// Package prompts is the operator's prompt catalog: one ordered, global list of
// short behavioural instructions chartr owns and hands to the agents it
// launches. A preset is a stable id, a human name, and non-empty prompt text —
// nothing else. There is no grouping, no frontmatter, no per-provider format:
// these are chartr's own, and the point of owning them is that they work the
// same way across every harness chartr launches.
//
// The catalog lives in `prompts.toml` under the config root, beside the space
// registry and the agent library, as an array of tables with no order field:
// file order is creation order and the only order there is. Selections are held
// per space in the registry and composed in this order.
//
// A missing file is an empty catalog — the first-run state, not an error. A file
// this package cannot fully read yields *no* presets and one warning naming it,
// and every mutation is refused while it stands: half-executing a catalog would
// send an agent instructions the operator cannot see, and rewriting one would
// throw away bytes they wrote by hand. Fixing the file is the operator's, and
// chartr touches nothing until they do.
//
// Written atomically (0600 under a 0700 root, temp-then-rename) like the space
// registry and the source list, and safe for concurrent use.
package prompts

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"

	"github.com/BurntSushi/toml"
)

// fileName is the catalog under the operator's config root.
const fileName = "prompts.toml"

// Errors the HTTP surface above this package distinguishes.
var (
	ErrNotFound = errors.New("prompts: no preset with that id")
	ErrInvalid  = errors.New("prompts: a preset needs a name and some prompt text")
	// ErrMalformed refuses every mutation against a catalog this package could
	// not read, so an operator's hand-edit is never silently discarded by the
	// next click in the pane.
	ErrMalformed = errors.New("prompts: the catalog could not be read, so it will not be rewritten")
)

// Prompt is one preset: chartr's stable id for the row, the operator's name for
// it, and the text an agent is actually told.
type Prompt struct {
	ID   string `toml:"id" json:"id"`
	Name string `toml:"name" json:"name"`
	Body string `toml:"body" json:"body"`
}

// file is the document: the operator's presets and nothing beside them.
type file struct {
	Prompts []Prompt `toml:"prompt"`
}

// Catalog is the in-memory catalog backed by <configDir>/prompts.toml. Every
// mutation persists the whole file atomically.
type Catalog struct {
	path string

	mu   sync.Mutex
	rows []Prompt
	// warning is what made the file unreadable, empty when it read cleanly. It is
	// both what the model surfaces and what refuses a mutation.
	warning string
}

// Load reads the catalog under configDir. It never fails: a missing file is an
// empty catalog, and anything unreadable is a warning with no presets behind it.
func Load(configDir string) *Catalog {
	c := &Catalog{path: filepath.Join(configDir, fileName)}

	data, err := os.ReadFile(c.path)
	if os.IsNotExist(err) {
		return c
	}
	if err != nil {
		c.warning = fmt.Sprintf("%s could not be read (%v); no presets are available until it can be", c.path, err)
		return c
	}
	var f file
	if err := toml.Unmarshal(data, &f); err != nil {
		c.warning = fmt.Sprintf("%s could not be parsed (%v); no presets are available until it can be", c.path, err)
		return c
	}
	seen := map[string]bool{}
	for i, p := range f.Prompts {
		p.ID, p.Name = strings.TrimSpace(p.ID), strings.TrimSpace(p.Name)
		switch {
		case p.ID == "":
			c.warning = fmt.Sprintf("%s: prompt %d has no id; no presets are available until every prompt has an id, a name and a body", c.path, i+1)
		case seen[p.ID]:
			c.warning = fmt.Sprintf("%s: two prompts are called %q; no presets are available until each id appears once", c.path, p.ID)
		case p.Name == "" || strings.TrimSpace(p.Body) == "":
			c.warning = fmt.Sprintf("%s: the prompt %q needs both a name and a body; no presets are available until it has them", c.path, p.ID)
		}
		if c.warning != "" {
			c.rows = nil
			return c
		}
		seen[p.ID] = true
		c.rows = append(c.rows, p)
	}
	return c
}

// Warnings is what the load could not read, ready to surface beside the other
// config warnings. Advisory in the sense that chartr still runs; a catalog with
// a warning simply has no presets in it.
func (c *Catalog) Warnings() []string {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.warning == "" {
		return nil
	}
	return []string{c.warning}
}

// List returns the whole catalog in creation order.
func (c *Catalog) List() []Prompt {
	c.mu.Lock()
	defer c.mu.Unlock()
	return append([]Prompt(nil), c.rows...)
}

// Get returns one preset by id.
func (c *Catalog) Get(id string) (Prompt, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	for _, p := range c.rows {
		if p.ID == id {
			return p, true
		}
	}
	return Prompt{}, false
}

// Selected resolves a space's selection: the named presets in catalog order,
// plus the ids the catalog no longer holds. A missing id is reported and never
// substituted — receiving a preset the operator did not choose is worse than
// receiving one fewer — and a repeated id is composed once.
func (c *Catalog) Selected(ids []string) (chosen []Prompt, missing []string) {
	c.mu.Lock()
	defer c.mu.Unlock()

	want := make(map[string]bool, len(ids))
	for _, id := range ids {
		want[id] = true
	}
	held := make(map[string]bool, len(c.rows))
	for _, p := range c.rows {
		held[p.ID] = true
		if want[p.ID] {
			chosen = append(chosen, p)
		}
	}
	for _, id := range ids {
		if !held[id] {
			missing = append(missing, id)
		}
	}
	return chosen, missing
}

