// Package sources is the skill-source registry: the operator's ordered list of
// places skills come from, replacing the layer model's three fixed roots and its
// closed name set. A source is a local folder or a pinned git checkout; the list's
// position *is* resolution order, so which skill wins is something the operator
// reads rather than infers.
//
// The list lives in `sources.toml` under the config root as an array of tables.
// There is no order field — and therefore no densification, no duplicate-order
// case and no legacy decode. Identity is the operator's own name: trimmed, 1–64
// characters of letters, digits, space, hyphen or underscore, no `/` (reserved by
// the qualified form `Source/skill`), unique case-insensitively. `enabled` sits on
// the row and is written only when false.
//
// The `chartr-skills` row is synthetic: never written as a row, always last, not
// removable and not reorderable. Only its scalars persist beside the rows — its
// toggle, and the commit and timestamp a refresh records. What lands at its path
// is the seed's business, not this package's.
//
// The stance is the space registry's — written atomically at 0600 under a 0700
// root, temp-then-rename, and a missing or unparseable file is the default row
// alone: the first-run state, not an error. Losing the file costs re-registration
// and never work, with one honest asymmetry: git checkouts are orphaned by the
// loss and chartr does not collect them. None of that registry's machinery is
// borrowed — the hashed id, the dense order and the legacy decode exist there
// because its rows serialise sorted by path and key per-space state by id, and
// neither pressure exists here.
//
// Discovery is a bounded, uncached walk: a skill is any directory at depth 1–3
// below a source root holding `SKILL.md` at its top level, and the walk never
// descends into one that has it — everything below is that skill's supporting
// files. Every caller walks; the walk stats directories and reads no file, so a
// skill folder created a second ago is usable in the very next spawn.
package sources

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"
	"unicode"

	"github.com/BurntSushi/toml"
)

// DefaultName is the fixed name of the synthetic default source. It is reserved:
// a hand-edited row claiming it is dropped, because the name belongs to the row
// chartr always seats last.
const DefaultName = "chartr-skills"

// The two kinds of source. `kind` is declared, never inferred — two kinds in one
// table, because two tables could not interleave and resolution is one ordered
// list.
const (
	KindDir = "dir"
	KindGit = "git"
)

// What a walk of a source found, reported on its row rather than raised as an
// error: a registered source that vanished or holds nothing is a fact the
// operator fixes, not a failure that stops a spawn.
const (
	StatusOK          = "ok"
	StatusUnavailable = "unavailable" // the path is gone; the row survives, never auto-removed
	StatusEmpty       = "empty"       // the path is there and yields no skills
)

const (
	fileName     = "sources.toml"
	skillFile    = "SKILL.md"
	checkoutsDir = "sources" // git checkouts live under <configDir>/sources/<hash>
	maxDepth     = 3         // a skill sits at depth 1–3 below a source root
	nameMax      = 64
)

// Errors the callers above this package distinguish. Everything else degrades
// into a warning on load, because a malformed file must never cost the operator
// the rest of their list.
var (
	ErrNotFound      = errors.New("sources: not found")
	ErrDuplicateName = errors.New("sources: a source by that name is already registered")
	ErrBadName       = errors.New("sources: a source name is 1-64 characters of letters, digits, space, hyphen or underscore, and may not contain '/'")
	ErrGitMissing    = errors.New("sources: git is not on PATH")
	ErrProtected     = errors.New("sources: the default source cannot be removed or reordered")
	ErrBadReorder    = errors.New("sources: a reorder must name every registered source exactly once")
)

// Source is one entry in the list: what the operator registered, plus the git
// state a pinned checkout carries on its own row. Enabled and Default are held
// in memory rather than read off the row as-is — Enabled because it is written
// only when false, Default because the default row is never written at all.
type Source struct {
	Name string `json:"name"`
	Kind string `json:"kind"`
	// Path is the local directory the walk reads. A dir source's is the
	// operator's folder; a git source's is chartr's checkout, keyed by a hash of
	// the URL so renaming a source is a pure metadata edit.
	Path    string    `json:"path"`
	URL     string    `json:"url,omitempty"`
	Ref     string    `json:"ref,omitempty"`
	Commit  string    `json:"commit,omitempty"`
	Fetched time.Time `json:"fetched,omitempty"`
	Enabled bool      `json:"enabled"`
	// Default marks the synthetic `chartr-skills` row: always last, not
	// removable, not reorderable.
	Default bool `json:"default,omitempty"`
}

