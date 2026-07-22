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
	"github.com/rengwu/chartr/internal/prompt"
)

// Ticket 15 at the process boundary: the ideate on-ramp. A live, ticketless agent
// tab opened with the on-disk starter prompt typed in, sharing only the
// adapter's spawn primitive with a real session (spec, State model) — no map or
// ticket lookup, no claim commit, no lifecycle. It works in a space with no
// `.plan/` at all, which is the point of the on-ramp (planning ticket 07).
//
// Ticket 03 took away its one special case: it named no agent and quietly
// borrowed the `grill` binding's. It now names one like everything else, so every
// test here registers the agent it ideates with.

// ideateAgent registers the agent these tests ideate with, with a stub binary on
// PATH, and returns that stub's delivery log. The name is deliberately not a
// role's and not an adapter's — nothing about ideate resolves through either.
func ideateAgent(t *testing.T, h *chartrtest.Chartr) string {
	t.Helper()
	log := chartrtest.StubAgent(t, "some-harness")
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	return log
}

// Opening ideate in a mapless space spawns a live tab that carries no Session
// binding, and the starter prompt reaches the agent through the same
// read-this-file opener a real session uses, byte-matching the composed prompt.
func TestIdeateOpensLiveTicketlessTab(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	deliveryLog := ideateAgent(t, h)

	resp := register(t, h, repo)
	id := h.Ideate(resp.ID, "thinker")

	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	tab := findTerminal(t, s, id)
	if !tab.Alive {
		t.Errorf("ideate tab is not alive")
	}
	if tab.Session != nil {
		t.Errorf("ideate tab carries a Session binding %+v, want none — it must not read as a real session", tab.Session)
	}

	promptRel := filepath.Join(".chartr", "run", id, "payload.md")
	promptAbs := filepath.Join(repo, promptRel)
	got, err := os.ReadFile(promptAbs)
	if err != nil {
		t.Fatalf("ideate prompt not written: %v", err)
	}
	if want := prompt.Ideate(prompt.RootsFor(h.DataDir, h.ConfigDir, repo)); string(got) != want {
		t.Errorf("ideate prompt on disk does not match the composed starter prompt:\ngot:\n%s\nwant:\n%s", got, want)
	}

	log := chartrtest.WaitForFileContains(t, deliveryLog, promptAbs, 5*time.Second)
	if !strings.Contains(log, "Read the file") {
		t.Errorf("the opener the agent received did not read-this-file:\n%s", log)
	}
}

// No claim commit is ever written and no ticket's status ever moves — ideate
// leaves git history and every ticket exactly as it found them, even in a space
// that has both.
func TestIdeateWritesNoClaimCommit(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	ideateAgent(t, h)

	// A committed baseline so "no commit" is a real assertion, not just "no
	// commits exist yet".
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.Git(t, repo, "add", "-A")
	chartrtest.Git(t, repo, "commit", "-q", "-m", "baseline")
	before := commitCount(t, repo)

	resp := register(t, h, repo)
	h.Ideate(resp.ID, "thinker")

	if after := commitCount(t, repo); after != before {
		t.Errorf("ideate wrote a commit: HEAD went from %s to %s commits", before, after)
	}
	if st := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1).Status; st != "open" {
		t.Errorf("ideate changed an unrelated ticket's status to %q, want open", st)
	}
}

// An ideate agent that dies on its own drops from the model exactly like an
// ad-hoc shell — no pinning, no death halt, no lifecycle state ever derives for
// it (unlike a real session, which stays pinned to its ticket for resume/respawn/
// release).
func TestIdeateHasNoDeathHalt(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	marker := chartrtest.StubDyingAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	id := h.Ideate(resp.ID, "thinker")

	c, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()
	m := h.SnapshotUntil(c, func(m model.Model) bool {
		return !hasTerminal(findSpace(t, m, resp.ID), id)
	})
	if hasTerminal(findSpace(t, m, resp.ID), id) {
		t.Errorf("dead ideate tab %s (marker %s) is still listed — it should drop, not pin", id, marker)
	}
}

