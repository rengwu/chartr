package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/registry"
)

// Scratch is present in the server-authoritative model from first run, but its
// derived shape is deliberately only an identity, a home-directory path, and
// its live shells. Keeping every repository-derived field empty is the public
// proof that a rebuild did not discover, classify, or run git against $HOME.
func TestSnapshotAlwaysCarriesAThinScratchSpace(t *testing.T) {
	h := chartrtest.Start(t)
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("resolve home directory: %v", err)
	}

	snap := h.Snapshot(ctx(t))
	if len(snap.Spaces) != 1 {
		t.Fatalf("first-run snapshot has %d spaces, want only Scratch", len(snap.Spaces))
	}
	scratch := findSpace(t, snap, registry.ScratchID)
	if !scratch.Scratch {
		t.Error("Scratch space is not flagged")
	}
	if scratch.Name != "Scratch" {
		t.Errorf("Scratch name = %q, want Scratch", scratch.Name)
	}
	if scratch.Path != home {
		t.Errorf("Scratch path = %q, want home directory %q", scratch.Path, home)
	}
	if scratch.Branch != "" || scratch.Dirty || scratch.LastAgent != "" {
		t.Errorf("Scratch carries repository state: branch=%q dirty=%v lastAgent=%q",
			scratch.Branch, scratch.Dirty, scratch.LastAgent)
	}
	if len(scratch.Maps) != 0 || len(scratch.Skills) != 0 || len(scratch.Layers) != 0 {
		t.Errorf("Scratch is not thin: maps=%d skills=%d layers=%d",
			len(scratch.Maps), len(scratch.Skills), len(scratch.Layers))
	}
	if scratch.TrackerAdapter != nil {
		t.Errorf("Scratch carries a tracker-adapter offer: %+v", scratch.TrackerAdapter)
	}
	if len(scratch.Terminals) != 0 {
		t.Errorf("fresh Scratch carries %d terminals, want none", len(scratch.Terminals))
	}

	if _, err := os.Stat(filepath.Join(h.ConfigDir, "spaces.toml")); !os.IsNotExist(err) {
		t.Errorf("first run wrote a registry row for Scratch: stat error = %v", err)
	}
}

// Opening through the ordinary terminal endpoint proves Scratch is an ordinary
// registry lookup to the terminal manager: the shell starts in its entry path,
// the operator's home, and multiple shells group beneath the same space.
func TestScratchShellsOpenInTheHomeDirectory(t *testing.T) {
	fakeHome := t.TempDir()
	t.Setenv("HOME", fakeHome)
	h := chartrtest.Start(t)
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("resolve home directory: %v", err)
	}

	first := h.OpenTerminal(registry.ScratchID)
	tc := h.DialTerminal(ctx(t), first)
	defer tc.Close()
	tc.Send(ctx(t), "printf 'scratch-cwd<%s>\\n' \"$PWD\"\n")
	// macOS exposes /var as a symlink to /private/var; the shell reports the
	// physical spelling after chdir even though UserHomeDir returned the logical
	// one. They name the same directory.
	shellHome, err := filepath.EvalSymlinks(home)
	if err != nil {
		t.Fatalf("resolve home-directory symlinks: %v", err)
	}
	wantCWD := "scratch-cwd<" + shellHome + ">"
	out := tc.ReadUntil(ctx(t), wantCWD)
	if !strings.Contains(out, wantCWD) {
		t.Fatalf("Scratch shell did not report home as its working directory; output %q", out)
	}

	second := h.OpenTerminal(registry.ScratchID)
	scratch := findSpace(t, h.Snapshot(ctx(t)), registry.ScratchID)
	if !hasTerminal(scratch, first) || !hasTerminal(scratch, second) {
		t.Fatalf("Scratch terminals = %+v, want both %s and %s", scratch.Terminals, first, second)
	}
	if got := len(scratch.Terminals); got != 2 {
		t.Errorf("Scratch carries %d terminals, want 2", got)
	}

	// Opening shells mutates runtime state only: it neither registers $HOME nor
	// runs the registration path that could initialise a repository there.
	if _, err := os.Stat(filepath.Join(h.ConfigDir, "spaces.toml")); !os.IsNotExist(err) {
		t.Errorf("opening Scratch wrote spaces.toml: stat error = %v", err)
	}
	if _, err := os.Stat(filepath.Join(fakeHome, ".git")); !os.IsNotExist(err) {
		t.Errorf("opening Scratch ran git init in the home directory: stat error = %v", err)
	}
}

// A hidden Scratch row is absent from the list the chrome can post. The server
// puts it back at its current slot, while retaining the full-list validation for
// every registered space.
func TestReorderMayOmitOnlyScratch(t *testing.T) {
	h := chartrtest.Start(t)
	a := register(t, h, chartrtest.NewSpaceRepo(t))
	b := register(t, h, chartrtest.NewSpaceRepo(t))

	withScratch := []string{a.ID, registry.ScratchID, b.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": withScratch}); code != 204 {
		t.Fatalf("seat Scratch between registered spaces = %d, body %s", code, body)
	}

	visibleOrder := []string{b.ID, a.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": visibleOrder}); code != 204 {
		t.Fatalf("reorder omitting Scratch = %d, body %s", code, body)
	}
	want := []string{b.ID, registry.ScratchID, a.ID}
	if got := allSpaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Fatalf("sidebar after hidden-Scratch reorder = %v, want %v", got, want)
	}

	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": []string{b.ID}}); code != 400 {
		t.Errorf("reorder omitting a registered space = %d, want 400 (body %s)", code, body)
	}
	if got := allSpaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Errorf("sidebar after refused registered-space omission = %v, want untouched %v", got, want)
	}

	data, err := os.ReadFile(filepath.Join(h.ConfigDir, "spaces.toml"))
	if err != nil {
		t.Fatalf("read registry after reorder: %v", err)
	}
	if got := strings.Count(string(data), "[[space]]"); got != 2 {
		t.Errorf("registry carries %d space rows, want only the 2 registered spaces:\n%s", got, data)
	}
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("resolve home directory: %v", err)
	}
	if strings.Contains(string(data), "path = \""+home+"\"") {
		t.Errorf("registry persisted Scratch as a space row:\n%s", data)
	}
}

func allSpaceIDs(m model.Model) []string {
	out := make([]string, 0, len(m.Spaces))
	for _, s := range m.Spaces {
		out = append(out, s.ID)
	}
	return out
}
