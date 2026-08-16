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

// ErrBadReorder marks a terminals reorder whose ids are not exactly the space's
// live terminals — a stale view, a foreign id, or a repeated one. Like the
// sidebar's own reorder, it is all-or-nothing: the order is left untouched, so a
// partial list is a client bug refused rather than a partial rearrangement.
var ErrBadReorder = errors.New("terminal: a reorder must name every terminal in the space exactly once")

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
	// genTitle turns one completed transcript turn into a short label by spending a
	// cheap model on the tab's *own* adapter and profile — same-vendor and same
	// account, so a session's conversation never crosses to a harness or a login the
	// operator didn't run it on (server side, titlegen.go). Nil is the feature-off
	// state. It runs off the sampler on its own goroutine and must not block the
	// sampler; ok=false means "no title this time", swallowed silently.
	genTitle func(TitleRequest) (title string, ok bool)
	// bindTitles matches a live agent tab to the persisted session behind it. It is
	// seated with the production binder (titlebind.go) and replaced by tests, which
	// is what lets the whole title behaviour be driven through the manager with no
	// provider store, no PTY and no clock.
	bindTitles func(*Terminal) (TitleBinding, bool)
	// autoTitleEnabled gates the whole feature from the operator's per-machine
	// toggle (autotitle.toml). Default true; ConfigureAutoTitle updates it on every
	// rebuild. Guarded by mu, read once per sample.
	autoTitleEnabled bool
	// titleSem caps how many title generations run at once process-wide, so a burst
	// of tabs going idle together cannot fan out into a crowd of CLI processes. A
	// buffered channel used as a counting semaphore; nil until the first generator
	// is wired.
	titleSem chan struct{}

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
	// titleGenConcurrency caps how many auto-title generations run at once across
	// the whole process. Two keeps a burst of idling tabs from spawning a crowd of
	// CLI processes while still letting a couple of titles land promptly.
	titleGenConcurrency = 2
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
		terms:            make(map[string]*Terminal),
		onChange:         onChange,
		onFinished:       onFinished,
		notifyEnabled:    true,
		notifyAfter:      DefaultNotifyAfter,
		notifySettle:     DefaultNotifySettle,
		autoTitleEnabled: true,
		bindTitles:       bindTranscript,
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
		now := time.Now()
		if ev := t.updateRunClock(now); ev != nil && m.onFinished != nil {
			m.onFinished(*ev)
		}
	}
	// The transcript beat rides the slow tick. A title tracks what the agent
	// persisted, not what the screen is doing this frame, so there is nothing to
	// gain from asking three times a second — and the beat is deliberately outside
	// the sampling loop, because activity no longer has anything to say about it.
	if slowTick {
		if moved, _ := m.titleTick(terms); moved {
			changed = true
		}
	}
	if changed {
		m.notify()
	}
}

// titleTick advances every tab's transcript by one beat, publishes whatever
// native titles appeared, and launches the paid generations the completed turns
// authorized. It reports whether any tab's displayed title changed and how many
// generations it launched.
//
// The toggle is read once per beat, not once per tab, and gates the whole
// feature: off means no transcript body is read and nothing is spent. A tab with
// no generator wired is in the same state — there is nobody to spend with.
//
// Generations run off this goroutine behind the process-wide concurrency cap, so
// a burst of tabs completing turns together can neither stall the sampler nor
// fan out into a crowd of CLI processes.
func (m *Manager) titleTick(terms []*Terminal) (changed bool, launched int) {
	on := m.genTitle != nil && m.autoTitleOn()
	for _, t := range terms {
		attempt, spend, moved := t.titleBeat(on, m.bindTitles)
		if moved {
			changed = true
		}
		if spend {
			launched++
			go m.runTitleGen(t, attempt)
		}
	}
	return changed, launched
}

// SetTitleGenerator wires the cheap-model title generator the sampler spends on
// and seats the concurrency semaphore. The server sets it once after construction;
// passing nil turns auto-titling off. It is set before the first sample matters, so
// no lock guards it — the sampler reads it, the server writes it once at startup.
func (m *Manager) SetTitleGenerator(gen func(TitleRequest) (string, bool)) {
	m.genTitle = gen
	m.titleSem = make(chan struct{}, titleGenConcurrency)
}

// ConfigureAutoTitle applies the operator's per-machine auto-title toggle. Safe to
// call on every rebuild; it flips a single guarded flag the transcript beat reads,
// so turning the feature off stops both transcript observation and generation at
// once (a title already on screen simply stays, and stops updating).
func (m *Manager) ConfigureAutoTitle(enabled bool) {
	m.mu.Lock()
	m.autoTitleEnabled = enabled
	m.mu.Unlock()
}

// autoTitleOn reports the toggle under the lock — the sampler's one read per tick.
func (m *Manager) autoTitleOn() bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.autoTitleEnabled
}

