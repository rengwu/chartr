package server_test

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// Ticket 08 at the process boundary: the new-shell control's agent rows. `POST
// /launch` starts a **free session** — an agent chartr launched into a space with
// no ticket, no role, and no brief: nothing is injected, and the operator types
// their first message into the live TUI themselves. It shares only the adapter's
// spawn primitive with a real session (no map/ticket lookup, no claim, no Session,
// no death halt), and the tab is titled by the agent's registered name — the thing
// the operator clicked.

// freeAgent registers the agent these tests launch with, with a stub binary on
// PATH, and returns that stub's delivery log. The name is deliberately not a
// role's and not an adapter's — nothing about a free session resolves through
// either.
func freeAgent(t *testing.T, h *chartrtest.Chartr) string {
	t.Helper()
	log := chartrtest.StubAgent(t, "some-harness")
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	return log
}

// A free session opens a live tab carrying no Session, titled by the agent's
// registered name, and injects nothing: the agent launches bare, no payload is
// written to the run directory, and no opener is typed into its TUI.
func TestFreeSessionOpensATabTitledByTheAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	deliveryLog := freeAgent(t, h)

	resp := register(t, h, repo)
	id := h.Launch(resp.ID, "thinker")

	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	tab := findTerminal(t, s, id)
	if !tab.Alive {
		t.Errorf("free tab is not alive")
	}
	if tab.Session != nil {
		t.Errorf("free tab carries a Session binding %+v, want none — it must not read as a real session", tab.Session)
	}
	// Titled by the thing the operator clicked, not by the adapter behind it.
	if tab.Title != "thinker" {
		t.Errorf("free tab title = %q, want the agent's registered name %q", tab.Title, "thinker")
	}

	// Nothing injected: no payload written for the tab to point at.
	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run", id)); err == nil {
		t.Errorf("a free session wrote a payload into the run directory")
	}

	// The agent did launch — the stub records its env the moment it starts — but no
	// opener ever reached it: no read-this-file line and nothing typed on stdin.
	log := chartrtest.WaitForFileContains(t, deliveryLog, "env:", 5*time.Second)
	if strings.Contains(log, "Read the file") {
		t.Errorf("a free session injected a read-this-file opener:\n%s", log)
	}
	if strings.Contains(log, "stdin:") {
		t.Errorf("a free session typed an opener into the agent's TUI:\n%s", log)
	}
}

// A free session never counts toward the one-live-session-per-space gate: a
// space with a live session still takes one, and a live free session does not
// stand in the way of a real spawn. This is the whole reason `Terminal.session`
// stays nil for it.
func TestFreeSessionIsOutsideTheOneSessionGate(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-a.md", ticket(1, "A", "[]", "task", ""))
	chartrtest.WriteTicket(t, repo, "widget", "02-b.md", ticket(2, "B", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})
	freeAgent(t, h)
	resp := register(t, h, repo)

	// A free session first: it must not seat anything the gate can see.
	h.Launch(resp.ID, "thinker")
	mustSpawn(t, h, resp.ID, "widget", 1, "implement")

	// And with a real session live, a free session is still allowed — where a
	// second spawn would be refused 409.
	h.Launch(resp.ID, "thinker")
}

// A free session settles the agent on the same doorstep a spawn uses, refusing an
// unknown or PATH-absent agent the same two ways and in the same order — and
// opening nothing and writing nothing either way.
func TestFreeSessionRefusesAnUnknownOrAbsentAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.StubAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	// Registered, but its binary was never put on PATH.
	registerAgent(t, h, "ghost", map[string]any{"adapter": "no-such-harness"})

	if code, body := h.LaunchRaw(resp.ID, "nobody"); code != 400 {
		t.Errorf("free session with an unregistered agent = %d, want 400; body %s", code, body)
	}
	code, body := h.LaunchRaw(resp.ID, "ghost")
	if code != 409 {
		t.Errorf("free session with a PATH-absent agent = %d, want 409; body %s", code, body)
	}
	if !strings.Contains(body, "no-such-harness") {
		t.Errorf("the refusal does not name the missing binary: %s", body)
	}

	if s := findSpace(t, h.Snapshot(ctx(t)), resp.ID); len(s.Terminals) != 0 {
		t.Errorf("a refused free session opened a tab: %+v", s.Terminals)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run")); err == nil {
		t.Errorf("a refused free session wrote a payload into the run directory")
	}
}