// Editing the materialized `ideate` skill on disk changes what the very next
// ideate session is told — the on-disk hackable markdown the Done-when calls
// for.
func TestIdeateStarterPromptIsEditable(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	ideateAgent(t, h)

	materialized := filepath.Join(h.DataDir, "skills", "ideate", "SKILL.md")
	if _, err := os.Stat(materialized); err != nil {
		t.Fatalf("ideate skill was not materialized: %v", err)
	}
	if err := os.WriteFile(materialized, []byte("EDITED-IDEATE-STARTER on disk."), 0o644); err != nil {
		t.Fatalf("editing the materialized ideate skill: %v", err)
	}

	resp := register(t, h, repo)
	id := h.Ideate(resp.ID, "thinker")

	got, err := os.ReadFile(filepath.Join(repo, ".chartr", "run", id, "payload.md"))
	if err != nil {
		t.Fatalf("ideate prompt not written: %v", err)
	}
	if !strings.Contains(string(got), "EDITED-IDEATE-STARTER") {
		t.Errorf("edit to the materialized starter prompt did not reach the next ideate session:\n%s", got)
	}
}

// Ticket 03: ideate names its agent. The agent the request names is the one that
// runs, its flags reach the process verbatim and in order, and the tab it seats
// still carries no Session — naming an agent is the only thing that changed about
// ideate, so it must not have quietly become a session.
func TestIdeateLaunchesTheNamedAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	// Both binaries are on PATH, so what this proves is the *name* deciding — not
	// the old grill binding's adapter happening to be absent.
	claudeDelivery := chartrtest.StubAgent(t, "claude")
	delivery := chartrtest.StubAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "thinker", map[string]any{
		"adapter": "some-harness",
		"args":    []string{"-m", "big", "--think"},
		"prompt":  "argv",
	})

	id := h.Ideate(resp.ID, "thinker")

	promptAbs := filepath.Join(repo, ".chartr", "run", id, "payload.md")
	log := chartrtest.WaitForFileContains(t, delivery, promptAbs, 5*time.Second)
	want := []string{"argv: -m", "argv: big", "argv: --think", "argv: Read the file " + promptAbs}
	if !inOrder(log, want) {
		t.Errorf("the named agent's argv did not reach the process in order.\nwant %v\ngot:\n%s", want, log)
	}
	if b, _ := os.ReadFile(claudeDelivery); len(b) > 0 {
		t.Errorf("ideate still borrowed the grill binding's adapter:\n%s", b)
	}

	tab := findTerminal(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), id)
	if tab.Session != nil {
		t.Errorf("ideate tab carries a Session binding %+v, want none", tab.Session)
	}
}

// An agent ideate cannot run is refused on the doorstep, the same two ways and in
// the same order a spawn refuses it — and nothing is opened either way. Ideate
// writes no claim, so "leaves the space as it was" means no tab and no prompt.
func TestIdeateRefusesAnUnknownOrAbsentAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.StubAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	// Registered, but its binary was never put on PATH.
	registerAgent(t, h, "ghost", map[string]any{"adapter": "no-such-harness"})

	if code, body := h.IdeateRaw(resp.ID, "nobody"); code != 400 {
		t.Errorf("ideate with an unregistered agent = %d, want 400; body %s", code, body)
	}
	code, body := h.IdeateRaw(resp.ID, "ghost")
	if code != 409 {
		t.Errorf("ideate with a PATH-absent agent = %d, want 409; body %s", code, body)
	}
	if !strings.Contains(body, "no-such-harness") {
		t.Errorf("the refusal does not name the missing binary: %s", body)
	}

	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if len(s.Terminals) != 0 {
		t.Errorf("a refused ideate opened a tab: %+v", s.Terminals)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run")); err == nil {
		t.Errorf("a refused ideate wrote a starter prompt into %s", filepath.Join(repo, ".chartr", "run"))
	}
}
