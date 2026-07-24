// Package prompt owns chartr's hackable skill library and the payload a
// session would be told. Every injected prompt — the common core, the four role
// prompts, the ideate on-ramp, and the tracker convention — is a standard
// `SKILL.md` directory: `name`/`description` frontmatter over a markdown body,
// with supporting files alongside it. They are vendored from the wayfinder
// skills, embedded in the binary, and materialized to disk so the operator can
// read, edit, and reuse them in any agent CLI that reads the standard.
//
// Resolution is **whole-skill shadowing** across three layers — shipped built-in
// (`<dataDir>/skills/`) ‹ local user (`<configDir>/skills/`) ‹ committed
// workspace (`<space>/.chartr/skills/`): the most specific layer that
// defines a skill of a given name wins its entire directory. The precedence is
// the content half of ADR 0009's reconciling rule — what the project ships wins,
// so a committed workspace skill beats a local one. There is no per-file merge to
// reason about; a fork records what it forked from in `forked_from:` frontmatter
// and is surfaced as behind, never auto-merged.
//
// chartr keeps composing the payload itself (ADR 0002, reaffirmed): it reads
// the resolved core + role bodies with their frontmatter stripped and assembles
// them with a context bundle (map body, ticket, blockers' answers, and the
// glossary sourced from the `tracker-convention` skill) into one markdown
// document, built fresh every time and never accumulated. Supporting files stay
// on disk rather than being inlined, so a session can zoom into them on demand at
// no payload cost.
package prompt

import (
	"crypto/sha256"
	"embed"
	"encoding/hex"
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"

	"github.com/rengwu/chartr/internal/config"
)

// SourceRepo and SourceCommit record where the skill library was vendored from
// (spec: "recording the upstream commit per sync"). Keeping the diff surface
// small — a few hundred lines of markdown — is what makes a sync reviewable;
// bump SourceCommit whenever the embedded skills are re-vendored.
const (
	SourceRepo   = "github.com/rengwu/skills"
	SourceCommit = "9e8b5ea"
)

// The skills chartr knows by name. CoreSkill is injected first before any
// role skill; IdeateSkill is the ideate on-ramp, composed alone (no core, no
// context bundle) because an ideate session is ticketless and mapless;
// TrackerSkill restates the wayfinder map format and carries the method glossary
// as its supporting file, which the context bundle sources. The four method
// skills — WayfinderSkill, DomainSkill, SpecSkill, TicketsSkill — are shipped,
// resolved, and materialized like the rest, but never auto-composed into a
// session payload: they serve charting, speccing, and ticket-breaking work done
// outside a composed session.
const (
	CoreSkill    = "core"
	IdeateSkill  = "ideate"
	TrackerSkill = "tracker-convention"
	GlossaryFile = "glossary.md"

	WayfinderSkill = "wayfinder"
	DomainSkill    = "domain-modeling"
	SpecSkill      = "to-spec"
	TicketsSkill   = "to-tickets"
)

// Segment layer tags. The three skill layers name where a skill resolved from —
// shipped floor, the operator's own fork, or a space's committed library; "context"
// tags a context-bundle part, which is assembled rather than resolved through the
// layers.
const (
	LayerBuiltin   = "built-in"
	LayerUser      = "user"
	LayerWorkspace = "workspace"
	LayerContext   = "context"
)

const (
	// libDirName is the operator's own skill library directory under the config
	// root and (below dotDirName) a space's repo.
	libDirName = "skills"
	// builtinLibDirName is where the shipped library is materialized under the
	// config root — a sibling of libDirName so the operator's own skills and the
	// editable built-in defaults never share a directory.
	builtinLibDirName = "builtin-skills"
	dotDirName        = ".chartr"
	// skillFile is the standard entry point of a skill directory.
	skillFile = "SKILL.md"
	// embedRoot is where the shipped library sits inside the binary.
	embedRoot = "assets/skills"
)

//go:embed assets/skills
var assets embed.FS

// Roots are the three skill-library roots resolution walks, lowest precedence
// first. Any of them may be empty, which simply means that layer defines nothing.
type Roots struct {
	// Builtin is where the shipped library is materialized
	// (`<configDir>/builtin-skills`). When a skill is absent from it — a fresh
	// install, a directory the operator deleted — resolution falls back to the
	// copy embedded in the binary, so the built-in layer is never missing.
	Builtin string
	// User is the operator's local library (`<configDir>/skills`): uncommitted,
	// machine-local forks.
	User string
	// Workspace is a space's committed library (`.chartr/skills`):
	// shared, versioned, and — for content — the winning layer (ADR 0009).
	Workspace string
}

