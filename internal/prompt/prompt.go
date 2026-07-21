// Package prompt owns the harness's hackable prompt library and the payload a
// session would be told (ticket 08). Four role prompts plus a common core are
// vendored from the wayfinder skills, embedded in the binary, and materialized
// to disk as plain markdown so the operator can read and edit exactly what a
// session receives (stories 45–47).
//
// Resolution per part walks three layers — shipped built-in ‹ local user ‹
// committed workspace — with `replace` (resets the base) and `append` (stacks)
// semantics per layer. The precedence is the content half of ADR 0009's
// reconciling rule: what the project ships wins, so a committed workspace
// replacement beats a user one. A replacement forked from an older shipped
// default is surfaced, never auto-merged (story 47) — the vendored-sync duty
// made visible rather than silently reconciled.
//
// Payload composition (ADR 0005) assembles core + role prompt + a context bundle
// (glossary, map body, ticket, blockers' answers) into one markdown document,
// built fresh every time and never accumulated.
package prompt

import (
	"crypto/sha256"
	"embed"
	"encoding/hex"
	"fmt"
	"os"
	"path/filepath"

	"github.com/rengwu/wayfinder-harness/internal/config"
)

// SourceRepo and SourceCommit record where the prompt library was vendored from
// (spec: "recording the upstream commit per sync"). Keeping the diff surface
// small — a few hundred lines of markdown — is what makes a sync reviewable;
// bump SourceCommit whenever the embedded assets are re-vendored.
const (
	SourceRepo   = "github.com/rengwu/skills"
	SourceCommit = "vendored-2026-07 (bump on each sync)"
)

// CorePart is the common core, injected first before any role prompt.
// GlossaryPart is the method glossary carried in the context bundle. IdeatePart
// is the ideate on-ramp's starter prompt (ticket 15) — filed as a non-role part
// so the five-role set stays closed: it is materialized and editable like a role
// prompt, but composed alone, with no core, no role, and no context bundle,
// since an ideate session is ticketless and mapless by design.
const (
	CorePart     = "core"
	GlossaryPart = "glossary"
	IdeatePart   = "ideate"
)

// Segment layer tags. The three prompt layers reuse config's names so provenance
// reads the same everywhere; "context" tags a context-bundle part, which is
// assembled rather than resolved through the layers.
const (
	LayerBuiltin   = string(config.LayerBuiltin)
	LayerUser      = string(config.LayerUser)
	LayerWorkspace = string(config.LayerWorkspace)
	LayerContext   = "context"
)

// libDirName is the prompt library directory under both the user data root and,
// as a committed overlay, a space's repo (below dotDirName).
const (
	libDirName = "prompts"
	dotDirName = ".wayfinder-harness"
)

//go:embed assets/*.md
var assets embed.FS

// promptParts are the resolvable parts in composition order sans the role slot:
// the core comes first, then each role. The glossary is a bundle artifact, not a
// resolvable prompt, so it is not here.
func promptParts() []string {
	parts := []string{CorePart}
	for _, r := range config.Roles {
		parts = append(parts, string(r))
	}
	return parts
}

// Segment is one contiguous piece of a composed part, tagged with the layer it
// came from so field-level provenance survives onto the wire and into the
// preview. A resolved prompt part is one or more segments (a base plus appends,
// or a replacement); a context part is a single segment tagged "context".
type Segment struct {
	Layer string `json:"layer"`
	Label string `json:"label,omitempty"`
	Text  string `json:"text"`
}

// Part is one labelled block of the payload — a resolved prompt (core, a role) or
// a context artifact (glossary, map, ticket, a blocker's answer). Kind is
// "prompt" or "context".
type Part struct {
	Name     string    `json:"name"`
	Kind     string    `json:"kind"`
	Segments []Segment `json:"segments"`
}

// Payload is the whole composed result for one ticket and role: the parts with
// their provenance, any warnings (a behind-default replacement), and the single
// markdown document the parts render to — exactly what a session would be told.
type Payload struct {
	Role      string   `json:"role"`
	TicketNum int      `json:"ticketNum"`
	Parts     []Part   `json:"parts"`
	Warnings  []string `json:"warnings,omitempty"`
	Markdown  string   `json:"markdown"`
}

// embedded returns a part's shipped default bytes from the binary.
func embedded(part string) (string, bool) {
	b, err := assets.ReadFile("assets/" + part + ".md")
	if err != nil {
		return "", false
	}
	return string(b), true
}

// shortHash is the 8-hex prefix of a part's content hash — the marker a
// replacement records to declare which shipped default it forked from.
func shortHash(s string) string {
	sum := sha256.Sum256([]byte(s))
	return hex.EncodeToString(sum[:])[:8]
}

// DefaultHash is the short content hash of a part's shipped (embedded) default.
// A replacement whose recorded fork hash differs is behind the shipped default.
func DefaultHash(part string) string {
	e, _ := embedded(part)
	return shortHash(e)
}

// Materialize writes the embedded defaults to <dataDir>/prompts as plain markdown
// so the operator can read and edit the library (story 45), and drops a README
// recording the source and the editing model. Existing files are never
// overwritten — an operator's edits are the point, and they compose on the next
// preview (Done-when: editable on disk, edits show up in the next composition).
func Materialize(dataDir string) error {
	if dataDir == "" {
		return nil
	}
	dir := filepath.Join(dataDir, libDirName)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	for _, part := range append(promptParts(), GlossaryPart, IdeatePart) {
		e, ok := embedded(part)
		if !ok {
			continue
		}
		p := filepath.Join(dir, part+".md")
		if _, err := os.Stat(p); err == nil {
			continue // preserve the operator's edits
		}
		if err := os.WriteFile(p, []byte(e), 0o644); err != nil {
			return err
		}
	}
	readme := filepath.Join(dir, "README.md")
	if _, err := os.Stat(readme); os.IsNotExist(err) {
		if err := os.WriteFile(readme, []byte(readmeText()), 0o644); err != nil {
			return err
		}
	}
	return nil
}

