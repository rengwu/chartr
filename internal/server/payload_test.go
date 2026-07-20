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

// Ticket 08 at the process boundary: the prompt library and payload preview. The
// harness materializes a hackable prompt library, resolves each part across
// built-in ‹ user ‹ workspace with replace/append semantics, surfaces a
// replacement forked from an older shipped default, and composes core + role +
// context bundle into one payload — with the review role's Done-when and spec
// guaranteed by assembly. Every assertion is on the public payload the preview
// endpoint returns and on the files on disk; no test reaches into the package.

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

// writeUserPrompt drops a file into the operator's local prompt library (the user
// layer), under the harness data root.
func writeUserPrompt(t *testing.T, dataDir, name, body string) {
	t.Helper()
	dir := filepath.Join(dataDir, "prompts")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatalf("mkdir user prompts: %v", err)
	}
	if err := os.WriteFile(filepath.Join(dir, name), []byte(body), 0o644); err != nil {
		t.Fatalf("write user prompt %s: %v", name, err)
	}
}

// writeWorkspacePrompt drops a file into the committed workspace prompt overlay
// inside the space's repo.
func writeWorkspacePrompt(t *testing.T, repo, name, body string) {
	t.Helper()
	harnesstest.WriteFile(t, repo, filepath.Join(".wayfinder-harness", "prompts", name), body)
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

// Resolution walks built-in ‹ user ‹ workspace with `append` stacking and
// `replace` resetting the base — and, for a replace, the highest layer wins
// (committed workspace over local user), the content half of ADR 0009. The whole
// matrix is observable in the resolved core part's segments and their layer tags.
func TestPromptResolutionMatrix(t *testing.T) {
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

	// Append stacks at both overlay layers, in precedence order after the base.
	writeUserPrompt(t, h.DataDir, "core.append.md", "USER-APPEND-LINE")
	writeWorkspacePrompt(t, repo, "core.append.md", "WORKSPACE-APPEND-LINE")
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "grill")
	core := findPart(t, p, "core")
	if got := segLayers(core); !equalStrings(got, []string{"built-in", "user", "workspace"}) {
		t.Errorf("append layers = %v, want [built-in user workspace]", got)
	}
	txt := segText(core)
	if !strings.Contains(txt, "USER-APPEND-LINE") || !strings.Contains(txt, "WORKSPACE-APPEND-LINE") {
		t.Errorf("appends not stacked:\n%s", txt)
	}
	if !strings.Contains(txt, "wayfinder-harness session") {
		t.Errorf("append dropped the shipped base:\n%s", txt)
	}

	// A user replace resets the base — the shipped default is gone — while the
	// appends still stack after it.
	writeUserPrompt(t, h.DataDir, "core.replace.md", "USER-REPLACED-CORE")
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "grill")
	core = findPart(t, p, "core")
	txt = segText(core)
	if strings.Contains(txt, "wayfinder-harness session") {
		t.Errorf("user replace did not reset the base:\n%s", txt)
	}
	if !strings.Contains(txt, "USER-REPLACED-CORE") || !strings.Contains(txt, "WORKSPACE-APPEND-LINE") {
		t.Errorf("replace/append composition wrong:\n%s", txt)
	}
	if got := core.Segments[0].Layer; got != "user" {
		t.Errorf("first segment after user replace = %q, want user", got)
	}

	// A committed workspace replace wins over the user replace (content half of
	// ADR 0009): it resets the base above the user layer.
	writeWorkspacePrompt(t, repo, "core.replace.md", "WORKSPACE-REPLACED-CORE")
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "grill")
	core = findPart(t, p, "core")
	txt = segText(core)
	if strings.Contains(txt, "USER-REPLACED-CORE") {
		t.Errorf("workspace replace did not win over user replace:\n%s", txt)
	}
	if !strings.Contains(txt, "WORKSPACE-REPLACED-CORE") {
		t.Errorf("workspace replace not applied:\n%s", txt)
	}
	if got := core.Segments[0].Layer; got != "workspace" {
		t.Errorf("first segment after workspace replace = %q, want workspace", got)
	}
}