// RootsFor derives the three roots from the operator's config root and a space's
// repo. The built-in and user libraries both live under the config root; callers
// pass "" for a root that does not apply (the ideate on-ramp, for instance,
// resolves with no space).
func RootsFor(configDir, repoDir string) Roots {
	var r Roots
	if configDir != "" {
		r.Builtin = filepath.Join(configDir, builtinLibDirName)
		r.User = filepath.Join(configDir, libDirName)
	}
	if repoDir != "" {
		r.Workspace = filepath.Join(repoDir, dotDirName, libDirName)
	}
	return r
}

// Skill is one resolved skill: which layer won its whole directory, the metadata
// its frontmatter carries, its body with that frontmatter stripped, and the
// content hash covering the directory. Stale reports a fork recorded against a
// shipped default that has since moved on.
type Skill struct {
	Name        string `json:"name"`
	Layer       string `json:"layer"`
	Dir         string `json:"dir,omitempty"`
	Description string `json:"description,omitempty"`
	ForkedFrom  string `json:"forkedFrom,omitempty"`
	Hash        string `json:"hash"`
	Stale       bool   `json:"stale,omitempty"`

	Body string `json:"-"`
}

// Segment is one contiguous piece of a composed part, tagged with the layer it
// came from so field-level provenance survives onto the wire and into the
// preview. Under whole-skill shadowing a resolved skill is a single segment
// tagged with the layer that won it; a context part is a single segment tagged
// "context".
type Segment struct {
	Layer string `json:"layer"`
	Label string `json:"label,omitempty"`
	Text  string `json:"text"`
}

// Part is one labelled block of the payload — a resolved skill (core, a role) or
// a context artifact (glossary, map, ticket, a blocker's answer). Kind is
// "prompt" or "context".
type Part struct {
	Name     string    `json:"name"`
	Kind     string    `json:"kind"`
	Segments []Segment `json:"segments"`
}

// Payload is the whole composed result for one ticket and role: the parts with
// their provenance, the skills that were composed into it (which layer won each,
// with its content hash — the claim commit's provenance trailers), any warnings
// (a fork behind the shipped default), and the single markdown document the parts
// render to — exactly what a session would be told.
type Payload struct {
	Role      string   `json:"role"`
	TicketNum int      `json:"ticketNum"`
	Parts     []Part   `json:"parts"`
	Skills    []Skill  `json:"skills,omitempty"`
	Warnings  []string `json:"warnings,omitempty"`
	Markdown  string   `json:"markdown"`
}

// Names lists the skills chartr ships, in a stable order: the core, the
// roles, then the two library skills, then the four method skills.
func Names() []string {
	names := []string{CoreSkill}
	for _, r := range config.Roles {
		names = append(names, string(r))
	}
	return append(names, IdeateSkill, TrackerSkill,
		WayfinderSkill, DomainSkill, SpecSkill, TicketsSkill)
}

// shortHash is the 8-hex prefix of a content hash — short enough to read in
// frontmatter, long enough to identify a shipped version.
func shortHash(sum [32]byte) string { return hex.EncodeToString(sum[:])[:8] }

// hashFiles hashes a skill directory's files in a stable order, covering both
// paths and contents, so a change to a supporting file is as visible as a change
// to SKILL.md (story 24).
func hashFiles(files map[string][]byte) string {
	names := make([]string, 0, len(files))
	for n := range files {
		names = append(names, n)
	}
	sort.Strings(names)
	h := sha256.New()
	for _, n := range names {
		fmt.Fprintf(h, "%s\n%d\n", n, len(files[n]))
		h.Write(files[n])
	}
	var sum [32]byte
	copy(sum[:], h.Sum(nil))
	return shortHash(sum)
}

// embeddedFiles reads a shipped skill's whole directory out of the binary, keyed
// by path relative to the skill directory.
func embeddedFiles(name string) (map[string][]byte, bool) {
	root := path.Join(embedRoot, name)
	files := map[string][]byte{}
	err := fs.WalkDir(assets, root, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}
		b, err := assets.ReadFile(p)
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(root, p)
		if err != nil {
			return err
		}
		files[filepath.ToSlash(rel)] = b
		return nil
	})
	if err != nil || len(files) == 0 {
		return nil, false
	}
	return files, true
}

