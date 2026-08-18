//go:build !windows

package terminal

import (
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// What a free session is made of (preload.go): the tab is the operator's shell,
// and the agent is a command typed into it. Every claim below is a claim about
// that one fact, asserted at the process boundary — a real PTY, a real shell, a
// real command that quits — because the fact is only true if the shell is really
// there behind the agent.

// preloadStub writes an agent that announces itself, waits for one line, and
// exits — a Ctrl+C in the only shape a test can press: the agent ends and the
// tab's own process does not. It returns the stub's name.
func preloadStub(t *testing.T, dir string) string {
	t.Helper()
	const name = "quitter"
	script := "#!/bin/sh\nprintf 'AGENT-UP\\n'\nread line\nexit 0\n"
	if err := os.WriteFile(filepath.Join(dir, name), []byte(script), 0o755); err != nil {
		t.Fatalf("writing the stub agent: %v", err)
	}
	t.Setenv("PATH", dir+string(os.PathListSeparator)+os.Getenv("PATH"))
	return name
}

// The one the operator asked for. An agent that ends — Ctrl+C, `/exit`, a crash —
// hands the tab back to the shell it was started from: the tab stays listed and
// live, its scrollback intact, reading as the ad-hoc shell it always was.
func TestFreeTabOutlivesItsAgent(t *testing.T) {
	useTestShell(t)
	shrinkOpenerTiming(t)

	agent := preloadStub(t, t.TempDir())
	m := NewManager(nil, nil) // nil onChange: no background sampler, this test samples
	t.Cleanup(m.Shutdown)

	term, err := m.OpenFree("s1", t.TempDir(), "f1", agent, nil, nil, "", agent)
	if err != nil {
		t.Fatalf("opening the free tab: %v", err)
	}
	if !term.awaitForeground(5 * time.Second) {
		t.Fatal("the preloaded agent never took the terminal")
	}
	// An adapter chartr ships no manifest for reads the shell grammar, which is the
	// bargain a preloaded tab makes: it is identified out of its foreground.
	waitStatus(t, term, model.TerminalRunning)

	// The agent quits, which is the whole subject: its stdin is the tab's PTY, so
	// one line ends it exactly as an operator's Ctrl+C would.
	if _, err := term.Write([]byte("q\r")); err != nil {
		t.Fatalf("writing to the agent: %v", err)
	}

	// Back at a prompt: idle, under the shell's own name rather than the tab's
	// label, which is the agent the operator picked and no longer what is running.
	waitStatus(t, term, model.TerminalIdle)
	info := term.info()
	if !info.Alive {
		t.Fatalf("the free tab died with its agent; the shell it was started from should still hold it")
	}
	if info.Proc != "sh" {
		t.Errorf("foreground process = %q after the agent quit, want the shell %q", info.Proc, "sh")
	}
	if info.PromptTarget {
		t.Error("the tab still reads as a live agent to send a preset into, with only a shell in front of it")
	}
	if info.Session != nil {
		t.Errorf("a free tab grew a session binding %+v", info.Session)
	}

	// Still one of the space's tabs, with the session it just ran still readable.
	if tabs := m.ForSpace("s1"); len(tabs) != 1 || tabs[0].ID != "f1" {
		t.Errorf("space tabs after the agent quit = %+v, want the free tab still listed", tabs)
	}
	term.mu.Lock()
	scrollback := string(term.scrollback)
	term.mu.Unlock()
	if !strings.Contains(scrollback, "AGENT-UP") {
		t.Errorf("the agent's output did not survive it:\n%s", scrollback)
	}
}

// The tab ends when its shell does, and only then — the one lifecycle an ad-hoc
// shell has ever had, now a free session's too.
func TestFreeTabDropsWhenItsShellExits(t *testing.T) {
	useTestShell(t)
	shrinkOpenerTiming(t)

	agent := preloadStub(t, t.TempDir())
	dropped := make(chan struct{}, 8)
	m := NewManager(func() { dropped <- struct{}{} }, nil)
	t.Cleanup(m.Shutdown)

	term, err := m.OpenFree("s1", t.TempDir(), "f1", agent, nil, nil, "", agent)
	if err != nil {
		t.Fatalf("opening the free tab: %v", err)
	}
	if !term.awaitForeground(5 * time.Second) {
		t.Fatal("the preloaded agent never took the terminal")
	}
	// Quit the agent, then the shell behind it.
	if _, err := term.Write([]byte("q\r")); err != nil {
		t.Fatalf("writing to the agent: %v", err)
	}
	waitStatus(t, term, model.TerminalIdle)
	if _, err := term.Write([]byte("exit\r")); err != nil {
		t.Fatalf("writing to the shell: %v", err)
	}

	deadline := time.After(10 * time.Second)
	for {
		if len(m.ForSpace("s1")) == 0 {
			return
		}
		select {
		case <-deadline:
			t.Fatal("the shell exited but its tab is still listed")
		case <-time.After(20 * time.Millisecond):
		}
	}
}

// The other end of outliving the agent: closing the tab must still take the agent
// with it. A shell between chartr and the agent means the kill no longer lands on
// the agent's own pid, so what ends it is the hangup every terminal emulator
// ends its children with — the PTY closing under a foreground group. Nothing is
// left running with no tab to show for it.
func TestClosingAFreeTabTakesTheAgentWithIt(t *testing.T) {
	useTestShell(t)
	shrinkOpenerTiming(t)

	dir := t.TempDir()
	pidFile := filepath.Join(t.TempDir(), "agent.pid")
	// An agent that records its own pid and then refuses to end on its own.
	script := "#!/bin/sh\necho $$ > " + pidFile + "\nwhile true; do sleep 1; done\n"
	if err := os.WriteFile(filepath.Join(dir, "sleeper"), []byte(script), 0o755); err != nil {
		t.Fatalf("writing the stub agent: %v", err)
	}
	t.Setenv("PATH", dir+string(os.PathListSeparator)+os.Getenv("PATH"))

	m := NewManager(nil, nil)
	t.Cleanup(m.Shutdown)

	term, err := m.OpenFree("s1", t.TempDir(), "f1", "sleeper", nil, nil, "", "sleeper")
	if err != nil {
		t.Fatalf("opening the free tab: %v", err)
	}
	if !term.awaitForeground(5 * time.Second) {
		t.Fatal("the preloaded agent never took the terminal")
	}
	pid := waitForPID(t, pidFile)

	if err := m.Close("f1"); err != nil {
		t.Fatalf("closing the tab: %v", err)
	}
	deadline := time.Now().Add(5 * time.Second)
	for {
		if err := syscall.Kill(pid, 0); err != nil {
			return // gone, which is the whole assertion
		}
		if time.Now().After(deadline) {
			_ = syscall.Kill(pid, syscall.SIGKILL)
			t.Fatalf("agent pid %d outlived the tab the operator closed", pid)
		}
		time.Sleep(50 * time.Millisecond)
	}
}

// waitForPID reads the pid a stub agent wrote for itself, waiting for the shell
// to have started it at all.
func waitForPID(t *testing.T, path string) int {
	t.Helper()
	deadline := time.Now().Add(5 * time.Second)
	for {
		if b, err := os.ReadFile(path); err == nil && len(b) > 0 {
			pid, err := strconv.Atoi(strings.TrimSpace(string(b)))
			if err == nil && pid > 0 {
				return pid
			}
		}
		if time.Now().After(deadline) {
			t.Fatalf("the stub agent never recorded its pid in %s", path)
		}
		time.Sleep(20 * time.Millisecond)
	}
}

// The command is typed at a shell, so what the shell reads back has to be exactly
// what chartr would have exec'd: an opener delivered on the argv is a whole
// English sentence, and a flag's value can carry anything a path can. This asserts
// the round trip through a real shell rather than the quoting's spelling.
func TestPreloadedArgumentsSurviveTheShell(t *testing.T) {
	dir := t.TempDir()
	script := "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done\n"
	if err := os.WriteFile(filepath.Join(dir, "echoargs"), []byte(script), 0o755); err != nil {
		t.Fatalf("writing the argv echo: %v", err)
	}
	t.Setenv("PATH", dir+string(os.PathListSeparator)+os.Getenv("PATH"))

	args := []string{
		"--model", "sonnet",
		"Read the file /tmp/a b/payload.md in full — it is your complete brief.",
		"it's got a quote", "$HOME `date` \"quoted\" *", "-test.run=^TestX$",
	}
	line := preloadCommand("echoargs", args)
	if !strings.HasPrefix(line, shellQuote(filepath.Join(dir, "echoargs"))) {
		t.Errorf("preloaded line does not start with the binary resolved off chartr's own PATH: %s", line)
	}

	out, err := exec.Command("/bin/sh", "-c", line).Output()
	if err != nil {
		t.Fatalf("running the preloaded line %q: %v", line, err)
	}
	got := strings.Split(strings.TrimSuffix(string(out), "\n"), "\n")
	if len(got) != len(args) {
		t.Fatalf("the shell read %d arguments from %q, want %d: %q", len(got), line, len(args), got)
	}
	for i := range args {
		if got[i] != args[i] {
			t.Errorf("argument %d came back as %q, want %q", i, got[i], args[i])
		}
	}
}