// Skill is one directory a walk found: its basename, where it sits, and which
// source yielded it. Shadowed marks a name an earlier enabled source already
// claimed — not an error, it is what the order is for, and the loser stays
// reachable through the qualified form.
type Skill struct {
	Name     string `json:"name"`
	Dir      string `json:"dir"`
	Source   string `json:"source"`
	Shadowed bool   `json:"shadowed,omitempty"`
}

// State is a source with what a walk of it found — the shape a settings row and
// a payload's sources block are both built from. Warnings names what the walk
// itself lost: a duplicate basename inside this one source, where sorted walk
// order wins.
type State struct {
	Source
	Status   string   `json:"status"`
	Skills   []Skill  `json:"skills"`
	Warnings []string `json:"warnings,omitempty"`
}

// Registry is the in-memory list backed by <configDir>/sources.toml, safe for
// concurrent use: every mutation persists the whole file atomically.
type Registry struct {
	configDir string
	path      string

	mu       sync.Mutex
	rows     []Source // the operator's rows, in file order — resolution order
	def      Source   // the synthetic default row, always last
	warnings []string // what the load dropped, and why
}

// row is the on-disk shape. Enabled is a pointer so an absent key reads as
// enabled and a written `false` survives a round trip; Fetched is one so a row
// that has never been refreshed carries no zero timestamp.
type row struct {
	Name    string     `toml:"name"`
	Kind    string     `toml:"kind"`
	URL     string     `toml:"url,omitempty"`
	Ref     string     `toml:"ref,omitempty"`
	Commit  string     `toml:"commit,omitempty"`
	Path    string     `toml:"path,omitempty"`
	Fetched *time.Time `toml:"fetched,omitempty"`
	Enabled *bool      `toml:"enabled,omitempty"`
}

// file is the document. The default row's scalars sit beside the rows rather
// than among them — a [[source]] row means "a source the operator registered",
// which is what keeps deleting this file and re-registering those sources a
// complete recovery. They are declared before Sources because a scalar cannot
// follow an array of tables in TOML.
type file struct {
	DefaultEnabled *bool      `toml:"default_enabled,omitempty"`
	DefaultCommit  string     `toml:"default_commit,omitempty"`
	DefaultFetched *time.Time `toml:"default_fetched,omitempty"`

	Sources []row `toml:"source"`
}

// DefaultPath is where the default source's skills live: a named directory
// beside the git checkouts, which are keyed by hash and cannot collide with it.
// What lands there — the seed, or a checkout the operator pinned — is the seed's
// business.
func DefaultPath(configDir string) string {
	return filepath.Join(configDir, checkoutsDir, DefaultName)
}

// FilePath is where the list lives. It is exported because the absence of this
// file is what the first-run migration fires on: the one durable signal that no
// build with skill sources has ever started against this config root.
func FilePath(configDir string) string {
	return filepath.Join(configDir, fileName)
}

// HasSkills reports whether the bounded discovery walk finds at least one skill
// below dir. It is the same walk a registered source is read with, so a directory
// that would contribute no rows to the library answers no here — which is what
// lets the migration skip registering an empty legacy directory rather than
// leaving a permanently `empty` row behind.
func HasSkills(dir string) bool {
	if dir == "" {
		return false
	}
	return len(discover(dir, 1)) > 0
}