// dirFiles reads a skill directory off disk, keyed by path relative to it.
// Nested supporting files travel too, so a skill is a directory, not a file pair.
func dirFiles(dir string) (map[string][]byte, bool) {
	if dir == "" {
		return nil, false
	}
	if _, err := os.Stat(filepath.Join(dir, skillFile)); err != nil {
		return nil, false // a directory without SKILL.md does not define a skill
	}
	files := map[string][]byte{}
	err := filepath.WalkDir(dir, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}
		b, err := os.ReadFile(p)
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(dir, p)
		if err != nil {
			return err
		}
		files[filepath.ToSlash(rel)] = b
		return nil
	})
	if err != nil {
		return nil, false
	}
	return files, true
}

// ShippedHash is the content hash of a skill's shipped (embedded) directory. A
// fork whose recorded `forked_from` differs from it is behind the default.
func ShippedHash(name string) string {
	files, ok := embeddedFiles(name)
	if !ok {
		return ""
	}
	return hashFiles(files)
}

// Resolve resolves one skill by name across the three layers with whole-skill
// shadowing: the most specific layer that defines it — workspace, then user, then
// the materialized built-in — wins its entire directory, and only the shipped
// embedded copy is left as the floor. It never fails: an unreadable layer simply
// resolves to the one below it.
func Resolve(name string, roots Roots) (Skill, bool) {
	for _, cand := range []struct {
		layer string
		dir   string
	}{
		{LayerWorkspace, joinSkill(roots.Workspace, name)},
		{LayerUser, joinSkill(roots.User, name)},
		{LayerBuiltin, joinSkill(roots.Builtin, name)},
	} {
		if files, ok := dirFiles(cand.dir); ok {
			return newSkill(name, cand.layer, cand.dir, files), true
		}
	}
	files, ok := embeddedFiles(name)
	if !ok {
		return Skill{}, false
	}
	return newSkill(name, LayerBuiltin, "", files), true
}

func joinSkill(root, name string) string {
	if root == "" {
		return ""
	}
	return filepath.Join(root, name)
}

func newSkill(name, layer, dir string, files map[string][]byte) Skill {
	meta, body := splitFrontmatter(string(files[skillFile]))
	s := Skill{
		Name:        name,
		Layer:       layer,
		Dir:         dir,
		Description: meta["description"],
		ForkedFrom:  strings.ToLower(meta["forked_from"]),
		Hash:        hashFiles(files),
		Body:        strings.TrimSpace(body),
	}
	s.Stale = s.ForkedFrom != "" && s.ForkedFrom != ShippedHash(name)
	return s
}

// Support returns one of a skill's supporting files (the glossary, say) from
// whichever layer won the directory — off disk when a layer won it, out of the
// binary when the shipped copy did.
func (s Skill) Support(name string) (string, bool) {
	if s.Dir != "" {
		b, err := os.ReadFile(filepath.Join(s.Dir, name))
		if err != nil {
			return "", false
		}
		return string(b), true
	}
	b, err := assets.ReadFile(path.Join(embedRoot, s.Name, name))
	if err != nil {
		return "", false
	}
	return string(b), true
}

// staleWarning is the sentence the cockpit shows for a fork that has fallen
// behind the shipped default: what drifted, in which layer, and that nothing was
// merged for the operator (story 23).
func staleWarning(s Skill) string {
	return fmt.Sprintf(
		"the %s skill %q is behind the shipped default (forked from %s, shipped is now %s); review and re-fork it — it is never auto-merged",
		s.Layer, s.Name, s.ForkedFrom, ShippedHash(s.Name),
	)
}

// Library resolves every shipped skill for a space, so the cockpit can show which
// layer won each directory and whether it has drifted.
func Library(roots Roots) []Skill {
	var out []Skill
	for _, name := range Names() {
		if s, ok := Resolve(name, roots); ok {
			out = append(out, s)
		}
	}
	return out
}

// LibraryWarnings resolves every skill for a space just to collect the stale-fork
// surfacing, so a drifted fork is visible on the space (and the preview) without
// the operator opening every role.
func LibraryWarnings(roots Roots) []string {
	var w []string
	for _, s := range Library(roots) {
		if s.Stale {
			w = append(w, staleWarning(s))
		}
	}
	return w
}

