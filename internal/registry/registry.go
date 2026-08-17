// Package registry is the space registry: the operator's list of registered
// spaces plus the one synthetic Scratch entry, held in memory as a rebuildable
// index rather than a source of truth (ticket 02, ADR 0003). Everything
// authoritative — maps, committed workspace config — lives in the space folders;
// the registry holds only registered paths, the operator's sidebar order, and
// each space's local recency. Losing it costs re-adding folders, never work
// (their arrangement included), so a deleted spaces.toml is not an error: the
// operator re-registers and each folder picks up exactly as it sits.
//
// Registering runs no version-control command of its own: chartr does not init,
// commit, or otherwise touch the operator's VCS (ADR 0008, revised), so a plain
// folder, a git repo, a jujutsu or mercurial tree, and a subdirectory deep inside
// a monorepo all register the same way — the selected folder becomes a space and
// nothing on disk is turned into a repository. De-registering forgets the entry
// and touches nothing in the folder — forget, not destroy (story 4).
package registry

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

	"github.com/BurntSushi/toml"
)

// ScratchID is the fixed identity of chartr's one synthetic Scratch space. A
// registered space id is twelve hexadecimal characters derived from its absolute
// path, so this human-readable value cannot collide with one.
const ScratchID = "scratch"

// Entry is one space in the in-memory registry: its path plus the local,
// per-machine order and recency. A registered entry's ID is derived from its
// path; Scratch instead carries the fixed ScratchID and is never persisted.
type Entry struct {
	ID   string `toml:"-"`
	Path string `toml:"path"`
	// Scratch marks the one built-in entry. It is held in memory alongside the
	// registered entries but is never encoded as a [[space]] row: the only part of
	// it that reaches the file is its Order, written as a scalar beside the rows.
	Scratch bool `toml:"-"`
	// Order is the space's position in the sidebar: the operator's arrangement,
	// stored rather than derived, and the only thing List sorts by. It is dense
	// (0..n-1 across the whole registry) and rewritten wholesale on every save,
	// so a duplicate or missing index is unrepresentable after any successful
	// write. A file carrying no order at all — one written before the sidebar had
	// one — is frozen into today's sequence on load rather than reset, which is
	// what makes the upgrade invisible.
	Order      int       `toml:"order"`
	LastActive time.Time `toml:"last_active"`
	// LastAgent is the registered agent this space last spawned with — state, not
	// config: chartr writes it after a successful launch and nothing edits it.
	// It sits here because the registry is already per-space, local, chartr-owned
	// and rebuildable, which is exactly this value's lifecycle. A name that no
	// longer resolves against the library is *not* rewritten away: it is reported
	// as it stands and read as nothing remembered, so deleting an agent costs no
	// registry surgery.
	LastAgent string `toml:"last_agent,omitempty"`
	// Name is the operator's own display name for the space, overriding the folder
	// basename the sidebar shows by default. It is presentation only — purely how
	// the space reads in the chrome, changing nothing about the path, the id, or
	// what any action does — and it belongs here for the same reasons Order and
	// LastAgent do: per-space, per-machine, chartr-owned and rebuildable, keyed by
	// the path-derived id so a re-register of the same folder lines back up with
	// it. Empty means no override: the folder basename stands.
	Name string `toml:"name,omitempty"`
	// Prompts are the catalog ids this space selects at launch — the operator's
	// `At launch` choice, remembered per space. It is a set, not an order: the
	// catalog's own creation order is the only order there is, and composition
	// follows it however these ids happen to sit here. The ids are held exactly as
	// written: one the catalog no longer holds is surfaced rather than rewritten
	// away, because a selection that quietly repaired itself would hide a preset
	// the operator lost. It lives here for the same reasons LastAgent does —
	// per-space, per-machine, chartr-owned, rebuildable.
	Prompts []string `toml:"prompts,omitempty"`
}

// Registry is the in-memory registry backed by <dataDir>/spaces.toml. It is
// safe for concurrent use: the HTTP action handlers mutate it from many
// goroutines, and every mutation persists the whole file atomically.
type Registry struct {
	path string // the spaces.toml file

	mu      sync.Mutex
	entries map[string]Entry // keyed by ID
}

