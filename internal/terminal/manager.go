package terminal

import (
	"errors"
	"fmt"
	"sync"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/model"
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

	stop     chan struct{}
	stopOnce sync.Once
}

// sampleInterval is how often the manager re-reads each live shell's foreground
// process and activity. Fast enough that a shell going busy or idle refreshes
// the sidebar promptly, slow enough that the poll — one ioctl per shell, and a
// `ps` only when a new command takes the foreground — costs nothing noticeable.
const sampleInterval = 900 * time.Millisecond

// NewManager builds an empty Manager. onChange is called after a terminal is
// opened or ends, and whenever a live shell's activity changes; a nil onChange
// is tolerated (the manager is usable without a push, as in a focused unit
// test) and, with no one to notify, the background sampler is not started.
func NewManager(onChange func()) *Manager {
	m := &Manager{terms: make(map[string]*Terminal), onChange: onChange}
	if onChange != nil {
		m.stop = make(chan struct{})
		go m.sampleLoop()
	}
	return m
}

// sampleLoop re-samples every live shell on a fixed cadence and pushes a fresh
// model only when something changed, so a shell going busy or idle drives the
// sidebar with no filesystem or socket event behind it. It runs until Shutdown.
func (m *Manager) sampleLoop() {
	tick := time.NewTicker(sampleInterval)
	defer tick.Stop()
	for {
		select {
		case <-m.stop:
			return
		case <-tick.C:
			m.sampleOnce()
		}
	}
}

// sampleOnce samples every current terminal off the manager lock (sampling may
// exec to resolve a process name, which must not block Open/Close/ForSpace) and
// pushes once if any shell's activity changed.
func (m *Manager) sampleOnce() {
	m.mu.Lock()
	terms := make([]*Terminal, 0, len(m.terms))
	for _, id := range m.order {
		if t := m.terms[id]; t != nil {
			terms = append(terms, t)
		}
	}
	m.mu.Unlock()

	changed := false
	for _, t := range terms {
		if t.sample() {
			changed = true
		}
	}
	if changed {
		m.notify()
	}
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

// Info is one terminal's public shape on the pushed model: enough for a session
// row — its tab title, the process in its foreground, its activity, and its
// liveness — never the PTY itself.
type Info struct {
	ID     string
	Title  string
	Proc   string
	Status string
	Alive  bool
}

// ForSpace returns the space's terminals in creation order, so sessions seat top
// to bottom in the order the operator opened them, each carrying its last
// sampled foreground process and activity.
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
		alive, proc, state := t.alive, t.proc, t.state
		t.mu.Unlock()
		if proc == "" {
			proc = t.Title
		}
		if state == "" {
			state = model.TerminalIdle
		}
		out = append(out, Info{ID: t.ID, Title: t.Title, Proc: proc, Status: state, Alive: alive})
	}
	return out
}

// Shutdown stops the sampler and ends every terminal — used when the server
// drains on exit.
func (m *Manager) Shutdown() {
	if m.stop != nil {
		m.stopOnce.Do(func() { close(m.stop) })
	}
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
