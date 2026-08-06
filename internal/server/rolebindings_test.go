package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The seed and the role bindings at the process boundary. Both of these writes
// are *silent* — no UI reports them, nothing asks the operator's permission —
// so a test is the only place they are visible at all, which is why they are
// asserted here rather than in a unit.

// seededRoles is the `[roles]` table startup writes. Tests that replace the whole
// `user.toml` carry it, because bindings are seeded once and never auto-refilled.
const seededRoles = "[roles]\ngrill = \"chartr-skills/grill\"\nprototype = \"chartr-skills/prototype\"\nresearch = \"chartr-skills/research\"\nimplement = \"chartr-skills/implement\"\n"

func userConfig(t *testing.T, configDir string) string {
	t.Helper()
	b, err := os.ReadFile(filepath.Join(configDir, "user.toml"))
	if err != nil {
		t.Fatalf("reading user.toml: %v", err)
	}
	return string(b)
}

// A clean config root, with nothing fetched: the seed materializes, the four
// bindings are written qualified, and a grilling ticket composes a payload
// carrying the seed's own grill body. This is the offline first run — nothing in
// it touches the network, which is the entire reason the seed exists.
func TestFirstRunSeedsTheSkillsAndTheBindings(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "grilling", ""))
	resp := register(t, h, repo)

	// The seed is on disk under the default source's path.
	seedDir := filepath.Join(h.ConfigDir, "sources", "chartr-skills")
	for _, name := range []string{"grill", "prototype", "research", "implement", "wayfinder", "to-spec", "to-tickets"} {
		if _, err := os.Stat(filepath.Join(seedDir, name, "SKILL.md")); err != nil {
			t.Errorf("the seed is missing %s: %v", name, err)
		}
	}

	// The bindings are written, and every one of them is source-qualified.
	cfg := userConfig(t, h.ConfigDir)
	for _, want := range []string{
		`grill = "chartr-skills/grill"`,
		`prototype = "chartr-skills/prototype"`,
		`research = "chartr-skills/research"`,
		`implement = "chartr-skills/implement"`,
	} {
		if !strings.Contains(cfg, want) {
			t.Errorf("user.toml missing %s:\n%s", want, cfg)
		}
	}

	// And a grilling ticket composes with the seed's grill body, not a layer's.
	code, p, body := getPayload(t, h, resp.ID, "widget", 1, "grill")
	if code != 200 {
		t.Fatalf("payload preview = %d, body %s", code, body)
	}
	shipped, err := os.ReadFile(filepath.Join(seedDir, "grill", "SKILL.md"))
	if err != nil {
		t.Fatal(err)
	}
	marker := "An option nobody argued against was never tested"
	if !strings.Contains(string(shipped), marker) {
		t.Fatalf("the seed's grill skill no longer carries the marker this test reads for")
	}
	if !strings.Contains(segText(findPart(t, p, "grill")), marker) {
		t.Errorf("the composed grill part is not the seed's body:\n%s", segText(findPart(t, p, "grill")))
	}
	if got := findPart(t, p, "grill").Origin; got != "chartr-skills" {
		t.Errorf("grill provenance = %q, want chartr-skills", got)
	}
}

// A binding that resolves to nothing refuses the spawn outright — no terminal, no
// claim commit — and the refusal names the role, the binding as recorded, and
// which of the three unresolvable shapes it hit, because each is fixed somewhere
// different.
func TestUnresolvableBindingRefusesTheSpawnWithoutClaiming(t *testing.T) {
	for _, tc := range []struct {
		name, binding string
		wantIn        []string
	}{
		{
			name:    "source removed",
			binding: "gone/grill",
			wantIn:  []string{"grill role", "gone/grill", "no source named"},
		},
		{
			name:    "skill missing from the source",
			binding: "chartr-skills/nonesuch",
			wantIn:  []string{"grill role", "chartr-skills/nonesuch", "has no skill"},
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := chartrtest.Start(t)
			repo := chartrtest.NewSpaceRepo(t)
			chartrtest.WriteMap(t, repo, "widget", mapBody)
			chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "grilling", ""))
			chartrtest.StubAgent(t, "claude")
			resp := register(t, h, repo)
			registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})

			// Rebind by hand. Bindings are read fresh at every composition, so this
			// reaches the very next spawn with no restart.
			cfg := userConfig(t, h.ConfigDir)
			cfg = strings.Replace(cfg, `grill = "chartr-skills/grill"`, `grill = "`+tc.binding+`"`, 1)
			chartrtest.WriteFile(t, h.ConfigDir, "user.toml", cfg)

			code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "grill", "claude")
			if code != 400 {
				t.Fatalf("spawn on an unresolvable binding = %d, body %s", code, body)
			}
			for _, want := range tc.wantIn {
				if !strings.Contains(body, want) {
					t.Errorf("the refusal does not say %q:\n%s", want, body)
				}
			}

			// Nothing was claimed: HEAD is still unborn and the ticket is takeable.
			if _, err := gitHEAD(repo); err == nil {
				t.Error("a refused spawn wrote a claim commit")
			}
			tk := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1)
			if tk.Status != "open" || !tk.Frontier {
				t.Errorf("ticket after a refused spawn = %q (frontier %v), want open and on the frontier", tk.Status, tk.Frontier)
			}
		})
	}
}

// The third shape: a source that is registered but switched off. It needs the
// list to be in that state before the server loads it, so the sources file is
// written first and the server started over it.
func TestSpawnRefusedWhenTheBoundSourceIsDisabled(t *testing.T) {
	configDir := t.TempDir()
	if err := os.WriteFile(filepath.Join(configDir, "sources.toml"), []byte("default_enabled = false\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "grilling", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})

	code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "grill", "claude")
	if code != 400 {
		t.Fatalf("spawn into a disabled source = %d, body %s", code, body)
	}
	for _, want := range []string{"grill role", "chartr-skills/grill", "disabled"} {
		if !strings.Contains(body, want) {
			t.Errorf("the refusal does not say %q:\n%s", want, body)
		}
	}
	if _, err := gitHEAD(repo); err == nil {
		t.Error("a refused spawn wrote a claim commit")
	}
}

// Deleting the seeded directory is the whole reset story: the next startup
// re-materializes it, and a `.git` inside it stops chartr writing there at all.
func TestSeedResetsOnDeleteAndStopsAtAGitDirectory(t *testing.T) {
	configDir := t.TempDir()
	chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	seedDir := filepath.Join(configDir, "sources", "chartr-skills")
	grill := filepath.Join(seedDir, "grill", "SKILL.md")
	shipped, err := os.ReadFile(grill)
	if err != nil {
		t.Fatalf("the first start did not materialize the seed: %v", err)
	}

	if err := os.RemoveAll(seedDir); err != nil {
		t.Fatal(err)
	}
	chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	if b, err := os.ReadFile(grill); err != nil || string(b) != string(shipped) {
		t.Errorf("a restart did not re-materialize the deleted seed: %v", err)
	}

	// Pinned: the operator's own bytes, which a restart must not revert.
	if err := os.MkdirAll(filepath.Join(seedDir, ".git"), 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(grill, []byte("MY OWN GRILL"), 0o600); err != nil {
		t.Fatal(err)
	}
	chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	if b, _ := os.ReadFile(grill); string(b) != "MY OWN GRILL" {
		t.Errorf("a restart wrote into a pinned checkout: %q", b)
	}
}
