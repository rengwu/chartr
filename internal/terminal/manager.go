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

// ErrSessionExists is returned when a space already has a live session and one
// more is asked for. One session per space at a time is the invariant (spec,
// State model): parallelism is many spaces, never many sessions in one working
// tree. Ad-hoc shells do not count — a space may carry any number of those
// alongside its one session.
var ErrSessionExists = errors.New("terminal: the space already has a live session")

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

	// quietAfter is how long a session's PTY may stay silent before the sampler
	// marks it quiet (ticket 10). Held here so the whole manager samples on one
	// threshold; the server sets it (short in tests, a calm default in production).
	quietAfter time.Duration

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
// quietAfter is the session silence threshold; a non-positive value falls back to
// defaultQuietAfter so a zero value is never mistaken for "quiet immediately".
func NewManager(onChange func(), quietAfter time.Duration) *Manager {
	if quietAfter <= 0 {
		quietAfter = defaultQuietAfter
	}
	m := &Manager{terms: make(map[string]*Terminal), onChange: onChange, quietAfter: quietAfter}
	if onChange != nil {
		m.stop = make(chan struct{})
		go m.sampleLoop()
	}
	return m
}

// defaultQuietAfter is the silence a session may keep before the sampler marks it
// quiet when the server names no other threshold. Long enough that an agent
// thinking, compiling, or waiting on a slow tool is not dressed as stuck; the hint
// is a nudge, never an alarm, and never enacted (ticket 10).
const defaultQuietAfter = 45 * time.Second

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
		if t.sample(m.quietAfter) {
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

	t.start(func() { m.onExit(id) })

	m.notify()
	return t, nil
}

// OpenSession launches an agent in a PTY in cwd, bound to exactly one ticket, and
// seats it as a tab under spaceID. It is the session sibling of Open: same PTY and
// tab plumbing, but the tab carries a Session, runs the adapter's command instead
// of a shell, and has the one-line opener typed into it once it is up — the
// intervention channel is a live TUI, so the opener arrives as the session's first
// keystrokes, not an argv flag (spec, Sessions and adapters). id is chosen by the
// caller (the same id the claim commit and payload archive are keyed by), so the
// whole spawn refers to one session everywhere.
//
// It refuses with ErrSessionExists if the space already has a live session — one
// session per space at a time. A launch failure leaves nothing seated; the caller,
// having already written the claim, surfaces it (the stale claim stands until the
// human acts, ADR 0008).
func (m *Manager) OpenSession(spaceID, cwd, id, name string, args []string, opener string, s Session) (*Terminal, error) {
	m.mu.Lock()
	for _, tid := range m.order {
		if t := m.terms[tid]; t != nil && t.SpaceID == spaceID && t.isLiveSession() {
			m.mu.Unlock()
			return nil, ErrSessionExists
		}
	}
	m.mu.Unlock()

	sess := s
	t, err := newProc(id, spaceID, cwd, launchSpec{
		name:    name,
		args:    args,
		title:   sessionTitle(s),
		session: &sess,
	})
	if err != nil {
		return nil, err
	}

	// Record the tab before the read loop starts, so an agent that exits instantly
	// cannot remove itself before it has been listed (as Open does).
	m.mu.Lock()
	m.terms[id] = t
	m.order = append(m.order, id)
	m.mu.Unlock()

	t.start(func() { m.onExit(id) })

	// Type the opener into the live TUI. The PTY's input buffer holds it until the
	// agent reads, so no readiness handshake is needed; a write error only means the
	// agent already exited, which the read loop is reaping in parallel.
	if opener != "" {
		_, _ = t.Write([]byte(opener))
	}

	m.notify()
	return t, nil
}