// Load reads the list under configDir. A missing or unparseable file is the
// default row alone — the first-run state, not an error — and a row this package
// cannot read is dropped with a warning while the rest of the list stands.
func Load(configDir string) (*Registry, error) {
	r := &Registry{
		configDir: configDir,
		path:      filepath.Join(configDir, fileName),
		def: Source{
			Name:    DefaultName,
			Kind:    KindGit,
			Path:    DefaultPath(configDir),
			Enabled: true,
			Default: true,
		},
	}

	data, err := os.ReadFile(r.path)
	if err != nil {
		return r, nil // missing, unreadable: first-run state
	}
	var f file
	if err := toml.Unmarshal(data, &f); err != nil {
		r.warnings = append(r.warnings, fmt.Sprintf("%s could not be parsed (%v); reading it as a fresh list with the default source alone", r.path, err))
		return r, nil
	}

	if f.DefaultEnabled != nil {
		r.def.Enabled = *f.DefaultEnabled
	}
	r.def.Commit = f.DefaultCommit
	if f.DefaultFetched != nil {
		r.def.Fetched = *f.DefaultFetched
	}

	seen := map[string]bool{}
	for _, in := range f.Sources {
		s, warn := readRow(in)
		if warn != "" {
			r.warnings = append(r.warnings, warn)
			continue
		}
		if seen[strings.ToLower(s.Name)] {
			r.warnings = append(r.warnings, fmt.Sprintf("two sources are named %q; the first row wins and the later one is dropped", s.Name))
			continue
		}
		seen[strings.ToLower(s.Name)] = true
		r.rows = append(r.rows, s)
	}
	return r, nil
}

// readRow validates one decoded row, returning the warning that drops it. Every
// rejection names the row, because a dropped source the operator cannot identify
// is worse than one that stayed.
func readRow(in row) (Source, string) {
	name := strings.TrimSpace(in.Name)
	if !validName(name) {
		return Source{}, fmt.Sprintf("a source named %q was dropped: %v", in.Name, ErrBadName)
	}
	if strings.EqualFold(name, DefaultName) {
		return Source{}, fmt.Sprintf("a row named %q was dropped: that name belongs to the default source", in.Name)
	}
	switch in.Kind {
	case KindDir:
		if strings.TrimSpace(in.Path) == "" {
			return Source{}, fmt.Sprintf("the source %q was dropped: a %q source needs a path", name, KindDir)
		}
	case KindGit:
		if strings.TrimSpace(in.URL) == "" || strings.TrimSpace(in.Path) == "" {
			return Source{}, fmt.Sprintf("the source %q was dropped: a %q source needs a url and a checkout path", name, KindGit)
		}
	default:
		return Source{}, fmt.Sprintf("the source %q was dropped: %q is not a kind of source I know", name, in.Kind)
	}

	s := Source{
		Name:    name,
		Kind:    in.Kind,
		Path:    strings.TrimSpace(in.Path),
		URL:     strings.TrimSpace(in.URL),
		Ref:     in.Ref,
		Commit:  in.Commit,
		Enabled: in.Enabled == nil || *in.Enabled,
	}
	if in.Fetched != nil {
		s.Fetched = *in.Fetched
	}
	return s, ""
}

// validName is the identity rule: trimmed, 1-64 characters of letters, digits,
// space, hyphen or underscore. `/` is excluded by construction — it separates a
// source from a skill in the qualified form.
func validName(name string) bool {
	if name == "" || len([]rune(name)) > nameMax || name != strings.TrimSpace(name) {
		return false
	}
	for _, c := range name {
		switch {
		case unicode.IsLetter(c), unicode.IsDigit(c), c == ' ', c == '-', c == '_':
		default:
			return false
		}
	}
	return true
}

// Warnings are what the load dropped: rows this package could not read, and the
// file it could not parse. They are advisory and never block a resolve.
func (r *Registry) Warnings() []string {
	r.mu.Lock()
	defer r.mu.Unlock()
	return append([]string(nil), r.warnings...)
}

// List returns the whole list in resolution order: the operator's rows in file
// order, then the synthetic default row. Disabled sources are listed — the
// screen shows them; only resolution skips them.
func (r *Registry) List() []Source {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.listLocked()
}

func (r *Registry) listLocked() []Source {
	out := make([]Source, 0, len(r.rows)+1)
	out = append(out, r.rows...)
	return append(out, r.def)
}

