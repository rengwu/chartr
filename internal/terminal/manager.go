package terminal

import (
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/terminal/detect"
)

// ErrNoTerminal is returned when an id names no live terminal — it never
// existed, or it has already ended.
var ErrNoTerminal = errors.New("terminal: no such terminal")

// ErrSessionExists is returned when a space already has a live session and one
// more is asked for without the operator's override. One session per space is the
// default (spec, State model): parallelism is meant to be many spaces, never many
// sessions in one working tree. Ad-hoc shells do not count — a space may carry any
// number of those alongside its one session.
//
// It is a default rather than a hard rule (ADR 0003 as amended): OpenSession's
// force argument seats a second session anyway, because only the operator can know
// whether two agents will actually collide in the tree. Everything downstream of
// the gate is unchanged — a forced session is an ordinary session.
var ErrSessionExists = errors.New("terminal: the space already has a live session")

// Manager owns every ad-hoc terminal in the chartr process, keyed by id and
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
	// onFinished is the one event seam tickets 03 and 04 consume. This ticket
	// seats and drives the clock; a nil callback deliberately consumes nothing.
	onFinished func(RunFinished)

	notifyEnabled bool
	notifyAfter   time.Duration
	notifySettle  time.Duration

	stop     chan struct{}
	stopOnce sync.Once
}

// The sampler's two cadences. A tab with a known agent in its foreground is the
// demanding one: it wants roughly three samples inside the second so the
// publishing hysteresis can confirm an idle without the indicator feeling
// sluggish. A tab without one is answering the same question it always did — is a
// command in the foreground — and re-reading that three times a second buys
// nothing while costing every open tab a wakeup, so it keeps the old cadence.
//
// The loop ticks at the fast rate and samples the slow tabs every shellSampleEvery
// ticks. The cost of the split is identification latency: a shell that has just
// launched an agent is noticed within one slow tick, and sampled fast from then on.
const (
	sampleInterval   = 300 * time.Millisecond
	shellSampleEvery = 3 // 300ms × 3 ≈ the 900ms cadence shells have always had
)

// agentEngine is the parsed set of shipped agent manifests, built once for the
// whole process. The manifests are embedded and static, so this cannot fail at
// runtime.
var agentEngine = detect.Builtin()

// NewManager builds an empty Manager. onChange is called after a terminal is
// opened or ends, and whenever a live tab's activity changes. onFinished receives
// each qualifying run once; nil is valid until a consumer is wired. A nil
// onChange is tolerated (the manager is usable without a push, as in a focused
// unit test) and, with no one to notify, the background sampler is not started.
func NewManager(onChange func(), onFinished func(RunFinished)) *Manager {
	m := &Manager{
		terms:         make(map[string]*Terminal),
		onChange:      onChange,
		onFinished:    onFinished,
		notifyEnabled: true,
		notifyAfter:   DefaultNotifyAfter,
		notifySettle:  DefaultNotifySettle,
	}
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
	for n := uint64(0); ; n++ {
		select {
		case <-m.stop:
			return
		case <-tick.C:
			m.sampleOnce(n%shellSampleEvery == 0)
		}
	}
}

// sampleOnce samples the current terminals off the manager lock (sampling may
// exec to resolve a process name, which must not block Open/Close/ForSpace) and
// pushes once if any tab's activity changed. slowTick says whether this is one of
// the ticks on which tabs with no identified agent are sampled too; agent-bearing
// tabs are sampled on every tick.
func (m *Manager) sampleOnce(slowTick bool) {
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
		if !slowTick && !t.hasAgent() {
			continue
		}
		if t.sample(agentEngine) {
			changed = true
		}
		if ev := t.updateRunClock(time.Now()); ev != nil && m.onFinished != nil {
			m.onFinished(*ev)
		}
	}
	if changed {
		m.notify()
	}
}

