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
	// lastActivity is when this PTY last produced output. Silence past a threshold
	// is what earns an AFK session the "quiet" hint (ticket 10); it is refreshed on
	// every chunk broadcast, so any agent output — even a spinner redraw — resets it.
	lastActivity time.Time
	// spoke records that this PTY has produced at least one byte. lastActivity is
	// seeded at launch (so silence is measured from there), which makes it unable to
	// tell "has drawn nothing yet" from "has been quiet a while" — the distinction
	// awaitReady needs to know a TUI is up rather than merely slow to start.
	spoke bool
	// proc/state are the last sampled foreground process and activity (one of the
	// model.Terminal* states); lastPgrp caches the foreground group so a name is
	// only re-resolved when the foreground actually changes. silent is the last
	// sampled silence verdict for a session (alive and quiet past the threshold),
	// which the server folds together with the role to decide whether the tab
	// actually reads quiet.
	proc     string
	state    string
	lastPgrp int
	silent   bool
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
		alive, spoke, quiet := t.alive, t.spoke, time.Since(t.lastActivity)
		t.mu.Unlock()
		switch {
		case !alive:
			return false
		case spoke && quiet >= settle, time.Now().After(deadline):
			return true
		}
		time.Sleep(poll)
	}
}

// Resize sets the PTY window size (columns, rows) so the shell reflows to the
// browser's terminal geometry.
func (t *Terminal) Resize(cols, rows int) error { return t.pty.Resize(cols, rows) }

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
func (t *Terminal) pump(done func()) {
	buf := make([]byte, 4096)
	for {
		n, err := t.pty.Read(buf)
		if n > 0 {
			t.broadcast(buf[:n])
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
		// A session reads the session grammar, not a shell's idle/working: seat it
		// as working until the first sample decides otherwise, so a just-spawned
		// session orbits rather than flashing a spurious idle tick.
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
		// Seed silence from launch, so an AFK session that never emits a byte still
		// crosses into quiet once the threshold elapses.
		lastActivity: time.Now(),
		// Seat the initial view under the tab's own name before the first sample.
		proc:  title,
		state: state,
	}, nil
}

// newTerminal opens an ad-hoc shell in cwd (see newProc).
func newTerminal(id, spaceID, cwd string) (*Terminal, error) {
	name, args := shellCommand()
	return newProc(id, spaceID, cwd, launchSpec{name: name, args: args})
}

// start begins the read loop; cleanup runs once the shell exits.
func (t *Terminal) start(cleanup func()) { go t.pump(cleanup) }

// sample recomputes a tab's activity, returning whether it changed since the last
// sample. The manager's sampler loop calls it off the manager lock, and a change
// is what triggers a fresh model push — so a shell going busy, or an AFK session
// falling silent, updates the sidebar on its own with no filesystem or socket
// event behind it. A session reads on output silence (its agent holds the PTY's
// foreground for its whole life, so a shell's foreground-group signal says
// nothing); an ad-hoc shell reads on its foreground group as before.
func (t *Terminal) sample(quietAfter time.Duration) bool {
	if t.session != nil {
		return t.sampleSession(quietAfter)
	}
	return t.sampleShell()
}

// sampleSession recomputes a session tab: dead once its process exits, otherwise
// working, with a silence verdict (alive and quiet past the threshold) the server
// folds together with the role to decide whether it actually reads quiet. The
// quiet decision is deliberately not made here — the terminal knows the silence
// but not the role's AFK-ness — so a threshold crossing still pushes, and the
// server has the last word.
func (t *Terminal) sampleSession(quietAfter time.Duration) bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	state, silent := model.TerminalWorking, false
	if !t.alive {
		state = model.TerminalDead
	} else {
		silent = time.Since(t.lastActivity) > quietAfter
	}
	if state == t.state && silent == t.silent {
		return false
	}
	t.state, t.silent = state, silent
	return true
}

// sampleShell recomputes an ad-hoc shell's foreground process and activity. The
// exec that resolves a process name happens outside the terminal lock, and only
// when the foreground group actually changes, so a busy shell doesn't pay for it
// every tick.
func (t *Terminal) sampleShell() bool {
	t.mu.Lock()
	alive := t.alive
	prevPgrp := t.lastPgrp
	prevProc := t.proc
	t.mu.Unlock()

	if !alive {
		t.mu.Lock()
		defer t.mu.Unlock()
		if t.state == model.TerminalExited {
			return false
		}
		t.state, t.proc = model.TerminalExited, t.Title
		return true
	}

	pgrp := foreground(t.pty)
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
	t.lastPgrp = pgrp
	if state == t.state && proc == t.proc {
		return false
	}
	t.state, t.proc = state, proc
	return true
}
