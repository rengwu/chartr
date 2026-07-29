// Package registry is the space registry: the operator's list of registered
// spaces, held in the local user-config layer as a rebuildable index rather
// than a source of truth (ticket 02, ADR 0003, ADR 0009). Everything
// authoritative — maps, committed workspace config, git history — lives in the
// repositories; the registry holds only registered paths, the operator's sidebar
// order, and each space's local pin and recency. Losing it costs re-adding
// folders, never work (their arrangement included), so a
// deleted spaces.toml is not an error: the operator re-registers and each
// repo picks up exactly as it sits.
//
// Registering a folder that is not yet a git repository runs `git init`,
// announced and never silent (story 2). De-registering forgets the entry and
// touches nothing in the repository — forget, not destroy (story 4).
package registry

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"sync"
	"time"

	"github.com/BurntSushi/toml"
)

// Entry is one registered space: its path plus the local, per-machine order,
// pin and recency. The ID is derived from the path (so a rebuilt registry
// re-derives the same identity) and is not persisted.
type Entry struct {
	ID   string `toml:"-"`
	Path string `toml:"path"`
	// Order is the space's position in the sidebar: the operator's arrangement,
	// stored rather than derived, and the only thing List sorts by. It is dense
	// (0..n-1 across the whole registry) and rewritten wholesale on every save,
	// so a duplicate or missing index is unrepresentable after any successful
	// write. A file carrying no order at all — one written before the sidebar had
	// one — is frozen into today's sequence on load rather than reset, which is
	// what makes the upgrade invisible.
	Order int `toml:"order"`
	// Pinned is vestigial: it is still read and written, but nothing orders by it
	// any more. Dragging a space to the top is everything pinning did, so the key
	// goes away entirely rather than racing the stored order for the sidebar.
	Pinned     bool      `toml:"pinned"`
	LastActive time.Time `toml:"last_active"`
	// LastAgent is the registered agent this space last spawned with — state, not
	// config: chartr writes it after a successful launch and nothing edits it.
	// It sits here because the registry is already per-space, local, chartr-owned
	// and rebuildable, which is exactly this value's lifecycle. A name that no
	// longer resolves against the library is *not* rewritten away: it is reported
	// as it stands and read as nothing remembered, so deleting an agent costs no
	// registry surgery.
	LastAgent string `toml:"last_agent,omitempty"`
	// TrackerDismissed records that the operator waved off the offer to install
	// chartr's tracker adapter for this space, so the prompt is not shown again.
	// Like the rest of the entry it is local, per-machine, chartr-owned state:
	// installing the adapter never sets it (an installed adapter simply reads as
	// up-to-date), so only an explicit "leave it" lands here.
	TrackerDismissed bool `toml:"tracker_dismissed,omitempty"`
}

// Registry is the in-memory registry backed by <dataDir>/spaces.toml. It is
// safe for concurrent use: the HTTP action handlers mutate it from many
// goroutines, and every mutation persists the whole file atomically.
type Registry struct {
	path string // the spaces.toml file

	mu      sync.Mutex
	entries map[string]Entry // keyed by ID
}

