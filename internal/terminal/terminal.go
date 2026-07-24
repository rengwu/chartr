// Package terminal owns chartr's PTYs and the ad-hoc shells that run in
// them. An ad-hoc shell is deliberately not a session (spec, State model): it
// carries no ticket and no lifecycle — it is a plain multiplexer
// terminal in a space's working tree, opened by the operator and ended by the
// operator. The one thing it shares with a real session is the PTY primitive
// this package provides, built from day one on a cross-platform, ConPTY-capable
// library so the session core never ossifies unix-only (ADR 0006 as amended).
//
// Raw PTY output is buffered server-side (bounded scrollback) and replayed to a
// browser on attach, so a reconnecting terminal walks back into its running
// shell rather than a blank pane (ADR 0006). The terminal socket that carries
// these bytes never shares a connection with the control socket, so a flooding
// terminal cannot head-of-line-block map updates (ADR 0010).
package terminal

import (
	"fmt"
	"os"
	"runtime"
	"sync"
	"time"

	"github.com/aymanbagabas/go-pty"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/terminal/detect"
)

// scrollbackCap bounds the server-side replay buffer per terminal. Raw PTY bytes
// are retained up to this many bytes; once past it the oldest are dropped, so a
// long-lived shell cannot grow the buffer without bound. Replay is best effort:
// a reattaching browser sees the last scrollbackCap bytes — enough to walk back
// into a running shell.
const scrollbackCap = 256 << 10 // 256 KiB

// subCapacity is the buffered depth of one attached socket's down-channel. A
// browser that cannot keep up past this is killed as a slow consumer and left to
// reattach and replay, rather than back-pressuring the PTY read loop — the same
// policy the control-socket hub uses, and the reason the two sockets never share
// a connection.
const subCapacity = 256

// Session is the ticket binding a tab carries when it is a session rather than an
// ad-hoc shell — a PTY running an agent against exactly one ticket (the
// session↔ticket invariant, spec State model). It records what the tab renders:
// the map and ticket the session is claimed on, the role it was spawned as, and
// the resolved agent and model driving it. Nil on an ad-hoc shell, which carries
// no ticket and no lifecycle. Immutable after the tab is created.
type Session struct {
	MapSlug   string
	TicketNum int
	Role      string
	// Agent is the adapter — the binary that actually ran. AgentName is the
	// operator's registered name for it, empty when no agent was named. Both are
	// kept because they answer different questions: the adapter is what a tab
	// renders and what means something anywhere, and the name is what a resume or
	// respawn relaunches, so "start over cleanly" changes the payload and not the
	// execution rather than re-deciding it from config.
	Agent     string
	AgentName string
}

