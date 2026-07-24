package terminal

import (
	"io"
	"sync"

	"github.com/charmbracelet/x/vt"
)

// gridDefaultCols/gridDefaultRows seed the grid before the browser reports the
// real geometry. A PTY launches at the platform default and is resized the moment
// a browser attaches (applyResize → Terminal.Resize → grid.resize), so this is a
// short-lived stand-in; agents lay out against the reported width, so the grid
// tracks the PTY to keep the reconstruction faithful.
const (
	gridDefaultCols = 80
	gridDefaultRows = 24
	// gridScrollbackLines caps the emulator's off-screen history. Detection only ever
	// reads the visible viewport (String), never the scrollback, so the library's
	// 10000-line default would just hold tens of MB of history per tab for nothing.
	// A small cap keeps the grid's memory near the viewport it is actually read for.
	gridScrollbackLines = 256
)

// grid is a server-side reconstruction of a terminal's visible screen. It is fed
// the same raw PTY bytes the browser renders, but only ever *read* — never
// replayed back — so the detection path can slice the screen structurally (ADR
// 0010: the browser keeps rendering through xterm.js; this is not a second display
// source). text() hands the rendered viewport to the rule engine.
//
// It wraps charmbracelet/x/vt's Emulator, which is not safe for concurrent use:
// the pump goroutine writes bytes while the sampler reads the rendered text and a
// browser resize reshapes it, so every access goes through mu. mu is the grid's
// own lock, independent of the Terminal's, so grid access never interacts with the
// terminal's state lock.
//
// The emulator answers terminal queries the agent makes — claude opens a session
// with OSC 11;? asking the background colour — by writing the reply into an
// internal pipe. With no reader that write blocks forever and wedges the emulator
// mid-stream (this is how the design spike first failed), so drain reads the reply
// end and discards it: chartr never talks back to the agent through the grid.
type grid struct {
	mu   sync.Mutex
	emu  *vt.Emulator
	done chan struct{}
}

// newGrid builds a grid sized to cols×rows and starts the query-reply drain. A
// non-positive dimension falls back to the default so a caller that has not yet
// learned the PTY geometry still gets a usable emulator.
func newGrid(cols, rows int) *grid {
	if cols <= 0 {
		cols = gridDefaultCols
	}
	if rows <= 0 {
		rows = gridDefaultRows
	}
	emu := vt.NewEmulator(cols, rows)
	emu.SetScrollbackSize(gridScrollbackLines)
	g := &grid{emu: emu, done: make(chan struct{})}
	go g.drain()
	return g
}

// drain empties the emulator's reply pipe so a query the agent makes (an OSC 11;?
// background-colour probe) never blocks the write that answers it. It exits when
// the emulator is closed, which makes Read return an error.
func (g *grid) drain() {
	buf := make([]byte, 256)
	for {
		if _, err := g.emu.Read(buf); err != nil {
			close(g.done)
			return
		}
	}
}

// write feeds a chunk of raw PTY output into the emulator. It is called from the
// pump goroutine, alongside the scrollback append and the OSC scan.
func (g *grid) write(p []byte) {
	g.mu.Lock()
	_, _ = g.emu.Write(p)
	g.mu.Unlock()
}

// resize reshapes the grid to follow the PTY, so a region anchored to the bottom
// of the screen stays meaningful after the browser reflows.
func (g *grid) resize(cols, rows int) {
	if cols <= 0 || rows <= 0 {
		return
	}
	g.mu.Lock()
	g.emu.Resize(cols, rows)
	g.mu.Unlock()
}

// text returns the reconstructed viewport as plain text, trailing blank rows
// trimmed — the evidence the rule engine's screen regions slice.
func (g *grid) text() string {
	g.mu.Lock()
	defer g.mu.Unlock()
	return g.emu.String()
}

// close stops the drain goroutine by closing the emulator's reply pipe, which makes
// its blocked Read return EOF. It deliberately closes the pipe directly rather than
// calling Emulator.Close: Close flips an internal `closed` flag that Read also reads
// unsynchronized, so the two racing across goroutines trips the race detector.
// Closing the pipe (an io.Pipe, which *is* safe for concurrent Read/Close) unblocks
// the drain without touching that flag. Idempotent.
func (g *grid) close() {
	if c, ok := g.emu.InputPipe().(io.Closer); ok {
		_ = c.Close()
	}
}
