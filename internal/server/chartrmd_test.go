package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The standing `CHARTR.md` at a space's root (chartr-md, ticket 01). The reader
// is an agent chartr never launched, so every assertion here is one an operator
// could make with `ls`, `cat` and `git status` — which is the only way that
// reader ever encounters this feature.

// readDoc returns the standing document in a space, failing if it is not there.
func readDoc(t *testing.T, repo string) string {
	t.Helper()
	b, err := os.ReadFile(filepath.Join(repo, "CHARTR.md"))
	if err != nil {
		t.Fatalf("reading the standing document: %v", err)
	}
	return string(b)
}

// countExcludeLines reports how many times the ignore line stands in a space's
// local git exclude — the number that has to be exactly one after any number of
// reconciles.
func countExcludeLines(t *testing.T, repo string) int {
	t.Helper()
	b, err := os.ReadFile(filepath.Join(repo, ".git", "info", "exclude"))
	if err != nil {
		t.Fatalf("reading .git/info/exclude: %v", err)
	}
	n := 0
	for _, l := range strings.Split(string(b), "\n") {
		if strings.TrimSpace(l) == "CHARTR.md" {
			n++
		}
	}
	return n
}

// The whole destination in one test: a registered space carries the document at
// its root, the document is true in the mouth of an agent chartr did not launch,
// and `git status` in that space is clean without the tracked `.gitignore` being
// touched. Registration is deliberately not a trigger, so the space is
// reconciled by the restart — which is also what proves startup is a trigger.
func TestStandingDocLandsInARegisteredSpaceAndGitIgnoresIt(t *testing.T) {
	cfg := t.TempDir()
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, repo, ".gitignore", "node_modules/\n")
	chartrtest.Git(t, repo, "add", ".gitignore")
	chartrtest.Git(t, repo, "commit", "-qm", "the repository's own ignore file")

	h := chartrtest.Start(t, chartrtest.WithConfigDir(cfg))
	register(t, h, repo)
	if _, err := os.Stat(filepath.Join(repo, "CHARTR.md")); !os.IsNotExist(err) {
		t.Errorf("registration wrote the standing document; it is deliberately not a trigger (%v)", err)
	}

	chartrtest.Start(t, chartrtest.WithConfigDir(cfg))
	doc := readDoc(t, repo)

	// The first sentence has to be true of an agent chartr never launched.
	if !strings.HasPrefix(doc, "chartr is the cockpit that drives this repository") {
		t.Errorf("the standing document does not open with a sentence true of its reader:\n%s", doc)
	}
	if strings.Contains(doc, "opened this terminal") || strings.Contains(doc, "This shell") {
		t.Errorf("the standing document claims chartr opened the reader's shell:\n%s", doc)
	}
	// The same content rules the free payload carries: the sources inventory, no
	// fetchable address, and no live fact about the space.
	if !strings.Contains(doc, "chartr-skills") {
		t.Errorf("the sources block did not reach the standing document:\n%s", doc)
	}
	if strings.Contains(doc, "https://") {
		t.Errorf("a source URL reached the standing document:\n%s", doc)
	}

	if got := countExcludeLines(t, repo); got != 1 {
		t.Errorf(".git/info/exclude carries the ignore line %d times, want 1", got)
	}
	if status := chartrtest.Git(t, repo, "status", "--porcelain"); status != "" {
		t.Errorf("the standing document dirtied the space:\n%s", status)
	}
	if got := chartrtest.Git(t, repo, "show", "HEAD:.gitignore"); got != "node_modules/" {
		t.Errorf("the space's tracked .gitignore was modified: %q", got)
	}

	// A second reconcile is the same reconcile: the line is matched literally, so
	// it is appended once and never twice.
	chartrtest.Start(t, chartrtest.WithConfigDir(cfg))
	if got := countExcludeLines(t, repo); got != 1 {
		t.Errorf("after a second run .git/info/exclude carries the ignore line %d times, want 1", got)
	}
}

