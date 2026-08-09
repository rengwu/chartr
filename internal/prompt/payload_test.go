package prompt_test

import (
	"flag"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/sources"
)

// The two payloads, whole (ticket 07). Everything else in this package's tests
// asserts one clause at a time; these two assert the *document*, byte for byte,
// against a golden file — because the property that matters most about the free
// payload cannot be spot-checked. chartr's own voice in it is four sentences, and
// nothing but a diff would notice a fifth: the restraint is that every sentence
// chartr writes must still be true if the agent does nothing about it, and the
// next addition will almost always arrive phrased as a *should*. The golden is
// the review.

var update = flag.Bool("update", false, "rewrite the payload golden files")

// grillMarker is the identifiable body line of the test grill skill — used to
// prove the skill's bytes themselves travelled into the payload.
const grillMarker = "GRILL-BODY-MARKER"

// fixture builds a real config root and a registered `dir` source of skills the
// payloads are composed against. chartr ships none of its own (ADR 0018), so the
// test provides the source: the goldens carry that source's inventory, not a
// shipped one.
func fixture(t *testing.T) (string, *sources.Registry, config.RoleBindings) {
	t.Helper()
	dir := t.TempDir()
	skills := t.TempDir()
	writeSkill(t, skills, "grill", grillMarker)
	reg, err := sources.Load(dir)
	if err != nil {
		t.Fatalf("loading sources: %v", err)
	}
	if _, err := reg.RegisterDir("house", skills); err != nil {
		t.Fatalf("registering the skills source: %v", err)
	}
	return dir, reg, config.RoleBindings{"grill": "house/grill"}
}

// writeSkill lays down a minimal skill directory: rel below root, a SKILL.md with
// frontmatter and an identifiable body line.
func writeSkill(t *testing.T, root, name, body string) {
	t.Helper()
	dir := filepath.Join(root, name)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	src := "---\nname: " + name + "\ndescription: the " + name + " skill\n---\n\n" + body + "\n"
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte(src), 0o644); err != nil {
		t.Fatal(err)
	}
}

// The ticket payload: chartr's core, the bound role skill's body concatenated
// whole, the conventions pointer, the operator's preferences, then the context
// region — sources, map, ticket, blockers. Instructions, then data.
func TestTicketPayloadGolden(t *testing.T) {
	dir, reg, bindings := fixture(t)
	if err := os.WriteFile(prompt.PreferencesPath(dir), []byte("Never use emoji in a commit message.\n"), 0o600); err != nil {
		t.Fatal(err)
	}

	p, err := prompt.Compose(prompt.ComposeInput{
		Role:      "grill",
		ConfigDir: dir,
		Sources:   reg,
		Bindings:  bindings,
		Bundle: prompt.Bundle{
			MapName: "Widget", MapBody: "THE-MAP-BODY",
			TicketNum: 2, TicketTitle: "Dependent work", TicketBody: "THE-TICKET-BODY",
			Blockers: []prompt.Blocker{
				{Num: 1, Title: "Base decision", Answer: "USE-THE-BASE-APPROACH"},
				{Num: 3, Title: "Unresolved", Answer: ""},
			},
		},
	})
	if err != nil {
		t.Fatalf("Compose: %v", err)
	}
	checkGolden(t, "ticket-payload.golden.md", p.Markdown, dir)

	// The parts, in order, each carrying the origin the preview badges. The role
	// part is named for the *role*, not the skill it is bound to, and its origin is
	// the source that yielded the body.
	wantParts := []struct{ name, kind, origin string }{
		{"core", "prompt", "chartr"},
		{"grill", "prompt", "house"},
		{"conventions", "prompt", "chartr"},
		{"preferences", "prompt", "operator"},
		{"sources", "context", "context"},
		{"map", "context", "context"},
		{"ticket", "context", "context"},
		{"blocker #01", "context", "context"},
		{"blocker #03", "context", "context"},
	}
	if len(p.Parts) != len(wantParts) {
		t.Fatalf("payload has %d parts, want %d: %+v", len(p.Parts), len(wantParts), p.Parts)
	}
	for i, want := range wantParts {
		got := p.Parts[i]
		if got.Name != want.name || got.Kind != want.kind || got.Origin != want.origin {
			t.Errorf("part %d = %s/%s/%s, want %s/%s/%s",
				i, got.Name, got.Kind, got.Origin, want.name, want.kind, want.origin)
		}
	}

	// The role body is concatenated, not pointed at — that is what makes the claim
	// trailer's skill record mean *this text ran* — and no origin line sits above
	// it, because the badge and the trailer both carry that already.
	if !strings.Contains(p.Parts[1].Text, grillMarker) {
		t.Errorf("the grill part does not carry the skill body (%q):\n%s", grillMarker, p.Parts[1].Text)
	}
	if !strings.Contains(p.Markdown, grillMarker) {
		t.Errorf("the composed document points at the role skill instead of carrying it:\n%s", p.Markdown)
	}

	// The claim trailer's record: chartr's core by the shipped hash the running
	// binary carries, the role by the source that resolved it.
	if len(p.Skills) != 2 || p.Skills[0].Name != "core" || p.Skills[1].Source != "house" {
		t.Errorf("composed skills = %+v, want the core then the house grill", p.Skills)
	}

	if _, err := prompt.Compose(prompt.ComposeInput{Role: "nonesuch", ConfigDir: dir, Sources: reg}); err == nil {
		t.Error("Compose accepted an unknown role")
	}
}

