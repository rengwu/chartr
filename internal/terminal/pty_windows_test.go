//go:build windows

package terminal

import (
	"strings"
	"testing"
	"time"
)

// The one supported artifact ships for Windows too (ADR 0011), so the ConPTY
// path under COMSPEC is not a build-only claim — CI smoke-tests a real PTY
// round-trip on windows-latest (ticket 16). This is the Windows sibling of the
// Unix procstat test: it does not probe the foreground-group refinement (ConPTY
// has no such notion — see procstat_windows.go), only the primitive the whole
// terminal layer stands on: bytes typed up into a shell come back down through
// the same buffered fan-out a browser attaches to.
//
// It drives the public Manager surface an operator's shell tab uses — Open,
// Attach, Write — rather than reaching into ConPTY directly, so a regression in
// the go-pty ConPTY binding, the pump loop, or the broadcast fan-out all surface
// here.
func TestConPTYRoundTrip(t *testing.T) {
	m := NewManager(nil, 0) // nil onChange: no background sampler; we only need the round-trip.
	term, err := m.Open("s1", t.TempDir())
	if err != nil {
		t.Fatalf("open shell: %v", err)
	}
	t.Cleanup(func() { _ = m.Close(term.ID) })

	// Attach the way a browser socket does: replay buffer first, then live frames.
	att := term.Attach()
	t.Cleanup(att.Detach)

	// A token cmd.exe cannot produce except by echoing our line back — split so
	// the literal string never appears in a single write, only in the shell's
	// output, defeating a false positive on the input echo of the command itself.
	token := "WF" + "_CONPTY_" + "OK"
	if _, err := term.Write([]byte("echo " + token + "\r\n")); err != nil {
		t.Fatalf("write to shell: %v", err)
	}

	// The round-trip: read down-frames (plus whatever the prompt already replayed)
	// until the echoed token appears, or the deadline passes. cmd.exe's own prompt
	// and command echo arrive first; the token line is the proof the write reached
	// the process and its output flowed back through the PTY.
	seen := string(att.Scrollback)
	deadline := time.After(10 * time.Second)
	for !strings.Contains(seen, token) {
		select {
		case chunk := <-att.Frames:
			seen += string(chunk)
		case <-att.Done:
			t.Fatalf("terminal ended before the round-trip completed; output so far:\n%s", seen)
		case <-deadline:
			t.Fatalf("token %q never round-tripped through ConPTY; output so far:\n%s", token, seen)
		}
	}
}
