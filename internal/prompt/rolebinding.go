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
// a composition error before it writes the claim commit, so a role bound to
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
	if !strings.Contains(ref, "/") {
		return "", fmt.Errorf(
			"the %s role is bound to the bare name %q; a binding names a source, like %q, so that what the role runs does not follow whatever source order happens to be",
			role, ref, sources.DefaultBinding(ref))
	}
	return ref, nil
}

// resolveRoleSkill reads the skill a role is bound to out of the registered
// sources, and loads its body off disk. The error a failure returns is the whole
// user-facing account of why nothing spawned, so it carries the role, the binding
// as recorded, and the registry's own account of which of the three unresolvable
// shapes it is.
func resolveRoleSkill(reg *sources.Registry, bindings config.RoleBindings, role string) (Skill, error) {
	ref, err := bindingRef(bindings, role)
	if err != nil {
		return Skill{}, err
	}
	if reg == nil {
		return Skill{}, fmt.Errorf(
			"the %s role is bound to %q, but no source list was loaded, so nothing can resolve it", role, ref)
	}
	found, err := reg.Resolve(ref)
	if err != nil {
		return Skill{}, fmt.Errorf("the %s role is bound to %q, which resolves to nothing: %w", role, ref, err)
	}

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
