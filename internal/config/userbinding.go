package config

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"

	"github.com/BurntSushi/toml"
)

// The three fields of a role binding, and the only keys the transparency surface
// may write. Anything outside this set is refused rather than written blind.
const (
	FieldAdapter = "adapter"
	FieldModel   = "model"
	FieldArgs    = "args"
)

// BindingEdit is one field of one role's binding in one space — the unit the
// transparency surface edits (ADR 0014). Clear removes the override instead of
// setting one, which reveals the layer beneath it; Value carries adapter and
// model, Args carries args (a nil Args with Clear false writes `args = []`,
// which is the explicit "inherit nothing" the resolver already understands).
type BindingEdit struct {
	SpacePath string
	Role      string
	Field     string
	Value     string
	Args      []string
	Clear     bool
}

// SetUserBinding returns the local user-config bytes with one field of one
// role's binding set — or cleared — for one space, given the file's current
// contents (nil or empty when the file is absent).
//
// This is the write half of the transparency surface, and it writes **only the
// user layer** (ADR 0009 as amended): bindings resolve user-over-workspace, so
// the user layer is where an operator's execution choice belongs, and a local UI
// never edits committed workspace config on someone else's behalf.
//
// The edit is key-level and comment-preserving. Unlike DeclareMapKind, which only
// ever appends a table it has proven absent, this reaches into an existing table
// to set, replace, or delete one key — so it works on the file's own lines rather
// than decoding and re-encoding: every surrounding byte (comments, key ordering,
// unrelated tables, the operator's spacing) survives untouched. Only when the
// target table is absent entirely does it append one, in DeclareMapKind's style.
//
// It refuses rather than guess. A role already bound through some other TOML
// shape than the canonical `[spaces."<path>".roles.<role>]` table — an inline
// table, a dotted key — is left alone with an error telling the human to edit it
// by hand, because writing the canonical table beside it would produce a
// duplicate key the decoder rejects wholesale.
func SetUserBinding(existing []byte, e BindingEdit) ([]byte, error) {
	if e.SpacePath == "" {
		return nil, fmt.Errorf("no space path to key the override by")
	}
	if !isRole(e.Role) {
		return nil, fmt.Errorf("unknown role %q; want one of grill, prototype, research, implement", e.Role)
	}
	switch e.Field {
	case FieldAdapter, FieldModel:
		if !e.Clear && strings.TrimSpace(e.Value) == "" {
			return nil, fmt.Errorf("%s needs a value; clear the override instead to fall back to the layer beneath", e.Field)
		}
	case FieldArgs:
	default:
		return nil, fmt.Errorf("unknown binding field %q; want adapter, model or args", e.Field)
	}

	lines, eol := splitLines(existing)
	want := []string{"spaces", e.SpacePath, "roles", e.Role}
	start, end, found := findTable(lines, want)

	if !found {
		if e.Clear {
			return existing, nil // nothing to reveal; the file already says nothing
		}
		if userRoleDeclared(existing, e.SpacePath, e.Role) {
			return nil, fmt.Errorf(
				"role %q is already bound for this space in a shape this editor does not rewrite (an inline or dotted table); edit your user config by hand",
				e.Role,
			)
		}
		return appendBindingTable(existing, e), nil
	}

	kStart, kEnd, hasKey := findKey(lines, start+1, end, e.Field)
	switch {
	case hasKey && e.Clear:
		lines = append(lines[:kStart], lines[kEnd:]...)
	case hasKey:
		lines = append(lines[:kStart], append([]string{indentOf(lines[kStart]) + renderBinding(e)}, lines[kEnd:]...)...)
	case e.Clear:
		return existing, nil // the override is not there; nothing to reveal
	default:
		at := insertPoint(lines, start, end)
		lines = append(lines[:at], append([]string{indentWithin(lines, start, end) + renderBinding(e)}, lines[at:]...)...)
	}
	return []byte(strings.Join(lines, eol)), nil
}

// appendBindingTable adds the canonical table for a role that has none, in
// DeclareMapKind's style: a blank line off whatever precedes it, then the table.
func appendBindingTable(existing []byte, e BindingEdit) []byte {
	var b bytes.Buffer
	b.Write(existing)
	if n := len(existing); n > 0 {
		if existing[n-1] != '\n' {
			b.WriteByte('\n')
		}
		b.WriteByte('\n')
	}
	// %q quotes the space path as a TOML basic string key, so a path carrying a
	// `.` (or a space) is never misread as a dotted key.
	fmt.Fprintf(&b, "[spaces.%q.roles.%s]\n%s\n", e.SpacePath, e.Role, renderBinding(e))
	return b.Bytes()
}

