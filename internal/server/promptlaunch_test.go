package server_test

import (
	"crypto/sha256"
	"encoding/hex"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The selection settled in ticket 01 taking effect on what a launch is actually
// told (prompt-presets, ticket 02): a ticket payload carries each selected preset
// as its own operator prompt part, a free launch in a space with a selection gets
// a small run payload through the ordinary opener, and a space with nothing
// selected launches exactly as it did before any of this existed.

func setSpacePrompts(t *testing.T, h *chartrtest.Chartr, spaceID string, ids ...string) {
	t.Helper()
	if code, body := h.Put("/api/spaces/"+spaceID+"/prompts", map[string]any{"ids": ids}); code != 204 {
		t.Fatalf("selecting %v = %d, body %s", ids, code, body)
	}
}

// One composition behind both surfaces: the preview shows the presets a spawn
// then writes, byte for byte, and the payload hash the claim records covers them
// — no second audit path for the preset bytes.
func TestSelectedPresetsRideTicketPreviewAndSpawn(t *testing.T) {
	chartrtest.StubAgent(t, "claude")
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-a.md", ticket(1, "A", "[]", "task", ""))
	space := register(t, h, repo)

	brief := createPrompt(t, h, "Keep it brief", "BRIEF-MARKER")
	commits := createPrompt(t, h, "Commit convention", "COMMITS-MARKER")
	// Selected in the wrong order on purpose: catalog order is the only order.
	setSpacePrompts(t, h, space.ID, commits, brief)

	code, p, body := getPayload(t, h, space.ID, "widget", 1, "implement")
	if code != 200 {
		t.Fatalf("payload preview = %d, body %s", code, body)
	}
	names := partNames(p)
	want := []string{"preferences", "preset " + brief, "preset " + commits}
	if !containsRun(names, want) {
		t.Fatalf("parts = %v, want %v consecutively", names, want)
	}
	first := findPart(t, p, "preset "+brief)
	if first.Origin != "operator" || first.Label != "Keep it brief" || first.Text != "BRIEF-MARKER" {
		t.Errorf("preset part = %+v, want the operator's own name and text", first)
	}

	sp := mustSpawn(t, h, space.ID, "widget", 1, "implement")
	got, err := os.ReadFile(filepath.Join(repo, ".chartr", "run", sp.SessionID, "payload.md"))
	if err != nil {
		t.Fatalf("reading the session payload: %v", err)
	}
	if string(got) != p.Markdown {
		t.Errorf("the spawned payload differs from the preview:\n--- preview ---\n%s\n--- spawned ---\n%s", p.Markdown, got)
	}
	sum := sha256.Sum256(got)
	if hex.EncodeToString(sum[:]) != sp.PayloadSha {
		t.Errorf("payloadSha %s does not cover the payload bytes", sp.PayloadSha)
	}
}

// Changing the selection changes only what is composed *next*. The already
// written payload keeps the bytes its session was told, and the standing
// `CHARTR.md` — which is not a launch payload — never moves either way.
func TestChangingTheSelectionAffectsOnlyLaterCompositions(t *testing.T) {
	chartrtest.StubAgent(t, "claude")
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-a.md", ticket(1, "A", "[]", "task", ""))
	chartrtest.WriteTicket(t, repo, "widget", "02-b.md", ticket(2, "B", "[]", "task", ""))
	space := register(t, h, repo)

	brief := createPrompt(t, h, "Keep it brief", "BRIEF-MARKER")
	setSpacePrompts(t, h, space.ID, brief)

	standing := filepath.Join(repo, "CHARTR.md")
	before, err := os.ReadFile(standing)
	if err != nil {
		t.Fatalf("reading CHARTR.md: %v", err)
	}

	first := mustSpawn(t, h, space.ID, "widget", 1, "implement")
	firstPayload := filepath.Join(repo, ".chartr", "run", first.SessionID, "payload.md")
	if got := mustRead(t, firstPayload); !strings.Contains(got, "BRIEF-MARKER") {
		t.Fatalf("the first session was not told the selected preset:\n%s", got)
	}

	// Deselect, then start the next session: the running one's bytes stand.
	setSpacePrompts(t, h, space.ID)
	if got := mustRead(t, firstPayload); !strings.Contains(got, "BRIEF-MARKER") {
		t.Errorf("deselecting rewrote a payload a session already has:\n%s", got)
	}
	_, p, _ := getPayload(t, h, space.ID, "widget", 2, "implement")
	if strings.Contains(p.Markdown, "BRIEF-MARKER") {
		t.Errorf("a later composition still carries the deselected preset:\n%s", p.Markdown)
	}

	if got := mustRead(t, standing); got != string(before) {
		t.Errorf("CHARTR.md changed:\n--- before ---\n%s\n--- after ---\n%s", before, got)
	}
}

// A free session in a space with a selection is no longer bare: it gets the
// presets as a gitignored, owner-only run payload and the ordinary
// read-this-file opener — carried by whichever delivery its agent uses, since
// this is the same opener seam a spawn goes through and neither half is new.
func TestFreeSessionWithSelectedPresetsGetsARunPayload(t *testing.T) {
	deliveries := []struct {
		agent, mode, tag string
	}{
		{"typist", "type", "stdin: "},
		{"arger", "argv", "argv: "},
		{"flagger", "--prompt", "argv: "},
	}
	for _, d := range deliveries {
		t.Run(d.mode, func(t *testing.T) {
			h := chartrtest.Start(t)
			repo := chartrtest.NewSpaceRepo(t)
			deliveryLog := chartrtest.StubAgent(t, "some-harness")
			registerAgent(t, h, d.agent, map[string]any{"adapter": "some-harness", "prompt": d.mode})
			space := register(t, h, repo)

			brief := createPrompt(t, h, "Keep it brief", "BRIEF-MARKER")
			setSpacePrompts(t, h, space.ID, brief)

			id := h.Launch(space.ID, d.agent)

			payload := filepath.Join(repo, ".chartr", "run", id, "payload.md")
			if got := mustRead(t, payload); got != "BRIEF-MARKER\n" {
				t.Errorf("free payload = %q, want the selected preset and nothing else", got)
			}
			info, err := os.Stat(payload)
			if err != nil || info.Mode().Perm() != 0o600 {
				t.Errorf("free payload mode = %v (%v), want 0600", info.Mode().Perm(), err)
			}
			if ignore := mustRead(t, filepath.Join(repo, ".chartr", "run", ".gitignore")); !strings.Contains(ignore, "*") {
				t.Errorf("the run directory is not gitignored: %q", ignore)
			}

			log := chartrtest.WaitForFileContains(t, deliveryLog, "Read the file", 5*time.Second)
			if !strings.Contains(log, d.tag+"Read the file "+payload) {
				t.Errorf("the opener did not arrive by %s delivery pointing at %s:\n%s", d.mode, payload, log)
			}
		})
	}
}

// Nothing selected is still the bare launch this control has always been: no
// payload, no opener, nothing injected.
func TestFreeSessionWithNoSelectionStaysBare(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	deliveryLog := freeAgent(t, h)
	space := register(t, h, repo)
	// A catalog with presets in it, none of them selected here.
	createPrompt(t, h, "Keep it brief", "BRIEF-MARKER")

	id := h.Launch(space.ID, "thinker")

	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run", id)); err == nil {
		t.Errorf("a free session with no selection wrote a payload")
	}
	log := chartrtest.WaitForFileContains(t, deliveryLog, "env:", 5*time.Second)
	if strings.Contains(log, "Read the file") || strings.Contains(log, "stdin:") {
		t.Errorf("a free session with no selection was told something:\n%s", log)
	}
}

func mustRead(t *testing.T, path string) string {
	t.Helper()
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("reading %s: %v", path, err)
	}
	return string(b)
}

// containsRun reports whether want appears in got as a consecutive run.
func containsRun(got, want []string) bool {
	for i := 0; i+len(want) <= len(got); i++ {
		if strings.Join(got[i:i+len(want)], "\x00") == strings.Join(want, "\x00") {
			return true
		}
	}
	return false
}
