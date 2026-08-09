package config

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
)

// Role bindings: which skill each of the four roles spawns with, held as a
// flat scalar table in the operator's own `user.toml`, beside the agent
// library.
//
//	[roles]
//	grill = "my-skills/grill"
//	prototype = "my-skills/prototype"
//	research = "my-skills/research"
//	implement = "my-skills/implement"
//
// where `my-skills` is a source the operator registered — chartr ships none
// of its own (ADR 0017).
//
// Flat, not a subtable — a binding is one fact. Always qualified, never
// bare — a bare name resolved through source order would silently change
// what a role runs whenever sources reorder or a higher source ships its
// own `grill`, with no line in the file showing it. This package holds the
// string; resolving it against sources is the composer's job.
//
// chartr writes no bindings on a first run: with no shipped skills there is
// nothing to bind to, so every role starts unbound and refuses its spawn
// until the operator binds it against a source they registered.

// rolesFile is the binding half of the operator's config: a flat table of
// role → source-qualified skill reference.
type rolesFile struct {
	Roles map[string]string `toml:"roles"`
}

// RoleBindings is what the `[roles]` table said, keyed by role. Only the
// four known roles are carried — a stray key is the operator's to remove
// and means nothing to a spawn.
type RoleBindings map[Role]string

// ReadRoleBindings parses the `[roles]` table out of the user config. The
// second result reports whether the table is present at all — the seeding
// test: an operator upgrading with an existing `user.toml` seeds like a new
// install, while one who deleted a row keeps their gap.
//
// Never errors: an unparseable file reads as no table, which seeds a fresh
// one next startup and leaves the operator's bytes alone until then.
func ReadRoleBindings(userTOML []byte) (RoleBindings, bool) {
	var rf rolesFile
	if len(userTOML) == 0 || !decodeTOML(userTOML, &rf) {
		return RoleBindings{}, false
	}
	if rf.Roles == nil {
		return RoleBindings{}, false
	}
	out := RoleBindings{}
	for _, r := range Roles {
		if v := strings.TrimSpace(rf.Roles[string(r)]); v != "" {
			out[r] = v
		}
	}
	return out, true
}

// SetUserRole writes one role's binding, returning the new user-config
// bytes. Goes through the same comment-preserving line surgery the agent
// library uses, so comments, key order and unrelated tables survive — the
// single-row rebind the settings surface offers for a deleted or
// gone-source row.
func SetUserRole(existing []byte, role Role, ref string) ([]byte, error) {
	if !IsRole(string(role)) {
		return nil, fmt.Errorf("%q is not a role; want one of %v", role, Roles)
	}
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return nil, fmt.Errorf("the %s role needs a skill to bind to", role)
	}
	if !strings.Contains(ref, "/") {
		return nil, fmt.Errorf(
			"a role binds to a source-qualified skill like %q, not the bare name %q — a bare name would follow whatever source order happens to be",
			"Source/"+ref, ref)
	}

	lines, eol := splitLines(existing)
	start, end, found := findTable(lines, []string{"roles"})
	if !found {
		return appendRolesTable(existing, RoleBindings{role: ref}), nil
	}
	lines, _ = setKeyInTable(lines, start, end, string(role), string(role)+" = "+strconv.Quote(ref), true)
	return []byte(strings.Join(lines, eol)), nil
}

// SeedRoleBindings writes the four default bindings only if the config
// carries no `[roles]` table at all, reporting whether it wrote. Tests for
// the table, not any individual row — a half-filled table is the
// operator's, and refilling it would be the auto-restore a deleted row is
// entitled not to get.
func SeedRoleBindings(existing []byte, binding func(Role) string) ([]byte, bool) {
	if _, present := ReadRoleBindings(existing); present {
		return existing, false
	}
	seed := RoleBindings{}
	for _, r := range Roles {
		seed[r] = binding(r)
	}
	return appendRolesTable(existing, seed), true
}

// appendRolesTable adds the `[roles]` table to a config that has none, in
// the same style as the agent-library writer: a blank line, then the
// table, roles in display order, unaligned like every other key chartr
// writes — a column an operator's own edit would silently break.
func appendRolesTable(existing []byte, bindings RoleBindings) []byte {
	var b bytes.Buffer
	b.Write(existing)
	if n := len(existing); n > 0 {
		if existing[n-1] != '\n' {
			b.WriteByte('\n')
		}
		b.WriteByte('\n')
	}
	b.WriteString("[roles]\n")
	for _, r := range Roles {
		ref, ok := bindings[r]
		if !ok {
			continue
		}
		fmt.Fprintf(&b, "%s = %s\n", r, strconv.Quote(ref))
	}
	return b.Bytes()
}
