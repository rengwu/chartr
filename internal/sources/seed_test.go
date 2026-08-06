package sources_test

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/sources"
)

// The seed at the package seam: the two states its directory can be in, the
// wholesale reconcile between them, and the one-way conversion a refresh makes.
// Everything here is driven against a temp config root through the public
// surface, with the filesystem as the assertion — which is the point, since the
// seed deliberately records nothing about itself anywhere else.

func seedFile(t *testing.T, cfg, rel string) string {
	t.Helper()
	b, err := os.ReadFile(filepath.Join(sources.DefaultPath(cfg), rel))
	if err != nil {
		t.Fatalf("reading %s out of the seeded directory: %v", rel, err)
	}
	return string(b)
}

// A fresh config root materializes the whole vendored copy, and a second startup
// over the same root changes nothing — the compare is on the file set and bytes,
// so an unchanged directory is not rewritten.
func TestReconcileMaterializesTheSeed(t *testing.T) {
	cfg := t.TempDir()
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("first reconcile: %v", err)
	}

	names := sources.SeedSkillNames()
	if len(names) == 0 {
		t.Fatal("the embedded seed carries no skills")
	}
	r, err := sources.Load(cfg)
	if err != nil {
		t.Fatalf("load: %v", err)
	}
	st, ok := r.Walk(sources.DefaultName)
	if !ok {
		t.Fatal("the default source is missing from the list")
	}
	if st.Status != sources.StatusOK || len(st.Skills) != len(names) {
		t.Fatalf("walked the seeded default = %s with %d skills, want ok with %d", st.Status, len(st.Skills), len(names))
	}
	// The four role skills are what a first run cannot spawn without.
	for _, want := range []string{"grill", "prototype", "research", "implement"} {
		if _, err := r.Resolve(sources.DefaultBinding(want)); err != nil {
			t.Errorf("a fresh install cannot resolve %s: %v", sources.DefaultBinding(want), err)
		}
	}

	before := seedFile(t, cfg, filepath.Join("grill", "SKILL.md"))
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("second reconcile: %v", err)
	}
	if got := seedFile(t, cfg, filepath.Join("grill", "SKILL.md")); got != before {
		t.Error("a second reconcile over an unchanged directory rewrote it")
	}
}

// The replacement is wholesale, not per-file: an edited skill is restored, and a
// file that is not the seed's is gone. Deleting the whole directory is therefore
// the reset — it is the same code path as any other difference.
func TestReconcileReplacesWholesale(t *testing.T) {
	cfg := t.TempDir()
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile: %v", err)
	}
	dir := sources.DefaultPath(cfg)
	shipped := seedFile(t, cfg, filepath.Join("grill", "SKILL.md"))

	if err := os.WriteFile(filepath.Join(dir, "grill", "SKILL.md"), []byte("MY OWN GRILL"), 0o600); err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(filepath.Join(dir, "invented"), 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "invented", "SKILL.md"), []byte("not the seed's"), 0o600); err != nil {
		t.Fatal(err)
	}

	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile after an edit: %v", err)
	}
	if got := seedFile(t, cfg, filepath.Join("grill", "SKILL.md")); got != shipped {
		t.Error("an edited seeded skill was not restored")
	}
	if _, err := os.Stat(filepath.Join(dir, "invented")); err == nil {
		t.Error("a directory the seed does not carry survived the reconcile")
	}

	// Deleting it is the reset, and it needs no action anywhere.
	if err := os.RemoveAll(dir); err != nil {
		t.Fatal(err)
	}
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile after a delete: %v", err)
	}
	if got := seedFile(t, cfg, filepath.Join("grill", "SKILL.md")); got != shipped {
		t.Error("deleting the directory did not re-materialize the seed")
	}
}

