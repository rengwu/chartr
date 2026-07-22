package config

import (
	"bytes"
	"fmt"

	"github.com/BurntSushi/toml"

	"github.com/rengwu/chartr/internal/model"
)

// DeclareMapKind returns the committed workspace-config bytes with map `slug`
// declared as `kind`, given the file's current contents (nil or empty when the
// file is absent). It is the write half of classification (ADR 0007): kind lives
// in committed workspace config, keyed by slug, so a teammate cloning the space
// inherits the classification rather than re-confirming it (story 15).
//
// The file is the operator's to read and edit (the hackability stance), so this
// appends a new [maps."<slug>"] table rather than decode-and-re-encode the whole
// file — the operator's own role bindings, comments, and formatting survive
// untouched. Because a classified map is no longer inert, the classify action
// only ever reaches an undeclared map, so the slug's table is expected to be
// absent. If it is already present, this refuses rather than write a duplicate
// table (which the TOML decoder would then reject wholesale) or silently rewrite
// the operator's bytes: the human is told to edit it by hand.
func DeclareMapKind(existing []byte, slug, kind string) ([]byte, error) {
	if !model.ValidKind(kind) {
		return nil, fmt.Errorf("unknown map kind %q; want %s or %s", kind, model.KindPlanning, model.KindImplementation)
	}
	if mapKindDeclared(existing, slug) {
		return nil, fmt.Errorf("map %q is already declared in committed config; edit %s by hand to change it", slug, WorkspaceConfigName)
	}

	var b bytes.Buffer
	b.Write(existing)
	// Separate the appended table from whatever precedes it — a blank line after
	// existing content, nothing at the head of an empty or absent file.
	if n := len(existing); n > 0 {
		if existing[n-1] != '\n' {
			b.WriteByte('\n')
		}
		b.WriteByte('\n')
	}
	// %q quotes the slug as a TOML basic string key, so a dotted key never
	// misreads a slug that carries a `.` or other punctuation.
	fmt.Fprintf(&b, "[maps.%q]\nkind = %q\n", slug, kind)
	return b.Bytes(), nil
}

// mapKindDeclared reports whether the committed config already carries a
// declaration for slug — under either the [maps.<slug>] sub-table or an inline
// [maps] form. It decodes rather than scans text so both forms are caught; a
// file too malformed to decode is treated as declaring nothing, so the append
// still proceeds (the file was already surfaced as malformed on resolve, and a
// valid appended table does not deepen the damage).
func mapKindDeclared(existing []byte, slug string) bool {
	if len(existing) == 0 {
		return false
	}
	var wf workspaceFile
	if _, err := toml.Decode(string(existing), &wf); err != nil {
		return false
	}
	_, ok := wf.Maps[slug]
	return ok
}
