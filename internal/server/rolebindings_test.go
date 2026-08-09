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

// The third shape: a source that is registered but switched off. chartr's own
// stand-in source is disabled through the API, then a grilling spawn is refused
// naming the disabled source.
func TestSpawnRefusedWhenTheBoundSourceIsDisabled(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "grilling", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})

	if code, body := h.Put("/api/config/sources/"+chartrtest.SkillSource+"/enabled",
		map[string]any{"enabled": false}); code != 200 {
		t.Fatalf("disable = %d, body %s", code, body)
	}

	code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "grill", "claude")
	if code != 400 {
		t.Fatalf("spawn into a disabled source = %d, body %s", code, body)
	}
	for _, want := range []string{"grill role", chartrtest.SkillSource + "/grill", "disabled"} {
		if !strings.Contains(body, want) {
			t.Errorf("the refusal does not say %q:\n%s", want, body)
		}
	}
	if _, err := gitHEAD(repo); err == nil {
		t.Error("a refused spawn wrote a claim commit")
	}
}
