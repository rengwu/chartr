package prompt_test

import (
	"os"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/prompts"
)

// The space's selected presets in a ticket payload (prompt-presets ticket 02):
// each one its own operator prompt part, in the order it was handed over, after
// the operator's preferences and before the context region. One composition, so
// the preview, the spawn, and the payload hash all see the same bytes.

func selected() []prompts.Prompt {
	return []prompts.Prompt{
		{ID: "keep-it-brief", Name: "Keep it brief", Body: "BRIEF-MARKER"},
		{ID: "commit-convention", Name: "Commit convention", Body: "COMMITS-MARKER"},
	}
}

func TestSelectedPresetsComposeBetweenPreferencesAndContext(t *testing.T) {
	dir, reg, bindings := fixture(t)
	if err := os.WriteFile(prompt.PreferencesPath(dir), []byte("PREFERENCES-MARKER\n"), 0o600); err != nil {
		t.Fatal(err)
	}

	p, err := prompt.Compose(prompt.ComposeInput{
		Role:      "grill",
		ConfigDir: dir,
		Sources:   reg,
		Bindings:  bindings,
		Prompts:   selected(),
		Bundle: prompt.Bundle{
			MapName: "Widget", MapBody: "THE-MAP-BODY",
			TicketNum: 2, TicketTitle: "Dependent work", TicketBody: "THE-TICKET-BODY",
		},
	})
	if err != nil {
		t.Fatalf("Compose: %v", err)
	}

	want := []string{
		"core", "grill", "conventions", "preferences",
		"preset keep-it-brief", "preset commit-convention",
		"sources", "map", "ticket",
	}
	var got []string
	for _, part := range p.Parts {
		got = append(got, part.Name)
	}
	if strings.Join(got, ",") != strings.Join(want, ",") {
		t.Fatalf("parts = %v, want %v", got, want)
	}

	// Identity and provenance: the id names the part, the operator's own name for
	// it labels it, and it badges as theirs — the presets are operator text.
	for i, part := range p.Parts[4:6] {
		if part.Kind != "prompt" || part.Origin != prompt.OriginOperator {
			t.Errorf("preset part %d = %s/%s, want prompt/%s", i, part.Kind, part.Origin, prompt.OriginOperator)
		}
		if part.Label != selected()[i].Name || part.Text != selected()[i].Body {
			t.Errorf("preset part %d = label %q text %q, want %q / %q",
				i, part.Label, part.Text, selected()[i].Name, selected()[i].Body)
		}
	}

	// And in the document itself, in that order and in that region.
	md := p.Markdown
	prefs, brief := strings.Index(md, "PREFERENCES-MARKER"), strings.Index(md, "BRIEF-MARKER")
	commits, ctx := strings.Index(md, "COMMITS-MARKER"), strings.Index(md, "# Context")
	if !(prefs < brief && brief < commits && commits < ctx) || prefs < 0 || ctx < 0 {
		t.Errorf("preset placement in the document is prefs=%d brief=%d commits=%d context=%d, want that order",
			prefs, brief, commits, ctx)
	}
}

// No selection composes exactly what it composed before the feature existed: the
// empty case adds no part and no byte. (The golden payloads assert the rest.)
func TestNoSelectionComposesNothingExtra(t *testing.T) {
	dir, reg, bindings := fixture(t)
	in := prompt.ComposeInput{
		Role: "grill", ConfigDir: dir, Sources: reg, Bindings: bindings,
		Bundle: prompt.Bundle{MapName: "Widget", TicketNum: 1, TicketTitle: "A", TicketBody: "B"},
	}
	bare, err := prompt.Compose(in)
	if err != nil {
		t.Fatalf("Compose: %v", err)
	}
	in.Prompts = nil
	empty, err := prompt.Compose(in)
	if err != nil {
		t.Fatalf("Compose: %v", err)
	}
	if len(bare.Parts) != len(empty.Parts) || bare.Markdown != empty.Markdown {
		t.Errorf("an empty selection changed the payload")
	}
	for _, part := range bare.Parts {
		if strings.HasPrefix(part.Name, "preset ") {
			t.Errorf("an empty selection composed %q", part.Name)
		}
	}
}

// The free session's document is the presets and nothing else — the same parts
// through the same renderer, so a preset reads identically wherever it lands.
func TestComposePresets(t *testing.T) {
	md := prompt.ComposePresets(selected())
	if md != "BRIEF-MARKER\n\nCOMMITS-MARKER\n" {
		t.Errorf("free preset document = %q", md)
	}
	if got := prompt.ComposePresets(nil); got != "" {
		t.Errorf("no presets = %q, want the empty document that keeps a launch bare", got)
	}
}
