package server_test

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
	"github.com/rengwu/wayfinder-harness/internal/model"
)

// Ticket 04 at the process boundary: kind is declared, never inferred (ADR
// 0007). A discovered map with no declaration is inert — rendered and readable,
// but offering no session actions — until a human classifies it; classification
// is one HTTP action that writes the committed workspace config keyed by slug;
// the convention guess is pre-filled for both kinds; a renamed directory dangles
// its entry back to unclassified-and-inert. Session actions do not exist yet
// (spawning is a later ticket), so "offers no session actions" is asserted
// through the gate that will govern them: the map's declared Kind. Every
// assertion is on the public control-socket snapshot and the files on disk.

func classify(t *testing.T, h *harnesstest.Harness, spaceID, slug, kind string) (int, string) {
	t.Helper()
	return h.Post(
		fmt.Sprintf("/api/spaces/%s/maps/%s/classify", spaceID, slug),
		map[string]string{"kind": kind},
	)
}

// An undeclared map is inert with the convention guess pre-filled; classifying
// it declares the kind into committed config keyed by slug (so a teammate's
// clone inherits it), clears the now-spent guess, and — because the write
// appends rather than rewrites — leaves the operator's existing role bindings
// untouched.
func TestClassifyDeclaresKindAndPreservesConfig(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	// A pre-existing committed config with a role binding the append must keep.
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", `
[roles.implement]
model = "sonnet-ws"
`)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)

	// Undeclared: inert (kind unclassified) with a guess to pre-fill the confirm.
	m := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget")
	if m.Kind != "" {
		t.Errorf("undeclared map kind = %q, want unclassified (inert)", m.Kind)
	}
	if m.KindGuess == "" {
		t.Error("undeclared map carries no guess to pre-fill the confirm")
	}

	if code, body := classify(t, h, resp.ID, "widget", "implementation"); code != 200 {
		t.Fatalf("classify = %d, body %s", code, body)
	}

	// Declared: kind set, guess cleared — the map is no longer inert.
	m = findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget")
	if m.Kind != "implementation" {
		t.Errorf("classified map kind = %q, want implementation", m.Kind)
	}
	if m.KindGuess != "" {
		t.Errorf("classified map still carries the spent guess %q", m.KindGuess)
	}

	// The declaration landed in the committed layer, keyed by slug.
	cfg := readFile(t, filepath.Join(repo, ".wayfinder-harness/config.toml"))
	if !strings.Contains(cfg, "widget") || !strings.Contains(cfg, "implementation") {
		t.Errorf("committed config does not declare the kind:\n%s", cfg)
	}

	// The pre-existing role binding survived the append (write, not rewrite).
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if impl := binding(t, s, "implement"); impl.Model != "sonnet-ws" {
		t.Errorf("classify clobbered the binding: implement.model = %q, want sonnet-ws", impl.Model)
	}
}

// The guess follows the breakable conventions ADR 0007 keeps alive only as a
// one-time proposal: the `-impl` suffix, and every ticket typed `task`. It is
// pre-filled for both kinds, so classification is a one-keystroke confirm
// without ever being automatic.
func TestClassifyGuessMatchesConventions(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	// `-impl` suffix → implementation.
	harnesstest.WriteMap(t, repo, "analytics-impl", mapBody)
	harnesstest.WriteTicket(t, repo, "analytics-impl", "01-a.md", ticket(1, "A", "[]", "task", ""))

	// No suffix, but every ticket typed `task` → implementation.
	harnesstest.WriteMap(t, repo, "all-tasks", mapBody)
	harnesstest.WriteTicket(t, repo, "all-tasks", "01-a.md", ticket(1, "A", "[]", "task", ""))
	harnesstest.WriteTicket(t, repo, "all-tasks", "02-b.md", ticket(2, "B", "[]", "task", ""))

	// No suffix and a non-task ticket present → planning.
	harnesstest.WriteMap(t, repo, "discovery", mapBody)
	harnesstest.WriteTicket(t, repo, "discovery", "01-a.md", ticket(1, "A", "[]", "research", ""))
	harnesstest.WriteTicket(t, repo, "discovery", "02-b.md", ticket(2, "B", "[]", "task", ""))

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	want := map[string]string{
		"analytics-impl": "implementation",
		"all-tasks":      "implementation",
		"discovery":      "planning",
	}
	for slug, guess := range want {
		if got := findMap(t, s, slug).KindGuess; got != guess {
			t.Errorf("map %q guess = %q, want %q", slug, got, guess)
		}
		// A guessed map is still undeclared: the guess never classifies on its own.
		if got := findMap(t, s, slug).Kind; got != "" {
			t.Errorf("map %q kind = %q, want unclassified — the guess must not auto-classify", slug, got)
		}
	}
}

// Renaming a classified map's directory from outside dangles its committed entry
// (keyed by the old slug) and returns the map to unclassified-and-inert with a
// fresh guess — never an error (ADR 0007). Discovery is by notice, so the test
// renames while watching and waits for the push.
func TestRenamedMapDanglesToUnclassified(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "orig", mapBody)
	harnesstest.WriteTicket(t, repo, "orig", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	if code, body := classify(t, h, resp.ID, "orig", "implementation"); code != 200 {
		t.Fatalf("classify = %d, body %s", code, body)
	}
	if got := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "orig").Kind; got != "implementation" {
		t.Fatalf("precondition: orig kind = %q, want implementation", got)
	}

	cc := h.DialControl(ctx(t))
	defer cc.Close()
	cc.ReadSnapshot(ctx(t)) // drain the latest snapshot before the rename

	oldDir := filepath.Join(repo, ".plan", "orig")
	newDir := filepath.Join(repo, ".plan", "renamed")
	if err := os.Rename(oldDir, newDir); err != nil {
		t.Fatalf("renaming map dir: %v", err)
	}

	last := cc.WaitFor(ctx(t), func(m model.Model) bool {
		return hasMap(findSpace(t, m, resp.ID), "renamed")
	})
	s := findSpace(t, last, resp.ID)

	rm := findMap(t, s, "renamed")
	if rm.Kind != "" {
		t.Errorf("renamed map kind = %q, want unclassified — the declaration dangled", rm.Kind)
	}
	if rm.KindGuess == "" {
		t.Error("renamed map offers no fresh guess to re-classify")
	}
	if hasMap(s, "orig") {
		t.Error("old slug still present after the directory was renamed")
	}
}

// Classification refuses what it cannot honour rather than corrupting the
// operator's committed file: an unrecognised kind and a missing space are
// rejected, and re-classifying an already-declared map refuses rather than
// silently rewrite the file (a classified map is not inert, so this only guards
// a race or a hand-edited entry).
func TestClassifyRejectsBadInput(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	if code, _ := classify(t, h, resp.ID, "widget", "planing"); code != 400 {
		t.Errorf("classify with a bogus kind = %d, want 400", code)
	}
	if code, _ := classify(t, h, "no-such-space", "widget", "planning"); code != 404 {
		t.Errorf("classify on a missing space = %d, want 404", code)
	}

	if code, body := classify(t, h, resp.ID, "widget", "planning"); code != 200 {
		t.Fatalf("first classify = %d, body %s", code, body)
	}
	if code, _ := classify(t, h, resp.ID, "widget", "implementation"); code != 400 {
		t.Errorf("re-classify of an already-declared map = %d, want 400", code)
	}
}

// A committed kind the harness does not recognise is surfaced as a warning and
// the map stays unclassified-and-inert — adoption is never gated on config lint,
// and no lifecycle runs on a value the harness cannot read.
func TestUnrecognisedCommittedKindStaysInert(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", `
[maps."widget"]
kind = "planing"
`)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	if got := findMap(t, s, "widget").Kind; got != "" {
		t.Errorf("map with an unrecognised committed kind = %q, want unclassified", got)
	}
	if !hasSubstring(s.Warnings, "planing") {
		t.Errorf("unrecognised kind produced no warning; warnings = %v", s.Warnings)
	}
}