// Terminal is one tab: a running process attached to a PTY, its bounded
// server-side scrollback, and the set of browser sockets currently watching it.
// It is created and owned by a Manager; callers reach it only through Attach,
// Write, and Resize. A tab is an ad-hoc shell by default; one opened through
// OpenSession additionally carries a Session, which is the only thing that makes
// it a session rather than a plain multiplexer terminal.
type Terminal struct {
	// ID is the terminal's stable identity within the chartr process. SpaceID
	// ties it to the space whose working tree it runs in; Title labels its tab.
	ID      string
	SpaceID string
	Title   string

	// session is the ticket binding when this tab is a session, nil for an ad-hoc
	// shell. Immutable after start.
	session *Session

	// shellPID is the shell process's pid, which (being a session leader on a new
	// PTY) is also its process-group id. The foreground group equals it exactly
	// when the shell sits at its prompt, which is how sample tells idle from
	// working. Immutable after start.
	shellPID int

	pty pty.Pty
	cmd *pty.Cmd

	mu         sync.Mutex
	scrollback []byte
	subs       map[*subscriber]struct{}
	alive      bool
	done       chan struct{}
	// killed records that the operator ended this tab (Close/Shutdown) rather than
	// its process dying on its own. It is the one thing that tells a death-halt from
	// a dismissal: a session whose process exits stays pinned to its ticket for the
	// operator to resume/respawn/release, but a session the operator closed — like
	// any ad-hoc shell — drops from the model (ticket 10).
	killed bool
	// lastActivity is when this PTY last produced output, refreshed on every chunk
	// broadcast. It is no longer an activity signal — a TUI repaints its cursor
	// forever, which is why the silence heuristic was retired — but awaitReady still
	// reads it to tell a TUI that has finished painting from one still coming up.
	lastActivity time.Time
	// spoke records that this PTY has produced at least one byte. lastActivity is
	// seeded at launch, which makes it unable to tell "has drawn nothing yet" from
	// "has been still a while" — the distinction awaitReady needs to know a TUI is
	// up rather than merely slow to start.
	spoke bool
	// proc/state are the last sampled foreground process and activity (one of the
	// model.Terminal* states); lastPgrp caches the foreground group so the group is
	// only re-read when the foreground actually changes.
	proc     string
	state    string
	lastPgrp int

	// The agent grammar's state, all guarded by mu.
	//
	// oscTitle/oscProgress are the latest OSC values the read loop sniffed out of
	// the PTY — the evidence the agent broadcasts about itself. They are cleared
	// whenever the foreground agent changes, so one agent never inherits another's.
	//
	// agent is the identified foreground agent ("" when none), and pub carries that
	// agent's publishing hysteresis. Both are reseated when the identified agent
	// changes.
	oscTitle    string
	oscProgress string
	agent       string
	pub         *publisher

	// grid reconstructs the terminal's visible screen server-side from the same
	// PTY bytes the browser renders, read by the sampler for detection only (never
	// replayed back — ADR 0010). It has its own lock, so it is reached directly
	// rather than under mu.
	grid *grid
}

// subscriber is one attached terminal socket. Down-frames are delivered through
// a buffered channel; dead is closed both when the socket is killed as a slow
// consumer and when the terminal itself ends, so a handler selecting on it wakes
// for either.
type subscriber struct {
	ch   chan []byte
	dead chan struct{}
	once sync.Once
}

func (s *subscriber) kill() { s.once.Do(func() { close(s.dead) }) }

// Attachment is one browser's live view of a terminal: the scrollback to replay
// first, then the stream of raw down-frames, plus a Done channel that fires when
// this attachment is torn down (terminal ended or dropped as a slow consumer).
type Attachment struct {
	// Scrollback is the buffered bytes to replay before streaming, captured at
	// attach time.
	Scrollback []byte
	// Frames carries raw PTY bytes to write down to xterm.js.
	Frames <-chan []byte
	// Done fires when the attachment ends — the terminal exited, or this socket
	// fell behind and was dropped to reattach and replay.
	Done <-chan struct{}

	sub *subscriber
	t   *Terminal
}

// Detach removes the attachment from its terminal. Idempotent and safe to call
// after the terminal has already ended.
func (a *Attachment) Detach() {
	a.t.mu.Lock()
	delete(a.t.subs, a.sub)
	a.t.mu.Unlock()
}

// Attach registers a new browser socket and returns its scrollback snapshot and
// live stream. Attaching to a terminal that has already ended returns its final
// scrollback with an already-fired Done, so a handler replays the record and
// closes cleanly rather than hanging.
func (t *Terminal) Attach() *Attachment {
	s := &subscriber{ch: make(chan []byte, subCapacity), dead: make(chan struct{})}
	t.mu.Lock()
	sb := make([]byte, len(t.scrollback))
	copy(sb, t.scrollback)
	if t.alive {
		t.subs[s] = struct{}{}
	} else {
		s.kill()
	}
	t.mu.Unlock()
	return &Attachment{Scrollback: sb, Frames: s.ch, Done: s.dead, sub: s, t: t}
}

// Write sends keystrokes up into the shell. It is the operator's intervention
// channel — raw bytes straight to the PTY.
func (t *Terminal) Write(p []byte) (int, error) { return t.pty.Write(p) }

