package sources_test

import (
	"os"
	"path/filepath"
	"testing"
)

// The mirror is the repo-local copy a sandboxed agent reads (ADR 0018). These
// drive it through the registry the way a spawn does: register real folders,
// mirror them into a destination, and assert on what landed on disk.

func TestMirrorCopiesEnabledSourcesInResolutionLayout(t *testing.T) {
	cfg := t.TempDir()
	libA, libB := t.TempDir(), t.TempDir()
	skillAt(t, libA, "grill")
	skillAt(t, libA, "research")
	skillAt(t, libB, "implement")

	r := load(t, cfg)
	if _, err := r.RegisterDir("House", libA); err != nil {
		t.Fatal(err)
	}
	if _, err := r.RegisterDir("Team", libB); err != nil {
		t.Fatal(err)
	}

	dest := filepath.Join(t.TempDir(), ".chartr", "skills")
	got, err := r.Mirror(dest)
	if err != nil {
		t.Fatalf("mirror: %v", err)
	}
	if len(got) != 3 {
		t.Fatalf("mirrored %d skills, want 3: %+v", len(got), got)
	}
	// Laid out as <source>/<skill>/, and the returned Dir points at it.
	for _, rel := range []string{"House/grill", "House/research", "Team/implement"} {
		if _, err := os.Stat(filepath.Join(dest, filepath.FromSlash(rel), "SKILL.md")); err != nil {
			t.Errorf("mirror is missing %s: %v", rel, err)
		}
	}
	for _, sk := range got {
		if _, err := os.Stat(filepath.Join(sk.Dir, "SKILL.md")); err != nil {
			t.Errorf("returned Dir %q does not hold the skill: %v", sk.Dir, err)
		}
	}
	// A self-contained ignore keeps the whole tree out of git.
	if b, err := os.ReadFile(filepath.Join(dest, ".gitignore")); err != nil || string(b) != "*\n" {
		t.Errorf("mirror ignore = %q (%v), want %q", b, err, "*\n")
	}
}

func TestMirrorSkipsDisabledSourcesAndSymlinks(t *testing.T) {
	cfg := t.TempDir()
	on, off := t.TempDir(), t.TempDir()
	grill := skillAt(t, on, "grill")
	skillAt(t, off, "hidden")

	// A supporting file is copied; a symlink — even one pointing inside — is not,
	// which is the minimal, safe handling the mirror settles on.
	if err := os.WriteFile(filepath.Join(grill, "ref.md"), []byte("supporting"), 0o644); err != nil {
		t.Fatal(err)
	}
	outside := filepath.Join(t.TempDir(), "secret.txt")
	if err := os.WriteFile(outside, []byte("secret"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.Symlink(outside, filepath.Join(grill, "escape.txt")); err != nil {
		t.Skipf("symlinks unsupported here: %v", err)
	}

	r := load(t, cfg)
	if _, err := r.RegisterDir("On", on); err != nil {
		t.Fatal(err)
	}
	if _, err := r.RegisterDir("Off", off); err != nil {
		t.Fatal(err)
	}
	if err := r.SetEnabled("Off", false); err != nil {
		t.Fatal(err)
	}

	dest := filepath.Join(t.TempDir(), "mirror")
	if _, err := r.Mirror(dest); err != nil {
		t.Fatalf("mirror: %v", err)
	}

	if _, err := os.Stat(filepath.Join(dest, "Off")); !os.IsNotExist(err) {
		t.Error("a disabled source was mirrored")
	}
	if _, err := os.Stat(filepath.Join(dest, "On", "grill", "ref.md")); err != nil {
		t.Errorf("a supporting file was not copied: %v", err)
	}
	if _, err := os.Lstat(filepath.Join(dest, "On", "grill", "escape.txt")); !os.IsNotExist(err) {
		t.Error("a symlink was copied into the mirror")
	}
}

func TestMirrorReconcilesInPlaceAndPrunes(t *testing.T) {
	cfg := t.TempDir()
	lib := t.TempDir()
	skillAt(t, lib, "grill")
	skillAt(t, lib, "research")

	r := load(t, cfg)
	if _, err := r.RegisterDir("House", lib); err != nil {
		t.Fatal(err)
	}
	dest := filepath.Join(t.TempDir(), "mirror")
	if _, err := r.Mirror(dest); err != nil {
		t.Fatalf("first mirror: %v", err)
	}

	// A skill removed upstream is pruned from the mirror on the next reconcile,
	// while the survivor stays put.
	if err := os.RemoveAll(filepath.Join(lib, "research")); err != nil {
		t.Fatal(err)
	}
	if _, err := r.Mirror(dest); err != nil {
		t.Fatalf("second mirror: %v", err)
	}
	if _, err := os.Stat(filepath.Join(dest, "House", "research")); !os.IsNotExist(err) {
		t.Error("a skill removed upstream survived in the mirror")
	}
	if _, err := os.Stat(filepath.Join(dest, "House", "grill", "SKILL.md")); err != nil {
		t.Errorf("the surviving skill was lost: %v", err)
	}
}
