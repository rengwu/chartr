package sources_test

import (
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/sources"
)

// The registry is driven the way registry_test drives the space registry: a real
// TOML file under a temp config root, through the package's public surface, with
// hand-written malformed files for the degradation cases. The spec's failure-mode
// table is what most of this file asserts — every row of it has a reading, and a
// reading nobody tests is a reading nobody kept.

// skillAt creates a skill directory: rel below root, holding a SKILL.md.
func skillAt(t *testing.T, root, rel string) string {
	t.Helper()
	dir := filepath.Join(root, filepath.FromSlash(rel))
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte("---\nname: x\n---\n\nbody\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	return dir
}

func names(list []sources.Skill) []string {
	out := make([]string, 0, len(list))
	for _, s := range list {
		out = append(out, s.Name)
	}
	return out
}

func sourceNames(list []sources.Source) []string {
	out := make([]string, 0, len(list))
	for _, s := range list {
		out = append(out, s.Name)
	}
	return out
}

func load(t *testing.T, cfg string) *sources.Registry {
	t.Helper()
	r, err := sources.Load(cfg)
	if err != nil {
		t.Fatalf("load: %v", err)
	}
	return r
}

func write(t *testing.T, cfg, body string) {
	t.Helper()
	if err := os.MkdirAll(cfg, 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(cfg, "sources.toml"), []byte(body), 0o600); err != nil {
		t.Fatal(err)
	}
}

// --- the file itself -------------------------------------------------------

func TestAMissingFileIsTheDefaultRowAlone(t *testing.T) {
	r := load(t, t.TempDir())
	if got := sourceNames(r.List()); len(got) != 1 || got[0] != sources.DefaultName {
		t.Fatalf("list = %v, want just the default row", got)
	}
	if w := r.Warnings(); len(w) != 0 {
		t.Fatalf("a missing file is the first-run state, not a warning: %v", w)
	}
}

func TestAnUnparseableFileIsTheDefaultRowAlone(t *testing.T) {
	cfg := t.TempDir()
	write(t, cfg, "[[source]\nname = \"broken\"\n")
	r := load(t, cfg)
	if got := sourceNames(r.List()); len(got) != 1 || got[0] != sources.DefaultName {
		t.Fatalf("list = %v, want just the default row", got)
	}
	if len(r.Warnings()) == 0 {
		t.Fatal("an unparseable file degrades with a warning")
	}
}

func TestPositionInTheFileIsResolutionOrder(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	skillAt(t, lib, "grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("First", lib); err != nil {
		t.Fatal(err)
	}
	if _, err := r.RegisterDir("Second", lib); err != nil {
		t.Fatal(err)
	}
	if got := sourceNames(r.List()); strings.Join(got, ",") != "First,Second,"+sources.DefaultName {
		t.Fatalf("list = %v", got)
	}

	if err := r.Reorder([]string{"Second", "First"}); err != nil {
		t.Fatal(err)
	}
	if got := sourceNames(load(t, cfg).List()); strings.Join(got, ",") != "Second,First,"+sources.DefaultName {
		t.Fatalf("reordered list = %v", got)
	}
	if err := r.Reorder([]string{"Second"}); !errors.Is(err, sources.ErrBadReorder) {
		t.Fatalf("a partial reorder is refused: %v", err)
	}
	if err := r.Reorder([]string{"Second", sources.DefaultName}); !errors.Is(err, sources.ErrProtected) {
		t.Fatalf("the default row is not reorderable: %v", err)
	}
	if err := r.Remove(sources.DefaultName); !errors.Is(err, sources.ErrProtected) {
		t.Fatalf("the default row is not removable: %v", err)
	}
}

func TestEnabledIsWrittenOnlyWhenFalse(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	r := load(t, cfg)
	if _, err := r.RegisterDir("House skills", lib); err != nil {
		t.Fatal(err)
	}

	path := filepath.Join(cfg, "sources.toml")
	body, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(body), "enabled") {
		t.Fatalf("an enabled row writes no enabled key:\n%s", body)
	}
	info, err := os.Stat(path)
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode().Perm() != 0o600 {
		t.Fatalf("sources.toml is %v, want 0600", info.Mode().Perm())
	}

	if err := r.SetEnabled("House skills", false); err != nil {
		t.Fatal(err)
	}
	body, _ = os.ReadFile(path)
	if !strings.Contains(string(body), "enabled = false") {
		t.Fatalf("a disabled row writes enabled = false:\n%s", body)
	}
	if got := load(t, cfg).List(); got[0].Enabled {
		t.Fatal("disabled did not survive the round trip")
	}
}

