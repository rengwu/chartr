package server_test

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
	"github.com/rengwu/wayfinder-harness/internal/prompt"
)

// The skill library and payload preview at the process boundary. The harness
// materializes a hackable `SKILL.md` library, resolves each skill across
// built-in ‹ user ‹ workspace with whole-skill shadowing, surfaces a fork behind
// the shipped default, and composes core + role + context bundle into one
// payload. Every assertion is on the public payload the preview endpoint returns
// and on the files on disk; no test reaches into the package. The focused
// resolution/assembly unit seam lives in internal/prompt (prompt_test.go).

func getPayload(t *testing.T, h *harnesstest.Harness, id, slug string, num int, role string) (int, prompt.Payload, string) {
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

func segLayers(part prompt.Part) []string {
	var out []string
	for _, s := range part.Segments {
		out = append(out, s.Layer)
	}
	return out
}

func segText(part prompt.Part) string {
	var out []string
	for _, s := range part.Segments {
		out = append(out, s.Text)
	}
	return strings.Join(out, "\n")
}

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
	harnesstest.WriteFile(t, repo,
		filepath.Join(".wayfinder-harness", "skills", name, "SKILL.md"), skillSource(name, extra, body))
}

// The preview composes a session's whole payload: the resolved core and role
// prompts (shipped built-in by default), then the context bundle assembled fresh
// — glossary, map body, this ticket, and each blocker's answer pulled inline. The
// composed markdown is the single document a session would be told.
func TestPayloadComposesWithProvenanceAndBundle(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	// A resolved blocker whose answer the bundle must inline.
	harnesstest.WriteTicket(t, repo, "widget", "01-base.md",
		ticket(1, "Base decision", "[]", "task", "## Answer\nUSE-THE-BASE-APPROACH."))
	harnesstest.WriteTicket(t, repo, "widget", "02-dependent.md",
		ticket(2, "Dependent work", "[1]", "task", ""))

	resp := register(t, h, repo)

	code, p, body := getPayload(t, h, resp.ID, "widget", 2, "implement")
	if code != 200 {
		t.Fatalf("payload preview = %d, body %s", code, body)
	}

	// Core comes first, then the chosen role, both shipped built-in by default.
	core := findPart(t, p, "core")
	if got := segLayers(core); len(got) != 1 || got[0] != "built-in" {
		t.Errorf("core layers = %v, want [built-in]", got)
	}
	impl := findPart(t, p, "implement")
	if got := segLayers(impl); len(got) != 1 || got[0] != "built-in" {
		t.Errorf("implement layers = %v, want [built-in]", got)
	}
	if !strings.Contains(segText(impl), "implementation map") {
		t.Errorf("implement prompt missing its shipped content:\n%s", segText(impl))
	}

	// The context bundle is present and fresh (ADR 0005).
	for _, name := range []string{"glossary", "map", "ticket"} {
		if !hasPart(p, name) {
			t.Errorf("payload missing context part %q; parts: %v", name, partNames(p))
		}
	}

	// The blocker's answer is pulled inline.
	blocker := findPart(t, p, "blocker #01")
	if !strings.Contains(segText(blocker), "USE-THE-BASE-APPROACH") {
		t.Errorf("blocker answer not inlined:\n%s", segText(blocker))
	}
	if l := blocker.Segments[0].Layer; l != "context" {
		t.Errorf("blocker segment layer = %q, want context", l)
	}

	// The composed markdown is one document carrying prompt and context together.
	if !strings.Contains(p.Markdown, "wayfinder-harness session") {
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
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-base.md",
		ticket(1, "Base decision", "[]", "task", "## Proposed Answer\nUSE-THE-UNBLESSED-APPROACH."))
	harnesstest.WriteTicket(t, repo, "widget", "02-dependent.md",
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

// Resolution walks built-in ‹ user ‹ workspace with whole-skill shadowing: the
// most specific layer defining a skill wins its entire directory, and a committed
// workspace skill wins over a local user one (the content half of ADR 0009). The
// whole matrix is observable in the resolved core part's single segment and its
// layer tag.
func TestSkillShadowingMatrix(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	// Baseline: the shipped default, one built-in segment.
	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "grill")
	if got := segLayers(findPart(t, p, "core")); len(got) != 1 || got[0] != "built-in" {
		t.Fatalf("baseline core layers = %v, want [built-in]", got)
	}

	// A user skill shadows the built-in whole directory — the shipped body is
	// gone, not stacked onto.
	writeUserSkill(t, h.ConfigDir, "core", "", "USER-CORE-SKILL")
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "grill")
	core := findPart(t, p, "core")
	if got := segLayers(core); !equalStrings(got, []string{"user"}) {
		t.Errorf("user-shadowed core layers = %v, want [user]", got)
	}
	if txt := segText(core); !strings.Contains(txt, "USER-CORE-SKILL") || strings.Contains(txt, "wayfinder-harness session") {
		t.Errorf("user skill did not shadow the shipped body:\n%s", txt)
	}

	// A committed workspace skill wins over the user one.
	writeWorkspaceSkill(t, repo, "core", "", "WORKSPACE-CORE-SKILL")
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "grill")
	core = findPart(t, p, "core")
	if got := segLayers(core); !equalStrings(got, []string{"workspace"}) {
		t.Errorf("workspace-shadowed core layers = %v, want [workspace]", got)
	}
	txt := segText(core)
	if !strings.Contains(txt, "WORKSPACE-CORE-SKILL") || strings.Contains(txt, "USER-CORE-SKILL") {
		t.Errorf("workspace skill did not win over the user one:\n%s", txt)
	}
	// Frontmatter is metadata for the cockpit, never payload.
	if strings.Contains(txt, "description:") {
		t.Errorf("frontmatter leaked into the composed body:\n%s", txt)
	}

	// Shadowing is per skill: the role skill is untouched by any of it.
	if got := segLayers(findPart(t, p, "grill")); !equalStrings(got, []string{"built-in"}) {
		t.Errorf("grill layers = %v, want the untouched [built-in]", got)
	}
}

