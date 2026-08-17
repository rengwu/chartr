package server_test

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// The prompt catalog and the per-space `At launch` selection at the process
// boundary (prompt-presets, ticket 01). The pane itself is Svelte; everything it
// is load-bearing for is server state — a preset that shows up in the snapshot,
// a selection remembered per space, a deletion that takes its references with
// it — so each of these asserts the action's effect on the snapshot an operator
// would read off that pane.

func promptIDs(m model.Model) []string {
	out := make([]string, 0, len(m.Prompts))
	for _, p := range m.Prompts {
		out = append(out, p.ID)
	}
	return out
}

func createPrompt(t *testing.T, h *chartrtest.Chartr, name, body string) string {
	t.Helper()
	code, resp := h.Post("/api/config/prompts", map[string]string{"name": name, "body": body})
	if code != 200 {
		t.Fatalf("create %q = %d, body %s", name, code, resp)
	}
	var r struct {
		ID string `json:"id"`
	}
	if err := json.Unmarshal([]byte(resp), &r); err != nil {
		t.Fatalf("create response not JSON: %v (%q)", err, resp)
	}
	return r.ID
}

// The whole of what the pane does to the catalog: create presets, see them in
// creation order, edit one in place, and delete one — none of it touching the
// operator's skills or preferences.
func TestPromptCatalogCRUDDrivesTheSnapshot(t *testing.T) {
	h := chartrtest.Start(t)

	brief := createPrompt(t, h, "Keep answers brief", "Answer in as few words as the question allows.")
	commits := createPrompt(t, h, "Commit convention", "Follow the repository's commit convention.")

	snap := h.Snapshot(ctx(t))
	if got := promptIDs(snap); len(got) != 2 || got[0] != brief || got[1] != commits {
		t.Fatalf("catalog = %v, want %v in creation order", got, []string{brief, commits})
	}
	if snap.Prompts[0].Name != "Keep answers brief" || !strings.HasPrefix(snap.Prompts[0].Body, "Answer in") {
		t.Errorf("first preset = %+v, want its name and body", snap.Prompts[0])
	}

	// An edit keeps the id and the row's place; only the text changes.
	if code, body := h.Put("/api/config/prompts/"+brief, map[string]string{
		"name": "Much briefer", "body": "One sentence.",
	}); code != 200 {
		t.Fatalf("edit = %d, body %s", code, body)
	}
	snap = h.Snapshot(ctx(t))
	if got := promptIDs(snap); len(got) != 2 || got[0] != brief {
		t.Fatalf("after an edit the catalog is %v, want the row where it was", got)
	}
	if snap.Prompts[0].Name != "Much briefer" || snap.Prompts[0].Body != "One sentence." {
		t.Errorf("edited preset = %+v, want the new name and body", snap.Prompts[0])
	}

	// Deleting drops the row and nothing else.
	if code, body := h.Delete("/api/config/prompts/" + brief); code != 200 {
		t.Fatalf("delete = %d, body %s", code, body)
	}
	if got := promptIDs(h.Snapshot(ctx(t))); len(got) != 1 || got[0] != commits {
		t.Errorf("after deletion the catalog is %v, want just %q", got, commits)
	}
}

// The refusals, each answered rather than silently absorbed: a preset needs both
// halves, and editing or deleting something that is not there is a 404.
func TestPromptCatalogRefusals(t *testing.T) {
	h := chartrtest.Start(t)

	if code, body := h.Post("/api/config/prompts", map[string]string{"name": "  ", "body": "text"}); code != 400 {
		t.Errorf("a nameless preset = %d (%s), want 400", code, body)
	}
	if code, body := h.Post("/api/config/prompts", map[string]string{"name": "Named", "body": " \n "}); code != 400 {
		t.Errorf("a bodyless preset = %d (%s), want 400", code, body)
	}
	if code, body := h.Put("/api/config/prompts/nope", map[string]string{"name": "A", "body": "b"}); code != 404 {
		t.Errorf("editing an unknown preset = %d (%s), want 404", code, body)
	}
	if code, body := h.Delete("/api/config/prompts/nope"); code != 404 {
		t.Errorf("deleting an unknown preset = %d (%s), want 404", code, body)
	}
	if got := promptIDs(h.Snapshot(ctx(t))); len(got) != 0 {
		t.Errorf("a refused action left %v in the catalog", got)
	}
}

