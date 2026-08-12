package prompt_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
)

// The two embedded cores an operator may shadow with a file in the config
// root: `core-ticket.md` and `core-space.md`. Absent → the binary's embedded
// default composes unchanged (the golden tests cover that); present → the
// operator's bytes compose in their place, and chartr never writes over the
// file. The write contract is deliberately not shadowable — it is a parser
// contract (see ReconcileSpaceConventions).

func composeTicket(t *testing.T, dir string) prompt.Payload {
	t.Helper()
	_, reg, bindings := fixture(t)
	p, err := prompt.Compose(prompt.ComposeInput{
		Role:      "grill",
		ConfigDir: dir,
		Sources:   reg,
		Bindings:  bindings,
		Bundle:    prompt.Bundle{MapName: "widget", TicketNum: 1, TicketTitle: "First"},
	})
	if err != nil {
		t.Fatalf("Compose: %v", err)
	}
	return p
}

func TestCoreTicketOverrideComposesInPlaceOfTheEmbeddedCore(t *testing.T) {
	dir := t.TempDir()
	body := "---\nname: core\ndescription: my own core\n---\n\nOPERATOR-CORE-BODY\n"
	if err := os.WriteFile(filepath.Join(dir, prompt.CoreTicketOverrideFile), []byte(body), 0o600); err != nil {
		t.Fatal(err)
	}

	core := findPromptPart(t, composeTicket(t, dir), "core")

	if !strings.Contains(core.Text, "OPERATOR-CORE-BODY") {
		t.Errorf("the override did not compose as the core:\n%s", core.Text)
	}
	if strings.Contains(core.Text, "one agent session") {
		t.Error("the embedded core composed even though an override was present")
	}
	// An overridden core is the operator's bytes, badged and recorded as such —
	// the claim trailer reads `core=operator`, not `core=chartr`.
	if core.Origin != prompt.OriginOperator {
		t.Errorf("overridden core origin = %q, want %q", core.Origin, prompt.OriginOperator)
	}
	// Frontmatter is stripped exactly as it is for the embedded asset.
	if strings.Contains(core.Text, "description:") {
		t.Errorf("frontmatter leaked from the override into the body:\n%s", core.Text)
	}
}

func TestNoCoreOverrideLeavesTheEmbeddedCoreAndItsChartrOrigin(t *testing.T) {
	dir := t.TempDir()
	core := findPromptPart(t, composeTicket(t, dir), "core")
	if !strings.Contains(core.Text, "one agent session") {
		t.Errorf("the embedded core did not compose:\n%s", core.Text)
	}
	if core.Origin != prompt.OriginChartr {
		t.Errorf("un-overridden core origin = %q, want %q", core.Origin, prompt.OriginChartr)
	}
}

func TestCoreSpaceOverrideComposesIntoTheStandingDocument(t *testing.T) {
	dir, reg, _ := fixture(t)
	if err := os.WriteFile(filepath.Join(dir, prompt.CoreSpaceOverrideFile), []byte("OPERATOR-SPACE-CORE\n"), 0o600); err != nil {
		t.Fatal(err)
	}

	p, err := prompt.ComposeStanding(dir, reg)
	if err != nil {
		t.Fatalf("ComposeStanding: %v", err)
	}
	core := findPromptPart(t, p, "core")
	if !strings.Contains(core.Text, "OPERATOR-SPACE-CORE") || core.Origin != prompt.OriginOperator {
		t.Errorf("space core override did not compose as operator bytes: %q/%s", core.Origin, core.Text)
	}
}

// An override file that exists but cannot be read fails the compose rather than
// silently falling back to the shipped bytes — the same rule preferences already
// follow. A directory in the file's place is the portable way to make the read
// fail.
func TestUnreadableOverrideFailsTheCompose(t *testing.T) {
	dir := t.TempDir()
	if err := os.Mkdir(filepath.Join(dir, prompt.CoreTicketOverrideFile), 0o755); err != nil {
		t.Fatal(err)
	}
	_, reg, bindings := fixture(t)
	if _, err := prompt.Compose(prompt.ComposeInput{
		Role: "grill", ConfigDir: dir, Sources: reg, Bindings: bindings,
	}); err == nil {
		t.Error("Compose ignored an unreadable core override instead of failing")
	}
}

func findPromptPart(t *testing.T, p prompt.Payload, name string) prompt.Part {
	t.Helper()
	for _, part := range p.Parts {
		if part.Name == name {
			return part
		}
	}
	t.Fatalf("no part named %q in %v", name, p.Parts)
	return prompt.Part{}
}
