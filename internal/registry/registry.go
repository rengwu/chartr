// Package registry is the space registry: the operator's list of registered
// spaces, held in the local user-config layer as a rebuildable index rather
// than a source of truth (ticket 02, ADR 0003, ADR 0009). Everything
// authoritative — maps, committed workspace config, git history — lives in the
// repositories; the registry holds only registered paths and each space's local
// pin and recency. Losing it costs re-adding folders, never work, so a
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

// Entry is one registered space: its path plus the local, per-machine pin and
// recency that order the sidebar. The ID is derived from the path (so a rebuilt
// registry re-derives the same identity) and is not persisted.
type Entry struct {
	ID         string    `toml:"-"`
	Path       string    `toml:"path"`
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
	for _, e := range f.Spaces {
		e.ID = spaceID(e.Path)
		r.entries[e.ID] = e
	}
	return r, nil
}

type file struct {
	Spaces []Entry `toml:"space"`
}

// Register makes path a space. It cleans the path to an absolute form, requires
// it to be an existing directory, and — if it is not already a git repository —
// runs `git init` there, reporting that it did so (never silent). Registering
// an already-registered path just refreshes its recency. The bool result is
// whether a `git init` was run.
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
		e = Entry{ID: id, Path: abs}
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

// SetPin sets whether a space is pinned (pinned spaces sort first). Pinning an
// unknown ID is a no-op.
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

// List returns the entries ordered as the sidebar shows them: pinned first,
// then the rest by recency (most-recently-active on top). An actionable signal
// may flag a row but never re-sorts it, so this order is stable against
// everything but pin and activity.
func (r *Registry) List() []Entry {
	r.mu.Lock()
	out := make([]Entry, 0, len(r.entries))
	for _, e := range r.entries {
		out = append(out, e)
	}
	r.mu.Unlock()

	sort.SliceStable(out, func(i, j int) bool {
		if out[i].Pinned != out[j].Pinned {
			return out[i].Pinned
		}
		if !out[i].LastActive.Equal(out[j].LastActive) {
			return out[i].LastActive.After(out[j].LastActive)
		}
		return out[i].Path < out[j].Path // stable tiebreak
	})
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
// crash mid-write cannot corrupt it. The caller holds r.mu.
func (r *Registry) saveLocked() error {
	if err := os.MkdirAll(filepath.Dir(r.path), 0o755); err != nil {
		return fmt.Errorf("registry: creating config dir: %w", err)
	}

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
