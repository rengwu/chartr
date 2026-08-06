package server_test

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/prompt"
)

// The two payload previews at the process boundary: the ticket payload for a
// chosen ticket and role, and the free payload, which names no space because it
// holds no fact about one. Every assertion is on the public payload the endpoints
// return and on the files on disk; no test reaches into the package. The focused
// composition seam, with both goldens, lives in internal/prompt.

func getPayload(t *testing.T, h *chartrtest.Chartr, id, slug string, num int, role string) (int, prompt.Payload, string) {
	t.Helper()
	code, body := h.Get(fmt.Sprintf("/api/spaces/%s/maps/%s/tickets/%d/payload?role=%s", id, slug, num, role))
	var p prompt.Payload
	if code == 200 {
		if err := json.Unmarshal([]byte(body), &p); err != nil {
			t.Fatalf("payload response not JSON: %v (%q)", err, body)
		}
	}
	return code, p, body
}

func findPart(t *testing.T, p prompt.Payload, name string) prompt.Part {
	t.Helper()
	for _, part := range p.Parts {
		if part.Name == name {
			return part
		}
	}
	t.Fatalf("part %q not in payload (parts: %s)", name, strings.Join(partNames(p), ", "))
	return prompt.Part{}
}

func hasPart(p prompt.Payload, name string) bool {
	for _, part := range p.Parts {
		if part.Name == name {
			return true
		}
	}
	return false
}

func partNames(p prompt.Payload) []string {
	var out []string
	for _, part := range p.Parts {
		out = append(out, part.Name)
	}
	return out
}

func segText(part prompt.Part) string { return part.Text }

// skillSource renders a SKILL.md: the standard frontmatter contract over a body,
// with any extra frontmatter lines (a `forked_from:`) folded in.
func skillSource(name, extra, body string) string {
	return fmt.Sprintf("---\nname: %s\ndescription: a test %s skill\n%s---\n\n%s\n", name, name, extra, body)
}

// writeUserSkill defines a skill in the operator's local library (the user layer)
// under their config root.
func writeUserSkill(t *testing.T, configDir, name, extra, body string) {
	t.Helper()
	dir := filepath.Join(configDir, "skills", name)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatalf("mkdir user skill: %v", err)
	}
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte(skillSource(name, extra, body)), 0o644); err != nil {
		t.Fatalf("write user skill %s: %v", name, err)
	}
}

// writeWorkspaceSkill defines a skill in the space's committed library.
func writeWorkspaceSkill(t *testing.T, repo, name, extra, body string) {
	t.Helper()
	chartrtest.WriteFile(t, repo,
		filepath.Join(".chartr", "skills", name, "SKILL.md"), skillSource(name, extra, body))
}

// The preview composes a session's whole payload: chartr's embedded core and the
// role's bound skill, then the contract files, then the context region assembled
// fresh — the sources block, the map body, this ticket, and each blocker's answer
// pulled inline. The composed markdown is the single document a session is told.
func TestPayloadComposesWithProvenanceAndBundle(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	// A resolved blocker whose answer the bundle must inline.
	chartrtest.WriteTicket(t, repo, "widget", "01-base.md",
		ticket(1, "Base decision", "[]", "task", "## Answer\nUSE-THE-BASE-APPROACH."))
	chartrtest.WriteTicket(t, repo, "widget", "02-dependent.md",
		ticket(2, "Dependent work", "[1]", "task", ""))

	resp := register(t, h, repo)

	code, p, body := getPayload(t, h, resp.ID, "widget", 2, "implement")
	if code != 200 {
		t.Fatalf("payload preview = %d, body %s", code, body)
	}

	// Core comes first, shipped built-in; then the chosen role, resolved through
	// its seeded binding into the default source rather than through the layers
	// (skill-sources ticket 05).
	core := findPart(t, p, "core")
	if core.Origin != "chartr" {
		t.Errorf("core origin = %q, want chartr", core.Origin)
	}
	impl := findPart(t, p, "implement")
	if impl.Origin != "chartr-skills" {
		t.Errorf("implement origin = %q, want chartr-skills", impl.Origin)
	}
	if !strings.Contains(segText(impl), "implementation map") {
		t.Errorf("implement prompt missing its shipped content:\n%s", segText(impl))
	}

	// The context bundle is present and fresh (ADR 0005).
	for _, name := range []string{"sources", "map", "ticket"} {
		if !hasPart(p, name) {
			t.Errorf("payload missing context part %q; parts: %v", name, partNames(p))
		}
	}

	// The blocker's answer is pulled inline.
	blocker := findPart(t, p, "blocker #01")
	if !strings.Contains(segText(blocker), "USE-THE-BASE-APPROACH") {
		t.Errorf("blocker answer not inlined:\n%s", segText(blocker))
	}
	if blocker.Origin != "context" {
		t.Errorf("blocker origin = %q, want context", blocker.Origin)
	}

	// The composed markdown is one document carrying prompt and context together.
	if !strings.Contains(p.Markdown, "chartr session") {
		t.Errorf("composed markdown missing the core prompt:\n%s", p.Markdown)
	}
	if !strings.Contains(p.Markdown, "# Context") || !strings.Contains(p.Markdown, "USE-THE-BASE-APPROACH") {
		t.Errorf("composed markdown missing the context bundle:\n%s", p.Markdown)
	}
}

