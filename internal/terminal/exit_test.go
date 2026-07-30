//go:build !windows

package terminal

import (
	"testing"
	"time"
)

// A process that ends must be noticed, and this is the only test that asserts it
// at the process boundary rather than through the server.
//
// The gap this fills shipped a release. Every path that reacts to a tab ending —
// dropping a closed shell, pinning a session that died, halting on a death —
// hangs off the pump goroutine seeing its PTY read fail. On macOS a master read
// returns EOF as soon as the child exits, so all of it worked. On Linux EIO
// arrives only when the last slave descriptor closes, and go-pty keeps one open
// in the parent, so the read blocked forever: the shell died, the tab stayed,
// and no death was ever detected. Nothing in this package covered the exit path,
// so the whole failure surfaced only as nine timing-out server tests that had
// been red for days.
//
// So this asserts the primitive directly: a process that exits on its own makes
// its terminal report dead, and the manager's onChange fires to publish it.
func TestExitedProcessIsReaped(t *testing.T) {
	reaped := make(chan struct{}, 4)
	m := NewManager(func() { reaped <- struct{}{} }, nil)
	defer m.Shutdown()

	// A shell is the honest subject here: it is what an ad-hoc tab actually runs,
	// and it exits by itself the moment its stdin closes.
	term, err := m.Open("space", t.TempDir())
	if err != nil {
		t.Fatalf("opening the shell: %v", err)
	}

	// `exit` rather than a kill, so this covers a process ending of its own accord
	// — the case a death-halt depends on, and the one no signal is there to force.
	if _, err := term.Write([]byte("exit\r")); err != nil {
		t.Fatalf("writing to the shell: %v", err)
	}

	deadline := time.After(10 * time.Second)
	for {
		if !term.info().Alive {
			break
		}
		select {
		case <-deadline:
			t.Fatal("the shell exited but its terminal never reported dead: " +
				"the pump goroutine is still blocked on a PTY read that will not return")
		case <-time.After(20 * time.Millisecond):
		}
	}

	select {
	case <-reaped:
	case <-time.After(5 * time.Second):
		t.Fatal("the terminal died without notifying the manager, so no fresh model is pushed and the tab lingers")
	}
}