// --- rows the load drops ---------------------------------------------------

func TestADuplicateSourceNameInAHandEditedFileKeepsTheFirstRow(t *testing.T) {
	cfg, a, b := t.TempDir(), t.TempDir(), t.TempDir()
	write(t, cfg, `
[[source]]
name = "House"
kind = "dir"
path = "`+a+`"

[[source]]
name = "house"
kind = "dir"
path = "`+b+`"
`)
	r := load(t, cfg)
	list := r.List()
	if len(list) != 2 || list[0].Path != a {
		t.Fatalf("first row wins: %v", sourceNames(list))
	}
	if w := r.Warnings(); len(w) != 1 || !strings.Contains(w[0], "house") {
		t.Fatalf("the dropped row is named in a warning: %v", w)
	}
}

func TestAnUnknownKindDropsOnlyItsOwnRow(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	write(t, cfg, `
[[source]]
name = "Strange"
kind = "svn"
path = "/tmp/whatever"

[[source]]
name = "Pathless"
kind = "dir"

[[source]]
name = "Fine"
kind = "dir"
path = "`+lib+`"
`)
	r := load(t, cfg)
	if got := sourceNames(r.List()); strings.Join(got, ",") != "Fine,"+sources.DefaultName {
		t.Fatalf("the rest of the list stands: %v", got)
	}
	if w := r.Warnings(); len(w) != 2 {
		t.Fatalf("both bad rows warn: %v", w)
	}
}

func TestARowNamedChartrSkillsIsDropped(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	write(t, cfg, `
[[source]]
name = "chartr-skills"
kind = "dir"
path = "`+lib+`"
`)
	r := load(t, cfg)
	if got := sourceNames(r.List()); len(got) != 1 || got[0] != sources.DefaultName {
		t.Fatalf("list = %v, want the synthetic row alone", got)
	}
	if w := r.Warnings(); len(w) != 1 || !strings.Contains(w[0], sources.DefaultName) {
		t.Fatalf("the drop is warned about by name: %v", w)
	}
	if _, err := r.RegisterDir(sources.DefaultName, lib); !errors.Is(err, sources.ErrDuplicateName) {
		t.Fatalf("registering the reserved name is refused: %v", err)
	}
}

func TestADuplicateSourceNameAtRegistrationIsRefused(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	if _, err := r.RegisterDir("  house  ", lib); !errors.Is(err, sources.ErrDuplicateName) {
		t.Fatalf("names are unique case-insensitively: %v", err)
	}
	for _, bad := range []string{"", "with/slash", strings.Repeat("x", 65)} {
		if _, err := r.RegisterDir(bad, lib); !errors.Is(err, sources.ErrBadName) {
			t.Fatalf("%q is not a source name: %v", bad, err)
		}
	}
	if got := len(load(t, cfg).List()); got != 2 {
		t.Fatalf("a refused registration wrote a row: %d rows", got)
	}
}

// --- the walk --------------------------------------------------------------

func TestTheWalkIsBoundedAtDepthThreeAndStopsAtASkill(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	skillAt(t, lib, "one")                   // depth 1
	skillAt(t, lib, "a/two")                 // depth 2
	skillAt(t, lib, "a/b/three")             // depth 3
	skillAt(t, lib, "a/b/c/four")            // depth 4 — out of bounds
	skillAt(t, lib, "one/nested")            // below a skill — its supporting files
	skillAt(t, lib, ".hidden/dot")           // dot-entry
	skillAt(t, lib, "node_modules/vendored") // skipped by name

	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	st, ok := r.Walk("House")
	if !ok {
		t.Fatal("no such source")
	}
	// Sorted walk order, depth first: "a" sorts before "one", so the deeper
	// skills come out first. What matters is that it is deterministic — it is
	// what decides a duplicate basename inside one source.
	if got := strings.Join(names(st.Skills), ","); got != "three,two,one" {
		t.Fatalf("walk found %q", got)
	}
	if st.Status != sources.StatusOK {
		t.Fatalf("status = %q", st.Status)
	}
}