// The free payload: four parts and nothing else. No map list, no frontier, no
// branch for a space without `.plan/` — it holds no live fact about the space it
// runs in, which is why it is composed from the config root and the registry
// alone and takes no space at all.
func TestFreePayloadGolden(t *testing.T) {
	dir, reg, _ := fixture(t)

	p, err := prompt.ComposeFree(dir, reg)
	if err != nil {
		t.Fatalf("ComposeFree: %v", err)
	}
	checkGolden(t, "free-payload.golden.md", p.Markdown, dir)

	wantParts := []struct{ name, origin string }{
		{"core", "chartr"},
		{"conventions", "chartr"},
		{"preferences", "operator"},
		{"sources", "context"},
	}
	if len(p.Parts) != len(wantParts) {
		t.Fatalf("free payload has %d parts, want %d: %+v", len(p.Parts), len(wantParts), p.Parts)
	}
	for i, want := range wantParts {
		if got := p.Parts[i]; got.Name != want.name || got.Origin != want.origin {
			t.Errorf("part %d = %s/%s, want %s/%s", i, got.Name, got.Origin, want.name, want.origin)
		}
	}
	if p.Role != "" || p.TicketNum != 0 {
		t.Errorf("a free payload named a role or a ticket: %q/%d", p.Role, p.TicketNum)
	}
}

// The standing document chartr keeps at every space's root (chartr-md, ticket
// 01): the free payload's four parts minus the one clause only a shell chartr
// opened can be told. Its first sentence has to be true in the mouth of a file
// read by an agent chartr never launched — that is the failure this golden
// exists to catch, and the same "every sentence is still true if the agent does
// nothing" test governs every line below it.
func TestStandingPayloadGolden(t *testing.T) {
	dir, reg, _ := fixture(t)

	p, err := prompt.ComposeStanding(dir, reg)
	if err != nil {
		t.Fatalf("ComposeStanding: %v", err)
	}
	checkGolden(t, "standing-payload.golden.md", p.Markdown, dir)

	// The standing document is the free payload with the launch clause dropped and
	// nothing else changed — no extra part, no live fact about the space, and no
	// fetchable address for an agent running with permissions skipped.
	free, err := prompt.ComposeFree(dir, reg)
	if err != nil {
		t.Fatal(err)
	}
	if len(p.Parts) != len(free.Parts) {
		t.Fatalf("the standing document has %d parts, the free payload %d", len(p.Parts), len(free.Parts))
	}
	for i := range p.Parts {
		if p.Parts[i].Name != free.Parts[i].Name || p.Parts[i].Origin != free.Parts[i].Origin {
			t.Errorf("part %d = %s/%s, want the free payload's %s/%s",
				i, p.Parts[i].Name, p.Parts[i].Origin, free.Parts[i].Name, free.Parts[i].Origin)
		}
	}
	if strings.Contains(p.Markdown, "opened this terminal") || strings.Contains(p.Markdown, "This shell") {
		t.Errorf("the standing document claims chartr opened the reader's shell:\n%s", p.Markdown)
	}
	if !strings.Contains(free.Markdown, "This shell") {
		t.Errorf("the free payload lost the clause that is true only of it:\n%s", free.Markdown)
	}
	if strings.Contains(p.Markdown, "https://") {
		t.Errorf("a source URL reached the standing document:\n%s", p.Markdown)
	}
}

