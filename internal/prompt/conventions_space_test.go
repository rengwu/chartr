package prompt_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
)

// The write contract's per-space copy (chartr's file-format contract moved out
// of the operator's config root and into `.chartr/TRACKER-CONVENTION.md`, so a
// session sandboxed to its own space can read it — see conventions.go).

func TestReconcileSpaceConventionsWritesTheCanonicalBytes(t *testing.T) {
	space := t.TempDir()

	if err := prompt.ReconcileSpaceConventions(space); err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	path := filepath.Join(space, filepath.FromSlash(prompt.ConventionsRelPath))
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("reading the materialized contract: %v", err)
	}
	if string(b) != prompt.Conventions() {
		t.Error("materialized conventions differ from the embedded bytes")
	}
}

func TestReconcileSpaceConventionsRestoresAnEditedFile(t *testing.T) {
	space := t.TempDir()
	if err := prompt.ReconcileSpaceConventions(space); err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	path := filepath.Join(space, filepath.FromSlash(prompt.ConventionsRelPath))
	if err := os.WriteFile(path, []byte("the operator's own rules\n"), 0o600); err != nil {
		t.Fatalf("edit conventions: %v", err)
	}
	if err := prompt.ReconcileSpaceConventions(space); err != nil {
		t.Fatalf("reconcile after edit: %v", err)
	}

	b, _ := os.ReadFile(path)
	if string(b) != prompt.Conventions() {
		t.Error("an edited conventions file was not restored to the canonical bytes")
	}
}

// The file lands inside `.chartr/`, marked git-ignored by a `*` marker written
// at that directory's root — the same self-contained device the skill mirror
// and run directory use one level down — so a repository the operator never
// edited its own `.gitignore` for still leaves this file (and the marker
// itself) untracked.
func TestReconcileSpaceConventionsSelfIgnores(t *testing.T) {
	space := t.TempDir()
	if err := prompt.ReconcileSpaceConventions(space); err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	ignore := filepath.Join(space, ".chartr", ".gitignore")
	b, err := os.ReadFile(ignore)
	if err != nil {
		t.Fatalf("reading the ignore marker: %v", err)
	}
	if string(b) != "*\n" {
		t.Errorf("ignore marker = %q, want %q", b, "*\n")
	}
}

// An unchanged file must not have its mtime touched, the same guarantee the
// config-root reconcile and the standing document give: a clean tree stays
// clean and no watch fires on chartr's own write.
func TestReconcileSpaceConventionsLeavesAnUnchangedFileAlone(t *testing.T) {
	space := t.TempDir()
	if err := prompt.ReconcileSpaceConventions(space); err != nil {
		t.Fatalf("reconcile: %v", err)
	}
	path := filepath.Join(space, filepath.FromSlash(prompt.ConventionsRelPath))
	before, err := os.Stat(path)
	if err != nil {
		t.Fatal(err)
	}

	if err := prompt.ReconcileSpaceConventions(space); err != nil {
		t.Fatalf("second reconcile: %v", err)
	}
	after, err := os.Stat(path)
	if err != nil {
		t.Fatal(err)
	}
	if !after.ModTime().Equal(before.ModTime()) {
		t.Error("an unchanged conventions file was rewritten; its mtime moved")
	}
}

func TestReconcileSpaceConventionsSkipsAnEmptyDirectory(t *testing.T) {
	if err := prompt.ReconcileSpaceConventions(""); err != nil {
		t.Errorf("reconcile with no space directory: %v", err)
	}
}
