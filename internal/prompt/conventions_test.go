package prompt_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
)

// The config root's one contract file: the operator's preferences, which
// chartr creates once and never writes again (skill-sources ticket 03). The
// conventions half of what used to be a two-file contract here now lives
// per-space instead — see conventions_space_test.go.

func TestAFreshConfigRootGetsThePreferencesFile(t *testing.T) {
	dir := t.TempDir()

	got, err := prompt.ReconcileContract(dir)
	if err != nil {
		t.Fatalf("reconcile: %v", err)
	}

	if got.Preferences != "" {
		t.Errorf("fresh preferences = %q, want empty", got.Preferences)
	}
	if _, err := os.Stat(prompt.PreferencesPath(dir)); err != nil {
		t.Fatalf("preferences.md not created: %v", err)
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

// A payload names the contract by a path relative to the space it will run in,
// not an absolute one under the operator's config root — the whole point of
// moving it out of ConfigDir (see conventions_space_test.go): a session
// sandboxed to its own working tree can resolve a relative path but not one
// that leaves the repo.
func TestComposedPayloadPointsAtTheRelativeConventionsPath(t *testing.T) {
	dir, reg, bindings := fixture(t)

	p, err := prompt.Compose(prompt.ComposeInput{Role: "grill", ConfigDir: dir, Sources: reg, Bindings: bindings})
	if err != nil {
		t.Fatalf("compose: %v", err)
	}

	var text string
	for _, part := range p.Parts {
		if part.Name == "conventions" {
			text = part.Text
		}
	}
	if text == "" {
		t.Fatal("no conventions part in the composed payload")
	}
	if !strings.Contains(text, "`"+prompt.ConventionsRelPath+"`") {
		t.Errorf("conventions sentence = %q, want it to name %q", text, prompt.ConventionsRelPath)
	}
	if strings.Contains(text, dir) {
		t.Errorf("conventions sentence names the config root:\n%s", text)
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

// ConventionsRelPath is what every payload and every space's materialized copy
// have to agree on: the same path, relative, under `.chartr/`.
func TestConventionsRelPathIsUnderTheSpaceLocalDirectory(t *testing.T) {
	if want := filepath.ToSlash(prompt.ConventionsRelPath); want != ".chartr/TRACKER-CONVENTION.md" {
		t.Errorf("ConventionsRelPath = %q, want .chartr/TRACKER-CONVENTION.md", want)
	}
}