// A catalog chartr could not read yields no presets, says so where the operator
// is looking, and refuses every mutation — the operator's own bytes are still
// exactly as they typed them afterwards.
func TestMalformedCatalogIsSurfacedAndNeverOverwritten(t *testing.T) {
	configDir := t.TempDir()
	catalog := filepath.Join(configDir, "prompts.toml")
	const original = "[[prompt]\nid = \"broken\"\n"
	if err := os.WriteFile(catalog, []byte(original), 0o600); err != nil {
		t.Fatalf("writing the catalog: %v", err)
	}

	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	space := register(t, h, chartrtest.NewSpaceRepo(t))

	snap := h.Snapshot(ctx(t))
	if got := promptIDs(snap); len(got) != 0 {
		t.Errorf("a malformed catalog offered %v, want no presets", got)
	}
	warned := false
	for _, w := range findSpace(t, snap, space.ID).Warnings {
		if strings.Contains(w, "prompts.toml") {
			warned = true
		}
	}
	if !warned {
		t.Errorf("no warning names the unreadable catalog: %v", findSpace(t, snap, space.ID).Warnings)
	}

	if code, body := h.Post("/api/config/prompts", map[string]string{"name": "A", "body": "b"}); code != 409 {
		t.Errorf("creating against a malformed catalog = %d (%s), want 409", code, body)
	}
	if code, body := h.Delete("/api/config/prompts/broken"); code != 409 {
		t.Errorf("deleting against a malformed catalog = %d (%s), want 409", code, body)
	}
	got, err := os.ReadFile(catalog)
	if err != nil || string(got) != original {
		t.Errorf("the operator's catalog is now %q (%v), want it untouched", got, err)
	}
}

// The `At launch` selection is remembered per space: one space's choice, in
// catalog order however it was sent, with every other space left alone.
func TestLaunchSelectionIsPerSpace(t *testing.T) {
	h := chartrtest.Start(t)
	brief := createPrompt(t, h, "Brief", "Keep it short.")
	commits := createPrompt(t, h, "Commits", "Use the convention.")

	alpha := register(t, h, chartrtest.NewSpaceRepo(t))
	beta := register(t, h, chartrtest.NewSpaceRepo(t))

	// Sent in the wrong order on purpose: catalog order is the only order.
	if code, body := h.Put("/api/spaces/"+alpha.ID+"/prompts", map[string]any{
		"ids": []string{commits, brief},
	}); code != 204 {
		t.Fatalf("selecting = %d, body %s", code, body)
	}

	snap := h.Snapshot(ctx(t))
	if got := findSpace(t, snap, alpha.ID).Prompts; len(got) != 2 || got[0] != brief || got[1] != commits {
		t.Errorf("alpha selects %v, want %v in catalog order", got, []string{brief, commits})
	}
	if got := findSpace(t, snap, beta.ID).Prompts; len(got) != 0 {
		t.Errorf("selecting in one space selected %v in another", got)
	}
	for _, s := range snap.Spaces {
		if s.Scratch && len(s.Prompts) != 0 {
			t.Errorf("Scratch carries a selection %v; it has none in this version", s.Prompts)
		}
	}

	// A selection is a whole-list write, so it also clears.
	if code, body := h.Put("/api/spaces/"+alpha.ID+"/prompts", map[string]any{"ids": []string{}}); code != 204 {
		t.Fatalf("clearing = %d, body %s", code, body)
	}
	if got := findSpace(t, h.Snapshot(ctx(t)), alpha.ID).Prompts; len(got) != 0 {
		t.Errorf("after clearing, alpha selects %v", got)
	}
}

// The selection survives a restart: it lives in the registry beside the space's
// other local state, so a chartr that starts against the same config root reads
// back exactly what the operator chose.
func TestLaunchSelectionSurvivesAReload(t *testing.T) {
	configDir := t.TempDir()
	repo := chartrtest.NewSpaceRepo(t)

	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	brief := createPrompt(t, h, "Brief", "Keep it short.")
	space := register(t, h, repo)
	if code, body := h.Put("/api/spaces/"+space.ID+"/prompts", map[string]any{
		"ids": []string{brief},
	}); code != 204 {
		t.Fatalf("selecting = %d, body %s", code, body)
	}

	next := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	snap := next.Snapshot(ctx(t))
	if got := promptIDs(snap); len(got) != 1 || got[0] != brief {
		t.Fatalf("the catalog reloaded as %v, want %v", got, []string{brief})
	}
	if got := findSpace(t, snap, space.ID).Prompts; len(got) != 1 || got[0] != brief {
		t.Errorf("the selection reloaded as %v, want %v", got, []string{brief})
	}
}