// A blocker carrying only an in-flight `## Proposed Answer` — wreckage from the
// retired review lifecycle — contributes *no* answer to a dependent's bundle. The
// heading is unknown to the reader and no human blessed what is under it, so the
// bundle says the blocker is not resolved rather than handing a session an
// unblessed proposal as though it were the answer (spec, ignore-don't-tolerate).
func TestProposedAnswerIsNotABlockersAnswer(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-base.md",
		ticket(1, "Base decision", "[]", "task", "## Proposed Answer\nUSE-THE-UNBLESSED-APPROACH."))
	chartrtest.WriteTicket(t, repo, "widget", "02-dependent.md",
		ticket(2, "Dependent work", "[1]", "task", ""))

	resp := register(t, h, repo)

	code, p, body := getPayload(t, h, resp.ID, "widget", 2, "implement")
	if code != 200 {
		t.Fatalf("payload preview = %d, body %s", code, body)
	}

	blocker := findPart(t, p, "blocker #01")
	if strings.Contains(segText(blocker), "USE-THE-UNBLESSED-APPROACH") {
		t.Errorf("proposed answer leaked into the bundle as an answer:\n%s", segText(blocker))
	}
	if !strings.Contains(segText(blocker), "not resolved") {
		t.Errorf("blocker without an answer should read as unresolved:\n%s", segText(blocker))
	}
	if strings.Contains(p.Markdown, "USE-THE-UNBLESSED-APPROACH") {
		t.Errorf("proposed answer leaked into the composed markdown:\n%s", p.Markdown)
	}
}

// A blocker corrected after it resolved hands its dependent the *corrected*
// answer. This repo amends a resolved ticket by appending a `## Correction` or
// `## Amendment (…)` section rather than editing the answer in place, so reading
// only up to the next `## ` would inline the superseded text and drop the
// retraction — a wrong statement travelling with a blessed answer's authority.
// The amendment's heading rides along too: it carries the correction's point and
// marks the prose below it as superseding.
func TestBlockerAnswerCarriesItsCorrections(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-base.md", ticket(1, "Base decision", "[]", "task",
		"## Answer\nUSE-THE-BASE-APPROACH.\n\n"+
			"## Correction — the base approach was wrong about the mechanism\n\n"+
			"USE-THE-CORRECTED-APPROACH.\n\n"+
			"## Out of scope\n\nUNRELATED-TRAILING-SECTION."))
	chartrtest.WriteTicket(t, repo, "widget", "02-dependent.md",
		ticket(2, "Dependent work", "[1]", "task", ""))

	resp := register(t, h, repo)

	code, p, body := getPayload(t, h, resp.ID, "widget", 2, "implement")
	if code != 200 {
		t.Fatalf("payload preview = %d, body %s", code, body)
	}

	blocker := segText(findPart(t, p, "blocker #01"))
	if !strings.Contains(blocker, "USE-THE-CORRECTED-APPROACH") {
		t.Errorf("the correction was dropped from the blocker's answer:\n%s", blocker)
	}
	if !strings.Contains(blocker, "## Correction — the base approach was wrong") {
		t.Errorf("the correction's heading was dropped, so the prose reads as a continuation:\n%s", blocker)
	}
	// The original still travels: an amendment supersedes in prose, and the
	// dependent reads both halves in order rather than a rewritten answer.
	if !strings.Contains(blocker, "USE-THE-BASE-APPROACH") {
		t.Errorf("the amended answer lost its original:\n%s", blocker)
	}
	// A heading that amends nothing still ends the answer.
	if strings.Contains(blocker, "UNRELATED-TRAILING-SECTION") {
		t.Errorf("an unrelated trailing section leaked into the answer:\n%s", blocker)
	}
	if !strings.Contains(p.Markdown, "USE-THE-CORRECTED-APPROACH") {
		t.Errorf("composed markdown missing the correction:\n%s", p.Markdown)
	}
}

