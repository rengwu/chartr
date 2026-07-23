package config

import "strings"

// Comment-preserving TOML line surgery: enough of TOML to find one table and one
// key inside it — table headers, quoted keys, comments, and values that run past
// their first line — treating anything it does not recognise as opaque text to
// leave alone. The agent-library writer (useragent.go) edits the operator's own
// file through these, so comments, key order, spacing, and unrelated tables
// survive every edit, and a shape this editor does not understand is refused with
// a pointer at a hand edit rather than rewritten blind.

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
// (`[[…]]`) are deliberately not ours: chartr never writes one, and one in
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
