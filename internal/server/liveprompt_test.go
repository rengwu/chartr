package server_test

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// Live preset delivery at the process boundary (prompt-presets, ticket 03). The
// whole chain is production: a real PTY running a stub agent that paints real
// OSC titles, the sampler reading them through the shipped `claude` manifest,
// the queue, and the two writes that land on the agent's stdin. What the pane
// will read — eligibility and the one pending preset — is asserted off the
// pushed snapshot, which is the only place it exists.

// livePromptAgent installs a stub agent named `claude` and registers it. The stub
// paints the braille working glyph, waits for a cue file to appear, then paints
// the ✳ idle glyph and records every line it is sent — so a test drives the tab
// from working to idle by creating one file, and reads back what chartr typed.
//
// \342\240\202 is U+2802 (a braille frame, the working rule); \342\234\263 is
// U+2733 (✳, the positive idle rule).
func livePromptAgent(t *testing.T, h *chartrtest.Chartr) (cue, log string) {
	t.Helper()
	dir := t.TempDir()
	cue, log = filepath.Join(dir, "go-idle"), filepath.Join(dir, "stdin.log")

	bin := t.TempDir()
	script := "#!/bin/sh\n" +
		"printf '\\033]0;\\342\\240\\202 working\\007'\n" +
		"while [ ! -f " + cue + " ]; do sleep 0.05; done\n" +
		"printf '\\033]0;\\342\\234\\263 ready\\007'\n" +
		"while IFS= read -r line; do printf 'stdin: %s\\n' \"$line\" >> " + log + "; done\n"
	if err := os.WriteFile(filepath.Join(bin, "claude"), []byte(script), 0o755); err != nil {
		t.Fatalf("writing the stub agent: %v", err)
	}
	t.Setenv("PATH", bin+string(os.PathListSeparator)+os.Getenv("PATH"))
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})
	return cue, log
}

func goIdle(t *testing.T, cue string) {
	t.Helper()
	if err := os.WriteFile(cue, nil, 0o600); err != nil {
		t.Fatalf("cueing the stub agent to go idle: %v", err)
	}
}

// waitTerminal polls the snapshot until the space's tab satisfies pred.
func waitTerminal(t *testing.T, h *chartrtest.Chartr, spaceID, termID string, why string, pred func(model.Terminal) bool) model.Terminal {
	t.Helper()
	c, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	m := h.SnapshotUntil(c, func(m model.Model) bool {
		for _, term := range findSpace(t, m, spaceID).Terminals {
			if term.ID == termID {
				return pred(term)
			}
		}
		return false
	})
	tab := findTerminal(t, findSpace(t, m, spaceID), termID)
	if !pred(tab) {
		t.Fatalf("tab %s never %s; it is {status:%s alive:%v promptTarget:%v pending:%q}",
			termID, why, tab.Status, tab.Alive, tab.PromptTarget, tab.PendingPrompt)
	}
	return tab
}

func sendPrompt(t *testing.T, h *chartrtest.Chartr, spaceID, termID, promptID string) (int, bool) {
	t.Helper()
	code, body := h.Post("/api/spaces/"+spaceID+"/terminals/"+termID+"/prompt",
		map[string]string{"id": promptID})
	if code != 200 {
		return code, false
	}
	var r struct {
		Queued bool `json:"queued"`
	}
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("send-preset response not JSON: %v (%q)", err, body)
	}
	return code, r.Queued
}

// An idle agent is told at once, and nothing is left pending behind it.
func TestIdleAgentIsSentThePresetImmediately(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	cue, log := livePromptAgent(t, h)
	goIdle(t, cue) // idle from the first frame

	const body = "Answer in as few words as the question allows."
	id := createPrompt(t, h, "Keep answers brief", body)
	resp := register(t, h, repo)
	term := h.Launch(resp.ID, "claude")

	waitTerminal(t, h, resp.ID, term, "read idle", func(x model.Terminal) bool {
		return x.Status == model.TerminalIdle && x.PromptTarget
	})

	code, queued := sendPrompt(t, h, resp.ID, term, id)
	if code != 200 || queued {
		t.Fatalf("sending to an idle agent = %d queued=%v, want 200 queued=false", code, queued)
	}
	chartrtest.WaitForFileContains(t, log, "stdin: "+body, 10*time.Second)

	tab := findTerminal(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), term)
	if tab.PendingPrompt != "" {
		t.Errorf("pending preset = %q after an immediate send, want none", tab.PendingPrompt)
	}
}