// Get returns one source by name, case-insensitively, including the default row.
func (r *Registry) Get(name string) (Source, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.getLocked(name)
}

func (r *Registry) getLocked(name string) (Source, bool) {
	for _, s := range r.listLocked() {
		if strings.EqualFold(s.Name, name) {
			return s, true
		}
	}
	return Source{}, false
}

// Walk reads one source's skills off disk. A vanished path reads as
// `unavailable` with zero skills, a path yielding nothing as `empty`; both keep
// the row. A duplicate basename inside one source is not an error — sorted walk
// order wins and the loser is named on the row.
func (r *Registry) Walk(name string) (State, bool) {
	s, ok := r.Get(name)
	if !ok {
		return State{}, false
	}
	return walk(s), true
}

// States walks every source and marks shadowing across the list: a skill name an
// earlier *enabled* source already claimed is shadowed here. A disabled source
// claims nothing, because resolution skips it.
func (r *Registry) States() []State {
	r.mu.Lock()
	list := r.listLocked()
	r.mu.Unlock()

	claimed := map[string]bool{}
	out := make([]State, 0, len(list))
	for _, s := range list {
		st := walk(s)
		for i, sk := range st.Skills {
			key := strings.ToLower(sk.Name)
			if claimed[key] {
				st.Skills[i].Shadowed = true
				continue
			}
			if s.Enabled {
				claimed[key] = true
			}
		}
		out = append(out, st)
	}
	return out
}

func walk(s Source) State {
	st := State{Source: s, Status: StatusOK, Skills: []Skill{}}
	if s.Path == "" {
		st.Status = StatusUnavailable
		return st
	}
	info, err := os.Stat(s.Path)
	if err != nil || !info.IsDir() {
		st.Status = StatusUnavailable
		return st
	}

	seen := map[string]string{}
	for _, dir := range discover(s.Path, 1) {
		name := filepath.Base(dir)
		if first, dup := seen[strings.ToLower(name)]; dup {
			st.Warnings = append(st.Warnings, fmt.Sprintf("%q appears twice in %q: %s wins over %s", name, s.Name, first, dir))
			continue
		}
		seen[strings.ToLower(name)] = dir
		st.Skills = append(st.Skills, Skill{Name: name, Dir: dir, Source: s.Name})
	}
	if len(st.Skills) == 0 {
		st.Status = StatusEmpty
	}
	return st
}

// discover is the bounded walk: directories at depth 1-3 below the root holding
// SKILL.md at their top level, in sorted order, never descending into one that
// has it. Dot-entries and node_modules are skipped; an unreadable directory
// simply contributes nothing.
func discover(dir string, depth int) []string {
	if depth > maxDepth {
		return nil
	}
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil
	}
	names := make([]string, 0, len(entries))
	for _, e := range entries {
		if !e.IsDir() && e.Type()&os.ModeSymlink == 0 {
			continue
		}
		n := e.Name()
		if strings.HasPrefix(n, ".") || n == "node_modules" {
			continue
		}
		names = append(names, n)
	}
	sort.Strings(names)

	var out []string
	for _, n := range names {
		p := filepath.Join(dir, n)
		if info, err := os.Stat(p); err != nil || !info.IsDir() {
			continue
		}
		if info, err := os.Stat(filepath.Join(p, skillFile)); err == nil && !info.IsDir() {
			out = append(out, p) // a skill; everything below is its supporting files
			continue
		}
		out = append(out, discover(p, depth+1)...)
	}
	return out
}