// Deleting a preset removes it from every space that had it selected, as part of
// the same action — no dangling id is left for a later launch to puzzle over.
func TestDeletingAPresetClearsItFromEverySpace(t *testing.T) {
	h := chartrtest.Start(t)
	brief := createPrompt(t, h, "Brief", "Keep it short.")
	doomed := createPrompt(t, h, "Doomed", "Not for long.")

	alpha := register(t, h, chartrtest.NewSpaceRepo(t))
	beta := register(t, h, chartrtest.NewSpaceRepo(t))
	for _, s := range []struct {
		id  string
		ids []string
	}{{alpha.ID, []string{brief, doomed}}, {beta.ID, []string{doomed}}} {
		if code, body := h.Put("/api/spaces/"+s.id+"/prompts", map[string]any{"ids": s.ids}); code != 204 {
			t.Fatalf("selecting in %s = %d, body %s", s.id, code, body)
		}
	}

	if code, body := h.Delete("/api/config/prompts/" + doomed); code != 200 {
		t.Fatalf("delete = %d, body %s", code, body)
	}

	snap := h.Snapshot(ctx(t))
	if got := findSpace(t, snap, alpha.ID).Prompts; len(got) != 1 || got[0] != brief {
		t.Errorf("alpha selects %v, want just the surviving preset", got)
	}
	if got := findSpace(t, snap, beta.ID).Prompts; len(got) != 0 {
		t.Errorf("beta selects %v, want nothing", got)
	}
	for _, s := range snap.Spaces {
		for _, w := range s.Warnings {
			if strings.Contains(w, doomed) {
				t.Errorf("an ordinary deletion left a warning behind: %q", w)
			}
		}
	}
}

// A selection naming a preset that is not in the catalog is refused rather than
// stored: it can only come from a stale client, and persisting it would record a
// choice the operator never made.
func TestSelectingAnUnknownPresetIsRefused(t *testing.T) {
	h := chartrtest.Start(t)
	brief := createPrompt(t, h, "Brief", "Keep it short.")
	space := register(t, h, chartrtest.NewSpaceRepo(t))

	if code, body := h.Put("/api/spaces/"+space.ID+"/prompts", map[string]any{
		"ids": []string{brief, "ghost"},
	}); code != 400 {
		t.Errorf("selecting an unknown preset = %d (%s), want 400", code, body)
	}
	if got := findSpace(t, h.Snapshot(ctx(t)), space.ID).Prompts; len(got) != 0 {
		t.Errorf("a refused selection stored %v", got)
	}
}

// Scratch is the home for ad-hoc shells and has no launch selection in this
// first version: the action is refused there the way every other repo-scoped one
// is, rather than quietly doing nothing.
func TestScratchTakesNoLaunchSelection(t *testing.T) {
	h := chartrtest.Start(t)
	brief := createPrompt(t, h, "Brief", "Keep it short.")

	if code, body := h.Put("/api/spaces/scratch/prompts", map[string]any{
		"ids": []string{brief},
	}); code != 400 {
		t.Errorf("selecting in Scratch = %d (%s), want 400", code, body)
	}
}

// A selected id the catalog no longer holds — a hand-edited registry, a deletion
// that half-finished — is skipped and named, never substituted with another
// preset.
func TestAMissingSelectedIDIsSurfacedNotSubstituted(t *testing.T) {
	configDir := t.TempDir()
	repo := chartrtest.NewSpaceRepo(t)

	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	brief := createPrompt(t, h, "Brief", "Keep it short.")
	space := register(t, h, repo)
	if code, body := h.Put("/api/spaces/"+space.ID+"/prompts", map[string]any{
		"ids": []string{brief},
	}); code != 204 {
		t.Fatalf("selecting = %d, body %s", code, body)
	}

	// The operator edits the registry by hand to name a preset that is not there.
	spaces := filepath.Join(configDir, "spaces.toml")
	data, err := os.ReadFile(spaces)
	if err != nil {
		t.Fatalf("reading the registry: %v", err)
	}
	edited := strings.Replace(string(data), `"`+brief+`"`, `"`+brief+`", "ghost"`, 1)
	if edited == string(data) {
		t.Fatalf("the registry does not record the selection: %s", data)
	}
	if err := os.WriteFile(spaces, []byte(edited), 0o600); err != nil {
		t.Fatalf("writing the registry: %v", err)
	}

	next := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	got := findSpace(t, next.Snapshot(ctx(t)), space.ID)
	if len(got.Prompts) != 1 || got.Prompts[0] != brief {
		t.Errorf("selection = %v, want only the preset that exists", got.Prompts)
	}
	named := false
	for _, w := range got.Warnings {
		if strings.Contains(w, "ghost") {
			named = true
		}
	}
	if !named {
		t.Errorf("the missing selection is not surfaced: %v", got.Warnings)
	}
}
