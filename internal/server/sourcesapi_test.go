package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/sources"
)

// The sources settings section at the process boundary (ticket 10). The screen
// itself is Svelte, but everything it is load-bearing *for* is server state: a
// row that shows up, an order that resolution follows, a binding that comes
// back. Each of these asserts the action's effect on the snapshot the section
// renders, which is the same thing an operator would read off it.

func findSource(t *testing.T, m model.Model, name string) model.Source {
	t.Helper()
	for _, s := range m.Sources {
		if s.Name == name {
			return s
		}
	}
	t.Fatalf("source %q not in snapshot (%d sources)", name, len(m.Sources))
	return model.Source{}
}

func sourceNames(m model.Model) []string {
	out := make([]string, 0, len(m.Sources))
	for _, s := range m.Sources {
		out = append(out, s.Name)
	}
	return out
}

// skillBody is the minimal SKILL.md the discovery walk recognizes — the walk
// stats directories and reads no file, so the body only has to parse.
const skillBody = "---\nname: s\ndescription: a skill\n---\n\nbody\n"

// The whole of what the section does to a folder source: register it, see its
// skill count, reorder, toggle it off, and remove it — all without hand-editing
// TOML. Starts from an empty list (chartr ships none, ADR 0018).
func TestSourcesSectionDrivesTheList(t *testing.T) {
	h := chartrtest.Start(t, chartrtest.WithoutSkills())

	dir := t.TempDir()
	writeSkill(t, dir, "grill", skillBody)
	writeSkill(t, dir, "housekeeping", skillBody)

	code, body := h.Post("/api/config/sources", map[string]string{
		"name": "mine", "kind": "dir", "path": dir,
	})
	if code != 200 {
		t.Fatalf("register = %d, body %s", code, body)
	}

	snap := h.Snapshot(ctx(t))
	got := findSource(t, snap, "mine")
	if got.Status != sources.StatusOK || len(got.Skills) != 2 {
		t.Errorf("row = status %q with %d skills, want ok with 2", got.Status, len(got.Skills))
	}
	if !got.Enabled {
		t.Error("a freshly registered source is disabled")
	}
	if names := sourceNames(snap); len(names) != 1 || names[0] != "mine" {
		t.Errorf("source list = %v, want just the registered row", names)
	}
	if len(got.Shadowed) != 0 {
		t.Errorf("the only source shadows nothing, got %v", got.Shadowed)
	}

	// A reorder names the operator's whole list.
	if code, body := h.Post("/api/config/sources/reorder", map[string]any{
		"names": []string{"mine"},
	}); code != 200 {
		t.Fatalf("reorder = %d, body %s", code, body)
	}

	// Toggling off keeps the row and its position, and drops it out of resolution.
	if code, body := h.Put("/api/config/sources/mine/enabled", map[string]any{"enabled": false}); code != 200 {
		t.Fatalf("disable = %d, body %s", code, body)
	}
	if findSource(t, h.Snapshot(ctx(t)), "mine").Enabled {
		t.Error("the row is still enabled after being toggled off")
	}

	// Removing it drops the row and leaves the operator's folder untouched.
	if code, body := h.Delete("/api/config/sources/mine"); code != 200 {
		t.Fatalf("remove = %d, body %s", code, body)
	}
	if names := sourceNames(h.Snapshot(ctx(t))); len(names) != 0 {
		t.Errorf("after removal sources = %v, want an empty list", names)
	}
	if _, err := os.Stat(filepath.Join(dir, "grill", "SKILL.md")); err != nil {
		t.Errorf("removing the row deleted the operator's folder: %v", err)
	}
}

// A source registered before the operator ever opens Settings — which is every
// source the silent first-run migration writes — is an ordinary row in the list.
// This is the whole of what makes that silence acceptable (ticket 07).
func TestASourceRegisteredBeforeFirstOpenIsVisible(t *testing.T) {
	configDir := t.TempDir()
	legacy := filepath.Join(configDir, "skills")
	writeSkill(t, legacy, "grill", skillBody)

	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	snap := h.Snapshot(ctx(t))
	names := sourceNames(snap)
	if len(names) != 1 || names[0] != "Legacy skills" {
		t.Fatalf("the migrated source is not the whole list: %v", names)
	}
	if got := findSource(t, snap, "Legacy skills"); got.Path != legacy || len(got.Skills) != 1 {
		t.Errorf("the migrated row = %+v, want %s with one skill", got, legacy)
	}
}

// Registering a git source with `git` absent from PATH is refused at the gate,
// naming why, before a row is written.
func TestAGitSourceIsRefusedWithoutGit(t *testing.T) {
	h := chartrtest.Start(t)
	// Emptied only after the rig has started, so the seed and everything else
	// startup does still find the tools it needs; the gate is checked at
	// registration time, which is what this asserts.
	t.Setenv("PATH", t.TempDir())

	code, body := h.Post("/api/config/sources", map[string]string{
		"name": "theirs", "kind": "git", "url": "https://example.invalid/skills.git",
	})
	if code != 400 {
		t.Fatalf("register without git = %d, body %s", code, body)
	}
	if !strings.Contains(body, "git is not on PATH") {
		t.Errorf("the refusal does not name why: %s", body)
	}
	for _, n := range sourceNames(h.Snapshot(ctx(t))) {
		if n == "theirs" {
			t.Error("a row was written for a refused registration")
		}
	}
}

// The three config files the section opens, each resolved server-side from a
// *name* — the client never sends a path, which is the whole security property
// of the open action. `conventions.md` used to be a fourth here; it is not any
// more, since it moved out of the config root to a per-space file with no
// single global path to name (see conventions.go and TestConventionsLandInARegisteredSpace).
func TestTheSourcesFilesAreOpenableByName(t *testing.T) {
	h := chartrtest.Start(t)
	snap := h.Snapshot(ctx(t))

	want := map[string]string{
		"sources-config": "sources.toml",
		"user-config":    "user.toml",
		"preferences":    "preferences.md",
	}
	seen := map[string]bool{}
	for _, l := range snap.Config {
		if base, ok := want[l.Name]; ok {
			seen[l.Name] = true
			if filepath.Base(l.Path) != base {
				t.Errorf("layer %q resolves to %s, want %s", l.Name, l.Path, base)
			}
		}
	}
	for name := range want {
		if !seen[name] {
			t.Errorf("layer %q is not on the settings surface", name)
		}
	}

	// And an unknown name is refused rather than treated as a path.
	if code, _ := h.Post("/api/config/open", map[string]string{"layer": "../../etc/passwd"}); code != 400 {
		t.Errorf("opening an unknown layer = %d, want 400", code)
	}
}
