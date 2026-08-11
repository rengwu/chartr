package prompt

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/sources"
)

// Resolving a role to the skill it spawns with, through the operator's binding
// table and their ordered source list — the swap that replaces the layer model's
// `Resolve(role, roots)` for the role half of a payload.
//
// Everything here is refusal-shaped on purpose. **An unresolvable binding refuses
// the spawn**: composition returns an error, and the spawn path already aborts on
// a composition error before it writes the claim, so a role bound to
// nothing costs the operator a message and never a half-claimed ticket. The error
// names the role, the recorded binding string, and which shape it hit — a source
// that is disabled, a source that is gone, or a skill missing from the source
// named — because those three are fixed in three different places.

// bindingRef is one role's recorded binding, validated as a qualified reference.
// A bare name is refused here rather than resolved through source order: the
// whole point of the table is that what a role runs is readable in one line.
func bindingRef(bindings config.RoleBindings, role string) (string, error) {
	ref := strings.TrimSpace(bindings[config.Role(role)])
	if ref == "" {
		return "", fmt.Errorf(
			"the %s role has no binding: `[roles]` in your user config names no skill for it, so there is nothing to spawn with — bind it in settings",
			role)
	}
	if ref == config.RoleBindingAuto {
		return ref, nil
	}
	if !strings.Contains(ref, "/") {
		return "", fmt.Errorf(
			"the %s role is bound to the bare name %q; a binding names a source, like %q, so that what the role runs does not follow whatever source order happens to be",
			role, ref, "Source/"+ref)
	}
	return ref, nil
}

// resolveRoleSkill reads the skill a role is bound to out of the registered
// sources, and loads its body off disk. The error a failure returns is the whole
// user-facing account of why nothing spawned, so it carries the role, the binding
// as recorded, and the registry's own account of which of the three unresolvable
// shapes it is.
//
// The operator's explicit binding is tried first. If it does not resolve, the
// role's accepted skill names are searched in source order so a repo that ships
// any of them works without hand-editing the binding.
func resolveRoleSkill(reg *sources.Registry, bindings config.RoleBindings, role string) (Skill, error) {
	ref, err := bindingRef(bindings, role)
	if err != nil {
		return Skill{}, err
	}
	if reg == nil {
		return Skill{}, fmt.Errorf(
			"the %s role is bound to %q, but no source list was loaded, so nothing can resolve it", role, ref)
	}

	// "auto" is the operator asking for precedence itself: no exact ref to try,
	// just the first enabled source that ships a skill the role accepts.
	if ref == config.RoleBindingAuto {
		if alias, ok := resolveAlias(reg, config.SkillAliases(config.Role(role))); ok {
			return loadSkill(reg, alias, role, ref)
		}
		return Skill{}, fmt.Errorf(
			"the %s role is set to resolve by precedence (%q), but no enabled source ships a skill it accepts", role, ref)
	}

	// A `Source/skill` ref is a pin: it resolves to exactly that source's skill or
	// it refuses. There is deliberately no alias fallback here — that would make a
	// pin into a disabled or gone source silently run a *different* source's skill,
	// which is the whole failure the qualified form exists to prevent. An operator
	// who wants precedence chooses "auto" instead.
	found, err := reg.Resolve(ref)
	if err == nil {
		return loadSkill(reg, found, role, ref)
	}
	return Skill{}, fmt.Errorf("the %s role is bound to %q, which resolves to nothing: %w", role, ref, err)
}

// resolveAlias searches enabled sources in resolution order for the first skill
// whose name matches one of the accepted names, case-insensitively.
func resolveAlias(reg *sources.Registry, names []string) (sources.Skill, bool) {
	for _, s := range reg.List() {
		if !s.Enabled {
			continue
		}
		st, ok := reg.Walk(s.Name)
		if !ok {
			continue
		}
		for _, sk := range st.Skills {
			for _, n := range names {
				if strings.EqualFold(sk.Name, n) {
					return sk, true
				}
			}
		}
	}
	return sources.Skill{}, false
}

// loadSkill reads a discovered skill's SKILL.md and builds the Skill value the
// payload carries. The recorded binding string is kept in errors so the operator
// sees what they wrote even when the body came from an alias fallback.
func loadSkill(reg *sources.Registry, found sources.Skill, role, ref string) (Skill, error) {
	body, err := os.ReadFile(filepath.Join(found.Dir, skillFile))
	if err != nil {
		return Skill{}, fmt.Errorf("the %s role is bound to %q, whose %s could not be read: %w", role, ref, skillFile, err)
	}
	meta, text := splitFrontmatter(string(body))

	s := Skill{
		Name:        found.Name,
		Source:      found.Source,
		Dir:         found.Dir,
		Description: meta["description"],
		Body:        strings.TrimSpace(text),
	}
	// The commit rides onto the claim trailer where the source carries one, so a
	// teammate reading the history can fetch the exact skill bytes a session was
	// told. A `dir` source has none, and the trailer simply names the source.
	if src, ok := reg.Get(found.Source); ok {
		s.Commit = src.Commit
	}
	return s, nil
}
