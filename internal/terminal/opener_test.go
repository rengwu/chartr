package terminal

import (
	"fmt"
	"io"
	"os"
	"os/exec"
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
		t.Skip("the raw-mode stub requires POSIX stty")
	}
	shrinkOpenerTiming(t)

	log := filepath.Join(t.TempDir(), "keystrokes.log")
	agent, args, env := rawModeAgent(log)

	m := NewManager(nil, nil)
	defer m.Shutdown()

	const opener = "Read the file /tmp/payload.md in full"
	if _, err := m.OpenSession("space", t.TempDir(), "s1", agent, args, env, opener, Session{
		MapSlug: "m", TicketNum: 1, Role: "implement", Agent: "stub",
	}, false); err != nil {
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
		t.Skip("the raw-mode stub requires POSIX stty")
	}
	shrinkOpenerTiming(t)

	log := filepath.Join(t.TempDir(), "keystrokes.log")
	agent, args, env := rawModeAgent(log)

	m := NewManager(nil, nil)
	defer m.Shutdown()

	if _, err := m.OpenSession("space", t.TempDir(), "s1", agent, args, env, "", Session{
		MapSlug: "m", TicketNum: 1, Role: "implement", Agent: "stub",
	}, false); err != nil {
		t.Fatalf("opening the session: %v", err)
	}

	time.Sleep(500 * time.Millisecond) // well past the grace the shrunk timing allows
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Errorf("keystrokes went to an agent that was already told on its argv: %q", b)
	}
}

// shrinkOpenerTiming collapses the readiness waits so the stub tests do not sit
// out production-sized settle, grace, and submit intervals.
func shrinkOpenerTiming(t *testing.T) {
	t.Helper()
	settle, grace, submit := openerSettle, openerGrace, openerSubmit
	openerSettle, openerGrace, openerSubmit = 30*time.Millisecond, 150*time.Millisecond, 20*time.Millisecond
	t.Cleanup(func() { openerSettle, openerGrace, openerSubmit = settle, grace, submit })
}

const rawAgentLogEnv = "CHARTR_TEST_RAW_AGENT_LOG"

// rawModeAgent relaunches this test binary as a stub agent. The child enters raw
// mode and reads stdin itself, avoiding a shell between the stty call and the
// reader that could restore terminal settings. Its invisible title is emitted
// only after raw mode is active, giving awaitReady the same paint signal a real
// TUI provides.
func rawModeAgent(log string) (string, []string, []string) {
	return os.Args[0], []string{"-test.run=^TestRawModeAgentProcess$"}, []string{rawAgentLogEnv + "=" + log}
}

func TestRawModeAgentProcess(_ *testing.T) {
	log := os.Getenv(rawAgentLogEnv)
	if log == "" {
		return
	}

	cmd := exec.Command("stty", "raw", "-echo")
	cmd.Stdin, cmd.Stdout, cmd.Stderr = os.Stdin, os.Stdout, os.Stderr
	if err := cmd.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "setting raw mode: %v\n", err)
		os.Exit(2)
	}

	f, err := os.OpenFile(log, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o600)
	if err != nil {
		fmt.Fprintf(os.Stderr, "opening keystroke log: %v\n", err)
		os.Exit(2)
	}
	_, _ = fmt.Fprint(os.Stdout, "\x1b]0;ready\a")
	if _, err := io.Copy(f, os.Stdin); err != nil {
		fmt.Fprintf(os.Stderr, "recording keystrokes: %v\n", err)
		os.Exit(2)
	}
	_ = f.Close()
	os.Exit(0)
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