// awaitReady blocks until this tab's process looks ready to be typed at: it has
// drawn something and then held still for settle. A TUI paints its frame as it
// comes up, so stillness after paint is the closest thing to a readiness
// handshake a PTY offers — and it beats a flat sleep in both directions, waiting
// longer for a slow agent and less for a fast one.
//
// It gives up after grace and reports true anyway: an agent that draws nothing
// (a stub, a pipe-like CLI) is not thereby unreachable, and the keystrokes sit in
// the PTY buffer until it reads. It reports false only for a process that has
// already exited, where there is nobody left to type to.
func (t *Terminal) awaitReady(settle, grace time.Duration) bool {
	const poll = 25 * time.Millisecond
	deadline := time.Now().Add(grace)
	for {
		t.mu.Lock()
		// `still` is how long this PTY has drawn nothing. It is a readiness signal
		// only — not the retired silence heuristic, which tried to read *activity*
		// off the same clock and could not, because a TUI repaints forever.
		alive, spoke, still := t.alive, t.spoke, time.Since(t.lastActivity)
		t.mu.Unlock()
		switch {
		case !alive:
			return false
		case spoke && still >= settle, time.Now().After(deadline):
			return true
		}
		time.Sleep(poll)
	}
}

// Resize sets the PTY window size (columns, rows) so the shell reflows to the
// browser's terminal geometry, and follows the reconstruction grid to the same
// size so a region anchored to the bottom of the screen stays meaningful.
func (t *Terminal) Resize(cols, rows int) error {
	t.grid.resize(cols, rows)
	return t.pty.Resize(cols, rows)
}

// detectionText returns the reconstructed screen the rule engine reads. It is read
// for detection only and never replayed to the browser (ADR 0010).
func (t *Terminal) detectionText() string { return t.grid.text() }

// broadcast appends a chunk to scrollback and fans it out to every attached
// socket. A socket whose buffer is full is a slow consumer: it is killed and
// left to reattach and replay, so one wedged browser never stalls the read loop.
func (t *Terminal) broadcast(chunk []byte) {
	b := make([]byte, len(chunk))
	copy(b, chunk)

	t.mu.Lock()
	t.lastActivity, t.spoke = time.Now(), true
	t.scrollback = appendCapped(t.scrollback, b, scrollbackCap)
	for s := range t.subs {
		select {
		case s.ch <- b:
		case <-s.dead:
		default:
			s.kill()
		}
	}
	t.mu.Unlock()
}

// pump copies raw PTY output into scrollback and out to attached sockets until
// the shell exits, then reaps the process and runs cleanup: it marks the
// terminal dead, wakes every attached socket, closes the PTY, and calls done
// (which drops the terminal from its Manager and pushes a fresh model).
//
// It is also where OSC sniffing happens, on the one goroutine that already sees
// every byte. The scanner lives across iterations because a title genuinely splits
// across two Reads, and it retains only the latest value of each — the sampler
// reads those, not the stream.
func (t *Terminal) pump(done func()) {
	buf := make([]byte, 4096)
	var osc oscScanner
	for {
		n, err := t.pty.Read(buf)
		if n > 0 {
			t.broadcast(buf[:n])
			osc.scan(buf[:n], t.setOSCTitle, t.setOSCProgress)
			// Feed the same bytes to the screen reconstruction. It is read only for
			// detection; the browser still renders from the raw scrollback (ADR 0010).
			t.grid.write(buf[:n])
		}
		if err != nil {
			break
		}
	}

	_ = t.cmd.Wait()

	t.mu.Lock()
	t.alive = false
	close(t.done)
	for s := range t.subs {
		s.kill()
	}
	t.subs = map[*subscriber]struct{}{}
	t.mu.Unlock()

	t.grid.close()
	_ = t.pty.Close()
	done()
}

// close ends the tab on the human's command. It marks the tab killed — so a
// pinned dead session the operator dismisses drops from the model rather than
// halting a second time — then, if the process is still live, kills it, which
// makes the read loop's Read return and runs the same cleanup path as a natural
// exit. It reports whether it killed a live process; a caller closing an
// already-dead pinned session gets false and drops the tab itself, since no exit
// will fire to do it.
func (t *Terminal) close() bool {
	t.mu.Lock()
	alive := t.alive
	proc := t.cmd.Process
	t.killed = true
	t.mu.Unlock()
	if alive && proc != nil {
		_ = proc.Kill()
		return true
	}
	return false
}