// A free session requires an agent exactly like a spawn (agent-selection ticket
// 04): naming none is refused, and with nothing registered at all the refusal is
// the distinct empty-library message that points at registration — the menu's
// disabled row routes there for the same reason.
func TestFreeSessionRefusedWithoutAnAgentAndWhenLibraryEmpty(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	// Empty library: the specific "register one" message, not "pick one".
	code, body := h.LaunchRaw(resp.ID, "")
	if code != 409 || !strings.Contains(body, "no agents are registered") {
		t.Errorf("free session against an empty library = %d (%s), want 409 empty-library", code, body)
	}

	// With an agent registered, naming none is the "pick one" refusal.
	chartrtest.StubAgent(t, "some-harness")
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	code, body = h.LaunchRaw(resp.ID, "")
	if code != 400 || !strings.Contains(body, "an agent is required") {
		t.Errorf("free session naming no agent = %d (%s), want 400 an-agent-is-required", code, body)
	}
}

// A free session remembers the agent it just ran — the same rule a real spawn
// follows — so the next spawn here is one click. There is nothing else to
// remember: the menu is a static list the operator picks from every time.
func TestFreeSessionRemembersTheAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	freeAgent(t, h)

	resp := register(t, h, repo)
	h.Launch(resp.ID, "thinker")

	if got := findSpace(t, h.Snapshot(ctx(t)), resp.ID).LastAgent; got != "thinker" {
		t.Fatalf("space remembered %q after a free session, want thinker", got)
	}
}

// No claim is ever recorded and no ticket's status ever moves — a free session is
// not a session, so nothing about the map changes when one opens.
func TestFreeSessionWritesNoClaim(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	freeAgent(t, h)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	before := auditCount(t, repo)

	resp := register(t, h, repo)
	h.Launch(resp.ID, "thinker")

	if after := auditCount(t, repo); after != before {
		t.Errorf("a free session recorded a claim: audit records went from %d to %d", before, after)
	}
	if st := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1).Status; st != "open" {
		t.Errorf("a free session changed an unrelated ticket's status to %q, want open", st)
	}
}

// A free-session agent that ends — the operator's Ctrl+C, an `/exit`, a crash —
// takes nothing with it. The tab it ran in is an ad-hoc shell with that agent
// preloaded (terminal/preload.go), so what is left when the agent goes is the
// shell it was started from: the tab stays listed and live, at its own prompt,
// with the whole conversation still in scrollback to read and the command still
// there to run again.
//
// It stays as an ad-hoc shell and nothing more. No pinning, no death halt, and no
// lifecycle state derives for it — that is a real session's, which is *its* agent's
// process and stays pinned to its ticket for resume/respawn/release.
func TestFreeSessionOutlivesItsAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	marker := chartrtest.StubDyingAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	id := h.Launch(resp.ID, "thinker")

	// Idle is the shell back at its prompt: the tab launches working, reads
	// running while the agent holds the terminal, and settles to its own prompt
	// once the agent is gone.
	c, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()
	m := h.SnapshotUntil(c, func(m model.Model) bool {
		s := findSpace(t, m, resp.ID)
		return hasTerminal(s, id) && findTerminal(t, s, id).Status == model.TerminalIdle
	})

	space := findSpace(t, m, resp.ID)
	if !hasTerminal(space, id) {
		t.Fatalf("free tab %s (marker %s) dropped when its agent quit — the shell behind it should still hold the tab", id, marker)
	}
	tab := findTerminal(t, space, id)
	if !tab.Alive {
		t.Errorf("free tab %s reads dead after its agent quit; its shell is what the tab runs", id)
	}
	if tab.Status != model.TerminalIdle {
		t.Errorf("free tab status = %q after its agent quit, want %q — a shell at its own prompt", tab.Status, model.TerminalIdle)
	}
	if tab.Session != nil {
		t.Errorf("free tab grew a session binding %+v — it is not a session and has no death to halt on", tab.Session)
	}
	if tab.PromptTarget {
		t.Error("free tab still offers to deliver a preset with only a shell in front of it")
	}
}