// Resolve takes both forms of reference. A bare name searches enabled sources in
// file order and then the default row, first hit winning. A `Source/skill`
// reference addresses one source exactly and **never falls through**: naming a
// source and silently receiving a different source's skill is worse than an
// error. A disabled source is skipped by both forms, and a qualified reference
// into one says so, because that is the failure fixed in a click.
func (r *Registry) Resolve(ref string) (Skill, error) {
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return Skill{}, fmt.Errorf("%w: no skill named", ErrNotFound)
	}

	if i := strings.Index(ref, "/"); i >= 0 {
		srcName, skillName := strings.TrimSpace(ref[:i]), strings.TrimSpace(ref[i+1:])
		s, ok := r.Get(srcName)
		if !ok {
			return Skill{}, fmt.Errorf("%w: no source named %q, so %q resolves to nothing", ErrNotFound, srcName, ref)
		}
		if !s.Enabled {
			return Skill{}, fmt.Errorf("%w: the source %q is disabled, so %q resolves to nothing", ErrNotFound, s.Name, ref)
		}
		for _, sk := range walk(s).Skills {
			if strings.EqualFold(sk.Name, skillName) {
				return sk, nil
			}
		}
		return Skill{}, fmt.Errorf("%w: the source %q has no skill %q", ErrNotFound, s.Name, skillName)
	}

	for _, s := range r.List() {
		if !s.Enabled {
			continue
		}
		for _, sk := range walk(s).Skills {
			if strings.EqualFold(sk.Name, ref) {
				return sk, nil
			}
		}
	}
	return Skill{}, fmt.Errorf("%w: no enabled source has a skill named %q", ErrNotFound, ref)
}

// RegisterDir appends a local folder to the list. The path is cleaned to an
// absolute form and nothing else is required of it: a folder that is empty
// today, or gone tomorrow, is a status on the row rather than a refusal here.
func (r *Registry) RegisterDir(name, path string) (Source, error) {
	name = strings.TrimSpace(name)
	if !validName(name) {
		return Source{}, fmt.Errorf("%w (got %q)", ErrBadName, name)
	}
	abs, err := filepath.Abs(path)
	if err != nil {
		return Source{}, fmt.Errorf("sources: resolving %q: %w", path, err)
	}

	r.mu.Lock()
	defer r.mu.Unlock()
	if err := r.freeNameLocked(name); err != nil {
		return Source{}, err
	}
	s := Source{Name: name, Kind: KindDir, Path: filepath.Clean(abs), Enabled: true}
	r.rows = append(r.rows, s)
	if err := r.saveLocked(); err != nil {
		r.rows = r.rows[:len(r.rows)-1]
		return Source{}, err
	}
	return s, nil
}

// Remove forgets a source. A dir source's folder is untouched; a git source's
// checkout is chartr's own, so it goes with the row. The default row cannot be
// removed.
func (r *Registry) Remove(name string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if strings.EqualFold(strings.TrimSpace(name), DefaultName) {
		return ErrProtected
	}
	for i, s := range r.rows {
		if !strings.EqualFold(s.Name, name) {
			continue
		}
		r.rows = append(r.rows[:i:i], r.rows[i+1:]...)
		if err := r.saveLocked(); err != nil {
			return err
		}
		if s.Kind == KindGit && s.Path != "" && strings.HasPrefix(s.Path, filepath.Join(r.configDir, checkoutsDir)) {
			_ = os.RemoveAll(s.Path)
		}
		return nil
	}
	return fmt.Errorf("%w: no source named %q", ErrNotFound, name)
}

// SetEnabled parks a source without removing it, the default row included:
// disabled means one thing, and both forms of resolution skip it.
func (r *Registry) SetEnabled(name string, enabled bool) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if strings.EqualFold(strings.TrimSpace(name), DefaultName) {
		r.def.Enabled = enabled
		return r.saveLocked()
	}
	for i, s := range r.rows {
		if strings.EqualFold(s.Name, name) {
			r.rows[i].Enabled = enabled
			return r.saveLocked()
		}
	}
	return fmt.Errorf("%w: no source named %q", ErrNotFound, name)
}