// pinOnDeath reports whether this tab should stay pinned to its ticket when its
// process exits — true only for a session that died on its own (ticket 10). An
// ad-hoc shell, or a session the operator killed, drops from the model instead.
func (t *Terminal) pinOnDeath() bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.session != nil && !t.killed
}

// appendCapped appends chunk to buf and, if the result exceeds capBytes, trims
// to the last capBytes into a fresh slice so the backing array does not grow
// without bound as the shell runs.
func appendCapped(buf, chunk []byte, capBytes int) []byte {
	buf = append(buf, chunk...)
	if len(buf) > capBytes {
		trimmed := make([]byte, capBytes)
		copy(trimmed, buf[len(buf)-capBytes:])
		buf = trimmed
	}
	return buf
}

// shellCommand picks the operator's interactive shell per platform. The PTY's
// line discipline echoes keystrokes regardless of the shell, so even a bare
// /bin/sh satisfies the echo-and-run baseline; a richer shell brings its own
// prompt. Windows runs ConPTY under COMSPEC (ADR 0006 as amended).
func shellCommand() (string, []string) {
	if runtime.GOOS == "windows" {
		if c := os.Getenv("COMSPEC"); c != "" {
			return c, nil
		}
		return "cmd.exe", nil
	}
	if sh := os.Getenv("SHELL"); sh != "" {
		return sh, nil
	}
	return "/bin/sh", nil
}

func shellTitle(name string) string {
	base := name
	for i := len(name) - 1; i >= 0; i-- {
		if name[i] == '/' || name[i] == '\\' {
			base = name[i+1:]
			break
		}
	}
	if base == "" {
		return "shell"
	}
	return base
}

// launchSpec is the command a tab runs and the identity it seats with. An ad-hoc
// shell fills name/args from the operator's shell; a session fills them from its
// adapter and carries a Session plus an explicit title (the agent name is not the
// tab's identity — the ticket it is bound to is).
type launchSpec struct {
	name    string
	args    []string
	title   string
	session *Session
}

// newProc opens a PTY and starts spec's command in cwd, returning a live Terminal
// whose read loop is *not* yet running. The caller registers it before starting
// the loop (via start), so a process that exits instantly cannot run its cleanup —
// which drops the terminal from its Manager — before it has been recorded. It is
// the one launch path both ad-hoc shells and sessions share (spec: the adapter's
// spawn primitive is all a session and a shell have in common).
func newProc(id, spaceID, cwd string, spec launchSpec) (*Terminal, error) {
	p, err := pty.New()
	if err != nil {
		return nil, fmt.Errorf("opening pty: %w", err)
	}
	c := p.Command(spec.name, spec.args...)
	c.Dir = cwd
	c.Env = append(os.Environ(), "TERM=xterm-256color")
	if err := c.Start(); err != nil {
		_ = p.Close()
		return nil, fmt.Errorf("starting %s: %w", spec.name, err)
	}

	title := spec.title
	if title == "" {
		title = shellTitle(spec.name)
	}
	state := model.TerminalIdle
	if spec.session != nil {
		// Seat a session as working until the first sample decides otherwise, so a
		// just-spawned session orbits rather than flashing a spurious idle tick while
		// its agent boots. The agent grammar's startup grace holds the same line once
		// the agent is identified.
		state = model.TerminalWorking
	}
	return &Terminal{
		ID:       id,
		SpaceID:  spaceID,
		Title:    title,
		session:  spec.session,
		shellPID: c.Process.Pid,
		pty:      p,
		cmd:      c,
		subs:     make(map[*subscriber]struct{}),
		alive:    true,
		done:     make(chan struct{}),
		// Seed from launch so awaitReady measures stillness from the moment the
		// process started rather than from an unset zero time.
		lastActivity: time.Now(),
		// Seat the initial view under the tab's own name before the first sample.
		proc:  title,
		state: state,
		// The screen reconstruction starts at the PTY's launch geometry and follows
		// it on every browser resize.
		grid: newGrid(gridDefaultCols, gridDefaultRows),
	}, nil
}