// Materialize writes the embedded skill library to <configDir>/builtin-skills as
// plain `SKILL.md` directories so the operator can read and edit exactly what a
// session receives, and drops a README recording the source and the layering
// model. Existing files are never overwritten — an operator's edits are the
// point, and they compose on the next preview.
func Materialize(configDir string) error {
	if configDir == "" {
		return nil
	}
	root := filepath.Join(configDir, builtinLibDirName)
	for _, name := range Names() {
		files, ok := embeddedFiles(name)
		if !ok {
			continue
		}
		for rel, b := range files {
			p := filepath.Join(root, name, filepath.FromSlash(rel))
			if err := os.MkdirAll(filepath.Dir(p), 0o755); err != nil {
				return err
			}
			if _, err := os.Stat(p); err == nil {
				continue // preserve the operator's edits
			}
			if err := os.WriteFile(p, b, 0o644); err != nil {
				return err
			}
		}
	}
	readme := filepath.Join(root, "README.md")
	if _, err := os.Stat(readme); os.IsNotExist(err) {
		if err := os.WriteFile(readme, []byte(readmeText()), 0o644); err != nil {
			return err
		}
	}
	return nil
}

// Ideate returns the ideate on-ramp's resolved body. Unlike Compose it resolves a
// single skill with no core, no role, and no context bundle — the ideate session
// is ticketless and mapless by design, so there is no ticket or map to inject.
// Editing the resolved `ideate` skill changes what the very next ideate session
// reads.
func Ideate(roots Roots) string {
	s, ok := Resolve(IdeateSkill, roots)
	if !ok {
		return ""
	}
	return s.Body
}

// splitFrontmatter peels a leading `---` delimited block off a SKILL.md, returning
// its simple `key: value` pairs and the body below it. The frontmatter is metadata
// for the cockpit and for drift detection — it never reaches the payload (story
// 27). A file without frontmatter is all body.
func splitFrontmatter(src string) (map[string]string, string) {
	meta := map[string]string{}
	lines := strings.Split(src, "\n")
	if len(lines) == 0 || strings.TrimSpace(lines[0]) != "---" {
		return meta, src
	}
	end := -1
	for i := 1; i < len(lines); i++ {
		if strings.TrimSpace(lines[i]) == "---" {
			end = i
			break
		}
	}
	if end < 0 {
		return meta, src
	}
	for _, l := range lines[1:end] {
		i := strings.Index(l, ":")
		if i < 0 {
			continue
		}
		key := strings.ToLower(strings.TrimSpace(l[:i]))
		val := strings.Trim(strings.TrimSpace(l[i+1:]), `"'`)
		if key != "" {
			meta[key] = val
		}
	}
	return meta, strings.Join(lines[end+1:], "\n")
}

func readmeText() string {
	return fmt.Sprintf(`# Skill library

These are the skills chartr injects into every session — standard `+"`SKILL.md`"+`
directories, yours to read, edit, and reuse in any agent CLI that reads the
format. Vendored from %s (%s).

## The skills

- `+"`core/`"+` — the common core, injected first for every role.
- `+"`grill/`, `prototype/`, `research/`, `implement/`"+` — one per role.
- `+"`ideate/`"+` — the ticketless ideate on-ramp, composed alone.
- `+"`tracker-convention/`"+` — the wayfinder map format, carrying `+"`glossary.md`"+`
  (the glossary each session's context bundle is built from) as a supporting file.
- `+"`wayfinder/`"+` — the map method: charting an effort and working its tickets.
- `+"`domain-modeling/`"+` — keep `+"`CONTEXT.md`"+` and the ADRs current as terms
  crystallise.
- `+"`to-spec/`"+` — synthesize a resolved planning map or conversation into a spec.
- `+"`to-tickets/`"+` — break a spec into an implementation map of tracer-bullet
  tickets.

The method skills are never auto-composed into a session payload; they serve
charting, speccing, and ticket-breaking work done outside a composed session.

Editing any of these changes what the next session is told. To read exactly what
a ticket and role would receive, open the payload preview in the cockpit.

## Layering

A skill of the same name may be defined in three places, and the most specific one
wins its **whole directory** — there is no per-file merge:

    built-in (here) ‹ user (<config>/chartr/skills/) ‹ workspace (<space>/.chartr/skills/)

A fork may record which shipped version it came from in its frontmatter:

    ---
    name: implement
    description: ...
    forked_from: a1b2c3d4
    ---

If the shipped default later changes, the cockpit surfaces that your copy is
behind — it is never auto-merged.
`, SourceRepo, SourceCommit)
}