// Load opens the registry under dataDir, reading spaces.toml if it exists, then
// seats Scratch in memory at the slot the file recorded for it. A missing file
// yields Scratch alone — that is the first-run state, not an error, and the same
// state a lost registry recovers from.
func Load(dataDir string) (*Registry, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("registry: resolving the operator's home directory: %w", err)
	}
	r := &Registry{
		path:    filepath.Join(dataDir, "spaces.toml"),
		entries: map[string]Entry{},
	}
	data, err := os.ReadFile(r.path)
	if os.IsNotExist(err) {
		r.addScratch(home, nil)
		return r, nil
	}
	if err != nil {
		return nil, fmt.Errorf("registry: reading %s: %w", r.path, err)
	}
	var f file
	if err := toml.Unmarshal(data, &f); err != nil {
		return nil, fmt.Errorf("registry: parsing %s: %w", r.path, err)
	}
	// A missing `order` and an order of 0 decode into the same int, so the rows
	// are read a second time as raw tables purely to learn what the typed Entry
	// cannot say. The document has already parsed once, so this cannot fail; both
	// decodes see the rows in file order.
	var raw struct {
		Spaces []map[string]any `toml:"space"`
	}
	_ = toml.Unmarshal(data, &raw)
	rows := make([]legacyRow, len(f.Spaces))
	for i := range f.Spaces {
		f.Spaces[i].ID = spaceID(f.Spaces[i].Path)
		if i < len(raw.Spaces) {
			_, rows[i].hasOrder = raw.Spaces[i]["order"]
			rows[i].pinned, _ = raw.Spaces[i]["pinned"].(bool)
		}
	}
	for _, e := range ordered(f.Spaces, rows) {
		r.entries[e.ID] = e
	}
	r.addScratch(home, f.ScratchOrder)
	return r, nil
}

// addScratch splices the synthetic entry into the loaded sidebar sequence at the
// slot the file recorded, and densifies what comes out. Its path follows this
// machine and is intentionally never read from spaces.toml — only its slot is.
//
// The registered rows arrive already compacted by ordered, so the file's gap
// where Scratch sat is closed before the splice and reopened by it: a save that
// wrote rows 0, 2, 3 beside scratch_order = 1 loads back to exactly that
// sequence. A file with no recorded slot — one written before Scratch had a
// remembered place, or before Scratch existed — appends it at the end, which is
// the arrangement the operator already had with one row after it. An index the
// sequence cannot hold degrades the same way, the way a mangled order does
// everywhere else here: the arrangement is what suffers, never the list.
func (r *Registry) addScratch(home string, slot *int) {
	seq := make([]Entry, 0, len(r.entries)+1)
	for _, e := range r.entries {
		seq = append(seq, e)
	}
	sortByOrder(seq)

	at := len(seq)
	if slot != nil && *slot >= 0 && *slot < at {
		at = *slot
	}
	scratch := Entry{ID: ScratchID, Path: filepath.Clean(home), Scratch: true}
	seq = append(seq, Entry{})
	copy(seq[at+1:], seq[at:])
	seq[at] = scratch

	for i, e := range seq {
		e.Order = i
		r.entries[e.ID] = e
	}
}

type file struct {
	// ScratchOrder is the Scratch space's slot in the sidebar, and the only thing
	// about the synthetic entry that is persisted. It is a scalar beside the rows
	// rather than a row of its own because a [[space]] row means "a folder the
	// operator registered", which is what keeps deleting this file and re-adding
	// those folders a complete recovery — Scratch needs no recovering, being
	// rebuilt from nothing on every run.
	//
	// It is a pointer so that a file carrying no slot at all is told apart from
	// one that records slot 0, exactly as the rows' own `order` is. It is written
	// on every save and read as the splice index on every load; it is declared
	// before Spaces because a scalar cannot follow an array of tables in TOML.
	ScratchOrder *int `toml:"scratch_order,omitempty"`

	Spaces []Entry `toml:"space"`
}

// legacyRow is what the raw second decode learns about a row that its typed
// Entry cannot say, and both facts matter only to the migration.
//
// hasOrder: whether the row carried an `order` key at all. Presence is what tells
// a pre-upgrade file — freeze it — apart from a hand-edited one, which degrades.
//
// pinned: the deleted `pinned` flag. It is gone from Entry, from the wire model
// and from the HTTP surface, and is never written again — but the freeze is
// defined as *today's* sequence, and today's sequence put pinned spaces first.
// Reading the dead key here, and only here, is what keeps the upgrade invisible
// for an operator whose file still carries it; it orders nothing afterwards.
type legacyRow struct {
	hasOrder bool
	pinned   bool
}

