package prompt

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/sources"
)

// A `Source/skill` ref is a pin: it never falls back to another source's skill,
// even when an accepted alias exists elsewhere. Pinning bound/grill when only a
// later source ships a grill alias refuses rather than silently running it — that
// is the whole point of the qualified form, and "auto" is how an operator asks
// for the fallback instead.
func TestResolveRoleSkillPinNeverFallsBackToAnotherSource(t *testing.T) {
	dir := t.TempDir()
	reg, err := sources.Load(dir)
	if err != nil {
		t.Fatalf("loading sources: %v", err)
	}

	// The pinned source exists but lacks the exact skill named in the binding.
	bound := makeSkillsDir(t, "bound", []string{"prototype", "research", "implement"})
	if _, err := reg.RegisterDir("bound", bound); err != nil {
		t.Fatalf("registering bound source: %v", err)
	}

	// A later source has one of the grill aliases — which a pin must ignore.
	other := makeSkillsDir(t, "other", []string{"grill-me"})
	if _, err := reg.RegisterDir("other", other); err != nil {
		t.Fatalf("registering other source: %v", err)
	}

	bindings := config.RoleBindings{config.RoleGrill: "bound/grill"}
	_, err = resolveRoleSkill(reg, bindings, "grill")
	if err == nil {
		t.Fatal("a pin resolved through another source's alias instead of refusing")
	}
	if !strings.Contains(err.Error(), "bound/grill") {
		t.Errorf("the refusal does not name the pin: %v", err)
	}
}

// When the explicit binding fails and no accepted alias exists in any enabled
// source, resolution refuses with the explicit binding named in the error.
func TestResolveRoleSkillRefusesWhenAliasAlsoMissing(t *testing.T) {
	dir := t.TempDir()
	reg, err := sources.Load(dir)
	if err != nil {
		t.Fatalf("loading sources: %v", err)
	}

	src := makeSkillsDir(t, "src", []string{"prototype"})
	if _, err := reg.RegisterDir("src", src); err != nil {
		t.Fatalf("registering source: %v", err)
	}

	bindings := config.RoleBindings{config.RoleGrill: "src/grill"}
	_, err = resolveRoleSkill(reg, bindings, "grill")
	if err == nil {
		t.Fatal("expected resolution to fail")
	}
	if !strings.Contains(err.Error(), "src/grill") {
		t.Errorf("error does not name the explicit binding: %v", err)
	}
}

// An exact binding still wins over a later alias match.
func TestResolveRoleSkillPrefersExactBinding(t *testing.T) {
	dir := t.TempDir()
	reg, err := sources.Load(dir)
	if err != nil {
		t.Fatalf("loading sources: %v", err)
	}

	first := makeSkillsDir(t, "first", []string{"grill"})
	if _, err := reg.RegisterDir("first", first); err != nil {
		t.Fatalf("registering first source: %v", err)
	}

	bindings := config.RoleBindings{config.RoleGrill: "first/grill"}
	skill, err := resolveRoleSkill(reg, bindings, "grill")
	if err != nil {
		t.Fatalf("resolveRoleSkill: %v", err)
	}
	if skill.Source != "first" || skill.Name != "grill" {
		t.Errorf("exact resolved to %s/%s, want first/grill", skill.Source, skill.Name)
	}
}

// "auto" is the operator asking for precedence itself: no exact ref, just the
// first enabled source that ships a skill the role accepts.
func TestResolveRoleSkillAutoResolvesByPrecedence(t *testing.T) {
	dir := t.TempDir()
	reg, err := sources.Load(dir)
	if err != nil {
		t.Fatalf("loading sources: %v", err)
	}

	// Both sources ship a grill alias; precedence order is registration order.
	first := makeSkillsDir(t, "first", []string{"grill"})
	if _, err := reg.RegisterDir("first", first); err != nil {
		t.Fatalf("registering first source: %v", err)
	}
	second := makeSkillsDir(t, "second", []string{"grill-me"})
	if _, err := reg.RegisterDir("second", second); err != nil {
		t.Fatalf("registering second source: %v", err)
	}

	bindings := config.RoleBindings{config.RoleGrill: config.RoleBindingAuto}
	skill, err := resolveRoleSkill(reg, bindings, "grill")
	if err != nil {
		t.Fatalf("resolveRoleSkill: %v", err)
	}
	if skill.Source != "first" || skill.Name != "grill" {
		t.Errorf("auto resolved to %s/%s, want first/grill", skill.Source, skill.Name)
	}
}

// An "auto" binding with no source shipping a skill the role accepts refuses the
// spawn, and the refusal says it is a precedence resolution that found nothing —
// distinct from a pinned ref that resolves to nothing.
func TestResolveRoleSkillAutoRefusesWhenNothingMatches(t *testing.T) {
	dir := t.TempDir()
	reg, err := sources.Load(dir)
	if err != nil {
		t.Fatalf("loading sources: %v", err)
	}
	src := makeSkillsDir(t, "src", []string{"prototype"})
	if _, err := reg.RegisterDir("src", src); err != nil {
		t.Fatalf("registering source: %v", err)
	}

	bindings := config.RoleBindings{config.RoleGrill: config.RoleBindingAuto}
	_, err = resolveRoleSkill(reg, bindings, "grill")
	if err == nil {
		t.Fatal("expected an auto binding with no match to refuse")
	}
	if !strings.Contains(err.Error(), "precedence") {
		t.Errorf("the refusal does not name precedence: %v", err)
	}
}

func makeSkillsDir(t *testing.T, label string, names []string) string {
	t.Helper()
	root := filepath.Join(t.TempDir(), label)
	for _, name := range names {
		dir := filepath.Join(root, name)
		if err := os.MkdirAll(dir, 0o755); err != nil {
			t.Fatal(err)
		}
		src := "---\nname: " + name + "\ndescription: the " + name + " skill\n---\n\nbody of " + name + "\n"
		if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte(src), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	return root
}