func TestAVanishedDirPathKeepsItsRow(t *testing.T) {
	cfg, lib := t.TempDir(), filepath.Join(t.TempDir(), "gone")
	skillAt(t, lib, "grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	if err := os.RemoveAll(lib); err != nil {
		t.Fatal(err)
	}

	st, _ := r.Walk("House")
	if st.Status != sources.StatusUnavailable || len(st.Skills) != 0 {
		t.Fatalf("status = %q with %d skills, want unavailable and none", st.Status, len(st.Skills))
	}
	if got := sourceNames(load(t, cfg).List()); len(got) != 2 {
		t.Fatalf("the row is never auto-removed: %v", got)
	}
}

func TestASourceYieldingNoSkillsIsEmpty(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	if err := os.MkdirAll(filepath.Join(lib, "docs"), 0o755); err != nil {
		t.Fatal(err)
	}
	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	if st, _ := r.Walk("House"); st.Status != sources.StatusEmpty || len(st.Skills) != 0 {
		t.Fatalf("status = %q with %d skills, want empty and none", st.Status, len(st.Skills))
	}
}

func TestADuplicateBasenameInsideOneSourceIsNamedOnTheRow(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	skillAt(t, lib, "a/grill")
	skillAt(t, lib, "b/grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	st, _ := r.Walk("House")
	if got := names(st.Skills); len(got) != 1 || got[0] != "grill" {
		t.Fatalf("sorted walk order wins: %v", got)
	}
	if st.Skills[0].Dir != filepath.Join(lib, "a", "grill") {
		t.Fatalf("the first in walk order won: %s", st.Skills[0].Dir)
	}
	if len(st.Warnings) != 1 || !strings.Contains(st.Warnings[0], filepath.Join(lib, "b", "grill")) {
		t.Fatalf("the loser is named: %v", st.Warnings)
	}
}

func TestADuplicateSkillNameAcrossSourcesIsShadowedNotAnError(t *testing.T) {
	cfg, high, low := t.TempDir(), t.TempDir(), t.TempDir()
	skillAt(t, high, "grill")
	skillAt(t, low, "grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("High", high); err != nil {
		t.Fatal(err)
	}
	if _, err := r.RegisterDir("Low", low); err != nil {
		t.Fatal(err)
	}

	states := r.States()
	if states[0].Skills[0].Shadowed {
		t.Fatal("the winning source's skill is not shadowed")
	}
	if !states[1].Skills[0].Shadowed {
		t.Fatal("the lower source's skill is marked shadowed")
	}

	got, err := r.Resolve("grill")
	if err != nil {
		t.Fatal(err)
	}
	if got.Source != "High" {
		t.Fatalf("bare resolution takes the first hit, got %q", got.Source)
	}
	got, err = r.Resolve("Low/grill")
	if err != nil {
		t.Fatal(err)
	}
	if got.Source != "Low" {
		t.Fatalf("the shadowed skill stays reachable by qualification, got %q", got.Source)
	}
}

// --- resolution ------------------------------------------------------------

func TestAQualifiedMissNeverFallsThrough(t *testing.T) {
	cfg, empty, full := t.TempDir(), t.TempDir(), t.TempDir()
	skillAt(t, full, "grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("Empty", empty); err != nil {
		t.Fatal(err)
	}
	if _, err := r.RegisterDir("Full", full); err != nil {
		t.Fatal(err)
	}

	if _, err := r.Resolve("Empty/grill"); !errors.Is(err, sources.ErrNotFound) {
		t.Fatalf("a qualified miss is not found, never a fall-through: %v", err)
	}
	if _, err := r.Resolve("Nowhere/grill"); !errors.Is(err, sources.ErrNotFound) {
		t.Fatalf("an unknown source is not found: %v", err)
	}
	if _, err := r.Resolve("grill"); err != nil {
		t.Fatalf("the bare form still finds it: %v", err)
	}
}

func TestADisabledSourceIsSkippedByBothFormsAndSaysSo(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	skillAt(t, lib, "grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	if err := r.SetEnabled("House", false); err != nil {
		t.Fatal(err)
	}

	if _, err := r.Resolve("grill"); !errors.Is(err, sources.ErrNotFound) {
		t.Fatalf("a disabled source is skipped by the bare form: %v", err)
	}
	_, err := r.Resolve("House/grill")
	if !errors.Is(err, sources.ErrNotFound) {
		t.Fatalf("a disabled source is skipped by the qualified form: %v", err)
	}
	if !strings.Contains(err.Error(), "disabled") {
		t.Fatalf("the one failure fixed in a click names itself: %v", err)
	}
}

func TestRemovingASourceLeavesItsFolderAlone(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	skillAt(t, lib, "grill")
	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	if err := r.Remove("house"); err != nil {
		t.Fatal(err)
	}
	if got := sourceNames(load(t, cfg).List()); len(got) != 1 {
		t.Fatalf("list = %v", got)
	}
	if _, err := os.Stat(filepath.Join(lib, "grill", "SKILL.md")); err != nil {
		t.Fatalf("removing a dir source touched the operator's folder: %v", err)
	}
	if err := r.Remove("House"); !errors.Is(err, sources.ErrNotFound) {
		t.Fatalf("removing an unknown source is not found: %v", err)
	}
}

// --- git -------------------------------------------------------------------

// originRepo is a real repository to clone from: one skill, one commit.
func originRepo(t *testing.T) string {
	t.Helper()
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git is not on PATH")
	}
	dir := t.TempDir()
	skillAt(t, dir, "grill")
	for _, args := range [][]string{
		{"init", "--quiet"},
		{"add", "."},
		{"-c", "user.email=t@example.com", "-c", "user.name=t", "commit", "--quiet", "-m", "skills"},
	} {
		cmd := exec.Command("git", args...)
		cmd.Dir = dir
		if out, err := cmd.CombinedOutput(); err != nil {
			t.Fatalf("git %v: %v\n%s", args, err, out)
		}
	}
	return dir
}