// ConfigureNotifications applies the resolved per-machine notification prefs.
// It is safe to call on every model rebuild: unchanged values preserve every
// in-progress clock, while an actual edit updates the constants in place. Turning
// the feature off removes each clock; turning it back on starts a fresh clock.
func (m *Manager) ConfigureNotifications(enabled bool, after, settle time.Duration) {
	if after <= 0 {
		after = DefaultNotifyAfter
	}
	if settle <= 0 {
		settle = DefaultNotifySettle
	}

	m.mu.Lock()
	defer m.mu.Unlock()
	if m.notifyEnabled == enabled && m.notifyAfter == after && m.notifySettle == settle {
		return
	}
	m.notifyEnabled, m.notifyAfter, m.notifySettle = enabled, after, settle
	for _, t := range m.terms {
		t.configureRunClock(enabled, after, settle)
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
	t.configureRunClock(m.notifyEnabled, m.notifyAfter, m.notifySettle)
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
// It refuses with ErrSessionExists if the space already has a live session, unless
// force says the operator confirmed the concurrency at the gate (ADR 0003 as
// amended). The check runs under the lock so it is the one that closes the race:
// the server's earlier HasLiveSession is only there to refuse before a claim is
// written, and two spawns that pass it together still serialise here. A launch
// failure leaves nothing seated; the caller, having already written the claim,
// surfaces it (the stale claim stands until the human acts, ADR 0008).
func (m *Manager) OpenSession(spaceID, cwd, id, name string, args, env []string, opener string, s Session, force bool) (*Terminal, error) {
	if !force {
		m.mu.Lock()
		for _, tid := range m.order {
			if t := m.terms[tid]; t != nil && t.SpaceID == spaceID && t.isLiveSession() {
				m.mu.Unlock()
				return nil, ErrSessionExists
			}
		}
		m.mu.Unlock()
	}

	sess := s
	t, err := newProc(id, spaceID, cwd, launchSpec{
		name:    name,
		args:    args,
		env:     env,
		title:   sessionTitle(s),
		session: &sess,
	})
	if err != nil {
		return nil, err
	}

	// Record the tab before the read loop starts, so an agent that exits instantly
	// cannot remove itself before it has been listed (as Open does).
	m.mu.Lock()
	t.configureRunClock(m.notifyEnabled, m.notifyAfter, m.notifySettle)
	m.terms[id] = t
	m.order = append(m.order, id)
	m.mu.Unlock()

	t.start(func() { m.onExit(id) })
	typeOpener(t, opener)

	m.notify()
	return t, nil
}

// OpenOnRamp launches an agent in a PTY in cwd with an on-ramp skill's starter
// prompt typed in, seated as a plain tab under spaceID titled by the skill — the
// launcher's spine, of which ideate is one skill (ticket 15). It shares
// OpenSession's launch and opener mechanics but carries no Session: an on-ramp
// launch is deliberately not a session (spec, State model — "ticketless, live,
// sharing only the adapter's spawn primitive"), so it never counts toward the
// one-live-session-per-space limit OpenSession enforces and never freezes dead the
// way a session does. Its activity reads like any other tab's: the agent grammar
// if a known agent holds the foreground (it usually does), the shell grammar
// otherwise. id is chosen by the caller, matching OpenSession's style,
// so the tab and the gitignored prompt file it points at share one identity.
func (m *Manager) OpenOnRamp(spaceID, cwd, id, name string, args, env []string, opener, title string) (*Terminal, error) {
	t, err := newProc(id, spaceID, cwd, launchSpec{name: name, args: args, env: env, title: title})
	if err != nil {
		return nil, err
	}

	// Record the tab before the read loop starts, so an agent that exits instantly
	// cannot remove itself before it has been listed (as Open and OpenSession do).
	m.mu.Lock()
	t.configureRunClock(m.notifyEnabled, m.notifyAfter, m.notifySettle)
	m.terms[id] = t
	m.order = append(m.order, id)
	m.mu.Unlock()

	t.start(func() { m.onExit(id) })
	typeOpener(t, opener)

	m.notify()
	return t, nil
}

// Opener typing thresholds. A TUI is ready for keystrokes once it has drawn
// something and then gone still, so readiness is read off the PTY's own output
// rather than guessed at with a flat sleep: openerSettle is the stillness that
// counts as drawn, openerGrace caps the wait for an agent that draws nothing at
// all, and openerSubmit is the beat between the line and its carriage return.
// Vars, not consts, so a test can shrink them.
var (
	openerSettle = 400 * time.Millisecond
	openerGrace  = 2 * time.Second
	openerSubmit = 150 * time.Millisecond
)

// typeOpener types a session's opening line into a live TUI and presses return —
// the fallback delivery, for agents that take no prompt on their command line
// (adapter.ModeType). An empty opener types nothing, which is the argv and flag
// deliveries: they already carried the line, so there is nothing to type.
//
// It is fussier than a single Write for two reasons, both learned from real TUIs:
//
//   - **Return is CR, not LF.** A `\n` is what Ctrl+J sends; TUIs that
//     distinguish the two (anything on Ink, which parses `\r` as return and `\n`
//     as linefeed) read it as "insert a newline" and leave the line sitting in the
//     composer unsent, waiting for a human to press enter.
//   - **The submit key must arrive in its own read.** Text and return in one
//     chunk look like a paste, and a TUI that buffers pastes swallows the return
//     along with the text.
//
// It runs off the caller's goroutine so a spawn's HTTP response does not wait on
// the TUI drawing itself. A write error only means the agent already exited, which
// the read loop is reaping in parallel.
func typeOpener(t *Terminal, opener string) {
	if opener == "" {
		return
	}
	go func() {
		if !t.awaitReady(openerSettle, openerGrace) {
			return // the agent died before it could be told anything
		}
		if _, err := t.Write([]byte(strings.TrimRight(opener, "\r\n"))); err != nil {
			return
		}
		time.Sleep(openerSubmit)
		_, _ = t.Write([]byte("\r"))
	}()
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
// PTY bound to a ticket), nil for an ad-hoc shell. Status is the sampler's final
// word: the agent grammar's reading where an agent holds the foreground, the shell
// grammar's otherwise, with nothing left for the server to fold in.
type Info struct {
	ID      string
	SpaceID string
	Title   string
	Proc    string
	Status  string
	Alive   bool
	Session *Session
	// FinishedUnseen is the dot: this tab finished a qualifying run and the
	// operator has not focused it since.
	FinishedUnseen bool
}

// info snapshots one terminal's public shape under its own lock.
func (t *Terminal) info() Info {
	t.mu.Lock()
	alive, proc, state, unseen := t.alive, t.proc, t.state, t.finishedUnseen
	t.mu.Unlock()
	if proc == "" {
		proc = t.Title
	}
	if state == "" {
		state = model.TerminalIdle
	}
	return Info{
		ID: t.ID, SpaceID: t.SpaceID, Title: t.Title, Proc: proc, Status: state,
		Alive: alive, Session: t.session, FinishedUnseen: unseen,
	}
}

// MarkFinishedUnseen raises the dot on a tab: it finished a run worth reporting
// and the operator has not looked at it. It is the dot's half of the one
// RunFinished event — the OS notification is the other, and neither knows about
// the other. An id that names no live terminal is a no-op: a tab can end between
// the clock emitting and this call, and there is then nothing left to mark.
func (m *Manager) MarkFinishedUnseen(id string) {
	m.mu.Lock()
	t := m.terms[id]
	m.mu.Unlock()
	if t == nil {
		return
	}
	t.mu.Lock()
	t.finishedUnseen = true
	t.mu.Unlock()
}

// MarkSeen clears the dot — the operator focused the tab, which is the whole
// acknowledgement. It reports whether it actually cleared anything, so the caller
// pushes a fresh model only when there was a dot to clear: focusing a tab that
// carries none is an ordinary no-op, not an error and not a push.
func (m *Manager) MarkSeen(id string) bool {
	m.mu.Lock()
	t := m.terms[id]
	m.mu.Unlock()
	if t == nil {
		return false
	}
	t.mu.Lock()
	defer t.mu.Unlock()
	if !t.finishedUnseen {
		return false
	}
	t.finishedUnseen = false
	return true
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
// claim, so it never claims a ticket it cannot then seat. It is also what the
// spawn path asks to decide whether the operator needs to confirm concurrency
// (ADR 0003 as amended); the confirmed spawn passes force through to OpenSession
// and this answer stops mattering.
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
//
// A drop does not flush the run clock, which is ticket 01's open question answered
// by ticket 03, the emitting seam: a run only ever ends on a sample. The reason is
// that every drop is a tab the operator is already standing at. A session that died
// on its own — the ending the whole feature exists for — *pins* rather than drops,
// keeps being sampled, and reports through the ordinary path as `dead`; what drops
// is an ad-hoc shell the operator exited, a session they killed, or an on-ramp tab
// whose agent quit. Flushing those would notify an operator about the thing they
// just did. The accepted cost is that an on-ramp tab whose agent exits on its own
// after a long run reports nothing.
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