// A skill that records the shipped default it forked from is surfaced as behind
// when that default has since moved on — never auto-merged (story 23). The notice
// rides both the space snapshot and the preview; a fork recording the current
// shipped hash draws no warning.
func TestBehindDefaultSurfaced(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	// A stale fork: a skill recording a `forked_from` that is not the shipped hash.
	writeUserSkill(t, h.ConfigDir, "implement", "forked_from: deadbeef\n", "MY OWN IMPLEMENT SKILL")

	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "implement")
	if !hasSubstring(p.Warnings, "behind the shipped default") {
		t.Errorf("behind-default not surfaced in preview warnings: %v", p.Warnings)
	}
	// The frontmatter is stripped from the composed body — meta never leaks into
	// the payload.
	if strings.Contains(segText(findPart(t, p, "implement")), "forked_from") {
		t.Errorf("fork frontmatter leaked into the payload:\n%s", segText(findPart(t, p, "implement")))
	}
	// It is also surfaced on the space, so a stale fork is visible without opening
	// the preview. The library lives under the data root, not the watched `.plan/`,
	// so it refreshes on the next rebuild rather than by notice — force one.
	if code, body := h.Post(fmt.Sprintf("/api/spaces/%s/pin", resp.ID), map[string]bool{"pinned": true}); code != 204 {
		t.Fatalf("pin to force a rebuild = %d, body %s", code, body)
	}
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if !hasSubstring(s.Warnings, "behind the shipped default") {
		t.Errorf("behind-default not surfaced on the space: %v", s.Warnings)
	}

	// A fork recording the *current* shipped hash is owned, not behind.
	writeUserSkill(t, h.ConfigDir, "implement", "forked_from: "+prompt.ShippedHash("implement")+"\n", "MY OWN IMPLEMENT SKILL")
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "implement")
	if hasSubstring(p.Warnings, "behind the shipped default") {
		t.Errorf("a fork on the current default should not warn: %v", p.Warnings)
	}
}

// The materialized library is editable on disk, and an edit shows up in the next
// composition with no restart. The materialized skill directory *is* the built-in
// layer, so an edit there composes without shadowing anything.
func TestMaterializedLibraryEditsCompose(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	// The library was materialized on start; edit a role skill in place.
	materialized := filepath.Join(h.DataDir, "skills", "research", "SKILL.md")
	if _, err := os.Stat(materialized); err != nil {
		t.Fatalf("library was not materialized: %v", err)
	}
	if err := os.WriteFile(materialized, []byte(skillSource("research", "", "EDITED-RESEARCH-SKILL on disk.")), 0o644); err != nil {
		t.Fatalf("editing materialized skill: %v", err)
	}

	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "research")
	research := findPart(t, p, "research")
	if !strings.Contains(segText(research), "EDITED-RESEARCH-SKILL") {
		t.Errorf("edit to the materialized library did not compose:\n%s", segText(research))
	}
	if got := research.Segments[0].Layer; got != "built-in" {
		t.Errorf("materialized base layer = %q, want built-in", got)
	}
}

// The preview refuses what it cannot compose, so a bad request is a response, not
// a surprise: an unknown or missing role, a missing space, map, or ticket.
func TestPayloadPreviewRejectsBadInput(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
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