// The queue, end to end: a working agent receives nothing and holds one visible
// pending preset; a second activation is refused while it waits; and the first
// idle the sampler observes submits it exactly once and clears the row.
func TestBusyAgentQueuesThePresetUntilItGoesIdle(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	cue, log := livePromptAgent(t, h)

	const body = "Follow the repository's commit convention."
	id := createPrompt(t, h, "Commit convention", body)
	other := createPrompt(t, h, "Keep answers brief", "One sentence.")
	resp := register(t, h, repo)
	term := h.Launch(resp.ID, "claude")

	waitTerminal(t, h, resp.ID, term, "read working", func(x model.Terminal) bool {
		return x.Status == model.TerminalWorking && x.PromptTarget
	})

	code, queued := sendPrompt(t, h, resp.ID, term, id)
	if code != 200 || !queued {
		t.Fatalf("sending to a working agent = %d queued=%v, want 200 queued=true", code, queued)
	}
	tab := waitTerminal(t, h, resp.ID, term, "showed the pending preset", func(x model.Terminal) bool {
		return x.PendingPrompt == id
	})
	if tab.Status != model.TerminalWorking {
		t.Errorf("tab status while queued = %q, want it still working", tab.Status)
	}

	// One pending item: a second activation is refused and leaves the first alone.
	if code, _ := sendPrompt(t, h, resp.ID, term, other); code != 409 {
		t.Errorf("a second activation while one is pending = %d, want 409", code)
	}
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Fatalf("a working agent was typed into: %q", b)
	}

	// It goes idle, and the queued preset lands — once — and the row clears.
	goIdle(t, cue)
	got := chartrtest.WaitForFileContains(t, log, "stdin: "+body, 10*time.Second)
	if n := strings.Count(got, body); n != 1 {
		t.Errorf("the preset arrived %d times, want exactly one: %q", n, got)
	}
	waitTerminal(t, h, resp.ID, term, "cleared the pending preset", func(x model.Terminal) bool {
		return x.PendingPrompt == ""
	})
}

// Cancelling drops the item before it is delivered: the agent goes idle and is
// told nothing, and the tab is free to be sent another preset.
func TestCancelledPresetNeverReachesTheAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	cue, log := livePromptAgent(t, h)

	id := createPrompt(t, h, "Keep answers brief", "One sentence.")
	resp := register(t, h, repo)
	term := h.Launch(resp.ID, "claude")

	waitTerminal(t, h, resp.ID, term, "read working", func(x model.Terminal) bool {
		return x.Status == model.TerminalWorking
	})
	if code, queued := sendPrompt(t, h, resp.ID, term, id); code != 200 || !queued {
		t.Fatalf("queueing = %d queued=%v, want 200 queued=true", code, queued)
	}
	if code, body := h.Delete("/api/spaces/" + resp.ID + "/terminals/" + term + "/prompt"); code != 204 {
		t.Fatalf("cancelling = %d, body %s", code, body)
	}
	waitTerminal(t, h, resp.ID, term, "cleared the cancelled preset", func(x model.Terminal) bool {
		return x.PendingPrompt == ""
	})

	// Idle now, with nothing to submit: the cancelled preset is simply gone.
	goIdle(t, cue)
	waitTerminal(t, h, resp.ID, term, "read idle", func(x model.Terminal) bool {
		return x.Status == model.TerminalIdle
	})
	time.Sleep(500 * time.Millisecond)
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Errorf("a cancelled preset reached the agent: %q", b)
	}

	// Cancelling again is an ordinary no-op, and the tab still accepts a preset.
	if code, body := h.Delete("/api/spaces/" + resp.ID + "/terminals/" + term + "/prompt"); code != 204 {
		t.Errorf("cancelling nothing = %d, body %s", code, body)
	}
	if code, queued := sendPrompt(t, h, resp.ID, term, id); code != 200 || queued {
		t.Errorf("sending after a cancel = %d queued=%v, want 200 queued=false", code, queued)
	}
}

