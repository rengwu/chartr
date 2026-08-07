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
// skill count, move it above the default, toggle it off, and remove it — all
// without hand-editing TOML, which is the Done-when in one test.
func TestSourcesSectionDrivesTheList(t *testing.T) {
	h := chartrtest.Start(t)

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
	// The default row is always last, and it is the one row that never moves.
	if names := sourceNames(snap); names[len(names)-1] != sources.DefaultName {
		t.Errorf("source order = %v, want %s last", names, sources.DefaultName)
	}

	// A new row lands ahead of the synthetic default, so it is *its* `grill` that
	// wins and nothing of its own is shadowed — order is the whole of which skill
	// wins, and it is the one thing the section lets the operator edit about it.
	if len(got.Shadowed) != 0 {
		t.Errorf("%q sits ahead of the default row and shadows nothing, got %v", got.Name, got.Shadowed)
	}
	if got := findSource(t, snap, sources.DefaultName); len(got.Shadowed) == 0 {
		t.Error("the default row's grill is not marked shadowed by the row ahead of it")
	}
	// A reorder names the operator's whole list; the default row is not part of it.
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

	// Removing it drops the row and leaves the operator's folder untouched — a
	// dir source is theirs, and chartr deletes nothing it does not own.
	if code, body := h.Delete("/api/config/sources/mine"); code != 200 {
		t.Fatalf("remove = %d, body %s", code, body)
	}
	if names := sourceNames(h.Snapshot(ctx(t))); len(names) != 1 || names[0] != sources.DefaultName {
		t.Errorf("after removal sources = %v, want the default row alone", names)
	}
	if _, err := os.Stat(filepath.Join(dir, "grill", "SKILL.md")); err != nil {
		t.Errorf("removing the row deleted the operator's folder: %v", err)
	}
}

// The default row is protected in exactly two ways and no more: it cannot be
// removed and it cannot be reordered. It still toggles like any other.
func TestTheDefaultSourceRowIsProtected(t *testing.T) {
	h := chartrtest.Start(t)

	if code, body := h.Delete("/api/config/sources/" + sources.DefaultName); code != 400 {
		t.Errorf("removing the default row = %d, want 400 (body %s)", code, body)
	}
	if code, _ := h.Post("/api/config/sources/reorder", map[string]any{
		"names": []string{sources.DefaultName},
	}); code != 400 {
		t.Errorf("reordering the default row = %d, want 400", code)
	}
	if code, body := h.Put("/api/config/sources/"+sources.DefaultName+"/enabled",
		map[string]any{"enabled": false}); code != 200 {
		t.Fatalf("disabling the default row = %d, body %s", code, body)
	}
	if findSource(t, h.Snapshot(ctx(t)), sources.DefaultName).Enabled {
		t.Error("the default row did not toggle off")
	}
}

// The default row reads "shipped with this build" until a refresh pins it, which
// the section renders off `seeded` rather than by inspecting the filesystem.
func TestTheDefaultSourceReadsAsShipped(t *testing.T) {
	h := chartrtest.Start(t)
	got := findSource(t, h.Snapshot(ctx(t)), sources.DefaultName)
	if !got.Seeded {
		t.Error("the freshly seeded default row does not read as shipped")
	}
	if !got.Default || got.Commit != "" {
		t.Errorf("default row = %+v, want the synthetic row with no pin", got)
	}
	if len(got.Skills) == 0 {
		t.Error("the seeded default row yields no skills")
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
	if len(names) < 2 {
		t.Fatalf("the migrated source is not in the list: %v", names)
	}
	if names[len(names)-1] != sources.DefaultName {
		t.Errorf("source order = %v, want %s last", names, sources.DefaultName)
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

// A deleted role binding has exactly one recovery, and it is this control. The
// deletion itself is legitimate — nothing refills it at startup — so the test
// asserts both halves: it stays gone across a rebuild, and the restore writes it
// back to the seeded ref.
func TestRestoringADeletedRoleBinding(t *testing.T) {
	configDir := t.TempDir()
	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))

	// Delete the implement binding the way an operator would: edit the file.
	path := filepath.Join(configDir, "user.toml")
	before := userConfig(t, configDir)
	after := strings.ReplaceAll(before, "implement = \"chartr-skills/implement\"\n", "")
	if after == before {
		t.Fatalf("the seeded implement binding is not in user.toml:\n%s", before)
	}
	if err := os.WriteFile(path, []byte(after), 0o600); err != nil {
		t.Fatalf("writing user.toml: %v", err)
	}

	// It reads as unbound, and nothing puts it back on its own.
	role := findRole(t, h.Snapshot(ctx(t)), "implement")
	if role.Ref != "" {
		t.Errorf("implement = %q after deletion, want unbound", role.Ref)
	}
	if role.Default != "chartr-skills/implement" {
		t.Errorf("implement default = %q", role.Default)
	}

	if code, body := h.Post("/api/config/roles/implement/restore", nil); code != 200 {
		t.Fatalf("restore = %d, body %s", code, body)
	}
	role = findRole(t, h.Snapshot(ctx(t)), "implement")
	if role.Ref != "chartr-skills/implement" || !role.Resolves {
		t.Errorf("implement = %+v after restore, want the seeded ref, resolving", role)
	}
	if !strings.Contains(userConfig(t, configDir), `implement = "chartr-skills/implement"`) {
		t.Error("the restore did not reach user.toml")
	}
}

func findRole(t *testing.T, m model.Model, role string) model.RoleBinding {
	t.Helper()
	for _, r := range m.Roles {
		if r.Role == role {
			return r
		}
	}
	t.Fatalf("role %q not in snapshot (%d roles)", role, len(m.Roles))
	return model.RoleBinding{}
}

// The four config files the section opens, each resolved server-side from a
// *name* — the client never sends a path, which is the whole security property
// of the open action.
func TestTheSourcesFilesAreOpenableByName(t *testing.T) {
	h := chartrtest.Start(t)
	snap := h.Snapshot(ctx(t))

	want := map[string]string{
		"sources-config": "sources.toml",
		"user-config":    "user.toml",
		"conventions":    "conventions.md",
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

// The free payload previews from this screen: the same seam a ticket's preview
// uses, four parts, no ticket and no role.
func TestTheFreePayloadPreviewsFromSettings(t *testing.T) {
	h := chartrtest.Start(t)
	code, body := h.Get("/api/payload/free")
	if code != 200 {
		t.Fatalf("free payload preview = %d, body %s", code, body)
	}
	if !strings.Contains(body, `"parts"`) || !strings.Contains(body, `"markdown"`) {
		t.Errorf("the free payload preview is not a composed payload: %s", body)
	}
}