// newTerminal opens an ad-hoc shell in cwd (see newProc).
func newTerminal(id, spaceID, cwd string) (*Terminal, error) {
	name, args := shellCommand()
	return newProc(id, spaceID, cwd, launchSpec{name: name, args: args})
}

// start begins the read loop; cleanup runs once the shell exits.
func (t *Terminal) start(cleanup func()) { go t.pump(cleanup) }

// hasAgent reports whether a known agent currently holds this tab's foreground —
// the one bit the sampler needs to decide which cadence this tab is on, cheap
// enough to ask on every tick.
func (t *Terminal) hasAgent() bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.agent != ""
}

// setOSCTitle / setOSCProgress retain the latest OSC value the read loop sniffed.
// They are the only writers; the sampler reads them under the same lock.
func (t *Terminal) setOSCTitle(s string) {
	t.mu.Lock()
	t.oscTitle = s
	t.mu.Unlock()
}

func (t *Terminal) setOSCProgress(s string) {
	t.mu.Lock()
	t.oscProgress = s
	t.mu.Unlock()
}

// sample recomputes a tab's activity, returning whether it changed since the last
// sample. The manager's sampler loop calls it off the manager lock, and a change
// is what triggers a fresh model push — so a tab going busy or idle updates the
// sidebar on its own with no filesystem or socket event behind it.
//
// There is one grammar, not two. A tab is resolved to a known agent or to nothing,
// and a known agent reads the *agent* grammar — its own broadcast evidence, through
// the rule engine and the publishing hysteresis — whether or not the tab carries a
// Session. The reported bug was an ad-hoc shell running `claude`, which is exactly
// the case a session-only grammar could not reach.
//
// Only *how* a tab is resolved differs, and only because the answer is already
// known for one of them: a session's agent comes from the binding chartr recorded
// when it launched it, while an ad-hoc shell's has to be read off whatever holds
// the PTY's foreground. Everything downstream of the resolution is shared.
func (t *Terminal) sample(eng *detect.Engine) bool {
	t.mu.Lock()
	alive, isSession := t.alive, t.session != nil
	prevPgrp, prevAgent := t.lastPgrp, t.agent
	t.mu.Unlock()

	if !alive {
		return t.sampleGone(isSession)
	}

	// A session does not need to be identified by inspection: chartr launched its
	// agent and recorded which adapter it ran, so the binding *is* the answer. That
	// is both cheaper and steadier than reading the PTY — and it keeps the ioctl and
	// the `ps` off session PTYs entirely, as they always were.
	if isSession {
		agent := ""
		if a := t.session.Agent; eng.Known(a) {
			agent = a
		}
		t.seatAgent(agent, prevAgent)
		if agent != "" {
			return t.sampleAgent(agent, eng)
		}
		// A session running an agent we ship no manifest for (kimi, opencode and pi
		// are ticket 02's) is still an agent tab. The shell grammar cannot speak for
		// it — a session's root process *is* the agent, so it holds the foreground
		// for its whole life and the foreground-group signal would read a permanent,
		// wrong "idle". It keeps what it has always read: working while alive.
		return t.sampleUnknownSession()
	}

	// An ad-hoc shell is the case that has to be inspected: the operator typed
	// something, and whatever holds the foreground decides which grammar the tab
	// reads. The group is re-read only when the foreground actually changes, so a
	// busy shell does not exec `ps` every tick.
	//
	// A non-positive pgrp means the platform could not tell us — not that the
	// foreground went away. Treating it as "no agent" would drop the identification
	// and reseat the hysteresis (restarting the startup grace) on every unreadable
	// tick, which strobes. An unreadable foreground changes nothing instead.
	pgrp := foreground(t.pty)
	agent := prevAgent
	if pgrp > 0 && pgrp != prevPgrp {
		agent = eng.Identify(procGroupNames(pgrp))
	}
	t.seatAgent(agent, prevAgent)

	if pgrp > 0 {
		t.mu.Lock()
		t.lastPgrp = pgrp
		t.mu.Unlock()
	}

	if agent != "" {
		return t.sampleAgent(agent, eng)
	}
	return t.sampleShell(pgrp, prevPgrp)
}