// runTitleGen spends the session's one attempt off the beat's goroutine and, on a
// usable title, stores it and pushes a fresh model. The process-wide semaphore
// bounds how many run at once.
//
// A failure — the adapter declined, its models were exhausted, the run was
// cancelled, nothing usable came back — changes nothing on screen and is not
// retried: the attempt was consumed when it was scheduled. Auto-titling never
// surfaces an error.
func (m *Manager) runTitleGen(t *Terminal, at titleAttempt) {
	m.titleSem <- struct{}{}
	defer func() { <-m.titleSem }()

	title, ok := m.genTitle(at.req)
	if !ok {
		return
	}
	if t.titleGenerated(at, title) {
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
// caller (the same id the claim and payload archive are keyed by), so the
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
		agent:   s.Agent,
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

// OpenFree launches an agent bare in a PTY in cwd, seated as a plain tab under
// spaceID titled by the agent the operator picked — the new-shell control's
// spine. It shares OpenSession's launch mechanics but injects nothing (a free
// session is told no brief — the operator types their first message themselves)
// and carries no Session: a free session is deliberately not a session (spec,
// State model — "ticketless, live, sharing only the adapter's spawn primitive"),
// so it never counts toward the one-live-session-per-space limit OpenSession
// enforces and never freezes dead the way a session does. Its activity reads like
// any other tab's: the agent grammar if a known agent holds the foreground (it
// usually does), the shell grammar otherwise. id is chosen by the caller, matching
// OpenSession's style.
//
// agent is the adapter being launched — carried for the same reason a session
// carries it: chartr chose this binary, so the sampler is told rather than made to
// rediscover it off the PTY, and a tab on an agent with no shipped manifest reads
// working-while-alive instead of a permanent, wrong idle.
func (m *Manager) OpenFree(spaceID, cwd, id, name string, args, env []string, title, agent string) (*Terminal, error) {
	t, err := newProc(id, spaceID, cwd, launchSpec{name: name, args: args, env: env, title: title, agent: agent})
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
	// AutoTitle is the tab's automatic title — the agent's own native title for its
	// session, or the one label chartr generated from its first completed turn.
	// Empty until one of those lands, and on tabs with no agent in front, which are
	// never titled.
	AutoTitle string
}

// info snapshots one terminal's public shape under its own lock.
func (t *Terminal) info() Info {
	t.mu.Lock()
	alive, proc, state, unseen, title := t.alive, t.proc, t.state, t.finishedUnseen, t.autoTitle
	t.mu.Unlock()
	if proc == "" {
		proc = t.Title
	}
	if state == "" {
		state = model.TerminalIdle
	}
	return Info{
		ID: t.ID, SpaceID: t.SpaceID, Title: t.Title, Proc: proc, Status: state,
		Alive: alive, Session: t.session, FinishedUnseen: unseen, AutoTitle: title,
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

// Reorder rearranges spaceID's terminals into the sequence `ids` — the whole
// ordered list of that space's terminal ids, never a per-row move — and leaves
// every other space's terminals exactly where they sit in the global order. A tab
// never crosses into another space: the drag is scoped to one card, so `ids` is
// only ever a permutation of this space's own terminals, and one that omits a
// terminal, names an unknown id, or repeats one is refused with ErrBadReorder and
// changes nothing (the same all-or-nothing contract the sidebar reorder keeps).
//
// It writes only the space's slots in the global order, in the new sequence, so a
// foreign id between two of this space's tabs keeps its position. A reorder that
// lands the same sequence still pushes, harmlessly — the client posts the whole
// list on every drop, and the server does not special-case a no-op it cannot
// cheaply tell apart from a real move.
func (m *Manager) Reorder(spaceID string, ids []string) error {
	m.mu.Lock()
	// The space's terminal ids in their current global order — the set `ids` must
	// be a permutation of.
	current := make([]string, 0)
	for _, id := range m.order {
		if t := m.terms[id]; t != nil && t.SpaceID == spaceID {
			current = append(current, id)
		}
	}
	if len(ids) != len(current) {
		m.mu.Unlock()
		return fmt.Errorf("%w: %d ids for %d terminals", ErrBadReorder, len(ids), len(current))
	}
	inSpace := make(map[string]bool, len(current))
	for _, id := range current {
		inSpace[id] = true
	}
	seen := make(map[string]bool, len(ids))
	for _, id := range ids {
		if !inSpace[id] {
			m.mu.Unlock()
			return fmt.Errorf("%w: no terminal %q in the space", ErrBadReorder, id)
		}
		if seen[id] {
			m.mu.Unlock()
			return fmt.Errorf("%w: %q appears twice", ErrBadReorder, id)
		}
		seen[id] = true
	}
	// Refill the space's slots in place, in the new order; every foreign slot is
	// left untouched.
	next := 0
	for i, id := range m.order {
		if t := m.terms[id]; t != nil && t.SpaceID == spaceID {
			m.order[i] = ids[next]
			next++
		}
	}
	m.mu.Unlock()
	m.notify()
	return nil
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
// is an ad-hoc shell the operator exited, a session they killed, or a free tab
// whose agent quit. Flushing those would notify an operator about the thing they
// just did. The accepted cost is that a free tab whose agent exits on its own
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
