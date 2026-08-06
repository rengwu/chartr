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
// chartr's embedded core and the bound role's body with its frontmatter stripped,
// appends the conventions pointer and the operator's preferences, and assembles
// them with a context region (the sources block, and for a ticket session the map
// body, the ticket, and its blockers' answers) into one markdown document, built
// fresh every time and never accumulated. Supporting files stay on disk rather
// than being inlined, so a session can zoom into them on demand at no payload
// cost.
package prompt

import (
	"bytes"
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

// The skills chartr knows by name. CoreSkill is the common core — read straight
// out of the binary by composition, never through the layers, because it is
// chartr's own voice and not the operator's to shadow; IdeateSkill is the ideate
// on-ramp, composed alone (no core, no context bundle) because an ideate session
// is ticketless and mapless. The four method skills — WayfinderSkill,
// DomainSkill, SpecSkill, TicketsSkill — are shipped, resolved, and materialized
// like the rest, but never auto-composed into a session payload: they serve
// charting, speccing, and ticket-breaking work done outside a composed session.
const (
	CoreSkill   = "core"
	IdeateSkill = "ideate"

	WayfinderSkill = "wayfinder"
	DomainSkill    = "domain-modeling"
	SpecSkill      = "to-spec"
	TicketsSkill   = "to-tickets"
)

// Skill layer tags: the shipped floor, the operator's own fork, or a space's
// committed library. They name where a *library* skill resolved from; a payload
// part carries an Origin instead, which is an open string.
const (
	LayerBuiltin   = "built-in"
	LayerUser      = "user"
	LayerWorkspace = "workspace"
)

// Part origins — where a block of the payload came from, as the preview badges
// it. The set is open on purpose: a resolved skill body's origin is the
// *registered source's name*, so the badge answers the one silent failure source
// order can cause (which source's copy of this skill actually ran). These three
// are the fixed members: chartr's own embedded text, the operator's preferences,
// and anything assembled fresh at compose time.
const (
	OriginChartr   = "chartr"
	OriginOperator = "operator"
	OriginContext  = "context"
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

	// Source and Commit name where a skill resolved from once it comes through the
	// source registry rather than the three layers: the registered source's name,
	// and the commit that source is pinned at where it carries one. A skill with a
	// Source has no Layer and no Hash — the pair replaces both on the claim
	// trailer, because a source name plus a commit identifies bytes a teammate can
	// fetch, where a content hash only says that something differed.
	Source string `json:"source,omitempty"`
	Commit string `json:"commit,omitempty"`

	// OnRamp marks a self-driving skill the launcher may open cold from the
	// sidebar (`on-ramp: true` in its frontmatter). NeedsContext marks one that
	// offers an optional one-line context box before it launches
	// (`needs-context: true`). Both ride whole-skill shadowing: a shadowing
	// layer's SKILL.md declares its own flags, so a user or workspace skill sets
	// its own on-ramp status.
	OnRamp       bool `json:"onRamp,omitempty"`
	NeedsContext bool `json:"needsContext,omitempty"`

	Body string `json:"-"`
}

// Part is one labelled block of the payload — chartr's core, a resolved role
// skill, the conventions pointer, the operator's preferences, or a context
// artifact (the sources block, the map, this ticket, a blocker's answer). Kind is
// "prompt" or "context"; Origin is where the text came from, and is what the
// preview badges.
//
// A part is one contiguous block of text. It was a list of tagged segments while
// a per-field merge across the skill layers looked likely; whole-skill shadowing
// never produced a second segment, and resolution through sources cannot, so the
// machinery is gone.
type Part struct {
	Name   string `json:"name"`
	Kind   string `json:"kind"`
	Origin string `json:"origin"`
	Label  string `json:"label,omitempty"`
	Text   string `json:"text"`
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
// roles, the ideate on-ramp, then the four method skills.
func Names() []string {
	names := []string{CoreSkill}
	for _, r := range config.Roles {
		names = append(names, string(r))
	}
	return append(names, IdeateSkill,
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
		Name:         name,
		Layer:        layer,
		Dir:          dir,
		Description:  meta["description"],
		ForkedFrom:   strings.ToLower(meta["forked_from"]),
		Hash:         hashFiles(files),
		OnRamp:       parseBool(meta["on-ramp"]),
		NeedsContext: parseBool(meta["needs-context"]),
		Body:         strings.TrimSpace(body),
	}
	s.Stale = s.ForkedFrom != "" && s.ForkedFrom != ShippedHash(name)
	return s
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

// MatchesShipped reports whether the materialized library at dir is still exactly
// what Materialize would write — the same file set, byte for byte, README
// included. An absent or empty directory reads as a match: there is nothing there
// an operator could have edited. An unreadable one reads as a mismatch, because
// the only thing this answer gates is whether chartr may move the directory
// aside, and a directory it cannot read is one it must leave alone.
//
// This is the byte comparison the skill-sources migration borrows for its one
// call site (ticket 06). It goes with the shipped library in ticket 09; nothing
// else should grow a dependency on it.
func MatchesShipped(dir string) bool {
	if dir == "" {
		return true
	}
	have, ok := treeFiles(dir)
	if !ok {
		return false
	}
	if len(have) == 0 {
		return true
	}
	want := map[string][]byte{"README.md": []byte(readmeText())}
	for _, name := range Names() {
		files, ok := embeddedFiles(name)
		if !ok {
			continue
		}
		for rel, b := range files {
			want[name+"/"+rel] = b
		}
	}
	if len(want) != len(have) {
		return false
	}
	for rel, b := range want {
		if !bytes.Equal(b, have[rel]) {
			return false
		}
	}
	return true
}

// treeFiles reads every file below dir, keyed by slash-separated relative path.
// An absent directory is an empty set rather than a failure — the first-run
// state. Reports false only when the walk itself failed.
func treeFiles(dir string) (map[string][]byte, bool) {
	files := map[string][]byte{}
	err := filepath.WalkDir(dir, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			if os.IsNotExist(err) {
				return nil
			}
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

// Launch composes an on-ramp skill's payload: the named skill's resolved body
// **alone** — no core, no role, no context bundle — exactly as the ideate on-ramp
// is composed, because an on-ramp launch is ticketless and mapless by design
// (there is no ticket or map to inject). When context is non-empty it rides in the
// same payload under a short `## Your task` trailer, so the agent reads its brief
// from the one on-disk file it already opens rather than a fragile typed-in second
// line; an empty context writes the body unchanged. Returns nil when the named
// skill resolves in no layer. Editing the resolved skill changes what the very
// next launch reads.
func Launch(roots Roots, skill, context string) []byte {
	s, ok := Resolve(skill, roots)
	if !ok {
		return nil
	}
	body := s.Body
	if c := strings.TrimSpace(context); c != "" {
		body += "\n\n---\n\n## Your task\n\n" + c
	}
	return []byte(body)
}

// Ideate returns the ideate on-ramp's resolved body — Launch pinned to the ideate
// skill with no context, kept so the `/ideate` route and its callers read
// unchanged while the launcher generalises the same spine to any on-ramp skill.
func Ideate(roots Roots) string {
	return string(Launch(roots, IdeateSkill, ""))
}

// parseBool reads a frontmatter boolean flag tolerantly (true/false, yes/1, and
// their casings); anything it cannot read as true is false, so an absent or
// malformed flag simply leaves the skill off the launcher.
func parseBool(v string) bool {
	switch strings.ToLower(strings.TrimSpace(v)) {
	case "true", "yes", "1":
		return true
	}
	return false
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