// Reorder sets the list to names: the whole ordered list, applied as file order.
// It is deliberately not a per-row move — a full-list write is idempotent and
// cannot leave the list half-moved when the operator drags twice in quick
// succession. The default row is not part of it: it is always last. The list is
// validated in full before anything is touched.
func (r *Registry) Reorder(names []string) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	if len(names) != len(r.rows) {
		return fmt.Errorf("%w: %d names for %d sources", ErrBadReorder, len(names), len(r.rows))
	}
	byName := make(map[string]Source, len(r.rows))
	for _, s := range r.rows {
		byName[strings.ToLower(s.Name)] = s
	}
	next := make([]Source, 0, len(names))
	seen := map[string]bool{}
	for _, n := range names {
		key := strings.ToLower(strings.TrimSpace(n))
		if key == strings.ToLower(DefaultName) {
			return ErrProtected
		}
		s, ok := byName[key]
		if !ok {
			return fmt.Errorf("%w: no source named %q", ErrBadReorder, n)
		}
		if seen[key] {
			return fmt.Errorf("%w: %q appears twice", ErrBadReorder, n)
		}
		seen[key] = true
		next = append(next, s)
	}

	prev := r.rows
	r.rows = next
	if err := r.saveLocked(); err != nil {
		r.rows = prev
		return err
	}
	return nil
}

// freeNameLocked refuses a name already taken, case-insensitively, and the
// reserved default name — before anything is created on disk. The caller holds
// r.mu.
func (r *Registry) freeNameLocked(name string) error {
	if strings.EqualFold(name, DefaultName) {
		return fmt.Errorf("%w: %q is the default source's name", ErrDuplicateName, DefaultName)
	}
	for _, s := range r.rows {
		if strings.EqualFold(s.Name, name) {
			return fmt.Errorf("%w: %q", ErrDuplicateName, s.Name)
		}
	}
	return nil
}

// Save writes the list as it stands. Every mutation on the seam already saves,
// so this exists for the one caller that must guarantee the *file* exists even
// when it registered nothing: the first-run migration, which fires on the file's
// absence and would otherwise run again at every startup on a machine with
// nothing to migrate.
func (r *Registry) Save() error {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.saveLocked()
}

// saveLocked writes the whole list atomically (temp file + rename) so a crash
// mid-write cannot corrupt it. The file records the absolute paths of the
// operator's skill folders and the URLs they fetch from, which is nobody else's
// business on a shared machine, so it is owner-only under an owner-only root.
// The chmod is not redundant with the write mode: a temp file left behind by a
// crashed save already exists, and os.WriteFile only applies its mode when it
// creates the file. The caller holds r.mu.
func (r *Registry) saveLocked() error {
	if err := os.MkdirAll(r.configDir, 0o700); err != nil {
		return fmt.Errorf("sources: creating config dir: %w", err)
	}

	f := file{Sources: make([]row, 0, len(r.rows))}
	if !r.def.Enabled {
		off := false
		f.DefaultEnabled = &off
	}
	f.DefaultCommit = r.def.Commit
	if !r.def.Fetched.IsZero() {
		t := r.def.Fetched.UTC()
		f.DefaultFetched = &t
	}
	for _, s := range r.rows {
		out := row{Name: s.Name, Kind: s.Kind, URL: s.URL, Ref: s.Ref, Commit: s.Commit, Path: s.Path}
		if !s.Fetched.IsZero() {
			t := s.Fetched.UTC()
			out.Fetched = &t
		}
		if !s.Enabled {
			off := false
			out.Enabled = &off
		}
		f.Sources = append(f.Sources, out)
	}

	data, err := toml.Marshal(f)
	if err != nil {
		return fmt.Errorf("sources: encoding: %w", err)
	}
	tmp := r.path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o600); err != nil {
		return fmt.Errorf("sources: writing %s: %w", tmp, err)
	}
	if err := os.Chmod(tmp, 0o600); err != nil {
		return fmt.Errorf("sources: securing %s: %w", tmp, err)
	}
	if err := os.Rename(tmp, r.path); err != nil {
		return fmt.Errorf("sources: replacing %s: %w", r.path, err)
	}
	return nil
}

// checkoutPath is where a git source's clone lands: keyed by a hash of its URL,
// so renaming a source is a pure metadata edit and two rows never fight over one
// directory.
func (r *Registry) checkoutPath(url string) string {
	sum := sha256.Sum256([]byte(url))
	return filepath.Join(r.configDir, checkoutsDir, hex.EncodeToString(sum[:])[:12])
}