// ordered resolves the sidebar sequence of a just-loaded file and densifies it,
// with rows[i] carrying what the raw decode learned about entry i.
//
// It is one rule doing two jobs. Entries with a unique order are well-formed and
// keep their sequence; entries with a duplicate or missing order are malformed,
// are sorted among themselves by the *old* rule — pinned first, then recency,
// then path — and appended after the well-formed ones. A pre-upgrade file, where
// no entry carries an order, is simply the case where every entry is malformed:
// the whole list sorts by the old rule and freezes into 0..n-1, so the sidebar
// after the upgrade is the sidebar before it. This is the only place the deleted
// `pinned` key is still read, and it is read as history: what the sidebar looked
// like at the moment of the upgrade, never as an ordering input afterwards.
// A hand-edit or a truncated file costs the operator their arrangement, never
// their list of spaces.
//
// The result is written back on the next save, not here: loading the registry
// stays a read.
func ordered(entries []Entry, rows []legacyRow) []Entry {
	count := map[int]int{}
	for i, e := range entries {
		if rows[i].hasOrder {
			count[e.Order]++
		}
	}

	// The malformed are held as indices, not entries, so each one's raw row — the
	// only place its `pinned` still lives — travels with it into the sort.
	wellFormed := make([]Entry, 0, len(entries))
	malformed := make([]int, 0, len(entries))
	for i, e := range entries {
		if rows[i].hasOrder && count[e.Order] == 1 {
			wellFormed = append(wellFormed, e)
		} else {
			malformed = append(malformed, i)
		}
	}

	sortByOrder(wellFormed)
	sort.SliceStable(malformed, func(a, b int) bool {
		i, j := malformed[a], malformed[b]
		if rows[i].pinned != rows[j].pinned {
			return rows[i].pinned
		}
		if !entries[i].LastActive.Equal(entries[j].LastActive) {
			return entries[i].LastActive.After(entries[j].LastActive)
		}
		return entries[i].Path < entries[j].Path // stable tiebreak
	})

	out := wellFormed
	for _, i := range malformed {
		out = append(out, entries[i])
	}
	for i := range out {
		out[i].Order = i
	}
	return out
}

// sortByOrder puts entries in sidebar sequence. The path tiebreak only exists to
// make the sort total: orders are unique after any load or save, and the map the
// entries come out of has no iteration order of its own.
func sortByOrder(entries []Entry) {
	sort.Slice(entries, func(i, j int) bool {
		if entries[i].Order != entries[j].Order {
			return entries[i].Order < entries[j].Order
		}
		return entries[i].Path < entries[j].Path
	})
}

// Register makes path a space. It cleans the path to an absolute form and
// requires it to be an existing directory; it runs no version-control command
// and never turns the folder into a repository (ADR 0008, revised), so a folder
// deep inside a monorepo registers as itself rather than sprouting a nested
// repo. A new space appends: it takes max(order)+1, because the stored order
// belongs to the operator and registering a folder is not a rearrangement.
// Registering an already-registered path just refreshes its recency and leaves
// it where it sits.
func (r *Registry) Register(path string) (Entry, error) {
	abs, err := filepath.Abs(path)
	if err != nil {
		return Entry{}, fmt.Errorf("registry: resolving %q: %w", path, err)
	}
	abs = filepath.Clean(abs)

	info, err := os.Stat(abs)
	if err != nil {
		return Entry{}, fmt.Errorf("registry: %q is not a folder I can register: %w", path, err)
	}
	if !info.IsDir() {
		return Entry{}, fmt.Errorf("registry: %q is a file, not a folder", path)
	}

	id := spaceID(abs)

	r.mu.Lock()
	e, existed := r.entries[id]
	if !existed {
		e = Entry{ID: id, Path: abs, Order: r.nextOrderLocked()}
	}
	e.LastActive = time.Now().UTC()
	r.entries[id] = e
	err = r.saveLocked()
	r.mu.Unlock()
	if err != nil {
		return Entry{}, err
	}
	return e, nil
}

// Deregister forgets a space: it removes the registry entry and its local order
// and recency, and touches nothing in the repository. Re-register any time and
// the repo picks up exactly as it sits. Forgetting an unknown ID is a no-op.
func (r *Registry) Deregister(id string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	// The HTTP refusal belongs to the repo-scoped action guard (ticket 02), but
	// the registry invariant is already absolute: Scratch is always present.
	if id == ScratchID {
		return nil
	}
	if _, ok := r.entries[id]; !ok {
		return nil
	}
	delete(r.entries, id)
	return r.saveLocked()
}

