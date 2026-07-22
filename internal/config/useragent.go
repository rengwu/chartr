package config

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
)

// The write half of the agent library. It shares SetUserBinding's stance and its
// TOML line surgery (userbinding.go): the operator's file is *theirs* — comments,
// key order, spacing, and unrelated tables survive every edit — and a shape this
// editor does not understand is refused with a pointer at a hand edit rather than
// rewritten blind.
//
// Registering an agent writes the global `[agents.<name>]` table in the user's own
// config, never a repository's committed one. That is the whole safety property of
// the library's placement: an agent carrying `--dangerously-skip-permissions` is
// something an operator grants themselves on one machine, and cannot arrive from a
// `git pull`.

// SetUserAgent registers or updates one named agent, returning the new user-config
// bytes. Every field is written, and a field set to its zero value is removed
// rather than written empty, so an agent that drops its flags reads as an agent
// with none rather than one with an empty list.
func SetUserAgent(existing []byte, name string, a Agent) ([]byte, error) {
	if err := ValidAgentName(name); err != nil {
		return nil, err
	}
	if err := ValidAgent(a); err != nil {
		return nil, err
	}

	lines, eol := splitLines(existing)
	want := []string{"agents", name}
	start, end, found := findTable(lines, want)
	if !found {
		if agentDeclared(existing, name) {
			return nil, fmt.Errorf(
				"agent %q is already registered in a shape this editor does not rewrite (an inline or dotted table); edit your config by hand", name)
		}
		return appendAgentTable(existing, name, a), nil
	}

	// Set each field in place — replacing the line where it already sits, appending
	// where it does not, deleting where the agent no longer carries it — so an edit
	// through the surface leaves the operator's own comments inside the table.
	for _, f := range agentFields(a) {
		lines, end = setKeyInTable(lines, start, end, f.key, f.render, f.set)
	}
	return []byte(strings.Join(lines, eol)), nil
}

// DeleteUserAgent removes one agent's table entirely. Roles still assigned to it
// are deliberately left alone: the assignment lives in a different table (and, for
// a different space, possibly a different part of the file), and rewriting other
// people's config on a delete is exactly the kind of quiet action this surface
// avoids. A dangling assignment resolves to a visible warning and falls back to
// the role's own fields.
func DeleteUserAgent(existing []byte, name string) ([]byte, error) {
	if err := ValidAgentName(name); err != nil {
		return nil, err
	}
	lines, eol := splitLines(existing)
	start, end, found := findTable(lines, []string{"agents", name})
	if !found {
		return existing, nil // already gone; nothing to do
	}
	// Take the blank lines the table left behind with it, so repeated register and
	// delete cycles cannot slowly fill the file with gaps.
	for start > 0 && strings.TrimSpace(lines[start-1]) == "" && (end >= len(lines) || strings.TrimSpace(lines[end]) == "") {
		start--
	}
	return []byte(strings.Join(append(lines[:start:start], lines[end:]...), eol)), nil
}

// AssignedRoles reports every space and role in the user config currently
// assigned to an agent — what the surface warns about before a delete, so the
// operator sees what they are about to leave dangling.
func AssignedRoles(existing []byte, name string) []string {
	var uf userFile
	if len(existing) == 0 || !decodeTOML(existing, &uf) {
		return nil
	}
	var out []string
	for path, sp := range uf.Spaces {
		for role, b := range sp.Roles {
			if b.Agent != nil && *b.Agent == name {
				out = append(out, path+" › "+role)
			}
		}
	}
	sortStrings(out)
	return out
}

// agentField is one key of an agent's table, with the line that renders it and
// whether this agent sets it at all.
type agentField struct {
	key    string
	render string
	set    bool
}

func agentFields(a Agent) []agentField {
	return []agentField{
		{key: "adapter", render: "adapter = " + strconv.Quote(a.Adapter), set: true},
		{key: "prompt", render: "prompt = " + strconv.Quote(a.Prompt), set: a.Prompt != ""},
		{key: "args", render: renderArgs(a.Args), set: len(a.Args) > 0},
		// `model` is retired. It is listed with set=false so rewriting an agent that
		// still carries one *removes* the dead key rather than leaving it to warn
		// forever — the edit is what retires it on disk.
		{key: "model", render: "", set: false},
	}
}

func renderArgs(args []string) string {
	parts := make([]string, 0, len(args))
	for _, s := range args {
		parts = append(parts, strconv.Quote(s))
	}
	return "args = [" + strings.Join(parts, ", ") + "]"
}

// setKeyInTable sets, replaces, or removes one key inside an already-located
// table, returning the new lines and the table's new end index (which moves as
// lines are added or removed).
func setKeyInTable(lines []string, start, end int, key, render string, set bool) ([]string, int) {
	kStart, kEnd, hasKey := findKey(lines, start+1, end, key)
	switch {
	case hasKey && !set:
		return append(lines[:kStart:kStart], lines[kEnd:]...), end - (kEnd - kStart)
	case hasKey:
		out := append(lines[:kStart:kStart], append([]string{indentOf(lines[kStart]) + render}, lines[kEnd:]...)...)
		return out, end - (kEnd - kStart) + 1
	case !set:
		return lines, end
	default:
		at := insertPoint(lines, start, end)
		out := append(lines[:at:at], append([]string{indentWithin(lines, start, end) + render}, lines[at:]...)...)
		return out, end + 1
	}
}

// appendAgentTable adds the table for an agent that has none, in the same style
// as the binding writer: a blank line off whatever precedes it, then the table.
// The name is validated to need no quoting, so the header is written bare.
func appendAgentTable(existing []byte, name string, a Agent) []byte {
	var b bytes.Buffer
	b.Write(existing)
	if n := len(existing); n > 0 {
		if existing[n-1] != '\n' {
			b.WriteByte('\n')
		}
		b.WriteByte('\n')
	}
	fmt.Fprintf(&b, "[agents.%s]\n", name)
	for _, f := range agentFields(a) {
		if f.set {
			fmt.Fprintf(&b, "%s\n", f.render)
		}
	}
	return b.Bytes()
}

// agentDeclared reports whether the config already registers this agent in some
// other TOML shape than the canonical table — the case the writer refuses rather
// than duplicating a key the decoder would reject wholesale.
func agentDeclared(existing []byte, name string) bool {
	var af agentsFile
	if len(existing) == 0 || !decodeTOML(existing, &af) {
		return false
	}
	_, ok := af.Agents[name]
	return ok
}
