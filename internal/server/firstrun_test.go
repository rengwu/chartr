package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/sources"
)

// The migration off the three-layer skill model, driven at the process boundary:
// a config root pre-populated the way an upgrading operator's is, a real server
// started against it, and the source list read back off disk. There is no HTTP or
// model surface for sources yet (that is tickets 08 and 10), so the file the
// migration writes is the seam these assert through.

// oldRoot builds a config root in the shape the retired layer model left behind
// and returns it. No `sources.toml`, which is what the migration fires on.
func oldRoot(t *testing.T) string {
	t.Helper()
	return t.TempDir()
}

// writeSkill drops a minimal skill directory — the shape the discovery walk
// recognizes — under root.
func writeSkill(t *testing.T, root, name, body string) {
	t.Helper()
	dir := filepath.Join(root, name)
	if err := os.MkdirAll(dir, 0o700); err != nil {
		t.Fatalf("creating %s: %v", dir, err)
	}
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte(body), 0o600); err != nil {
		t.Fatalf("writing %s: %v", name, err)
	}
}

// migratedRows is the operator's own list. chartr ships no default row any more
// (ADR 0018), so the whole list is what migrated.
func migratedRows(t *testing.T, configDir string) []sources.Source {
	t.Helper()
	r, err := sources.Load(configDir)
	if err != nil {
		t.Fatalf("loading the migrated source list: %v", err)
	}
	return r.List()
}

// The user layer of the old model becomes an ordinary registered folder, so an
// operator's own skills keep resolving after the upgrade without them doing
// anything.
func TestMigrationRegistersTheLegacySkillsDirectory(t *testing.T) {
	root := oldRoot(t)
	legacy := filepath.Join(root, "skills")
	writeSkill(t, legacy, "implement", "---\nname: implement\n---\n\nmine.\n")

	chartrtest.Start(t, chartrtest.WithConfigDir(root))

	rows := migratedRows(t, root)
	if len(rows) != 1 {
		t.Fatalf("migrated rows = %+v, want exactly the legacy row", rows)
	}
	if rows[0].Name != "Legacy skills" {
		t.Errorf("legacy row name = %q, want %q", rows[0].Name, "Legacy skills")
	}
	if rows[0].Kind != sources.KindDir || rows[0].Path != legacy {
		t.Errorf("legacy row = %+v, want a dir source at %s", rows[0], legacy)
	}
	if !rows[0].Enabled {
		t.Error("the migrated legacy row is disabled; it drove skills yesterday")
	}
}

// An empty or absent old user layer contributes nothing — a permanently `empty`
// row is worse than no row.
func TestMigrationSkipsAnEmptyLegacySkillsDirectory(t *testing.T) {
	root := oldRoot(t)
	if err := os.MkdirAll(filepath.Join(root, "skills"), 0o700); err != nil {
		t.Fatalf("creating the legacy directory: %v", err)
	}

	chartrtest.Start(t, chartrtest.WithConfigDir(root))

	if rows := migratedRows(t, root); len(rows) != 0 {
		t.Errorf("migrated rows = %+v, want none from an empty legacy directory", rows)
	}
	// The file must still be written, or the migration fires again at every start.
	if _, err := os.Stat(sources.FilePath(root)); err != nil {
		t.Errorf("nothing to migrate left no source list behind: %v", err)
	}
}

// A materialized built-in library is now kept exactly where it is and registered
// as an ordinary folder, after the legacy row — the relative order the old model
// gave them.
//
// The cut (ticket 09) deleted the embedded shipped library, and with it the byte
// comparison this used to split two ways: an untouched copy was renamed aside and
// only a diverging one was registered. With nothing left to compare against, the
// branch that touches nothing on disk is the only honest one, and it is the
// direction ticket 06 already argued for — keeping too much is the right error to
// make. An empty or absent directory still contributes no row, exactly like the
// legacy one.
func TestMigrationRegistersTheBuiltinLibraryInPlace(t *testing.T) {
	root := oldRoot(t)
	writeSkill(t, filepath.Join(root, "skills"), "implement", "---\nname: implement\n---\n\nmine.\n")
	builtin := filepath.Join(root, "builtin-skills")
	writeSkill(t, builtin, "core", "---\nname: core\n---\n\nmy edited core.\n")

	chartrtest.Start(t, chartrtest.WithConfigDir(root))

	rows := migratedRows(t, root)
	if len(rows) != 2 {
		t.Fatalf("migrated rows = %+v, want the legacy row then the built-in one", rows)
	}
	if rows[0].Name != "Legacy skills" || rows[1].Name != "Migrated built-in skills" {
		t.Fatalf("migrated order = %q, %q; want %q before %q",
			rows[0].Name, rows[1].Name, "Legacy skills", "Migrated built-in skills")
	}
	if rows[1].Kind != sources.KindDir || rows[1].Path != builtin {
		t.Errorf("built-in row = %+v, want a dir source left where it was (%s)", rows[1], builtin)
	}
	if _, err := os.Stat(filepath.Join(root, "builtin-skills.migrated")); !os.IsNotExist(err) {
		t.Errorf("the built-in library was moved aside; nothing may move it now: stat error = %v", err)
	}
	// The operator's bytes survive untouched — the whole reason this is kept in place.
	after, err := os.ReadFile(filepath.Join(builtin, "core", "SKILL.md"))
	if err != nil || !strings.Contains(string(after), "my edited core") {
		t.Errorf("the operator's copy did not survive the migration (err = %v)", err)
	}
}