// Create appends a preset and persists the catalog. The id is derived from the
// name once, here, and never changes again: an edit keeps it, so every space
// that selected the preset keeps pointing at the same row.
func (c *Catalog) Create(name, body string) (Prompt, error) {
	name, body = strings.TrimSpace(name), strings.TrimSpace(body)
	if name == "" || body == "" {
		return Prompt{}, fmt.Errorf("%w (got name %q)", ErrInvalid, name)
	}

	c.mu.Lock()
	defer c.mu.Unlock()
	if err := c.readableLocked(); err != nil {
		return Prompt{}, err
	}
	p := Prompt{ID: c.freeIDLocked(name), Name: name, Body: body}
	c.rows = append(c.rows, p)
	if err := c.saveLocked(); err != nil {
		c.rows = c.rows[:len(c.rows)-1]
		return Prompt{}, err
	}
	return p, nil
}

// Update rewrites one preset's name and text in place. The id and the row's
// position are untouched, which is what makes an edit change what future
// launches receive everywhere the preset is selected, and nothing else.
func (c *Catalog) Update(id, name, body string) (Prompt, error) {
	name, body = strings.TrimSpace(name), strings.TrimSpace(body)
	if name == "" || body == "" {
		return Prompt{}, fmt.Errorf("%w (got name %q)", ErrInvalid, name)
	}

	c.mu.Lock()
	defer c.mu.Unlock()
	if err := c.readableLocked(); err != nil {
		return Prompt{}, err
	}
	for i, p := range c.rows {
		if p.ID != id {
			continue
		}
		c.rows[i] = Prompt{ID: id, Name: name, Body: body}
		if err := c.saveLocked(); err != nil {
			c.rows[i] = p
			return Prompt{}, err
		}
		return c.rows[i], nil
	}
	return Prompt{}, fmt.Errorf("%w: %q", ErrNotFound, id)
}

// Delete drops a preset from the catalog. Cleaning the deleted id out of the
// spaces that selected it belongs to the caller: the registry is where a
// selection lives, and this package knows nothing about spaces.
func (c *Catalog) Delete(id string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if err := c.readableLocked(); err != nil {
		return err
	}
	for i, p := range c.rows {
		if p.ID != id {
			continue
		}
		c.rows = append(c.rows[:i:i], c.rows[i+1:]...)
		if err := c.saveLocked(); err != nil {
			c.rows = append(c.rows[:i:i], append([]Prompt{p}, c.rows[i:]...)...)
			return err
		}
		return nil
	}
	return fmt.Errorf("%w: %q", ErrNotFound, id)
}

// readableLocked refuses a mutation while the file on disk is one this package
// could not read. The caller holds c.mu.
func (c *Catalog) readableLocked() error {
	if c.warning == "" {
		return nil
	}
	return fmt.Errorf("%w: %s", ErrMalformed, c.warning)
}

// freeIDLocked derives an unused id from a name: lower-kebab-case, with a
// numeric suffix when the obvious id is taken, so two presets named the same
// thing are two presets. The caller holds c.mu.
func (c *Catalog) freeIDLocked(name string) string {
	base := kebab(name)
	if base == "" {
		base = "prompt"
	}
	taken := make(map[string]bool, len(c.rows))
	for _, p := range c.rows {
		taken[p.ID] = true
	}
	id := base
	for n := 2; taken[id]; n++ {
		id = fmt.Sprintf("%s-%d", base, n)
	}
	return id
}

// kebab lowercases a name and joins its alphanumeric runs with hyphens. It is
// deliberately lossy: the id is an identity, not a translation of the name, and
// a name that survives none of this simply falls back to a generic base.
func kebab(name string) string {
	var b strings.Builder
	dash := false
	for _, r := range strings.ToLower(name) {
		switch {
		case r >= 'a' && r <= 'z', r >= '0' && r <= '9':
			if dash && b.Len() > 0 {
				b.WriteByte('-')
			}
			dash = false
			b.WriteRune(r)
		default:
			dash = true
		}
	}
	return b.String()
}

// saveLocked writes the whole catalog atomically (temp file + rename) so a crash
// mid-write cannot corrupt it. The file is the operator's own standing
// instructions to their agents, so it is owner-only under an owner-only root;
// the chmod is not redundant with the write mode, because a temp file left by a
// crashed save already exists and os.WriteFile only applies its mode on create.
// The caller holds c.mu.
func (c *Catalog) saveLocked() error {
	if err := os.MkdirAll(filepath.Dir(c.path), 0o700); err != nil {
		return fmt.Errorf("prompts: creating config dir: %w", err)
	}
	data, err := toml.Marshal(file{Prompts: c.rows})
	if err != nil {
		return fmt.Errorf("prompts: encoding: %w", err)
	}
	tmp := c.path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o600); err != nil {
		return fmt.Errorf("prompts: writing %s: %w", tmp, err)
	}
	if err := os.Chmod(tmp, 0o600); err != nil {
		return fmt.Errorf("prompts: securing %s: %w", tmp, err)
	}
	if err := os.Rename(tmp, c.path); err != nil {
		return fmt.Errorf("prompts: replacing %s: %w", c.path, err)
	}
	return nil
}