func TestGitAbsentFromPathRefusesRegistrationButNotResolution(t *testing.T) {
	cfg, lib := t.TempDir(), t.TempDir()
	skillAt(t, lib, "grill")
	// A checkout already on disk, hand-written as a git row: resolution reads
	// directories and never runs git, so it keeps working with none available.
	write(t, cfg, `
[[source]]
name = "Pinned"
kind = "git"
url = "https://example.com/skills"
path = "`+lib+`"
commit = "abc123def456"
`)
	t.Setenv("PATH", "")

	r := load(t, cfg)
	_, err := r.RegisterGit("New", "https://example.com/other", "main")
	if !errors.Is(err, sources.ErrGitMissing) {
		t.Fatalf("registering a git source is refused at the gate: %v", err)
	}
	if got := len(load(t, cfg).List()); got != 2 {
		t.Fatalf("the refusal wrote a row: %d rows", got)
	}
	if _, err := r.Resolve("Pinned/grill"); err != nil {
		t.Fatalf("an existing checkout keeps resolving: %v", err)
	}
	if _, err := r.Refresh("Pinned"); !errors.Is(err, sources.ErrGitMissing) {
		t.Fatalf("only refresh fails: %v", err)
	}
}

func TestACloneFailingPartwayLeavesNeitherRowNorDirectory(t *testing.T) {
	originRepo(t) // skips when git is unavailable
	cfg := t.TempDir()
	r := load(t, cfg)

	_, err := r.RegisterGit("Broken", filepath.Join(t.TempDir(), "no-such-repo"), "main")
	if err == nil {
		t.Fatal("cloning a repository that is not there should fail")
	}
	if !strings.Contains(err.Error(), "no-such-repo") {
		t.Fatalf("git's own error is reported: %v", err)
	}
	if got := len(load(t, cfg).List()); got != 1 {
		t.Fatalf("a failed clone wrote a row: %d rows", got)
	}
	entries, _ := os.ReadDir(filepath.Join(cfg, "sources"))
	if len(entries) != 0 {
		t.Fatalf("a failed clone left %d directories behind", len(entries))
	}
}

