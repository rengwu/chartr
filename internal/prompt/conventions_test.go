package prompt_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
)

// The config root's two contract files (skill-sources ticket 03): the generated
// conventions, which chartr owns and restores, and the operator's preferences,
// which chartr creates once and never writes again.

func TestAFreshConfigRootGetsBothContractFiles(t *testing.T) {
	dir := t.TempDir()

	got, err := prompt.ReconcileContract(dir)
	if err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	if got.ConventionsPath != filepath.Join(dir, "conventions.md") {
		t.Errorf("conventions path = %q", got.ConventionsPath)
	}
	if got.Preferences != "" {
		t.Errorf("fresh preferences = %q, want empty", got.Preferences)
	}
	b, err := os.ReadFile(got.ConventionsPath)
	if err != nil {
		t.Fatalf("read conventions: %v", err)
	}
	if string(b) != prompt.Conventions() {
		t.Error("materialized conventions differ from the embedded bytes")
	}
	if _, err := os.Stat(prompt.PreferencesPath(dir)); err != nil {
		t.Fatalf("preferences.md not created: %v", err)
	}
}

func TestAnEditedConventionsFileIsRestored(t *testing.T) {
	dir := t.TempDir()
	if _, err := prompt.ReconcileContract(dir); err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	path := prompt.ConventionsPath(dir)
	if err := os.WriteFile(path, []byte("the operator's own rules\n"), 0o600); err != nil {
		t.Fatalf("edit conventions: %v", err)
	}
	if _, err := prompt.ReconcileContract(dir); err != nil {
		t.Fatalf("reconcile after edit: %v", err)
	}

	b, _ := os.ReadFile(path)
	if string(b) != prompt.Conventions() {
		t.Error("an edited conventions file was not restored to the canonical bytes")
	}
}

// The conventions file is a parser contract, so composing anything restores it —
// not only a restart. This is the guarantee that makes an upgrade take effect in
// a running cockpit.
func TestComposingRestoresTheConventionsFile(t *testing.T) {
	dir, reg, bindings := fixture(t)

	path := prompt.ConventionsPath(dir)
	if err := os.WriteFile(path, []byte("truncated"), 0o600); err != nil {
		t.Fatalf("seed edited conventions: %v", err)
	}

	if _, err := prompt.Compose(prompt.ComposeInput{Role: "grill", ConfigDir: dir, Sources: reg, Bindings: bindings}); err != nil {
		t.Fatalf("compose: %v", err)
	}
	b, _ := os.ReadFile(path)
	if string(b) != prompt.Conventions() {
		t.Error("composing did not restore the conventions file")
	}
}

func TestPreferencesAreReadVerbatimAndNeverRewritten(t *testing.T) {
	dir := t.TempDir()
	if _, err := prompt.ReconcileContract(dir); err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	const own = "Always answer in Dutch.\n"
	if err := os.WriteFile(prompt.PreferencesPath(dir), []byte(own), 0o600); err != nil {
		t.Fatalf("write preferences: %v", err)
	}

	got, err := prompt.ReconcileContract(dir)
	if err != nil {
		t.Fatalf("reconcile after write: %v", err)
	}
	if got.Preferences != own {
		t.Errorf("preferences = %q, want %q", got.Preferences, own)
	}
	b, _ := os.ReadFile(prompt.PreferencesPath(dir))
	if string(b) != own {
		t.Error("preferences.md was rewritten; it is the operator's file")
	}
}

func TestADeletedPreferencesFileIsRecreatedEmpty(t *testing.T) {
	dir := t.TempDir()
	if _, err := prompt.ReconcileContract(dir); err != nil {
		t.Fatalf("reconcile: %v", err)
	}
	if err := os.Remove(prompt.PreferencesPath(dir)); err != nil {
		t.Fatalf("remove preferences: %v", err)
	}

	got, err := prompt.ReconcileContract(dir)
	if err != nil {
		t.Fatalf("reconcile after delete: %v", err)
	}
	if got.Preferences != "" {
		t.Errorf("recreated preferences = %q, want empty", got.Preferences)
	}
	if _, err := os.Stat(prompt.PreferencesPath(dir)); err != nil {
		t.Fatalf("preferences.md not recreated: %v", err)
	}
}

// The one failure that must never be silent: preferences exist but cannot be
// read. Composing without them would drop the operator's own instructions.
func TestAnUnreadablePreferencesFileFailsComposition(t *testing.T) {
	if os.Geteuid() == 0 {
		t.Skip("root reads unreadable files")
	}
	dir, reg, bindings := fixture(t)
	if _, err := prompt.ReconcileContract(dir); err != nil {
		t.Fatalf("reconcile: %v", err)
	}
	if err := os.Chmod(prompt.PreferencesPath(dir), 0o000); err != nil {
		t.Fatalf("chmod preferences: %v", err)
	}
	t.Cleanup(func() { _ = os.Chmod(prompt.PreferencesPath(dir), 0o600) })

	_, err := prompt.Compose(prompt.ComposeInput{Role: "grill", ConfigDir: dir, Sources: reg, Bindings: bindings})
	if err == nil {
		t.Fatal("compose succeeded with an unreadable preferences.md")
	}
	if !strings.Contains(err.Error(), "preferences.md") {
		t.Errorf("error %q does not name the file the operator must fix", err)
	}
}

// The document states the rules chartr's own code keeps. These are the two the
// parser enforces, so a drift between the text and the code shows up here.
func TestTheConventionsStateTheRulesTheParserKeeps(t *testing.T) {
	text := prompt.Conventions()
	for _, want := range []string{
		".plan/maps/",
		"`status` is forbidden",
		"## Done when",
		"## Ruled out",
		"frontier",
	} {
		if !strings.Contains(text, want) {
			t.Errorf("conventions do not mention %q", want)
		}
	}
}
