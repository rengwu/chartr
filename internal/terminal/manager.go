package terminal

import (
	"errors"
	"fmt"
	"sync"
)

// ErrNoTerminal is returned when an id names no live terminal — it never
// existed, or it has already ended.
var ErrNoTerminal = errors.New("terminal: no such terminal")

// Manager owns every ad-hoc terminal in the harness process, keyed by id and
// grouped by space. It is the one seam the server reaches: opening a shell,
// finding one to attach a socket to, ending one, and listing a space's terminals
// for the pushed model. onChange is the server's rebuild-and-push, called
// whenever the set of terminals changes so a tab appears or disappears on its
// own.
type Manager struct {
	mu       sync.Mutex
	terms    map[string]*Terminal
	order    []string
	seq      int
	onChange func()
}

// NewManager builds an empty Manager. onChange is called after a terminal is
// opened or ends; a nil onChange is tolerated (the manager is usable without a
// push, as in a focused unit test).
func NewManager(onChange func()) *Manager {
	return &Manager{terms: make(map[string]*Terminal), onChange: onChange}
}

// Open spawns a shell in cwd, tags it to spaceID, and returns the live terminal.
// The new terminal is immediately listed for the space and the model is pushed,
// so the tab appears without a refresh.
func (m *Manager) Open(spaceID, cwd string) (*Terminal, error) {
	m.mu.Lock()
	m.seq++
	id := fmt.Sprintf("t%d", m.seq)
	m.mu.Unlock()

	t, err := newTerminal(id, spaceID, cwd)
	if err != nil {
		return nil, err
	}

	// Record the terminal before its read loop starts, so a shell that exits
	// instantly cannot remove itself before it has been listed.
	m.mu.Lock()
	m.terms[id] = t
	m.order = append(m.order, id)
	m.mu.Unlock()

	t.start(func() { m.remove(id) })

	m.notify()
	return t, nil
}

// Get returns the live terminal with id, or false if none.
func (m *Manager) Get(id string) (*Terminal, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	t, ok := m.terms[id]
	return t, ok
}

// Close ends the terminal on the human's command. The terminal drops from the
// listing and the model is pushed once its process finishes exiting (through the
// same cleanup path a natural exit takes).
func (m *Manager) Close(id string) error {
	t, ok := m.Get(id)
	if !ok {
		return ErrNoTerminal
	}
	t.close()
	return nil
}

// Info is one terminal's public shape on the pushed model: enough for a tab and
// its liveness, never the PTY itself.
type Info struct {
	ID    string
	Title string
	Alive bool
}

// ForSpace returns the space's terminals in creation order, so tabs seat left to
// right in the order the operator opened them.
func (m *Manager) ForSpace(spaceID string) []Info {
	m.mu.Lock()
	defer m.mu.Unlock()
	out := make([]Info, 0)
	for _, id := range m.order {
		t := m.terms[id]
		if t == nil || t.SpaceID != spaceID {
			continue
		}
		t.mu.Lock()
		alive := t.alive
		t.mu.Unlock()
		out = append(out, Info{ID: t.ID, Title: t.Title, Alive: alive})
	}
	return out
}

// Shutdown ends every terminal — used when the server drains on exit.
func (m *Manager) Shutdown() {
	m.mu.Lock()
	terms := make([]*Terminal, 0, len(m.terms))
	for _, t := range m.terms {
		terms = append(terms, t)
	}
	m.mu.Unlock()
	for _, t := range terms {
		t.close()
	}
}

// remove drops a terminal from the listing after its process has exited, then
// pushes a fresh model so the tab disappears on its own.
func (m *Manager) remove(id string) {
	m.mu.Lock()
	delete(m.terms, id)
	for i, x := range m.order {
		if x == id {
			m.order = append(m.order[:i], m.order[i+1:]...)
			break
		}
	}
	m.mu.Unlock()
	m.notify()
}

func (m *Manager) notify() {
	if m.onChange != nil {
		m.onChange()
	}
}