// A replacement that records the shipped default it forked from is surfaced as
// behind when that default has since moved on — never auto-merged (story 47). The
// notice rides both the space snapshot and the preview; a fork recording the
// current shipped hash draws no warning.
func TestBehindDefaultSurfaced(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	// A stale fork: a replace marked with a hash that is not the shipped one.
	writeUserPrompt(t, h.DataDir, "implement.replace.md",
		"<!-- forked from deadbeef -->\nMY OWN IMPLEMENT PROMPT")

	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "implement")
	if !hasSubstring(p.Warnings, "forked from an older shipped default") {
		t.Errorf("behind-default not surfaced in preview warnings: %v", p.Warnings)
	}
	// The marker line is stripped from the composed prompt — meta never leaks into
	// the payload.
	if strings.Contains(segText(findPart(t, p, "implement")), "forked from") {
		t.Errorf("fork marker leaked into the payload:\n%s", segText(findPart(t, p, "implement")))
	}
	// It is also surfaced on the space, so a stale fork is visible without opening
	// the preview. The library lives under the data root, not the watched `.plan/`,
	// so it refreshes on the next rebuild rather than by notice — force one.
	if code, body := h.Post(fmt.Sprintf("/api/spaces/%s/pin", resp.ID), map[string]bool{"pinned": true}); code != 204 {
		t.Fatalf("pin to force a rebuild = %d, body %s", code, body)
	}
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if !hasSubstring(s.Warnings, "forked from an older shipped default") {
		t.Errorf("behind-default not surfaced on the space: %v", s.Warnings)
	}

	// A fork recording the *current* shipped hash is owned, not behind.
	writeUserPrompt(t, h.DataDir, "implement.replace.md",
		fmt.Sprintf("<!-- forked from %s -->\nMY OWN IMPLEMENT PROMPT", prompt.DefaultHash("implement")))
	_, p, _ = getPayload(t, h, resp.ID, "widget", 1, "implement")
	if hasSubstring(p.Warnings, "forked from an older shipped default") {
		t.Errorf("a fork on the current default should not warn: %v", p.Warnings)
	}
}

// The review payload always carries the ticket's Done-when and the spec by
// assembly (story 53), so a reviewer can never be handed only a diff. The spec is
// discovered from the map's own link to a spec.md, not a hard-coded path. Neither
// part is added for a non-review role.
func TestReviewPayloadCarriesDoneWhenAndSpec(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	// The map names its spec in its own body — the harness follows the link.
	reviewMap := "# Impl Map\n\n## Destination\nBuild it. The [spec](../shared/spec.md) is the source of truth.\n\n## Decisions so far\n"
	harnesstest.WriteMap(t, repo, "builder", reviewMap)
	harnesstest.WriteFile(t, repo, filepath.Join(".plan", "shared", "spec.md"),
		"# Spec\n\nThe one true SPEC-CONTENT-MARKER.\n")

	ticketBody := "---\ntype: task\nblocked_by: []\n---\n\n# Gate me\n\n" +
		"## Question\nDo the thing.\n\nDone when: THE-DONE-WHEN-MARKER holds and tests pass.\n"
	harnesstest.WriteTicket(t, repo, "builder", "01-gate-me.md", ticketBody)

	resp := register(t, h, repo)

	// Review: Done-when and spec both present, by assembly.
	_, rp, body := getPayload(t, h, resp.ID, "builder", 1, "review")
	if !hasPart(rp, "done-when") {
		t.Fatalf("review payload has no Done-when part; parts: %v\n%s", partNames(rp), body)
	}
	if !strings.Contains(segText(findPart(t, rp, "done-when")), "THE-DONE-WHEN-MARKER") {
		t.Errorf("Done-when part missing its content:\n%s", segText(findPart(t, rp, "done-when")))
	}
	if !hasPart(rp, "spec") {
		t.Fatalf("review payload has no spec part; parts: %v", partNames(rp))
	}
	if !strings.Contains(segText(findPart(t, rp, "spec")), "SPEC-CONTENT-MARKER") {
		t.Errorf("spec part did not resolve the linked spec.md:\n%s", segText(findPart(t, rp, "spec")))
	}
	// The guarantee is provable in the single composed document too.
	if !strings.Contains(rp.Markdown, "THE-DONE-WHEN-MARKER") || !strings.Contains(rp.Markdown, "SPEC-CONTENT-MARKER") {
		t.Errorf("composed review markdown missing Done-when or spec")
	}

	// The guarantee is review-specific: an implement payload adds neither part.
	_, ip, _ := getPayload(t, h, resp.ID, "builder", 1, "implement")
	if hasPart(ip, "done-when") || hasPart(ip, "spec") {
		t.Errorf("non-review payload should not carry the review guarantees; parts: %v", partNames(ip))
	}
}

// The materialized library is editable on disk, and an edit shows up in the next
// composition with no restart (Done-when). The materialized file is the base the
// composition starts from.
func TestMaterializedLibraryEditsCompose(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	// The library was materialized on start; edit a role prompt in place.
	materialized := filepath.Join(h.DataDir, "prompts", "research.md")
	if _, err := os.Stat(materialized); err != nil {
		t.Fatalf("library was not materialized: %v", err)
	}
	if err := os.WriteFile(materialized, []byte("EDITED-RESEARCH-PROMPT on disk."), 0o644); err != nil {
		t.Fatalf("editing materialized prompt: %v", err)
	}

	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "research")
	research := findPart(t, p, "research")
	if !strings.Contains(segText(research), "EDITED-RESEARCH-PROMPT") {
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
