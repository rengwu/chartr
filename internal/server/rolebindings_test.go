package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
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
			binding: "mine/grill",
			wantIn:  []string{"grill role", "mine/grill", "has no skill"},
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			// Start without the seeded stand-in source, so the only binding in play is
			// the pin under test.
			h := chartrtest.Start(t, chartrtest.WithoutSkills())
			repo := chartrtest.NewSpaceRepo(t)
			chartrtest.WriteMap(t, repo, "widget", mapBody)
			chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "grilling", ""))
			chartrtest.StubAgent(t, "claude")

			src := t.TempDir()
			writeSkill(t, src, "spelunk", "---\nname: spelunk\n---\n\na skill with no grill alias\n")
			if code, body := h.Post("/api/config/sources", map[string]string{
				"name": "mine", "kind": "dir", "path": src,
			}); code != 200 {
				t.Fatalf("registering source: %d %s", code, body)
			}

			resp := register(t, h, repo)
			registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})

			// Pin the grill role to a skill the source does not carry. A pin never
			// falls back to another source, so the spawn refuses.
			cfg := userConfig(t, h.ConfigDir)
			cfg = cfg + "\n[roles]\ngrill = \"" + tc.binding + "\"\n"
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

			// Nothing was claimed: the audit log is empty and the ticket is takeable.
			if n := auditCount(t, repo); n != 0 {
				t.Errorf("a refused spawn recorded %d audit entries, want none", n)
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
	if n := auditCount(t, repo); n != 0 {
		t.Errorf("a refused spawn recorded %d audit entries, want none", n)
	}
}

// An "auto" binding resolves by precedence: the role's accepted skill names are
// searched across enabled sources in order, so any repo that ships the right
// names satisfies the role with no exact ref to hand-pick.
func TestSpawnResolvesThroughAutoBinding(t *testing.T) {
	h := chartrtest.Start(t, chartrtest.WithoutSkills())
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "grilling", ""))
	chartrtest.StubAgent(t, "claude")

	// Register a source that has no skill named "grill" but does have an accepted
	// grill alias.
	src := t.TempDir()
	writeSkill(t, src, "grill-me", "---\nname: grill-me\n---\n\nalias-grill-body\n")
	if code, body := h.Post("/api/config/sources", map[string]string{
		"name": "mine", "kind": "dir", "path": src,
	}); code != 200 {
		t.Fatalf("registering source: %d %s", code, body)
	}

	resp := register(t, h, repo)
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})

	// Set the role to resolve by precedence; it should find mine/grill-me by alias
	// and spawn successfully with no exact ref named.
	cfg := userConfig(t, h.ConfigDir)
	cfg = cfg + "\n[roles]\ngrill = \"auto\"\n"
	chartrtest.WriteFile(t, h.ConfigDir, "user.toml", cfg)

	code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "grill", "claude")
	if code != 200 {
		t.Fatalf("spawn through auto resolution = %d, body %s", code, body)
	}

	// The payload preview shows the resolved skill body, not the spawn response.
	_, p, _ := getPayload(t, h, resp.ID, "widget", 1, "grill")
	grill := findPart(t, p, "grill")
	if grill.Origin != "mine" {
		t.Errorf("alias fallback resolved from %s, want mine", grill.Origin)
	}
	if len(p.Skills) < 2 || p.Skills[1].Name != "grill-me" {
		t.Errorf("alias fallback resolved to skill %q, want grill-me", p.Skills[1].Name)
	}
	if !strings.Contains(grill.Text, "alias-grill-body") {
		t.Errorf("spawn did not run the alias skill body:\n%s", grill.Text)
	}

	// The claim was recorded because the spawn succeeded.
	if n := auditCount(t, repo); n != 1 {
		t.Errorf("a successful spawn recorded %d audit entries, want the one claim", n)
	}
}