// The narrow target. Everything that is not a live agent chartr launched is
// refused, and the snapshot says so before the operator can even try: an
// ordinary shell and a foreign tab carry no eligibility, and an unknown preset
// is refused before any tab is touched.
func TestLiveDeliveryRefusesEverythingButALaunchedAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo, otherRepo := chartrtest.NewSpaceRepo(t), chartrtest.NewSpaceRepo(t)
	cue, _ := livePromptAgent(t, h)
	goIdle(t, cue)

	id := createPrompt(t, h, "Keep answers brief", "One sentence.")
	resp := register(t, h, repo)
	other := register(t, h, otherRepo)
	term := h.Launch(resp.ID, "claude")
	waitTerminal(t, h, resp.ID, term, "read idle", func(x model.Terminal) bool {
		return x.Status == model.TerminalIdle && x.PromptTarget
	})

	// An id the catalog does not hold is refused before any tab is written to.
	if code, _ := sendPrompt(t, h, resp.ID, term, "no-such-preset"); code != 404 {
		t.Errorf("an unknown preset = %d, want 404", code)
	}
	// A tab that does not exist, and one that belongs to another space.
	if code, _ := sendPrompt(t, h, resp.ID, "nope", id); code != 404 {
		t.Errorf("an unknown terminal = %d, want 404", code)
	}
	if code, _ := sendPrompt(t, h, other.ID, term, id); code != 404 {
		t.Errorf("another space's terminal = %d, want 404", code)
	}

	// An ordinary shell is not an agent chartr launched: no eligibility on the
	// snapshot, and the action refused.
	shell := h.OpenTerminal(resp.ID)
	tab := findTerminal(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), shell)
	if tab.PromptTarget {
		t.Error("an ordinary shell reads as a live delivery target")
	}
	if code, _ := sendPrompt(t, h, resp.ID, shell, id); code != 409 {
		t.Errorf("an ordinary shell = %d, want 409", code)
	}
}

// A manually launched agent is the case the eligibility rule exists for: chartr
// *sees* an agent in this tab — it reads the agent grammar and its status comes
// from claude's own title — but chartr did not launch it, so it is refused. The
// difference is who started the binary, not what is running.
func TestManuallyLaunchedAgentIsRefused(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	cue, _ := livePromptAgent(t, h)
	goIdle(t, cue)

	id := createPrompt(t, h, "Keep answers brief", "One sentence.")
	resp := register(t, h, repo)

	shell := h.OpenTerminal(resp.ID)
	tc := h.DialTerminal(ctx(t), shell)
	defer tc.Close()
	tc.Send(ctx(t), "claude\n")

	// chartr identifies the agent the operator started: the tab reads the agent
	// grammar's idle under claude's own name.
	tab := waitTerminal(t, h, resp.ID, shell, "identified the operator's own claude", func(x model.Terminal) bool {
		return x.Proc == "claude" && x.Status == model.TerminalIdle
	})
	if tab.PromptTarget {
		t.Error("an agent the operator started themselves reads as a live delivery target")
	}
	if code, _ := sendPrompt(t, h, resp.ID, shell, id); code != 409 {
		t.Errorf("a manually launched agent = %d, want 409", code)
	}
}

// A tab whose process is gone is no target: the dead session stays pinned to its
// ticket, and the pane offers it nothing.
func TestDeadSessionIsNoDeliveryTarget(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubDyingAgent(t, "claude")

	id := createPrompt(t, h, "Keep answers brief", "One sentence.")
	resp := register(t, h, repo)
	sid := spawnThenDie(t, h, resp.ID, "widget", 1, "implement")

	tab := findTerminal(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), sid)
	if tab.PromptTarget {
		t.Error("a dead session reads as a live delivery target")
	}
	if code, _ := sendPrompt(t, h, resp.ID, sid, id); code != 409 {
		t.Errorf("a dead session = %d, want 409", code)
	}
}