// nextOrderLocked is the order a newly registered space takes: one past the last
// row of the sidebar, so it appends. The caller holds r.mu.
func (r *Registry) nextOrderLocked() int {
	next := 0
	for _, e := range r.entries {
		if e.Order >= next {
			next = e.Order + 1
		}
	}
	return next
}

// ErrBadReorder marks a reorder that does not name every registered space
// exactly once. Scratch alone may be absent because the chrome hides it while it
// is empty; every other omission remains a client bug.
var ErrBadReorder = errors.New("registry: a reorder must name every registered space exactly once")

// Reorder sets the sidebar sequence to ids: the whole ordered list, applied as
// order 0..n-1. It is deliberately not a per-row move — a full-list write is
// idempotent, it is already the shape every save writes, and it cannot leave the
// sidebar half-moved when the operator drags twice in quick succession.
//
// The list is validated in full before anything is touched, so a rejected
// reorder leaves the arrangement exactly as it was. If Scratch is omitted it is
// spliced back at its current slot before the orders are applied.
func (r *Registry) Reorder(ids []string) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	if len(ids) != len(r.entries) && len(ids) != len(r.entries)-1 {
		return fmt.Errorf("%w: %d ids for %d spaces", ErrBadReorder, len(ids), len(r.entries))
	}
	seen := make(map[string]bool, len(ids))
	for _, id := range ids {
		if _, ok := r.entries[id]; !ok {
			return fmt.Errorf("%w: no space %q", ErrBadReorder, id)
		}
		if seen[id] {
			return fmt.Errorf("%w: %q appears twice", ErrBadReorder, id)
		}
		seen[id] = true
	}
	for id, e := range r.entries {
		if !e.Scratch && !seen[id] {
			return fmt.Errorf("%w: missing registered space %q", ErrBadReorder, id)
		}
	}

	order := append([]string(nil), ids...)
	if !seen[ScratchID] {
		slot := r.entries[ScratchID].Order
		if slot > len(order) {
			slot = len(order)
		}
		order = append(order, "")
		copy(order[slot+1:], order[slot:])
		order[slot] = ScratchID
	}
	if len(order) != len(r.entries) {
		return fmt.Errorf("%w: %d ids for %d spaces", ErrBadReorder, len(ids), len(r.entries))
	}

	for i, id := range order {
		e := r.entries[id]
		e.Order = i
		r.entries[id] = e
	}
	return r.saveLocked()
}

// SetLastAgent records the registered agent a space just spawned with, so the
// next spawn there can reuse it without asking again. Only a *successful* launch
// calls it, so a refused spawn leaves the memory exactly as it was. Recording an
// unknown ID, or the name already held, is a no-op.
func (r *Registry) SetLastAgent(id, name string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	e, ok := r.entries[id]
	if !ok || e.LastAgent == name {
		return nil
	}
	e.LastAgent = name
	r.entries[id] = e
	return r.saveLocked()
}

// Rename sets a space's display name, or clears the override when name is empty
// (after trimming) so the folder basename stands again. Presentation only: it
// touches nothing but this one field, so it never reorders the sidebar or moves a
// row. Renaming an unknown ID, or setting the name already held, is a no-op.
// Scratch is rebuilt from nothing on every run and carries no persisted row, so
// there is nothing here to rename — the repo-scoped action guard refuses it, and
// this stays a no-op for it besides.
func (r *Registry) Rename(id, name string) error {
	name = strings.TrimSpace(name)
	r.mu.Lock()
	defer r.mu.Unlock()
	if id == ScratchID {
		return nil
	}
	e, ok := r.entries[id]
	if !ok || e.Name == name {
		return nil
	}
	e.Name = name
	r.entries[id] = e
	return r.saveLocked()
}

// SetPrompts records the prompt presets a space launches agents with: the whole
// selected list in one write, never a per-row toggle, so two clicks in quick
// succession cannot leave a half-applied selection. A repeat is dropped — the
// selection is a set — and an empty list is a legitimate value meaning nothing is
// selected. Scratch carries no persisted row, so setting one there is a no-op the
// way a rename of it is; the repo-scoped action guard refuses it above.
func (r *Registry) SetPrompts(id string, ids []string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if id == ScratchID {
		return nil
	}
	e, ok := r.entries[id]
	if !ok {
		return nil
	}
	next := make([]string, 0, len(ids))
	seen := map[string]bool{}
	for _, p := range ids {
		if p == "" || seen[p] {
			continue
		}
		seen[p] = true
		next = append(next, p)
	}
	if equalIDs(e.Prompts, next) {
		return nil
	}
	e.Prompts = next
	r.entries[id] = e
	return r.saveLocked()
}