// A `.git` inside the directory means the operator owns the bytes, and chartr
// never writes there again — an upgrade must not silently revert an ownership
// they asserted with a fetch.
func TestReconcileNeverOverwritesAPinnedCheckout(t *testing.T) {
	cfg := t.TempDir()
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile: %v", err)
	}
	dir := sources.DefaultPath(cfg)
	if err := os.MkdirAll(filepath.Join(dir, ".git"), 0o700); err != nil {
		t.Fatal(err)
	}
	mine := filepath.Join(dir, "grill", "SKILL.md")
	if err := os.WriteFile(mine, []byte("MY OWN GRILL"), 0o600); err != nil {
		t.Fatal(err)
	}

	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile over a pinned checkout: %v", err)
	}
	if b, _ := os.ReadFile(mine); string(b) != "MY OWN GRILL" {
		t.Errorf("chartr wrote into a pinned checkout: %q", b)
	}
	if !sources.Pinned(dir) {
		t.Error("a directory with a .git does not read as pinned")
	}
}

// Refreshing the default source is the seeded→pinned conversion, and it happens
// exactly once: the first refresh clones the upstream over chartr's bytes, the
// `.git` it leaves stops the reconcile, and the commit and timestamp it records
// are what the settings row reads instead of "shipped with this build".
func TestRefreshConvertsTheSeedToAPin(t *testing.T) {
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git is not on PATH")
	}
	// A local repository stands in for the upstream: a clone from a path is the
	// same code path as a clone from a URL, and it keeps the suite off the network.
	upstream := t.TempDir()
	dir := skillAt(t, upstream, "grill")
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte("---\nname: grill\n---\n\nUPSTREAM GRILL\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	for _, args := range [][]string{
		{"init", "-b", "main"}, {"add", "-A"},
		{"-c", "user.email=t@t", "-c", "user.name=t", "commit", "-m", "skills"},
	} {
		cmd := exec.Command("git", args...)
		cmd.Dir = upstream
		if out, err := cmd.CombinedOutput(); err != nil {
			t.Fatalf("preparing the upstream repo (%v): %v\n%s", args, err, out)
		}
	}
	t.Cleanup(func(was string) func() { return func() { sources.SeedURL = was } }(sources.SeedURL))
	sources.SeedURL = upstream

	cfg := t.TempDir()
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile: %v", err)
	}
	r, err := sources.Load(cfg)
	if err != nil {
		t.Fatalf("load: %v", err)
	}

	got, err := r.Refresh(sources.DefaultName)
	if err != nil {
		t.Fatalf("refreshing the seeded default: %v", err)
	}
	if got.Commit == "" || got.Fetched.IsZero() {
		t.Errorf("a refreshed default row records no commit or timestamp: %+v", got)
	}
	if !sources.Pinned(sources.DefaultPath(cfg)) {
		t.Fatal("a refreshed default source is not pinned")
	}
	if b := seedFile(t, cfg, filepath.Join("grill", "SKILL.md")); !strings.Contains(b, "UPSTREAM GRILL") {
		t.Errorf("the upstream bytes did not land: %q", b)
	}

	// The two scalars survive a reload — they are what the row reads at load,
	// without inspecting the filesystem.
	again, err := sources.Load(cfg)
	if err != nil {
		t.Fatalf("reload: %v", err)
	}
	reloaded, _ := again.Get(sources.DefaultName)
	if reloaded.Commit != got.Commit || reloaded.Fetched.IsZero() {
		t.Errorf("the default row after reload = %+v, want commit %s and a timestamp", reloaded, got.Commit)
	}

	// And the seed is dead to it: a reconcile leaves the operator's checkout alone.
	if err := sources.Reconcile(cfg); err != nil {
		t.Fatalf("reconcile after the pin: %v", err)
	}
	if b := seedFile(t, cfg, filepath.Join("grill", "SKILL.md")); !strings.Contains(b, "UPSTREAM GRILL") {
		t.Errorf("the reconcile reverted a pinned checkout: %q", b)
	}
}