// The exclude is git's file and the operator's, not chartr's to rewrite: what is
// already in it survives, in its own order, with one line appended.
func TestStandingDocKeepsAnExistingGitExclude(t *testing.T) {
	cfg := t.TempDir()
	repo := chartrtest.NewSpaceRepo(t)
	excl := filepath.Join(repo, ".git", "info", "exclude")
	if err := os.MkdirAll(filepath.Dir(excl), 0o755); err != nil {
		t.Fatal(err)
	}
	// No trailing newline, which is the case that would otherwise glue chartr's
	// line onto the operator's last one.
	if err := os.WriteFile(excl, []byte("# mine\nscratch.txt"), 0o644); err != nil {
		t.Fatal(err)
	}

	h := chartrtest.Start(t, chartrtest.WithConfigDir(cfg))
	register(t, h, repo)
	chartrtest.Start(t, chartrtest.WithConfigDir(cfg))

	b, err := os.ReadFile(excl)
	if err != nil {
		t.Fatal(err)
	}
	if got, want := string(b), "# mine\nscratch.txt\nCHARTR.md\n"; got != want {
		t.Errorf(".git/info/exclude = %q, want %q", got, want)
	}
}

// Scratch follows the operator's home directory, which is not chartr's to write
// into and not necessarily a git repository at all.
func TestScratchSpaceGetsNoStandingDoc(t *testing.T) {
	home := t.TempDir()
	t.Setenv("HOME", home)

	chartrtest.Start(t)

	if _, err := os.Stat(filepath.Join(home, "CHARTR.md")); !os.IsNotExist(err) {
		t.Errorf("chartr wrote a standing document into the home directory (%v)", err)
	}
}

// A sources mutation is the second trigger, and the reason there is one: the
// sources block is half of what the document says. A registered space whose
// directory is gone must not take the settings save down with it, and a space
// whose bytes did not change must not have its file touched.
func TestSourcesMutationRewritesTheStandingDoc(t *testing.T) {
	cfg := t.TempDir()
	repo := chartrtest.NewSpaceRepo(t)
	gone := chartrtest.NewSpaceRepo(t)

	h := chartrtest.Start(t, chartrtest.WithConfigDir(cfg))
	register(t, h, repo)
	register(t, h, gone)
	h = chartrtest.Start(t, chartrtest.WithConfigDir(cfg))
	readDoc(t, repo) // the startup trigger, before the mutation under test

	// One registered space is unreachable from here on. It converges on the next
	// startup or sources change; it never fails one.
	if err := os.RemoveAll(gone); err != nil {
		t.Fatal(err)
	}

	skills := t.TempDir()
	writeSkill(t, skills, "spelunk", skillBody)
	code, body := h.Post("/api/config/sources", map[string]string{
		"name": "mine", "kind": "dir", "path": skills,
	})
	if code != 200 {
		t.Fatalf("register source = %d, body %s (an unreachable space must not fail a settings save)", code, body)
	}
	if doc := readDoc(t, repo); !strings.Contains(doc, "`mine` at `"+skills+"` — spelunk") {
		t.Errorf("the registered source did not reach the standing document:\n%s", doc)
	}

	// Toggling it off takes it back out — a disabled source is not in the
	// inventory at all.
	if code, body := h.Put("/api/config/sources/mine/enabled", map[string]bool{"enabled": false}); code != 200 {
		t.Fatalf("disable = %d, body %s", code, body)
	}
	if doc := readDoc(t, repo); strings.Contains(doc, "spelunk") {
		t.Errorf("a disabled source stayed in the standing document:\n%s", doc)
	}

	// A mutation that changes nothing the document says must not touch the file:
	// an unchanged space stays byte-for-byte and mtime-for-mtime as it was, so a
	// clean tree stays clean and no watcher fires.
	before, err := os.Stat(filepath.Join(repo, "CHARTR.md"))
	if err != nil {
		t.Fatal(err)
	}
	time.Sleep(10 * time.Millisecond)
	if code, body := h.Post("/api/config/sources/reorder", map[string][]string{
		"names": {"mine"}, // the whole reorderable list, in the order it is already in
	}); code != 200 {
		t.Fatalf("reorder = %d, body %s", code, body)
	}
	after, err := os.Stat(filepath.Join(repo, "CHARTR.md"))
	if err != nil {
		t.Fatal(err)
	}
	if !after.ModTime().Equal(before.ModTime()) {
		t.Error("an unchanged standing document was rewritten; its mtime moved")
	}
}
