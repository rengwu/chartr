//go:build !windows

package terminal

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// The reported bug, at the process boundary: an *ad-hoc shell* — not a session —
// running `claude` must read the agent's own broadcast state rather than being
// pinned to "working" for as long as the agent lives. The stub is a real
// executable named `claude` that chartr launches in a real PTY and that paints a
// real OSC title; everything under it (the foreground group, the identification,
// the read loop's scanner, the rule engine, the hysteresis) is the production
// path.
func TestAdHocShellRunningAnAgentReadsItsTitle(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("the stub agent is a POSIX shell script")
	}

	// A stub `claude` on PATH: paint the working glyph, hold it, then paint the
	// idle marker and block so the tab stays live — exactly the shape of a turn.
	// \342\240\202 is U+2802 (a braille frame); \342\234\263 is U+2733 (✳).
	bin := t.TempDir()
	script := "#!/bin/sh\n" +
		"printf '\\033]0;\\342\\240\\202 counting\\007'\n" +
		"sleep 1\n" +
		"printf '\\033]0;\\342\\234\\263 counting\\007'\n" +
		"cat\n"
	if err := os.WriteFile(filepath.Join(bin, "claude"), []byte(script), 0o755); err != nil {
		t.Fatalf("writing stub agent: %v", err)
	}
	t.Setenv("PATH", bin+string(os.PathListSeparator)+os.Getenv("PATH"))

	m := NewManager(nil) // nil onChange: no background sampler, we drive sample() by hand
	term, err := m.Open("s1", t.TempDir())
	if err != nil {
		t.Fatalf("open shell: %v", err)
	}
	t.Cleanup(func() { _ = m.Close(term.ID) })

	// At its own prompt the shell is just a shell — no agent, the old grammar.
	waitStatus(t, term, model.TerminalIdle)

	if _, err := term.Write([]byte("claude\n")); err != nil {
		t.Fatalf("write to shell: %v", err)
	}

	// The agent takes the foreground and the tab reads working.
	started := time.Now()
	waitStatus(t, term, model.TerminalWorking)

	term.mu.Lock()
	agent, proc := term.agent, term.proc
	term.mu.Unlock()
	if agent != "claude" {
		t.Fatalf("foreground agent = %q, want %q — identification never resolved the stub", agent, "claude")
	}
	if proc != "claude" {
		t.Errorf("tab proc = %q, want the agent %q", proc, "claude")
	}

	// Then the agent says it is done, and the tab reads idle — the bug, gone.
	waitStatus(t, term, model.TerminalIdle)

	term.mu.Lock()
	title := term.oscTitle
	term.mu.Unlock()
	if !strings.Contains(title, "✳") {
		t.Errorf("retained title = %q, want the ✳ the stub painted — the idle must come from the title", title)
	}
	// It read idle off a title the agent announced, not off the absence-confirming
	// path, which could not have fired this early anyway.
	if elapsed := time.Since(started); elapsed >= agentStartupGrace {
		t.Logf("note: idle took %s, at or past the %s startup grace", elapsed, agentStartupGrace)
	}
}

// The other half of the collapsed grammar: a tab whose foreground is *not* a known
// agent keeps reading exactly as it did before — idle at the prompt, working under
// the command's own name. TestSampleTracksForegroundCommand asserts the happy
// path; this asserts the boundary, that a plain command is never mistaken for an
// agent and never picks up the agent grammar.
func TestNonAgentCommandKeepsTheShellGrammar(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("foreground process groups are a unix affordance")
	}

	m := NewManager(nil)
	term, err := m.Open("s1", t.TempDir())
	if err != nil {
		t.Fatalf("open shell: %v", err)
	}
	t.Cleanup(func() { _ = m.Close(term.ID) })

	waitStatus(t, term, model.TerminalIdle)
	if _, err := term.Write([]byte("sleep 5\n")); err != nil {
		t.Fatalf("write to shell: %v", err)
	}
	waitStatus(t, term, model.TerminalWorking)

	term.mu.Lock()
	agent, proc := term.agent, term.proc
	term.mu.Unlock()
	if agent != "" {
		t.Errorf("`sleep` resolved to agent %q; a plain command must resolve to none", agent)
	}
	if proc != "sleep" {
		t.Errorf("tab proc = %q, want %q — the shell grammar names the command", proc, "sleep")
	}
}