// baseText returns the base text for a part — the materialized file if present
// (so operator edits compose), else the embedded default — alongside the hash of
// the *shipped* default, which behind-default detection compares a fork against.
func baseText(dataDir, part string) (text, shippedHash string) {
	shippedHash = DefaultHash(part)
	if dataDir != "" {
		if b, err := os.ReadFile(filepath.Join(dataDir, libDirName, part+".md")); err == nil {
			return string(b), shippedHash
		}
	}
	e, _ := embedded(part)
	return e, shippedHash
}

// overlay reads a layer's `replace` or `append` file for a part, if present.
func overlay(root, part, kind string) (string, bool) {
	if root == "" {
		return "", false
	}
	b, err := os.ReadFile(filepath.Join(root, part+"."+kind+".md"))
	if err != nil {
		return "", false
	}
	return string(b), true
}

type layerRef struct {
	layer string
	root  string
}

// overlayRoots returns the user then workspace overlay roots, in the order
// resolution processes them (low precedence to high): the workspace layer is last
// so a committed replacement wins over a local one (ADR 0009, content half).
func overlayRoots(dataDir, repoDir string) []layerRef {
	var refs []layerRef
	if dataDir != "" {
		refs = append(refs, layerRef{LayerUser, filepath.Join(dataDir, libDirName)})
	}
	if repoDir != "" {
		refs = append(refs, layerRef{LayerWorkspace, filepath.Join(repoDir, dotDirName, libDirName)})
	}
	return refs
}

// resolvePart resolves one part across the three layers into its provenance-
// tagged segments, appending a warning for any replacement forked from an older
// shipped default. A `replace` resets the accumulated base (discarding the layers
// below it); an `append` stacks after whatever the current base is, so a layer
// carrying both replaces then appends its own addition.
func resolvePart(part, dataDir, repoDir string, warnings *[]string) []Segment {
	text, shipped := baseText(dataDir, part)
	segs := []Segment{{Layer: LayerBuiltin, Label: "default", Text: text}}

	for _, lr := range overlayRoots(dataDir, repoDir) {
		if rep, ok := overlay(lr.root, part, "replace"); ok {
			forked, body := splitForkMarker(rep)
			if forked != "" && forked != shipped {
				*warnings = append(*warnings, fmt.Sprintf(
					"the %s prompt for %q is a replacement forked from an older shipped default (forked from %s, shipped is now %s); review and re-fork it",
					lr.layer, part, forked, shipped,
				))
			}
			segs = []Segment{{Layer: lr.layer, Label: "replace (resets base)", Text: body}}
		}
		if app, ok := overlay(lr.root, part, "append"); ok {
			segs = append(segs, Segment{Layer: lr.layer, Label: "append", Text: app})
		}
	}
	return segs
}

// Ideate returns the ideate on-ramp's starter prompt: the materialized editable
// copy if the operator has one, else the shipped default (ticket 15). Unlike
// Compose it resolves a single part with no core, no role, and no context
// bundle — the ideate session is ticketless and mapless, so there is no ticket or
// map to inject, and it is a non-role part, deliberately outside the workspace
// replace/append overlay that role prompts and the payload preview use. Editing
// `<dataDir>/prompts/ideate.md` changes what the very next ideate session reads.
func Ideate(dataDir string) string {
	text, _ := baseText(dataDir, IdeatePart)
	return text
}

// LibraryWarnings resolves every prompt part for a space just to collect the
// behind-default surfacing, so a stale fork is visible on the space (and the
// preview) without the operator opening every role. It never fails: an unreadable
// overlay simply resolves to the layers below it.
func LibraryWarnings(dataDir, repoDir string) []string {
	var w []string
	for _, part := range promptParts() {
		_ = resolvePart(part, dataDir, repoDir, &w)
	}
	return w
}

func validRole(role string) bool {
	for _, r := range config.Roles {
		if string(r) == role {
			return true
		}
	}
	return false
}

func readmeText() string {
	return fmt.Sprintf(`# Prompt library

These are the prompts the harness injects into every session — plain markdown,
yours to read and edit. Vendored from %s (%s).

## The parts

- `+"`core.md`"+` — the common core, injected first for every role.
- `+"`grill.md`, `prototype.md`, `research.md`, `implement.md`"+` — one per role.
- `+"`glossary.md`"+` — the method glossary carried in each session's context.

Editing any of these files changes what the next session is told. To read exactly
what a ticket and role would receive, open the payload preview in the cockpit.

## Overriding without forking

Instead of editing a shipped file you can layer on top of it, per part:

- `+"`<part>.append.md`"+` — stacked after the base (house rules that ride alongside).
- `+"`<part>.replace.md`"+` — replaces the base entirely (a full fork).

A committed copy of these under a space's `+"`.wayfinder-harness/prompts/`"+` wins
over the local ones here, which win over the shipped default. A `+"`replace`"+` may
record which shipped version it forked from with a first line:

    <!-- forked from a1b2c3d4 -->

If the shipped default later changes, the cockpit surfaces that your replacement
is behind — it is never auto-merged.
`, SourceRepo, SourceCommit)
}