func renderBinding(e BindingEdit) string {
	if e.Field == FieldArgs {
		parts := make([]string, 0, len(e.Args))
		for _, a := range e.Args {
			parts = append(parts, strconv.Quote(a))
		}
		return "args = [" + strings.Join(parts, ", ") + "]"
	}
	return e.Field + " = " + strconv.Quote(e.Value)
}

// userRoleDeclared reports whether the user config already binds this role for
// this space, whatever TOML shape it used. A file too malformed to decode
// declares nothing — it was already surfaced as malformed on resolve, and
// appending a well-formed table does not deepen the damage.
func userRoleDeclared(existing []byte, spacePath, role string) bool {
	if len(existing) == 0 {
		return false
	}
	var uf userFile
	if _, err := toml.Decode(string(existing), &uf); err != nil {
		return false
	}
	_, ok := uf.Spaces[spacePath].Roles[role]
	return ok
}

// --- TOML line surgery ----------------------------------------------------
//
// Everything below works on the file's own lines. It understands just enough
// TOML to find one table and one key inside it — table headers, quoted keys,
// comments, and values that run past their first line — and treats anything it
// does not recognise as opaque text to leave alone.

// splitLines splits a file into lines and reports the line ending it uses, so a
// rewrite hands back the operator's own convention rather than normalising it.
func splitLines(data []byte) ([]string, string) {
	s := string(data)
	if strings.Contains(s, "\r\n") {
		return strings.Split(s, "\r\n"), "\r\n"
	}
	return strings.Split(s, "\n"), "\n"
}

// findTable locates the table whose key path is want, returning the index of its
// header line and the index one past its last line (the next header, or EOF).
func findTable(lines []string, want []string) (start, end int, found bool) {
	for i, l := range lines {
		path, ok := parseTableHeader(l)
		if !ok {
			continue
		}
		if found {
			return start, i, true // the next header closes the one we found
		}
		if equalPath(path, want) {
			start, found = i, true
		}
	}
	if found {
		return start, len(lines), true
	}
	return 0, 0, false
}

// findKey locates a key line named key within [from, to), returning its span —
// one past its last line, so a value that runs across lines (an array over
// several lines) is replaced or deleted whole rather than half.
func findKey(lines []string, from, to int, key string) (start, end int, found bool) {
	for i := from; i < to; i++ {
		name, ok := parseKeyName(lines[i])
		if !ok || name != key {
			continue
		}
		return i, i + valueSpan(lines, i, to), true
	}
	return 0, 0, false
}

// valueSpan counts how many lines a key's value occupies, by balancing the
// brackets and braces that are outside strings. An unbalanced tail (a malformed
// file) stops at the table's end rather than running away.
func valueSpan(lines []string, at, to int) int {
	depth := 0
	for i := at; i < to; i++ {
		depth += bracketDelta(lines[i])
		if depth <= 0 {
			return i - at + 1
		}
	}
	return to - at
}

func bracketDelta(line string) int {
	d := 0
	scanTOML(line, func(r rune) {
		switch r {
		case '[', '{':
			d++
		case ']', '}':
			d--
		}
	})
	return d
}

// insertPoint picks where a new key goes inside an existing table: after the
// last key already there, so the addition reads as the newest line rather than
// jumping the queue — and immediately after the header when the table is empty.
// Either way nothing already in the file moves.
func insertPoint(lines []string, start, end int) int {
	at := start + 1
	for i := start + 1; i < end; i++ {
		if _, ok := parseKeyName(lines[i]); ok {
			at = i + valueSpan(lines, i, end)
		}
	}
	return at
}

// indentOf returns a line's leading whitespace, so a replacement sits exactly
// where the line it replaces did.
func indentOf(line string) string {
	return line[:len(line)-len(strings.TrimLeft(line, " \t"))]
}