// Neither half of the instruction is a skill layer's to move. The core is
// chartr's own voice, read straight out of the binary, and the role resolves
// through the operator's binding into their sources — so a `core` or a `grill`
// written into either skill layer changes nothing about what a session is told.
//
// This replaces the shadowing matrix that stood here. The layers still resolve
// the shipped library for the settings surface, but no layer reaches a payload
// any more, and the whole model goes with ticket 09.
func TestNeitherCoreNorRoleIsShadowableByASkillLayer(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	_, base, _ := getPayload(t, h, resp.ID, "widget", 1, "grill")

	writeUserSkill(t, h.ConfigDir, "core", "", "USER-CORE-SKILL")
	writeWorkspaceSkill(t, repo, "grill", "", "WORKSPACE-GRILL-SKILL")

	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "grill")
	core := findPart(t, p, "core")
	if core.Origin != "chartr" || strings.Contains(core.Text, "USER-CORE-SKILL") {
		t.Errorf("a user skill reached the core: %s\n%s", core.Origin, core.Text)
	}
	if !strings.Contains(core.Text, "chartr session") {
		t.Errorf("the embedded core is not what composed:\n%s", core.Text)
	}
	// Frontmatter is metadata for the cockpit, never payload.
	if strings.Contains(core.Text, "description:") {
		t.Errorf("frontmatter leaked into the composed body:\n%s", core.Text)
	}
	grill := findPart(t, p, "grill")
	if grill.Origin != "chartr-skills" || strings.Contains(grill.Text, "WORKSPACE-GRILL-SKILL") {
		t.Errorf("a workspace skill reached the role: %s\n%s", grill.Origin, grill.Text)
	}
	if p.Markdown != base.Markdown {
		t.Error("writing into the skill layers changed the composed document")
	}
}

// The free payload's preview takes no space, and returns the same four parts
// whichever tree the operator happens to have registered — no map list, no
// frontier, no ticket, no role. It is the operator's one window onto what an
// agent chartr launches without a ticket is told.
func TestFreePayloadPreviewIsSpaceIndependent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	register(t, h, repo)

	code, body := h.Get("/api/payload/free")
	if code != 200 {
		t.Fatalf("free payload preview = %d, body %s", code, body)
	}
	var p prompt.Payload
	if err := json.Unmarshal([]byte(body), &p); err != nil {
		t.Fatalf("free payload response not JSON: %v (%q)", err, body)
	}

	if got := partNames(p); !equalStrings(got, []string{"core", "conventions", "preferences", "sources"}) {
		t.Errorf("free payload parts = %v, want core, conventions, preferences, sources", got)
	}
	// Not one live fact about the space it will run in.
	for _, forbidden := range []string{"widget", "First", "frontier", ".plan/maps/widget"} {
		if strings.Contains(p.Markdown, forbidden) {
			t.Errorf("the free payload carries %q, a live fact about the space:\n%s", forbidden, p.Markdown)
		}
	}
	if !strings.Contains(p.Markdown, "chartr is the cockpit") {
		t.Errorf("the free payload is missing its core:\n%s", p.Markdown)
	}
}

// The preview refuses what it cannot compose, so a bad request is a response, not
// a surprise: an unknown or missing role, a missing space, map, or ticket.
func TestPayloadPreviewRejectsBadInput(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	if code, _ := h.Get(fmt.Sprintf("/api/spaces/%s/maps/widget/tickets/1/payload?role=nonesuch", resp.ID)); code != 400 {
		t.Errorf("unknown role = %d, want 400", code)
	}
	if code, _ := h.Get(fmt.Sprintf("/api/spaces/%s/maps/widget/tickets/1/payload", resp.ID)); code != 400 {
		t.Errorf("missing role = %d, want 400", code)
	}
	if code, _ := h.Get("/api/spaces/no-such-space/maps/widget/tickets/1/payload?role=implement"); code != 404 {
		t.Errorf("missing space = %d, want 404", code)
	}
	if code, _ := h.Get(fmt.Sprintf("/api/spaces/%s/maps/nope/tickets/1/payload?role=implement", resp.ID)); code != 404 {
		t.Errorf("missing map = %d, want 404", code)
	}
	if code, _ := h.Get(fmt.Sprintf("/api/spaces/%s/maps/widget/tickets/99/payload?role=implement", resp.ID)); code != 404 {
		t.Errorf("missing ticket = %d, want 404", code)
	}
}
