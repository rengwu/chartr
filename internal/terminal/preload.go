package terminal

import (
	"os/exec"
	"runtime"
	"strings"
	"time"
)

// A free session is an ad-hoc shell with one command preloaded. The tab runs the
// operator's own shell, and chartr types the agent's command line into it the way
// the operator would have — so the agent is a *child* of the tab rather than the
// tab itself.
//
// That is the whole of it, and everything the free session's behaviour is made of
// follows from it:
//
//   - **Ctrl+C leaves a shell, not a corpse.** Quitting the agent returns the tab
//     to its prompt with the whole session still in scrollback, and the command
//     one line up in the shell's history to run again. Nothing has to notice the
//     death and nothing has to be respawned: the tab's process never ended, so
//     there is no tab to drop (Manager.onExit). This is what the operator asked
//     for when they hit Ctrl+C in a real terminal, and it is what they get.
//   - **It reverts to exactly what it was launched as.** With no agent in front
//     of it the tab is an ad-hoc shell in every way the model can see: the shell
//     grammar's idle, no agent seated, no preset target. Nothing had to be
//     un-set, because a free tab never carried a launched agent to begin with —
//     it is identified out of its PTY's foreground like any other shell, which is
//     also how the agent is recognised again if the operator runs it a second
//     time.
//   - **A tab ends only when its shell does**, on the operator's `exit` or their
//     close, which is the one lifecycle an ad-hoc shell has ever had.
//
// The cost is the cost of every ad-hoc shell: identification is the sampler's
// foreground read, so an adapter chartr ships no manifest for reads the shell
// grammar rather than the agent's own, and a platform with no foreground-group
// notion (Windows, procstat_windows.go) reads a free tab as a plain live shell.
// A session is unchanged — it *is* its agent's process, because a session ends
// when its agent does and has a death to halt on.

// The preload's readiness thresholds. A shell prompt is one write, not a TUI
// painting itself, so it settles far faster than an agent does — and the command
// wants to be typed before the sampler's first pass, so the tab is never seen
// idling at a prompt it is about to leave. preloadGrace caps both of the waits
// that can go unanswered: a shell that prints no prompt at all, and an agent that
// never takes the terminal. It is generous because neither wait costs anything
// until it is the last one. Vars, not consts, so a test can shrink them.
var (
	preloadSettle = 150 * time.Millisecond
	preloadGrace  = 5 * time.Second
)

// preloadCommand renders a launch as the line the operator would have typed: the
// binary, then its arguments, each quoted for the shell that will read them.
//
// The binary is resolved to an absolute path first, out of chartr's own PATH —
// the same lookup the exec would have done, and the reason a free session runs
// the binary the agent's registration probe found rather than whatever the
// operator's shell rc later put in front of it. An unresolvable name is left as
// written, so the shell reports it in the tab the way it reports any other
// command it cannot find.
func preloadCommand(name string, args []string) string {
	if name == "" {
		return ""
	}
	if path, err := exec.LookPath(name); err == nil {
		name = path
	}
	parts := make([]string, 0, len(args)+1)
	parts = append(parts, shellQuote(name))
	for _, a := range args {
		parts = append(parts, shellQuote(a))
	}
	return strings.Join(parts, " ")
}

// shellQuote renders one argv token so a shell reads it back as the single word
// it was. It is not cosmetic: an opener delivered on the argv is a whole English
// sentence, and a model flag or a payload path can carry anything a path can.
//
// A POSIX shell gets single quotes, which have no escapes at all inside them —
// the one quoting in the language that cannot be got wrong. An embedded single
// quote is the only case they cannot hold, and it is spliced in the way it always
// is: close the quoting, escape the quote, open it again. Windows runs COMSPEC
// (ADR 0006 as amended), where double quotes are the only grouping there is and
// an embedded one doubles.
//
// A token a shell would read back whole on its own is left alone. That much *is*
// cosmetic, and it earns its place: the line is echoed into a tab the operator is
// looking at, and `claude --model sonnet` is the command they would have typed
// where a fully quoted one is a machine's transcript of it.
func shellQuote(s string) string {
	if !needsQuoting(s) {
		return s
	}
	if runtime.GOOS == "windows" {
		return `"` + strings.ReplaceAll(s, `"`, `""`) + `"`
	}
	return `'` + strings.ReplaceAll(s, `'`, `'\''`) + `'`
}

// needsQuoting reports whether a shell would read s as anything other than the
// one literal word it is. The safe set is the conservative one: letters, digits,
// and the punctuation that carries no meaning to any shell in any position — so
// a flag, a version, and an absolute path pass, and a space, a glob, a quote, an
// expansion or an empty token does not.
func needsQuoting(s string) bool {
	if s == "" {
		return true
	}
	for _, r := range s {
		switch {
		case r >= 'a' && r <= 'z', r >= 'A' && r <= 'Z', r >= '0' && r <= '9':
		case strings.ContainsRune("_@%+=:,./-", r):
		default:
			return true
		}
	}
	return false
}

// startPreload types the tab's preloaded command into its shell once the shell is
// at its prompt, and then hands opener to the agent that command started.
//
// It runs off the caller's goroutine so a launch's HTTP response does not wait on
// a shell drawing its prompt, and it is one goroutine rather than two because the
// order is the point: the command first, the opener only into what it started.
// The wait between them is the foreground itself — the agent is not ready to be
// told anything until it holds the terminal, and typing a brief at a shell prompt
// would run it as a command. A write that fails, or a tab that ends, only means
// there is nobody left to type to; the read loop is reaping it in parallel.
func startPreload(t *Terminal, opener string) {
	if t.preload == "" {
		return
	}
	go func() {
		if !t.awaitReady(preloadSettle, preloadGrace) {
			return // the shell died before it could be told anything
		}
		t.submitPrompt(t.preload)

		if opener == "" {
			return // an argv or flag delivery: the command line already carried it
		}
		if !t.awaitForeground(preloadGrace) {
			return // it never took the terminal; the brief is not the shell's to run
		}
		if !t.awaitReady(openerSettle, openerGrace) {
			return
		}
		t.submitPrompt(opener)
	}()
}

// awaitForeground blocks until something other than the tab's own shell holds the
// PTY's foreground — the preloaded agent taking the terminal, which is the only
// honest sign that a keystroke will reach the agent rather than the prompt it was
// started from.
//
// It is false for a tab whose process is gone, and false for a shell still
// holding its own foreground when the grace runs out: a command that never
// started is a shell sitting at a prompt, and a brief typed there would be run as
// a command rather than read as a brief. Where the platform has no
// foreground-group notion to read (Windows, procstat_windows.go) there is nothing
// to wait for and nothing to refuse, so it is true at once and the opener falls
// back to the paint signal a session's opener has always waited on.
func (t *Terminal) awaitForeground(grace time.Duration) bool {
	const poll = 25 * time.Millisecond
	deadline := time.Now().Add(grace)
	for {
		t.mu.Lock()
		alive, shell := t.alive, t.shellPID
		t.mu.Unlock()
		if !alive {
			return false
		}
		if pgrp := foreground(t.pty); pgrp <= 0 || pgrp != shell {
			return true
		}
		if time.Now().After(deadline) {
			return false
		}
		time.Sleep(poll)
	}
}