// indentWithin matches the indentation of the keys already in a table, so an
// inserted key lines up with its neighbours in an indented file.
func indentWithin(lines []string, start, end int) string {
	for i := start + 1; i < end; i++ {
		if _, ok := parseKeyName(lines[i]); ok {
			return indentOf(lines[i])
		}
	}
	return ""
}

// parseTableHeader parses `[a.b."c"]` into its key path. Array-of-table headers
// (`[[…]]`) are deliberately not ours: the chartr never writes one, and one in
// the operator's file is left strictly alone.
func parseTableHeader(line string) ([]string, bool) {
	s := strings.TrimSpace(stripComment(line))
	if len(s) < 2 || s[0] != '[' || s[len(s)-1] != ']' || strings.HasPrefix(s, "[[") {
		return nil, false
	}
	return splitKeyPath(strings.TrimSpace(s[1 : len(s)-1]))
}

// parseKeyName parses the key of a `key = value` line, unquoting a quoted key.
// A line that is blank, a comment, or a table header is not a key line.
func parseKeyName(line string) (string, bool) {
	s := stripComment(line)
	if strings.TrimSpace(s) == "" {
		return "", false
	}
	if t := strings.TrimSpace(s); t[0] == '[' {
		return "", false
	}
	eq := indexOutsideStrings(s, '=')
	if eq < 0 {
		return "", false
	}
	path, ok := splitKeyPath(strings.TrimSpace(s[:eq]))
	if !ok || len(path) != 1 {
		return "", false // a dotted key is a shape this editor leaves alone
	}
	return path[0], true
}

// splitKeyPath splits a dotted TOML key path into its segments, unquoting basic
// and literal string keys. It reports false on anything it cannot read, so an
// exotic header is skipped rather than half-understood.
func splitKeyPath(s string) ([]string, bool) {
	var out []string
	var seg strings.Builder
	quoted := false // this segment came from a quoted key
	i := 0
	for i < len(s) {
		switch c := s[i]; c {
		case '"', '\'':
			if seg.Len() > 0 {
				return nil, false
			}
			val, n, ok := readString(s[i:])
			if !ok {
				return nil, false
			}
			seg.WriteString(val)
			quoted = true
			i += n
		case '.':
			out = append(out, strings.TrimSpace(seg.String()))
			seg.Reset()
			quoted = false
			i++
		case ' ', '\t':
			i++
		default:
			if quoted {
				return nil, false // trailing junk after a quoted key
			}
			seg.WriteByte(c)
			i++
		}
	}
	out = append(out, strings.TrimSpace(seg.String()))
	for _, p := range out {
		if p == "" {
			return nil, false
		}
	}
	return out, true
}

// readString reads a TOML basic (") or literal (') string at the head of s,
// returning its value and how many bytes it consumed.
func readString(s string) (string, int, bool) {
	q := s[0]
	var b strings.Builder
	for i := 1; i < len(s); i++ {
		c := s[i]
		if q == '"' && c == '\\' && i+1 < len(s) {
			switch s[i+1] {
			case '"':
				b.WriteByte('"')
			case '\\':
				b.WriteByte('\\')
			default:
				b.WriteByte(c)
				b.WriteByte(s[i+1])
			}
			i++
			continue
		}
		if c == q {
			return b.String(), i + 1, true
		}
		b.WriteByte(c)
	}
	return "", 0, false
}

// stripComment removes a trailing `#` comment, ignoring one inside a string.
func stripComment(line string) string {
	if i := indexOutsideStrings(line, '#'); i >= 0 {
		return line[:i]
	}
	return line
}

func indexOutsideStrings(s string, target byte) int {
	found := -1
	scanBytes(s, func(i int, c byte) {
		if c == target && found < 0 {
			found = i
		}
	})
	return found
}

func scanTOML(s string, fn func(rune)) {
	scanBytes(s, func(_ int, c byte) { fn(rune(c)) })
}

// scanBytes visits every byte of a line that sits outside a string literal and
// outside a comment — the only bytes that carry TOML structure.
func scanBytes(s string, fn func(int, byte)) {
	for i := 0; i < len(s); i++ {
		switch c := s[i]; c {
		case '"', '\'':
			_, n, ok := readString(s[i:])
			if !ok {
				return // an unterminated string swallows the rest of the line
			}
			i += n - 1
		case '#':
			fn(i, c)
			return
		default:
			fn(i, c)
		}
	}
}

func equalPath(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
