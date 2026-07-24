package prompt_test

import (
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
)

// The skill library's own seam (ticket 04): resolution across the three layers,
// the composed payload, and fork drift. Everything here is asserted through the
// package's public surface — Resolve, Compose, Library — on real directories on
// disk; the server's payload_test.go keeps exercising the same composer through
// the preview endpoint.

// roots lays out the three layer roots under one temp dir. Each is created only
// when a test writes into it, so an absent layer is the default.
func roots(t *testing.T) prompt.Roots {
	t.Helper()
	dir := t.TempDir()
	return prompt.Roots{
		Builtin:   filepath.Join(dir, "data", "skills"),
		User:      filepath.Join(dir, "config", "skills"),
		Workspace: filepath.Join(dir, "repo", ".chartr", "skills"),
	}
}

// writeSkill defines a skill in one layer: a SKILL.md carrying the standard
// frontmatter, plus any supporting files.
func writeSkill(t *testing.T, root, name, frontmatter, body string, support map[string]string) {
	t.Helper()
	dir := filepath.Join(root, name)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatalf("mkdir skill %s: %v", name, err)
	}
	src := fmt.Sprintf("---\nname: %s\ndescription: a test %s skill\n%s---\n\n%s\n", name, name, frontmatter, body)
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte(src), 0o644); err != nil {
		t.Fatalf("write SKILL.md: %v", err)
	}
	for n, c := range support {
		if err := os.WriteFile(filepath.Join(dir, n), []byte(c), 0o644); err != nil {
			t.Fatalf("write support file %s: %v", n, err)
		}
	}
}

func resolve(t *testing.T, name string, r prompt.Roots) prompt.Skill {
	t.Helper()
	s, ok := prompt.Resolve(name, r)
	if !ok {
		t.Fatalf("skill %q did not resolve in any layer", name)
	}
	return s
}

// Whole-skill shadowing: the most specific layer that defines a skill wins its
// *entire* directory — body and supporting files together — and the layers below
// it contribute nothing. Nothing is merged per file, so a shadowing skill that
// omits a supporting file the layer below has simply does not have one.
func TestWholeSkillShadowing(t *testing.T) {
	r := roots(t)

	// With no layer defining it, the shipped copy embedded in the binary is the
	// floor, and it reads as built-in.
	base := resolve(t, "implement", r)
	if base.Layer != "built-in" {
		t.Errorf("unlayered implement resolved from %q, want built-in", base.Layer)
	}
	if !strings.Contains(base.Body, "implementation map") {
		t.Errorf("shipped implement body missing its content:\n%s", base.Body)
	}

	// The materialized built-in layer wins over the embedded floor.
	writeSkill(t, r.Builtin, "implement", "", "BUILTIN-BODY", nil)
	if got := resolve(t, "implement", r); got.Layer != "built-in" || got.Body != "BUILTIN-BODY" {
		t.Errorf("built-in layer = %q/%q, want built-in/BUILTIN-BODY", got.Layer, got.Body)
	}

	// A user skill shadows the built-in whole directory.
	writeSkill(t, r.User, "implement", "", "USER-BODY", map[string]string{"notes.md": "USER-NOTES"})
	got := resolve(t, "implement", r)
	if got.Layer != "user" || got.Body != "USER-BODY" {
		t.Errorf("user layer = %q/%q, want user/USER-BODY", got.Layer, got.Body)
	}
	if s, ok := got.Support("notes.md"); !ok || s != "USER-NOTES" {
		t.Errorf("supporting file did not travel with the winning directory: %q (%v)", s, ok)
	}

	// A committed workspace skill wins over the local one (ADR 0009, content
	// half), and takes its whole directory with it: the user layer's supporting
	// file is gone, not merged in.
	writeSkill(t, r.Workspace, "implement", "", "WORKSPACE-BODY", nil)
	got = resolve(t, "implement", r)
	if got.Layer != "workspace" || got.Body != "WORKSPACE-BODY" {
		t.Errorf("workspace layer = %q/%q, want workspace/WORKSPACE-BODY", got.Layer, got.Body)
	}
	if _, ok := got.Support("notes.md"); ok {
		t.Error("the user layer's supporting file survived a workspace shadow; shadowing must be whole-directory")
	}

	// Shadowing is per skill, not per library: an untouched skill still resolves
	// from the floor.
	if got := resolve(t, "grill", r); got.Layer != "built-in" {
		t.Errorf("grill resolved from %q, want the untouched built-in", got.Layer)
	}
}