// The two things that make the free payload vary, and the only two: the sources
// block and the preferences bytes. Everything else is the same in an empty tree
// and a tree mid-effort.
func TestFreePayloadVariesOnlyWithSourcesAndPreferences(t *testing.T) {
	dir, reg, _ := fixture(t)
	base, err := prompt.ComposeFree(dir, reg)
	if err != nil {
		t.Fatal(err)
	}

	if err := os.WriteFile(prompt.PreferencesPath(dir), []byte("PREFER-THIS\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	extra := filepath.Join(t.TempDir(), "mine")
	if err := os.MkdirAll(filepath.Join(extra, "spelunk"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(extra, "spelunk", "SKILL.md"), []byte("---\nname: spelunk\n---\n\nbody\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	if _, err := reg.RegisterDir("mine", extra); err != nil {
		t.Fatal(err)
	}

	got, err := prompt.ComposeFree(dir, reg)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(got.Markdown, "PREFER-THIS") {
		t.Errorf("the operator's preferences did not reach the free payload:\n%s", got.Markdown)
	}
	if !strings.Contains(got.Markdown, "`mine` at `"+sources.MirrorDir+"/mine` — spelunk") {
		t.Errorf("a registered source did not reach the sources block:\n%s", got.Markdown)
	}
	if got.Markdown == base.Markdown {
		t.Error("the free payload did not change when its two inputs did")
	}

	// A disabled source is not in the inventory at all, and a git source's URL is
	// never printed — it is a fetchable address in a document handed to an agent
	// running with permissions skipped.
	if err := reg.SetEnabled("mine", false); err != nil {
		t.Fatal(err)
	}
	off, err := prompt.ComposeFree(dir, reg)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(off.Markdown, "spelunk") {
		t.Errorf("a disabled source appeared in the sources block:\n%s", off.Markdown)
	}
	if strings.Contains(off.Markdown, "https://") {
		t.Errorf("a source URL reached the payload:\n%s", off.Markdown)
	}
}

// A source whose directory is gone is the one failure that silently changes what
// a payload says, so it is the one composition warning that survives.
func TestUnavailableSourceWarns(t *testing.T) {
	dir, reg, _ := fixture(t)
	gone := filepath.Join(t.TempDir(), "vanished")
	if err := os.MkdirAll(gone, 0o755); err != nil {
		t.Fatal(err)
	}
	if _, err := reg.RegisterDir("ghost", gone); err != nil {
		t.Fatal(err)
	}
	if err := os.RemoveAll(gone); err != nil {
		t.Fatal(err)
	}

	p, err := prompt.ComposeFree(dir, reg)
	if err != nil {
		t.Fatal(err)
	}
	if !contains(p.Warnings, "ghost") || !contains(p.Warnings, "unavailable") {
		t.Errorf("a vanished source did not warn: %v", p.Warnings)
	}
}

// checkGolden compares a composed document with its golden, with the throwaway
// config root replaced by a stable placeholder so the file is diffable. Rewrite
// with `go test ./internal/prompt -update`.
func checkGolden(t *testing.T, name, got, configDir string) {
	t.Helper()
	got = strings.ReplaceAll(got, configDir, "<config>")
	path := filepath.Join("testdata", name)
	if *update {
		if err := os.MkdirAll("testdata", 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(path, []byte(got), 0o644); err != nil {
			t.Fatal(err)
		}
		return
	}
	want, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("reading the golden: %v (run `go test ./internal/prompt -update`)", err)
	}
	if got != string(want) {
		t.Errorf("the composed payload no longer matches %s.\n--- got ---\n%s\n--- want ---\n%s", name, got, want)
	}
}

func contains(hay []string, needle string) bool {
	for _, s := range hay {
		if strings.Contains(s, needle) {
			return true
		}
	}
	return false
}