// Load opens the registry under dataDir, reading spaces.toml if it exists. A
// missing file yields an empty registry — that is the first-run state, not an
// error, and the same state a lost registry recovers from.
func Load(dataDir string) (*Registry, error) {
	r := &Registry{
		path:    filepath.Join(dataDir, "spaces.toml"),
		entries: map[string]Entry{},
	}
	data, err := os.ReadFile(r.path)
	if os.IsNotExist(err) {
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
	// are read a second time as raw tables purely to learn which entries carried
	// the key at all. Presence is what tells a pre-upgrade file — freeze it —
	// apart from a hand-edited one, which degrades. The document has already
	// parsed once, so this cannot fail; both decodes see the rows in file order.
	var raw struct {
		Spaces []map[string]any `toml:"space"`
	}
	_ = toml.Unmarshal(data, &raw)
	carried := make([]bool, len(f.Spaces))
	for i := range f.Spaces {
		f.Spaces[i].ID = spaceID(f.Spaces[i].Path)
		if i < len(raw.Spaces) {
			_, carried[i] = raw.Spaces[i]["order"]
		}
	}
	for _, e := range ordered(f.Spaces, carried) {
		r.entries[e.ID] = e
	}
	return r, nil
}

type file struct {
	Spaces []Entry `toml:"space"`
}

// ordered resolves the sidebar sequence of a just-loaded file and densifies it,
// with carried[i] reporting whether entry i actually carried an `order` key.
//
// It is one rule doing two jobs. Entries with a unique order are well-formed and
// keep their sequence; entries with a duplicate or missing order are malformed,
// are sorted among themselves by the *old* rule — pinned first, then recency,
// then path — and appended after the well-formed ones. A pre-upgrade file, where
// no entry carries an order, is simply the case where every entry is malformed:
// the whole list sorts by the old rule and freezes into 0..n-1, so the sidebar
// after the upgrade is the sidebar before it. A hand-edit or a truncated file
// costs the operator their arrangement, never their list of spaces.
//
// The result is written back on the next save, not here: loading the registry
// stays a read.
func ordered(entries []Entry, carried []bool) []Entry {
	count := map[int]int{}
	for i, e := range entries {
		if carried[i] {
			count[e.Order]++
		}
	}

	wellFormed := make([]Entry, 0, len(entries))
	malformed := make([]Entry, 0, len(entries))
	for i, e := range entries {
		if carried[i] && count[e.Order] == 1 {
			wellFormed = append(wellFormed, e)
		} else {
			malformed = append(malformed, e)
		}
	}

	sortByOrder(wellFormed)
	sort.SliceStable(malformed, func(i, j int) bool {
		if malformed[i].Pinned != malformed[j].Pinned {
			return malformed[i].Pinned
		}
		if !malformed[i].LastActive.Equal(malformed[j].LastActive) {
			return malformed[i].LastActive.After(malformed[j].LastActive)
		}
		return malformed[i].Path < malformed[j].Path // stable tiebreak
	})

	out := append(wellFormed, malformed...)
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

// Register makes path a space. It cleans the path to an absolute form, requires
// it to be an existing directory, and — if it is not already a git repository —
// runs `git init` there, reporting that it did so (never silent). A new space
// appends: it takes max(order)+1, because the stored order belongs to the
// operator and registering a repo is not a rearrangement. Registering an
// already-registered path just refreshes its recency and leaves it where it sits.
// The bool result is whether a `git init` was run.
func (r *Registry) Register(path string) (Entry, bool, error) {
	abs, err := filepath.Abs(path)
	if err != nil {
		return Entry{}, false, fmt.Errorf("registry: resolving %q: %w", path, err)
	}
	abs = filepath.Clean(abs)

	info, err := os.Stat(abs)
	if err != nil {
		return Entry{}, false, fmt.Errorf("registry: %q is not a folder I can register: %w", path, err)
	}
	if !info.IsDir() {
		return Entry{}, false, fmt.Errorf("registry: %q is a file, not a folder", path)
	}

	gitInited := false
	if !isGitRepo(abs) {
		if err := gitInit(abs); err != nil {
			return Entry{}, false, err
		}
		gitInited = true
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
		return Entry{}, false, err
	}
	return e, gitInited, nil
}

// Deregister forgets a space: it removes the registry entry and its local pin
// and recency, and touches nothing in the repository. Re-register any time and
// the repo picks up exactly as it sits. Forgetting an unknown ID is a no-op.
func (r *Registry) Deregister(id string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
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

// SetPin sets whether a space is pinned. It no longer orders anything — the
// sidebar sorts by the stored order alone — and is kept only until the pin
// surface is removed. Pinning an unknown ID is a no-op.
func (r *Registry) SetPin(id string, pinned bool) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	e, ok := r.entries[id]
	if !ok {
		return nil
	}
	e.Pinned = pinned
	r.entries[id] = e
	return r.saveLocked()
}

// SetTrackerDismissed records whether the operator waved off the tracker-adapter
// offer for a space, so it is not shown again. Setting it on an unknown ID, or to
// the value already held, is a no-op.
func (r *Registry) SetTrackerDismissed(id string, dismissed bool) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	e, ok := r.entries[id]
	if !ok || e.TrackerDismissed == dismissed {
		return nil
	}
	e.TrackerDismissed = dismissed
	r.entries[id] = e
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

// List returns the entries ordered as the sidebar shows them: the operator's
// stored order, and nothing else. Pin and recency are still recorded but sort
// nothing, and an actionable signal may flag a row but never re-sorts it — so
// only an explicit reorder, or a registration appending at the end, moves a row.
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
func (r *Registry) saveLocked() error {
	if err := os.MkdirAll(filepath.Dir(r.path), 0o755); err != nil {
		return fmt.Errorf("registry: creating config dir: %w", err)
	}
	r.densifyLocked()

	f := file{Spaces: make([]Entry, 0, len(r.entries))}
	for _, e := range r.entries {
		f.Spaces = append(f.Spaces, e)
	}
	sort.Slice(f.Spaces, func(i, j int) bool { return f.Spaces[i].Path < f.Spaces[j].Path })

	data, err := toml.Marshal(f)
	if err != nil {
		return fmt.Errorf("registry: encoding: %w", err)
	}

	tmp := r.path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o644); err != nil {
		return fmt.Errorf("registry: writing %s: %w", tmp, err)
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

func isGitRepo(dir string) bool {
	// A .git entry (directory for a normal clone, a file for a linked worktree)
	// marks the repository root; chartr registers repository roots.
	_, err := os.Stat(filepath.Join(dir, ".git"))
	return err == nil
}

func gitInit(dir string) error {
	cmd := exec.Command("git", "init")
	cmd.Dir = dir
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("registry: git init in %s failed: %w\n%s", dir, err, out)
	}
	return nil
}
