package config

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
)

// Role bindings: which skill each of the four roles spawns with, held as a flat
// scalar table in the operator's own `user.toml`, beside the agent library.
//
//	[roles]
//	grill = "chartr-skills/grill"
//	prototype = "chartr-skills/prototype"
//	research = "chartr-skills/research"
//	implement = "chartr-skills/implement"
//
// **Flat, not a subtable**: a binding is one fact, and a subtable holding exactly
// one key forever is ceremony. **Always qualified, never bare**: a bare name
// resolved through source order would reintroduce the implicit capture the table
// exists to prevent — reordering sources, or a higher source later shipping its
// own `grill`, would silently change what every grilling session runs with no line
// in the file showing it. This package holds the *string*; resolving it against
// the registered sources is the composer's job.
//
// The table is **seeded once**, on the first startup that finds no `[roles]` table
// at all, and **never auto-refilled**: a deleted row is a legitimate way to make a
// role refuse until it is rebound, so recovery is an explicit single-row action
// rather than something a restart quietly undoes.

// rolesFile is the binding half of the operator's config: a flat table of
// role → source-qualified skill reference.
type rolesFile struct {
	Roles map[string]string `toml:"roles"`
}

// RoleBindings is what the `[roles]` table said, keyed by role. Only the four
// known roles are carried: a stray key is the operator's to remove and means
// nothing to a spawn.
type RoleBindings map[Role]string

// ReadRoleBindings parses the `[roles]` table out of the user config. The second
// result reports whether the table is *present at all*, which is the seeding test
// — an operator upgrading with an existing `user.toml` seeds exactly like a new
// install, while one who deleted a single row keeps their gap.
//
// It never errors: an unparseable file reads as no table, which seeds a fresh one
// on the next startup and leaves the operator's bytes alone until then.
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

// SetUserRole writes one role's binding, returning the new user-config bytes. It
// goes through the same comment-preserving line surgery the agent library uses,
// so the operator's comments, key order and unrelated tables survive: this is the
// single-row rebind the settings surface offers for a row that was deleted or
// pointed at a source that is gone.
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
			"chartr-skills/"+ref, ref)
	}

	lines, eol := splitLines(existing)
	start, end, found := findTable(lines, []string{"roles"})
	if !found {
		return appendRolesTable(existing, RoleBindings{role: ref}), nil
	}
	lines, _ = setKeyInTable(lines, start, end, string(role), string(role)+" = "+strconv.Quote(ref), true)
	return []byte(strings.Join(lines, eol)), nil
}

// SeedRoleBindings writes the four default bindings if — and only if — the config
// carries no `[roles]` table at all, reporting whether it wrote. The presence test
// is the table rather than the file, and rather than any individual row: a
// half-filled table is the operator's, and refilling it would be exactly the
// auto-restore a deleted row is entitled not to get.
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

// appendRolesTable adds the `[roles]` table to a config that has none, in the
// same style as the agent-library writer: a blank line off whatever precedes it,
// then the table, with the roles in their display order. Unaligned, like every
// other key chartr writes — a column that only holds while chartr is the writer
// is a column an operator's own edit silently breaks.
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
