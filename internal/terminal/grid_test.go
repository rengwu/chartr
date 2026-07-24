package terminal

import (
	"strings"
	"testing"
	"time"
)

// The emulator answers a terminal query — the background-colour probe claude opens
// a session with, OSC 11;? — by writing the reply into an internal pipe. With no
// reader that write blocks forever and wedges the whole emulator mid-stream, which
// is how the design spike first failed. The drain goroutine reads the reply end, so
// feeding a query must return promptly rather than deadlock. Tested, not assumed.
func TestGridDrainsTerminalQueryReplies(t *testing.T) {
	g := newGrid(80, 24)
	defer g.close()

	done := make(chan struct{})
	go func() {
		// OSC 11;? asks the background colour; the emulator replies into its pipe. A
		// second query and some payload after it prove the stream keeps flowing past
		// the reply rather than wedging on it.
		g.write([]byte("\x1b]11;?\x07"))
		g.write([]byte("\x1b]10;?\x07"))
		g.write([]byte("hello after the query\r\n"))
		close(done)
	}()

	select {
	case <-done:
	case <-time.After(3 * time.Second):
		t.Fatal("writing an OSC 11;? query wedged the emulator — the reply pipe was not drained")
	}

	if got := g.text(); !strings.Contains(got, "hello after the query") {
		t.Errorf("screen after the query = %q, want it to contain the text written past the query", got)
	}
}

// The grid reconstructs the visible screen from raw PTY bytes and follows the PTY on
// resize, so a region anchored to the bottom stays meaningful. Closing it stops the
// drain goroutine.
func TestGridReconstructsAndResizes(t *testing.T) {
	g := newGrid(80, 24)

	g.write([]byte("first line\r\nsecond line\r\n"))
	if got := g.text(); !strings.Contains(got, "first line") || !strings.Contains(got, "second line") {
		t.Errorf("reconstructed screen = %q, want both written lines", got)
	}

	g.resize(120, 40) // a browser reflow; must not panic or lose the content
	g.write([]byte("after resize\r\n"))
	if got := g.text(); !strings.Contains(got, "after resize") {
		t.Errorf("screen after resize = %q, want the line written post-resize", got)
	}

	g.close()
	select {
	case <-g.done:
	case <-time.After(2 * time.Second):
		t.Fatal("closing the grid did not stop the drain goroutine")
	}
}