// A directory without a SKILL.md does not define a skill — the standard's entry
// point is what makes a directory one — so resolution falls through it.
func TestDirectoryWithoutSkillMDDoesNotDefine(t *testing.T) {
	r := roots(t)
	if err := os.MkdirAll(filepath.Join(r.User, "implement"), 0o755); err != nil {
		t.Fatal(err)
	}
	if got := resolve(t, "implement", r); got.Layer != "built-in" {
		t.Errorf("an empty directory shadowed the built-in: layer = %q", got.Layer)
	}
}

// Composition (ADR 0002, reaffirmed): chartr reads the resolved core and role
// bodies, strips their frontmatter, and appends the freshly-built context bundle —
// map, ticket, blockers' answers, and the glossary sourced from the
// tracker-convention skill.
func TestComposeCarriesResolvedBodiesAndBundle(t *testing.T) {
	r := roots(t)
	writeSkill(t, r.User, "core", "", "USER-CORE-BODY", nil)
	writeSkill(t, r.Workspace, "implement", "", "WORKSPACE-IMPLEMENT-BODY", nil)
	writeSkill(t, r.User, "tracker-convention", "", "TRACKER-BODY",
		map[string]string{"glossary.md": "USER-GLOSSARY-TERM"})

	p, err := prompt.Compose(prompt.ComposeInput{
		Role:  "implement",
		Roots: r,
		Bundle: prompt.Bundle{
			MapName: "widget", MapBody: "THE-MAP-BODY",
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

	// The resolved bodies, with the layer that won each recorded.
	core := part(t, p, "core")
	if core.Segments[0].Layer != "user" || !strings.Contains(core.Segments[0].Text, "USER-CORE-BODY") {
		t.Errorf("core part = %q/%q, want the user body", core.Segments[0].Layer, core.Segments[0].Text)
	}
	impl := part(t, p, "implement")
	if impl.Segments[0].Layer != "workspace" || !strings.Contains(impl.Segments[0].Text, "WORKSPACE-IMPLEMENT-BODY") {
		t.Errorf("implement part = %q/%q, want the workspace body", impl.Segments[0].Layer, impl.Segments[0].Text)
	}

	// Frontmatter is metadata, never payload (story 27).
	for _, meta := range []string{"description:", "name: core", "---"} {
		if strings.Contains(core.Segments[0].Text, meta) {
			t.Errorf("frontmatter %q leaked into the composed body:\n%s", meta, core.Segments[0].Text)
		}
	}
	if p.Skills[0].Description != "a test core skill" {
		t.Errorf("description not parsed off the frontmatter: %q", p.Skills[0].Description)
	}

	// The context bundle rides after the prompts: the glossary sourced from the
	// resolved tracker-convention skill, the map, the ticket, each blocker.
	if got := text(part(t, p, "glossary")); !strings.Contains(got, "USER-GLOSSARY-TERM") {
		t.Errorf("glossary not sourced from the resolved tracker-convention skill: %q", got)
	}

	// The skill-library manifest names every shipped skill with its use and the
	// path its winning layer sits at — a layer win points at that layer's
	// directory, the embedded floor at the materialized built-in root.
	manifest := text(part(t, p, "skill-library"))
	for _, want := range []string{"`core`", "`wayfinder`", "`domain-modeling`", "`to-spec`", "`to-tickets`"} {
		if !strings.Contains(manifest, want) {
			t.Errorf("skill-library manifest missing %s:\n%s", want, manifest)
		}
	}
	if !strings.Contains(manifest, filepath.Join(r.User, "core")) {
		t.Errorf("manifest should point core at the winning user layer:\n%s", manifest)
	}
	if !strings.Contains(manifest, filepath.Join(r.Builtin, "wayfinder")) {
		t.Errorf("manifest should point wayfinder at the materialized built-in root:\n%s", manifest)
	}
	if strings.Contains(manifest, "USER-CORE-BODY") {
		t.Error("the manifest carries reach, never a skill's body")
	}
	if got := text(part(t, p, "map")); got != "THE-MAP-BODY" {
		t.Errorf("map part = %q", got)
	}
	if got := text(part(t, p, "ticket")); got != "THE-TICKET-BODY" {
		t.Errorf("ticket part = %q", got)
	}
	if got := text(part(t, p, "blocker #01")); !strings.Contains(got, "USE-THE-BASE-APPROACH") {
		t.Errorf("blocker answer not inlined: %q", got)
	}
	if got := text(part(t, p, "blocker #03")); !strings.Contains(got, "not resolved") {
		t.Errorf("a blocker with no answer should read as unresolved: %q", got)
	}

	// One document, prompts first, then the bundle under `# Context`.
	for _, want := range []string{"USER-CORE-BODY", "WORKSPACE-IMPLEMENT-BODY", "# Context", "THE-TICKET-BODY"} {
		if !strings.Contains(p.Markdown, want) {
			t.Errorf("composed markdown missing %q:\n%s", want, p.Markdown)
		}
	}
	if strings.Index(p.Markdown, "USER-CORE-BODY") > strings.Index(p.Markdown, "# Context") {
		t.Error("the prompts must precede the context bundle in the composed document")
	}

	// The skills composed in are recorded with their winning layer and hash — the
	// claim commit's provenance trailers.
	if len(p.Skills) != 2 || p.Skills[0].Name != "core" || p.Skills[1].Name != "implement" {
		t.Fatalf("composed skills = %+v, want core then implement", p.Skills)
	}
	if p.Skills[1].Layer != "workspace" || p.Skills[1].Hash == "" {
		t.Errorf("skill provenance = %q/%q, want workspace with a hash", p.Skills[1].Layer, p.Skills[1].Hash)
	}

	if _, err := prompt.Compose(prompt.ComposeInput{Role: "nonesuch", Roots: r}); err == nil {
		t.Error("Compose accepted an unknown role")
	}
}

// Fork drift is detected over the whole directory hash: a shadowing skill that
// records the shipped version it forked from is surfaced as behind once that
// default has moved on — never auto-merged (story 23) — and a supporting file is
// part of what the hash covers (story 24).
func TestForkedFromDriftOverDirectoryHash(t *testing.T) {
	r := roots(t)

	// A fork recording a hash that is not the shipped one is behind.
	writeSkill(t, r.User, "implement", "forked_from: deadbeef\n", "MY OWN IMPLEMENT SKILL", nil)
	got := resolve(t, "implement", r)
	if !got.Stale {
		t.Errorf("a fork recording an old hash is not stale: %+v", got)
	}
	if w := prompt.LibraryWarnings(r); !contains(w, "behind the shipped default") {
		t.Errorf("stale fork not surfaced in the library warnings: %v", w)
	}

	// A fork recording the *current* shipped hash is owned, not behind.
	writeSkill(t, r.User, "implement", "forked_from: "+prompt.ShippedHash("implement")+"\n", "MY OWN IMPLEMENT SKILL", nil)
	if got := resolve(t, "implement", r); got.Stale {
		t.Errorf("a fork on the current shipped default reads stale: %+v", got)
	}
	if w := prompt.LibraryWarnings(r); contains(w, "behind the shipped default") {
		t.Errorf("a fork on the current default warned: %v", w)
	}

	// A skill with no recorded provenance is not a drift claim, so it never warns.
	writeSkill(t, r.User, "grill", "", "MY OWN GRILL SKILL", nil)
	if got := resolve(t, "grill", r); got.Stale {
		t.Error("a fork with no forked_from should not read stale")
	}

	// The hash covers supporting files, not just SKILL.md: changing one changes it.
	writeSkill(t, r.User, "research", "", "BODY", map[string]string{"notes.md": "ONE"})
	first := resolve(t, "research", r).Hash
	writeSkill(t, r.User, "research", "", "BODY", map[string]string{"notes.md": "TWO"})
	if second := resolve(t, "research", r).Hash; second == first {
		t.Errorf("a changed supporting file left the directory hash at %s", first)
	}
}

// The ideate on-ramp is a skill like any other — layered and shadowable — but
// composed alone: no core, no role, no context bundle, because an ideate session
// is ticketless and mapless (story 29).
func TestIdeateComposesAlone(t *testing.T) {
	r := roots(t)
	if got := prompt.Ideate(r); !strings.Contains(got, "ticketless") {
		t.Errorf("shipped ideate body missing its content:\n%s", got)
	}
	writeSkill(t, r.User, "ideate", "", "MY OWN IDEATE SKILL", nil)
	got := prompt.Ideate(r)
	if got != "MY OWN IDEATE SKILL" {
		t.Errorf("Ideate = %q, want just the resolved body", got)
	}
	if strings.Contains(got, "# Context") || strings.Contains(got, "chartr session") {
		t.Errorf("ideate carried core or a context bundle:\n%s", got)
	}
}

// The shipped library is the eleven skills chartr composes from, each a real
// SKILL.md carrying the standard frontmatter contract; the glossary lives inside
// tracker-convention, and domain-modeling carries its two format references as
// supporting files (stories 16–17).
func TestShippedLibraryIsElevenSkills(t *testing.T) {
	lib := prompt.Library(prompt.Roots{})
	want := []string{"core", "grill", "prototype", "research", "implement", "ideate", "tracker-convention",
		"wayfinder", "domain-modeling", "to-spec", "to-tickets"}
	if len(lib) != len(want) {
		t.Fatalf("shipped library has %d skills, want %d: %+v", len(lib), len(want), lib)
	}
	for i, name := range want {
		s := lib[i]
		if s.Name != name {
			t.Fatalf("library[%d] = %q, want %q", i, s.Name, name)
		}
		if s.Description == "" {
			t.Errorf("skill %q ships without a description", name)
		}
		if strings.TrimSpace(s.Body) == "" {
			t.Errorf("skill %q ships with an empty body", name)
		}
		if s.Hash == "" || s.Hash != prompt.ShippedHash(name) {
			t.Errorf("skill %q hash = %q, want the shipped hash %q", name, s.Hash, prompt.ShippedHash(name))
		}
	}
	tc, ok := prompt.Resolve("tracker-convention", prompt.Roots{})
	if !ok {
		t.Fatal("tracker-convention did not resolve")
	}
	if g, ok := tc.Support("glossary.md"); !ok || !strings.Contains(g, "**Map**") {
		t.Errorf("the glossary is not a supporting file of tracker-convention: %v", ok)
	}
	dm, ok := prompt.Resolve("domain-modeling", prompt.Roots{})
	if !ok {
		t.Fatal("domain-modeling did not resolve")
	}
	for _, f := range []string{"CONTEXT-FORMAT.md", "ADR-FORMAT.md"} {
		if _, ok := dm.Support(f); !ok {
			t.Errorf("%s is not a supporting file of domain-modeling", f)
		}
	}
}

// Materialize writes the library to disk as skill directories the operator can
// read and edit, and never overwrites an edit.
func TestMaterializePreservesEdits(t *testing.T) {
	data := t.TempDir()
	if err := prompt.Materialize(data); err != nil {
		t.Fatalf("Materialize: %v", err)
	}
	skill := filepath.Join(data, "builtin-skills", "implement", "SKILL.md")
	if _, err := os.Stat(skill); err != nil {
		t.Fatalf("library was not materialized: %v", err)
	}
	if _, err := os.Stat(filepath.Join(data, "builtin-skills", "tracker-convention", "glossary.md")); err != nil {
		t.Errorf("supporting files were not materialized: %v", err)
	}

	if err := os.WriteFile(skill, []byte("EDITED"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := prompt.Materialize(data); err != nil {
		t.Fatalf("Materialize again: %v", err)
	}
	b, err := os.ReadFile(skill)
	if err != nil || string(b) != "EDITED" {
		t.Errorf("Materialize overwrote an operator's edit: %q (%v)", b, err)
	}

	// And the edit composes: the materialized directory is the built-in layer.
	r := prompt.RootsFor(data, "")
	if got := resolve(t, "implement", r); got.Body != "EDITED" || got.Layer != "built-in" {
		t.Errorf("materialized edit did not compose: %q/%q", got.Layer, got.Body)
	}
}

// The launcher metadata: `on-ramp` and `needs-context` parse off a skill's own
// frontmatter, and — because they ride whole-skill shadowing — a shadowing layer
// declares its own, so an operator's fork sets its own launch status rather than
// inheriting the shipped default's. (This is the payoff the hackable library was
// built for: a user skill appears in the picker with no chartr-side change.)
func TestOnRampFlagsParseAndShadow(t *testing.T) {
	r := roots(t)

	// A built-in skill that opts in and wants context.
	writeSkill(t, r.Builtin, "explore", "on-ramp: true\nneeds-context: true\n", "explore body", nil)
	s := resolve(t, "explore", r)
	if !s.OnRamp || !s.NeedsContext {
		t.Errorf("explore built-in: onRamp=%v needsContext=%v, want both true", s.OnRamp, s.NeedsContext)
	}

	// A user layer shadows it and declares itself on-ramp but *not* needs-context —
	// the shadow's own flags win the whole directory, they are not merged.
	writeSkill(t, r.User, "explore", "on-ramp: true\n", "user explore body", nil)
	s = resolve(t, "explore", r)
	if s.Layer != "user" || s.Body != "user explore body" {
		t.Fatalf("shadow did not win: %+v", s)
	}
	if !s.OnRamp || s.NeedsContext {
		t.Errorf("user shadow: onRamp=%v needsContext=%v, want on-ramp only", s.OnRamp, s.NeedsContext)
	}

	// A skill that declares nothing is not on the launcher — the flag is opt-in.
	writeSkill(t, r.Builtin, "quiet", "", "quiet body", nil)
	if q := resolve(t, "quiet", r); q.OnRamp || q.NeedsContext {
		t.Errorf("quiet: onRamp=%v needsContext=%v, want both false", q.OnRamp, q.NeedsContext)
	}
}

// Launch composes the named skill's body alone — no core, no context bundle — and
// appends the optional context under a `## Your task` trailer only when it is
// present. An empty (or whitespace) context writes the body unchanged, so a
// self-driving skill launches bare exactly as ideate does.
func TestLaunchComposesSkillAloneWithOptionalContext(t *testing.T) {
	r := roots(t)
	writeSkill(t, r.Builtin, "explore", "on-ramp: true\n", "EXPLORE BODY", nil)

	bare := string(prompt.Launch(r, "explore", ""))
	if bare != "EXPLORE BODY" {
		t.Errorf("bare launch = %q, want just the body", bare)
	}
	if string(prompt.Launch(r, "explore", "   \n  ")) != "EXPLORE BODY" {
		t.Errorf("whitespace context should launch bare")
	}

	withCtx := string(prompt.Launch(r, "explore", "settle the widget question"))
	if !strings.HasPrefix(withCtx, "EXPLORE BODY") {
		t.Errorf("context launch dropped the body:\n%s", withCtx)
	}
	if !strings.Contains(withCtx, "## Your task") || !strings.Contains(withCtx, "settle the widget question") {
		t.Errorf("context did not ride in the payload under a trailer:\n%s", withCtx)
	}

	// A skill that resolves in no layer launches nothing.
	if got := prompt.Launch(r, "no-such-skill", "x"); got != nil {
		t.Errorf("Launch of an unresolved skill = %q, want nil", got)
	}
}

// The shipped library tags exactly the five self-driving on-ramp skills, and marks
// needs-context on the three that read a subject — ideate and wayfinder open cold.
// The augmentative and second-step skills stay off the launcher.
func TestShippedOnRampTagging(t *testing.T) {
	onRamp := map[string]bool{}
	needsContext := map[string]bool{}
	for _, s := range prompt.Library(prompt.Roots{}) {
		if s.OnRamp {
			onRamp[s.Name] = true
		}
		if s.NeedsContext {
			needsContext[s.Name] = true
		}
	}

	wantOnRamp := map[string]bool{"ideate": true, "wayfinder": true, "grill": true, "research": true, "prototype": true}
	if !reflect.DeepEqual(onRamp, wantOnRamp) {
		t.Errorf("on-ramp skills = %v, want %v", onRamp, wantOnRamp)
	}
	wantNeedsContext := map[string]bool{"grill": true, "research": true, "prototype": true}
	if !reflect.DeepEqual(needsContext, wantNeedsContext) {
		t.Errorf("needs-context skills = %v, want %v", needsContext, wantNeedsContext)
	}

	// needs-context is meaningless off the launcher: nothing carries it without
	// on-ramp too.
	for name := range needsContext {
		if !onRamp[name] {
			t.Errorf("%s marks needs-context but not on-ramp", name)
		}
	}
}

func part(t *testing.T, p prompt.Payload, name string) prompt.Part {
	t.Helper()
	for _, part := range p.Parts {
		if part.Name == name {
			return part
		}
	}
	t.Fatalf("part %q not in the payload", name)
	return prompt.Part{}
}

func text(p prompt.Part) string {
	var out []string
	for _, s := range p.Segments {
		out = append(out, s.Text)
	}
	return strings.Join(out, "\n")
}

func contains(hay []string, needle string) bool {
	for _, s := range hay {
		if strings.Contains(s, needle) {
			return true
		}
	}
	return false
}