// RemovePrompt drops a deleted preset's id from every space that selected it, so
// deleting one from the catalog leaves no dangling reference behind. It is one
// write of the whole registry: an id nothing selected changes nothing at all.
func (r *Registry) RemovePrompt(promptID string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	changed := false
	for id, e := range r.entries {
		next := make([]string, 0, len(e.Prompts))
		for _, p := range e.Prompts {
			if p != promptID {
				next = append(next, p)
			}
		}
		if len(next) == len(e.Prompts) {
			continue
		}
		e.Prompts = next
		r.entries[id] = e
		changed = true
	}
	if !changed {
		return nil
	}
	return r.saveLocked()
}

func equalIDs(a, b []string) bool {
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

// List returns the entries ordered as the sidebar shows them: the operator's
// stored order, and nothing else. Recency is still recorded but sorts nothing,
// and an actionable signal may flag a row but never re-sorts it — so only an
// explicit reorder, or a registration appending at the end, moves a row.
func (r *Registry) List() []Entry {
	r.mu.Lock()
	out := make([]Entry, 0, len(r.entries))
	for _, e := range r.entries {
		out = append(out, e)
	}
	r.mu.Unlock()

	sortByOrder(out)
	return out
}

// Get returns one entry by ID.
func (r *Registry) Get(id string) (Entry, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	e, ok := r.entries[id]
	return e, ok
}

// saveLocked writes the whole registry atomically (temp file + rename) so a
// crash mid-write cannot corrupt it, and densifies the order first so every
// successful write leaves 0..n-1 on disk. The caller holds r.mu.
//
// The file is the absolute path of every repository the operator works in, which
// is nobody else's business on a shared machine, so it is written owner-only under
// an owner-only config root. The chmod is not redundant with the write mode: a
// temp file left behind by a crashed save already exists, and os.WriteFile only
// applies its mode when it creates the file (websocket-origin-fix, ticket 05).
func (r *Registry) saveLocked() error {
	if err := os.MkdirAll(filepath.Dir(r.path), 0o700); err != nil {
		return fmt.Errorf("registry: creating config dir: %w", err)
	}
	r.densifyLocked()

	f := file{Spaces: make([]Entry, 0, len(r.entries)-1)}
	for _, e := range r.entries {
		if e.Scratch {
			// Scratch is written as its slot and nothing else. Densification ran over
			// the whole arrangement, so the registered rows below carry the orders it
			// gave them and the file keeps a gap exactly where this index sits.
			slot := e.Order
			f.ScratchOrder = &slot
			continue
		}
		f.Spaces = append(f.Spaces, e)
	}
	sort.Slice(f.Spaces, func(i, j int) bool { return f.Spaces[i].Path < f.Spaces[j].Path })

	data, err := toml.Marshal(f)
	if err != nil {
		return fmt.Errorf("registry: encoding: %w", err)
	}

	tmp := r.path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o600); err != nil {
		return fmt.Errorf("registry: writing %s: %w", tmp, err)
	}
	if err := os.Chmod(tmp, 0o600); err != nil {
		return fmt.Errorf("registry: securing %s: %w", tmp, err)
	}
	if err := os.Rename(tmp, r.path); err != nil {
		return fmt.Errorf("registry: replacing %s: %w", r.path, err)
	}
	return nil
}

// densifyLocked rewrites every entry's order to its index in the current
// sequence, so the stored order is 0..n-1 with no gap and no duplicate. It runs
// on every save because holes appear honestly — a deregistration leaves one, a
// degraded file loads with one, a registration appends past the end — and none
// of them should outlive the next write. The caller holds r.mu.
func (r *Registry) densifyLocked() {
	seq := make([]Entry, 0, len(r.entries))
	for _, e := range r.entries {
		seq = append(seq, e)
	}
	sortByOrder(seq)
	for i, e := range seq {
		e.Order = i
		r.entries[e.ID] = e
	}
}

// spaceID derives a stable identity from the absolute path, so a registry
// rebuilt after loss (or a re-register of the same path) re-derives the same ID
// and any local overrides keyed nearby still line up.
func spaceID(absPath string) string {
	sum := sha256.Sum256([]byte(absPath))
	return hex.EncodeToString(sum[:])[:12]
}