// The role picker's data and its one write: every enabled source that ships a
// skill the role accepts is offered as a candidate in precedence order, "no
// preference" writes the auto sentinel that resolves by that order, and pinning a
// lower-precedence source overrides it. An unknown role is refused.
func TestRolePickerListsCandidatesAndBinds(t *testing.T) {
	h := chartrtest.Start(t, chartrtest.WithoutSkills())

	a := t.TempDir()
	writeSkill(t, a, "grill", "---\nname: grill\n---\n\nbody-a\n")
	b := t.TempDir()
	writeSkill(t, b, "grill-me", "---\nname: grill-me\n---\n\nbody-b\n")
	for _, s := range []struct{ name, path string }{{"repo-a", a}, {"repo-b", b}} {
		if code, body := h.Post("/api/config/sources", map[string]string{
			"name": s.name, "kind": "dir", "path": s.path,
		}); code != 200 {
			t.Fatalf("registering %s: %d %s", s.name, code, body)
		}
	}

	// Both grill-alias skills are offered, in precedence (registration) order.
	grill := findRoleBinding(t, h.Snapshot(ctx(t)).Roles, "grill")
	if got := grill.Candidates; len(got) != 2 || got[0] != "repo-a/grill" || got[1] != "repo-b/grill-me" {
		t.Fatalf("grill candidates = %v, want [repo-a/grill repo-b/grill-me]", got)
	}

	// "No preference" writes the auto sentinel and resolves by precedence.
	if code, body := h.Put("/api/config/roles/grill", map[string]string{"ref": "auto"}); code != 200 {
		t.Fatalf("bind auto: %d %s", code, body)
	}
	grill = findRoleBinding(t, h.Snapshot(ctx(t)).Roles, "grill")
	// Auto resolves to the precedence winner — the higher source's skill.
	if grill.Ref != "auto" || grill.Resolved != "repo-a/grill" || !grill.Resolves {
		t.Errorf("after auto: ref=%q resolved=%q resolves=%v, want auto/repo-a/grill/true",
			grill.Ref, grill.Resolved, grill.Resolves)
	}

	// Pinning the lower-precedence source overrides the order: it resolves to
	// exactly that skill, not the precedence winner.
	if code, body := h.Put("/api/config/roles/grill", map[string]string{"ref": "repo-b/grill-me"}); code != 200 {
		t.Fatalf("pin repo-b: %d %s", code, body)
	}
	grill = findRoleBinding(t, h.Snapshot(ctx(t)).Roles, "grill")
	if grill.Ref != "repo-b/grill-me" || grill.Resolved != "repo-b/grill-me" || !grill.Resolves {
		t.Errorf("after pin: ref=%q resolved=%q resolves=%v, want repo-b/grill-me pinned and resolved",
			grill.Ref, grill.Resolved, grill.Resolves)
	}

	// Disabling the pinned source makes the pin unresolved — no fallback to the
	// still-enabled repo-a, because a pin is a pin.
	if code, body := h.Put("/api/config/sources/repo-b/enabled", map[string]any{"enabled": false}); code != 200 {
		t.Fatalf("disable repo-b: %d %s", code, body)
	}
	grill = findRoleBinding(t, h.Snapshot(ctx(t)).Roles, "grill")
	if grill.Resolved != "" || grill.Resolves {
		t.Errorf("pin into a disabled source resolved to %q (resolves %v), want unresolved",
			grill.Resolved, grill.Resolves)
	}

	// An unknown role is refused before any write.
	if code, _ := h.Put("/api/config/roles/bogus", map[string]string{"ref": "auto"}); code != 400 {
		t.Errorf("binding a bogus role = %d, want 400", code)
	}
}

func findRoleBinding(t *testing.T, rows []model.RoleBinding, role string) model.RoleBinding {
	t.Helper()
	for _, r := range rows {
		if r.Role == role {
			return r
		}
	}
	t.Fatalf("no %q role in the snapshot", role)
	return model.RoleBinding{}
}
