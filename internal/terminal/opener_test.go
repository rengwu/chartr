package terminal

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"
)

// The typed delivery must press *return*, not linefeed, and must press it apart
// from the line it submits. Both are what a real TUI needs: a `\n` is Ctrl+J,
// which agents that tell the two apart read as "insert a newline" and leave the
// opener sitting unsent in the composer; and a submit key riding the same chunk
// as its text looks like a paste, which a TUI that buffers pastes swallows.
//
// The stub puts its tty in raw mode before reading, so the line discipline's
// CR→NL translation cannot forge the byte under test.
func TestTypedOpenerSubmitsWithCarriageReturn(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("the stub agent is a POSIX shell script")
	}
	shrinkOpenerTiming(t)

	log := filepath.Join(t.TempDir(), "keystrokes.log")
	agent := rawModeAgent(t, log)

	m := NewManager(nil, time.Second)
	defer m.Shutdown()

	const opener = "Read the file /tmp/payload.md in full"
	if _, err := m.OpenSession("space", t.TempDir(), "s1", agent, nil, opener, Session{
		MapSlug: "m", TicketNum: 1, Role: "implement", Agent: "stub",
	}); err != nil {
		t.Fatalf("opening the session: %v", err)
	}

	got := waitForFile(t, log, opener+"\r", 5*time.Second)
	if strings.Contains(got, "\n") {
		t.Errorf("the opener was submitted with a linefeed, which leaves it unsent in a real TUI: %q", got)
	}
}

// The argv and flag deliveries carry the opener themselves, so an empty opener
// must type nothing at all — not a stray return into a TUI that was already told.
func TestEmptyOpenerTypesNothing(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("the stub agent is a POSIX shell script")
	}
	shrinkOpenerTiming(t)

	log := filepath.Join(t.TempDir(), "keystrokes.log")
	agent := rawModeAgent(t, log)

	m := NewManager(nil, time.Second)
	defer m.Shutdown()

	if _, err := m.OpenSession("space", t.TempDir(), "s1", agent, nil, "", Session{
		MapSlug: "m", TicketNum: 1, Role: "implement", Agent: "stub",
	}); err != nil {
		t.Fatalf("opening the session: %v", err)
	}

	time.Sleep(500 * time.Millisecond) // well past the grace the shrunk timing allows
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Errorf("keystrokes went to an agent that was already told on its argv: %q", b)
	}
}

// shrinkOpenerTiming collapses the readiness waits so a test that drives a stub
// drawing nothing does not sit out the production grace period.
func shrinkOpenerTiming(t *testing.T) {
	t.Helper()
	settle, grace, submit := openerSettle, openerGrace, openerSubmit
	openerSettle, openerGrace, openerSubmit = 30*time.Millisecond, 150*time.Millisecond, 20*time.Millisecond
	t.Cleanup(func() { openerSettle, openerGrace, openerSubmit = settle, grace, submit })
}

// rawModeAgent installs a stub agent that turns off the line discipline (as every
// real TUI does) and copies its stdin verbatim to a log, so the test reads the
// exact bytes chartr typed rather than the kernel's translation of them.
func rawModeAgent(t *testing.T, log string) string {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, "stub-agent")
	script := fmt.Sprintf("#!/bin/sh\nstty raw -echo 2>/dev/null\ncat >> %q\n", log)
	if err := os.WriteFile(path, []byte(script), 0o755); err != nil {
		t.Fatalf("writing the stub agent: %v", err)
	}
	return path
}

func waitForFile(t *testing.T, path, want string, within time.Duration) string {
	t.Helper()
	deadline := time.Now().Add(within)
	for {
		b, _ := os.ReadFile(path)
		if strings.Contains(string(b), want) {
			return string(b)
		}
		if time.Now().After(deadline) {
			t.Fatalf("%s never contained %q within %s; got %q", path, want, within, b)
		}
		time.Sleep(20 * time.Millisecond)
	}
}