func TestAGitSourceClonesRefreshesAndResolves(t *testing.T) {
	origin := originRepo(t)
	cfg := t.TempDir()
	r := load(t, cfg)

	s, err := r.RegisterGit("Upstream", origin, "")
	if err != nil {
		t.Fatalf("register: %v", err)
	}
	if s.Kind != sources.KindGit || s.Commit == "" || s.Fetched.IsZero() {
		t.Fatalf("the row records the pin: %+v", s)
	}
	if !strings.HasPrefix(s.Path, filepath.Join(cfg, "sources")) {
		t.Fatalf("the checkout lives under the config root: %s", s.Path)
	}
	if _, err := r.Resolve("Upstream/grill"); err != nil {
		t.Fatalf("resolve: %v", err)
	}

	// A second commit upstream, and the explicit refresh that takes it.
	skillAt(t, origin, "research")
	for _, args := range [][]string{{"add", "."}, {"-c", "user.email=t@example.com", "-c", "user.name=t", "commit", "--quiet", "-m", "more"}} {
		cmd := exec.Command("git", args...)
		cmd.Dir = origin
		if out, err := cmd.CombinedOutput(); err != nil {
			t.Fatalf("git %v: %v\n%s", args, err, out)
		}
	}
	if _, err := r.Resolve("Upstream/research"); err == nil {
		t.Fatal("nothing fetches unattended")
	}

	after, err := r.Refresh("Upstream")
	if err != nil {
		t.Fatalf("refresh: %v", err)
	}
	if after.Commit == s.Commit {
		t.Fatal("refresh records the new short sha")
	}
	if _, err := r.Resolve("Upstream/research"); err != nil {
		t.Fatalf("the refreshed checkout resolves: %v", err)
	}
	if got := load(t, cfg).List()[0].Commit; got != after.Commit {
		t.Fatalf("the pin survived the round trip: %q", got)
	}

	if err := r.Remove("Upstream"); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(after.Path); !os.IsNotExist(err) {
		t.Fatalf("chartr's own checkout goes with the row: %v", err)
	}
}

// A path typed the way it is typed in a shell means the same thing here. Without
// the expansion a leading `~` reaches filepath.Abs as a relative segment and is
// joined onto chartr's working directory, producing a row that points at nothing
// and reads `unavailable` for a reason the operator cannot see.
func TestRegisteringATildePath(t *testing.T) {
	home := t.TempDir()
	t.Setenv("HOME", home)
	if runtime.GOOS == "windows" {
		t.Setenv("USERPROFILE", home)
	}

	cfg := t.TempDir()
	r := load(t, cfg)

	if err := os.MkdirAll(filepath.Join(home, "skills", "grill"), 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(home, "skills", "grill", "SKILL.md"),
		[]byte("---\nname: grill\n---\n"), 0o600); err != nil {
		t.Fatal(err)
	}

	s, err := r.RegisterDir("Mine", "~/skills")
	if err != nil {
		t.Fatal(err)
	}
	if want := filepath.Join(home, "skills"); s.Path != want {
		t.Fatalf("~/skills resolved to %q, want %q", s.Path, want)
	}
	// And it is the expanded path that persists — the file is read by chartr, not
	// by a shell, so a stored `~` would have to be re-expanded on every load.
	body, err := os.ReadFile(filepath.Join(cfg, "sources.toml"))
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(body), "~") {
		t.Fatalf("sources.toml stored an unexpanded path:\n%s", body)
	}
	if st, ok := r.Walk("Mine"); !ok || st.Status != sources.StatusOK || len(st.Skills) != 1 {
		t.Fatalf("the registered path does not walk: %+v", st)
	}

	// A bare `~` is the home directory itself; a `~` anywhere else is an ordinary
	// directory name and is left alone.
	s, err = r.RegisterDir("Home", "~")
	if err != nil {
		t.Fatal(err)
	}
	if s.Path != home {
		t.Fatalf("~ resolved to %q, want %q", s.Path, home)
	}
}