// An empty built-in directory answers the discovery walk with nothing, so it
// contributes no row rather than a permanently `empty` one.
func TestMigrationSkipsAnEmptyBuiltinLibrary(t *testing.T) {
	root := oldRoot(t)
	if err := os.MkdirAll(filepath.Join(root, "builtin-skills"), 0o700); err != nil {
		t.Fatalf("creating the built-in directory: %v", err)
	}

	chartrtest.Start(t, chartrtest.WithConfigDir(root))

	if rows := migratedRows(t, root); len(rows) != 0 {
		t.Errorf("migrated rows = %+v, want none from an empty built-in directory", rows)
	}
}

// The migration fires on the absence of the source list, so a root that already
// has one is never walked again — an operator who removed a migrated row does not
// get it back on the next start.
func TestMigrationRunsOnlyOnce(t *testing.T) {
	root := oldRoot(t)
	writeSkill(t, filepath.Join(root, "skills"), "implement", "---\nname: implement\n---\n\nmine.\n")

	chartrtest.Start(t, chartrtest.WithConfigDir(root))
	if rows := migratedRows(t, root); len(rows) != 1 {
		t.Fatalf("first start migrated %+v, want the legacy row", rows)
	}

	r, err := sources.Load(root)
	if err != nil {
		t.Fatalf("loading the source list: %v", err)
	}
	if err := r.Remove("Legacy skills"); err != nil {
		t.Fatalf("removing the migrated row: %v", err)
	}

	chartrtest.Start(t, chartrtest.WithConfigDir(root))
	if rows := migratedRows(t, root); len(rows) != 0 {
		t.Errorf("the migration ran again and restored %+v", rows)
	}
}

// Nothing this migration does reaches into a space, and nothing it does is
// announced: the operator finds the migrated rows when they next open Settings.
func TestMigrationWritesNothingIntoASpaceAndReportsNothing(t *testing.T) {
	root := oldRoot(t)
	writeSkill(t, filepath.Join(root, "skills"), "implement", "---\nname: implement\n---\n\nmine.\n")
	space := t.TempDir()
	if err := os.MkdirAll(filepath.Join(space, ".chartr", "skills", "implement"), 0o700); err != nil {
		t.Fatalf("creating the workspace layer: %v", err)
	}
	if err := os.WriteFile(
		filepath.Join(space, ".chartr", "skills", "implement", "SKILL.md"),
		[]byte("---\nname: implement\n---\n\ncommitted.\n"), 0o600,
	); err != nil {
		t.Fatalf("writing the workspace skill: %v", err)
	}

	h := chartrtest.Start(t, chartrtest.WithConfigDir(root))
	register(t, h, space)
	h.Snapshot(ctx(t))

	// `.chartr/skills` is now chartr's per-machine skill mirror, owned outright
	// (ADR 0018) — the one path it shares with the retired workspace layer. A
	// register now reconciles that directory to the enabled sources (none here,
	// so it empties), which reclaims the stale flat legacy layer rather than
	// preserving it. The mirror is gitignored, so this reclaim commits nothing;
	// the migration is still silent about it (asserted below).
	if _, err := os.Stat(filepath.Join(space, ".chartr", "skills", "implement", "SKILL.md")); !os.IsNotExist(err) {
		t.Errorf("the legacy workspace layer survived the register-time mirror: stat err = %v", err)
	}
	for _, s := range migratedRows(t, root) {
		if s.Path == filepath.Join(space, ".chartr", "skills") {
			t.Errorf("the workspace layer was registered as a source: %+v", s)
		}
	}
	if _, err := os.Stat(filepath.Join(space, "docs", "agents", "issue-tracker.md")); !os.IsNotExist(err) {
		t.Errorf("a tracker adapter was written into a space: stat error = %v", err)
	}
	for _, said := range []string{"Legacy skills", "migrat", "builtin-skills"} {
		if strings.Contains(h.Logged(), said) {
			t.Errorf("the migration announced itself (%q): %s", said, h.Logged())
		}
	}
}

// The retired tracker-adapter offer leaves an existing file exactly where it is.
// Its content stays true — nothing here moves or reshapes `.plan/maps/` — it
// merely stops being refreshed.
func TestAnExistingTrackerAdapterIsLeftUntouched(t *testing.T) {
	space := t.TempDir()
	if err := os.MkdirAll(filepath.Join(space, "docs", "agents"), 0o700); err != nil {
		t.Fatalf("creating docs/agents: %v", err)
	}
	adapter := filepath.Join(space, "docs", "agents", "issue-tracker.md")
	const body = "<!-- chartr-tracker-adapter: v1 -->\nold, and left alone.\n"
	if err := os.WriteFile(adapter, []byte(body), 0o600); err != nil {
		t.Fatalf("writing the existing adapter: %v", err)
	}

	h := chartrtest.Start(t)
	register(t, h, space)
	h.Snapshot(ctx(t))

	got, err := os.ReadFile(adapter)
	if err != nil {
		t.Fatalf("reading the adapter back: %v", err)
	}
	if string(got) != body {
		t.Errorf("the adapter was rewritten:\n%s", got)
	}
}