// seatAgent records a change of identified agent: it drops the retained OSC
// evidence so a new agent never inherits the old one's title, and reseats the
// publishing hysteresis (restarting its startup grace). A no-op when the agent is
// the same as last sample, which is the common case.
func (t *Terminal) seatAgent(agent, prev string) {
	if agent == prev {
		return
	}
	t.mu.Lock()
	defer t.mu.Unlock()
	t.oscTitle, t.oscProgress = "", ""
	t.agent = agent
	if agent == "" {
		t.pub = nil
		return
	}
	t.pub = newPublisher(time.Now())
}

// sampleUnknownSession is the fallback for a live session running an agent chartr
// has no manifest for: working, exactly as every session read before detection
// existed.
func (t *Terminal) sampleUnknownSession() bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.state == model.TerminalWorking {
		return false
	}
	t.state = model.TerminalWorking
	return true
}

// sampleGone settles a tab whose process is over. A session freezes dead — pinned
// to its ticket for the operator to resume, respawn, or release (ticket 10) — and
// an ad-hoc shell reads exited. Neither is touched by agent detection.
func (t *Terminal) sampleGone(isSession bool) bool {
	want := model.TerminalExited
	if isSession {
		want = model.TerminalDead
	}
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.state == want {
		return false
	}
	t.state = want
	if !isSession {
		t.proc = t.Title
	}
	return true
}

// sampleAgent reads the agent grammar: the rule engine's verdict on the evidence
// the agent broadcast about itself, folded through the publishing hysteresis so a
// positive signal lands at once and a bare absence is confirmed before it moves
// anything. The tab's process name reads as the agent, which is what the operator
// is actually looking at.
func (t *Terminal) sampleAgent(agent string, eng *detect.Engine) bool {
	// The screen is read off the grid's own lock, before taking t.mu — the two
	// locks never nest.
	screen := t.detectionText()

	t.mu.Lock()
	ev := detect.Evidence{Title: t.oscTitle, Progress: t.oscProgress, Screen: screen}
	pub := t.pub
	if pub == nil { // an agent is always seated with its hysteresis; belt and braces
		pub = newPublisher(time.Now())
		t.pub = pub
	}
	t.mu.Unlock()

	res := eng.Evaluate(agent, ev)
	now := time.Now()

	t.mu.Lock()
	defer t.mu.Unlock()
	state, changed := pub.update(res, now)
	if t.proc != agent {
		t.proc, changed = agent, true
	}
	if t.state != state {
		t.state, changed = state, true
	}
	return changed
}

// sampleShell recomputes a tab with no known agent in its foreground: today's
// grammar unchanged — idle while the shell sits at its prompt, working under the
// name of whatever command holds the foreground. The exec that resolves that name
// happens outside the terminal lock, and only when the foreground group actually
// changes, so a busy shell doesn't pay for it every tick.
func (t *Terminal) sampleShell(pgrp, prevPgrp int) bool {
	t.mu.Lock()
	prevProc := t.proc
	t.mu.Unlock()

	state, proc := model.TerminalIdle, t.Title
	if pgrp > 0 && pgrp != t.shellPID {
		state = model.TerminalWorking
		switch {
		case pgrp == prevPgrp && prevProc != "":
			proc = prevProc // same foreground group — reuse the resolved name
		default:
			if n := procName(pgrp); n != "" {
				proc = n
			}
		}
	}

	t.mu.Lock()
	defer t.mu.Unlock()
	if state == t.state && proc == t.proc {
		return false
	}
	t.state, t.proc = state, proc
	return true
}
