//go:build !windows

package terminal

import (
	"testing"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/model"
)

// A freshly opened shell sits at its prompt: sampling reports it idle under its
// own shell name. Running a foreground command flips it to working under that
// command's name, and the change is observable — which is the whole basis for
// the sidebar's live status indicator.
func TestSampleTracksForegroundCommand(t *testing.T) {
	m := NewManager(nil, 0) // nil onChange: no background sampler, we drive sample() by hand
	term, err := m.Open("s1", t.TempDir())
	if err != nil {
		t.Fatalf("open shell: %v", err)
	}
	t.Cleanup(func() { _ = m.Close(term.ID) })

	// At the prompt: idle, named for the shell itself.
	waitStatus(t, term, model.TerminalIdle)
	if info := m.ForSpace("s1"); len(info) != 1 || info[0].Status != model.TerminalIdle {
		t.Fatalf("fresh shell not reported idle: %+v", info)
	}
	if got := m.ForSpace("s1")[0].Proc; got != term.Title {
		t.Errorf("idle shell proc = %q, want the shell title %q", got, term.Title)
	}

	// Run a blocking foreground command; the shell goes working under its name.
	if _, err := term.Write([]byte("sleep 5\n")); err != nil {
		t.Fatalf("write to shell: %v", err)
	}
	waitStatus(t, term, model.TerminalWorking)
	if got := m.ForSpace("s1")[0].Proc; got != "sleep" {
		t.Errorf("working shell proc = %q, want %q", got, "sleep")
	}
}

// waitStatus samples until the terminal reaches want or the deadline passes,
// giving the shell time to reach its prompt / launch the command.
func waitStatus(t *testing.T, term *Terminal, want string) {
	t.Helper()
	deadline := time.Now().Add(3 * time.Second)
	for time.Now().Before(deadline) {
		term.sample(45 * time.Second)
		term.mu.Lock()
		got := term.state
		term.mu.Unlock()
		if got == want {
			return
		}
		time.Sleep(50 * time.Millisecond)
	}
	t.Fatalf("shell never reached status %q", want)
}