// OpenIdeate launches an agent in a PTY in cwd with a starter prompt typed in,
// seated as a plain tab under spaceID — the ideate on-ramp (ticket 15). It shares
// OpenSession's launch and opener mechanics but carries no Session: the ideate
// on-ramp is deliberately not a session (spec, State model — "ticketless, live,
// sharing only the adapter's spawn primitive"), so this tab reads
// exactly like an ad-hoc shell — idle/working/exited, never the session grammar's
// quiet hint — and never counts toward the one-live-session-per-space limit
// OpenSession enforces. id is chosen by the caller, matching OpenSession's style,
// so the tab and the gitignored prompt file it points at share one identity.
func (m *Manager) OpenIdeate(spaceID, cwd, id, name string, args []string, opener string) (*Terminal, error) {
	t, err := newProc(id, spaceID, cwd, launchSpec{name: name, args: args, title: "ideate"})
	if err != nil {
		return nil, err
	}

	// Record the tab before the read loop starts, so an agent that exits instantly
	// cannot remove itself before it has been listed (as Open and OpenSession do).
	m.mu.Lock()
	m.terms[id] = t
	m.order = append(m.order, id)
	m.mu.Unlock()

	t.start(func() { m.onExit(id) })

	if opener != "" {
		_, _ = t.Write([]byte(opener))
	}

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

// Close ends the terminal on the human's command. A live terminal is killed and
// drops from the listing once its process finishes exiting (the same cleanup path
// a natural exit takes). A pinned dead session — one that already died and is
// waiting for the operator — has no process left to exit, so Close drops it right
// away; this is how the operator dismisses a halted session without resuming,
// respawning, or releasing it.
func (m *Manager) Close(id string) error {
	t, ok := m.Get(id)
	if !ok {
		return ErrNoTerminal
	}
	if !t.close() {
		m.remove(id)
	}
	return nil
}

// Discard drops a tab from the listing without touching a process, then pushes a
// fresh model — the seam the death-halt actions (resume/respawn/release) use to
// clear the pinned dead session they are replacing. Dropping an id that names no
// terminal is a no-op.
func (m *Manager) Discard(id string) { m.remove(id) }

// Lookup returns the public Info for one terminal by id — enough for the
// death-halt handlers to read a tab's session binding and liveness without
// reaching into the PTY. false when the id names no terminal.
func (m *Manager) Lookup(id string) (Info, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	t := m.terms[id]
	if t == nil {
		return Info{}, false
	}
	return t.info(), true
}

// Info is one terminal's public shape on the pushed model: enough for a session
// row — its tab title, the process in its foreground, its activity, and its
// liveness — never the PTY itself. Session is set when the tab is a session (a
// PTY bound to a ticket), nil for an ad-hoc shell. Silent is the sampler's raw
// silence verdict for a live session (quiet past the threshold); the server, which
// alone knows the role's AFK-ness, turns it into the tab's final quiet reading.
type Info struct {
	ID      string
	SpaceID string
	Title   string
	Proc    string
	Status  string
	Alive   bool
	Silent  bool
	Session *Session
}

// info snapshots one terminal's public shape under its own lock.
func (t *Terminal) info() Info {
	t.mu.Lock()
	alive, proc, state, silent := t.alive, t.proc, t.state, t.silent
	t.mu.Unlock()
	if proc == "" {
		proc = t.Title
	}
	if state == "" {
		state = model.TerminalIdle
	}
	return Info{ID: t.ID, SpaceID: t.SpaceID, Title: t.Title, Proc: proc, Status: state, Alive: alive, Silent: silent, Session: t.session}
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
		out = append(out, t.info())
	}
	return out
}

// isLiveSession reports whether this tab is a session (bound to a ticket) whose
// process is still alive — the thing the one-session-per-space guard counts.
func (t *Terminal) isLiveSession() bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.session != nil && t.alive
}

// sessionTitle labels a session tab by the ticket it is bound to and the role it
// runs as — the tab's identity is its ticket, not the agent binary in it (the
// foreground process name is sampled separately into Proc).
func sessionTitle(s Session) string {
	return fmt.Sprintf("%s #%02d", s.Role, s.TicketNum)
}

// HasLiveSession reports whether spaceID already has a live session — the
// one-session-per-space precondition the spawn path checks before it writes a
// claim, so it never claims a ticket it cannot then seat.
func (m *Manager) HasLiveSession(spaceID string) bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	for _, id := range m.order {
		if t := m.terms[id]; t != nil && t.SpaceID == spaceID && t.isLiveSession() {
			return true
		}
	}
	return false
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

// onExit runs when a terminal's process exits. An ad-hoc shell, or a session the
// operator killed, drops from the listing so its tab disappears on its own. A
// session that died on its own stays put — pinned to its ticket with its
// scrollback intact — for the operator to resume, respawn, or release (ticket 10);
// it is already marked dead by the read loop's cleanup, so the push just re-renders
// it frozen. Either way a fresh model is pushed.
func (m *Manager) onExit(id string) {
	m.mu.Lock()
	t := m.terms[id]
	pin := t != nil && t.pinOnDeath()
	if !pin {
		m.drop(id)
	}
	m.mu.Unlock()
	m.notify()
}

// remove drops a terminal from the listing, then pushes a fresh model so the tab
// disappears on its own.
func (m *Manager) remove(id string) {
	m.mu.Lock()
	m.drop(id)
	m.mu.Unlock()
	m.notify()
}

// drop deletes a terminal from the map and order. The caller holds m.mu.
func (m *Manager) drop(id string) {
	delete(m.terms, id)
	for i, x := range m.order {
		if x == id {
			m.order = append(m.order[:i], m.order[i+1:]...)
			break
		}
	}
}

func (m *Manager) notify() {
	if m.onChange != nil {
		m.onChange()
	}
}
