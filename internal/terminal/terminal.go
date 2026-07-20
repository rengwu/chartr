// Package terminal owns the harness's PTYs and the ad-hoc shells that run in
// them. An ad-hoc shell is deliberately not a session (spec, State model): it
// carries no ticket, no lifecycle, and no review — it is a plain multiplexer
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

	"github.com/aymanbagabas/go-pty"
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

// Terminal is one ad-hoc shell: a running process attached to a PTY, its bounded
// server-side scrollback, and the set of browser sockets currently watching it.
// It is created and owned by a Manager; callers reach it only through Attach,
// Write, and Resize.
type Terminal struct {
	// ID is the terminal's stable identity within the harness process. SpaceID
	// ties it to the space whose working tree it runs in; Title labels its tab.
	ID      string
	SpaceID string
	Title   string

	pty pty.Pty
	cmd *pty.Cmd

	mu         sync.Mutex
	scrollback []byte
	subs       map[*subscriber]struct{}
	alive      bool
	done       chan struct{}
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

// close ends the shell on the human's command. Killing the process makes the
// read loop's Read return, which runs the same cleanup path as a natural exit.
// A terminal already dead is a no-op.
func (t *Terminal) close() {
	t.mu.Lock()
	alive := t.alive
	proc := t.cmd.Process
	t.mu.Unlock()
	if alive && proc != nil {
		_ = proc.Kill()
	}
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

// newTerminal opens a PTY and starts the shell in cwd, returning a live Terminal
// whose read loop is *not* yet running. The caller registers it before starting
// the loop (via start), so a shell that exits instantly cannot run its cleanup —
// which drops the terminal from its Manager — before it has been recorded.
func newTerminal(id, spaceID, cwd string) (*Terminal, error) {
	p, err := pty.New()
	if err != nil {
		return nil, fmt.Errorf("opening pty: %w", err)
	}
	name, args := shellCommand()
	c := p.Command(name, args...)
	c.Dir = cwd
	c.Env = append(os.Environ(), "TERM=xterm-256color")
	if err := c.Start(); err != nil {
		_ = p.Close()
		return nil, fmt.Errorf("starting %s: %w", name, err)
	}

	return &Terminal{
		ID:      id,
		SpaceID: spaceID,
		Title:   shellTitle(name),
		pty:     p,
		cmd:     c,
		subs:    make(map[*subscriber]struct{}),
		alive:   true,
		done:    make(chan struct{}),
	}, nil
}

// start begins the read loop; cleanup runs once the shell exits.
func (t *Terminal) start(cleanup func()) { go t.pump(cleanup) }
