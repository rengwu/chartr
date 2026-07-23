// Package env reconciles the environment chartr was launched with against the
// one its operator actually works in. The two are routinely not the same, and
// the gap is invisible until an agent that plainly exists cannot be found.
//
// The cockpit resolves every adapter binary against the *server process's* PATH:
// the registration probe calls exec.LookPath, and the spawn hands a bare name to
// go-pty, which stores it unresolved and lets exec.Command run the same lookup.
// Setting a child's Env does not redirect that lookup — Go resolves the binary
// out of the parent's PATH before the child's environment is ever consulted — so
// probe and spawn read one value and agree by construction. That agreement is
// worth keeping: it is what lets registration refuse a binary *before* the claim
// commit is written rather than stranding a claim on a launch that was never
// going to start (ADR 0008).
//
// What neither of them can do is see a PATH the process never inherited, and a
// desktop launch inherits almost nothing. A window opened from Finder or the
// Dock (ADR 0013) starts under launchd with `/usr/bin:/bin:/usr/sbin:/sbin` —
// no `/opt/homebrew/bin`, so Homebrew's own `claude` is unfindable on every
// Apple Silicon Mac; no `~/.local/bin`; nothing any version manager adds. Even
// from a terminal the inheritance is partial, because `~/.zshrc` and `~/.bashrc`
// are read by *interactive* shells only, and that is where operators are told to
// put an agent installer's PATH line.
//
// So chartr asks. Once, at startup, it runs the operator's login shell the way
// their terminal does and adopts the PATH that comes back. This is the fix VS
// Code and every other GUI-launched developer tool arrived at, for the same
// reason.
package env

import (
	"context"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"time"
)

// probeTimeout bounds the shell probe. An interactive shell runs the operator's
// full startup — plugin managers, version managers, prompt frameworks — and a
// heavy one takes real time, so the budget is generous. It exists to guarantee
// termination, not to police a slow `.zshrc`: a shell that blocks on a prompt or
// waits on a tty must not hold the cockpit's boot open, and the cost of giving up
// is only the PATH we already had.
const probeTimeout = 5 * time.Second

// disableVar lets an operator switch the probe off. Running a login shell at
// startup is a side effect on someone else's machine, and anyone with a reason to
// refuse it — a locked-down environment, a shell whose startup is not idempotent,
// a launcher that already exports the right PATH — is entitled to. Turning it off
// costs only the augmentation; chartr keeps the PATH it inherited.
const disableVar = "CHARTR_NO_PATH_PROBE"

// markers delimit the payload. A login shell is entitled to write to stdout — a
// MOTD, a version notice, a prompt framework's instant-prompt preamble — and none
// of that is ours to suppress. Bracketing the part we asked for lets the noise be
// discarded instead of parsed.
const (
	beginMarker = "__chartr_env_begin__"
	endMarker   = "__chartr_env_end__"
)

// HydratePATH augments this process's PATH with the operator's login-shell PATH,
// so that a binary they can run in their terminal is a binary chartr can find.
//
// It is deliberately additive. Entries already on the inherited PATH keep their
// position and their precedence, and the shell's contribute only what was missing;
// a PATH that already resolved an agent resolves the same agent afterwards. The
// probe can therefore only ever widen what is findable, which is what makes it
// safe to run unconditionally at startup — there is no configuration it can take
// away and no shadowing it can introduce.
//
// Every failure is silent and total: an unset SHELL, a shell that will not start,
// a timeout, output with no markers, an empty result. In each case the inherited
// PATH stands untouched and the operator meets the ordinary "not on your PATH"
// message, which stays true. A diagnostic here would be noise on the happy path
// and misdirection on the unhappy one — the operator's problem is a missing
// binary, not a shell probe.
//
// Callers run it once, before server.New, on the main goroutine: it mutates
// process-global state that every later lookup reads, so it must finish before
// anything can resolve a binary.
func HydratePATH() {
	// Windows keeps PATH in the registry and hands the same value to a process
	// launched from Explorer as to one launched from a console. There is no gap to
	// close, and no login shell whose startup files would close it.
	if runtime.GOOS == "windows" {
		return
	}
	if os.Getenv(disableVar) != "" {
		return
	}
	shell := os.Getenv("SHELL")
	if shell == "" {
		return
	}

	shellPATH, ok := loginPATH(shell)
	if !ok {
		return
	}
	if merged, changed := mergePATH(os.Getenv("PATH"), shellPATH); changed {
		_ = os.Setenv("PATH", merged)
	}
}

// loginPATH runs shell as a login *and* interactive shell and reads back the PATH
// it exports.
//
// Both flags are load-bearing, because the two startup files divide the work.
// `-l` reads the login files (`~/.zprofile`, `~/.profile`), and `-i` reads the
// interactive ones (`~/.zshrc`, `~/.bashrc`) — and the interactive half is the
// one that matters most here, since an agent installer appending `export PATH=…`
// almost always appends it to `.zshrc`. Asking for a login shell alone reproduces
// exactly the blind spot this function exists to remove.
//
// The payload is `env` rather than an expansion of `$PATH`, so the command is not
// written in any one shell's syntax. In fish a bare `$PATH` is a list and would
// come back space-separated; the *exported* variable `env` reads is colon-joined
// in every shell, because that is the format the OS defines. `printf` is a builtin
// in every shell that could plausibly be $SHELL, so the markers cost no process.
func loginPATH(shell string) (string, bool) {
	ctx, cancel := context.WithTimeout(context.Background(), probeTimeout)
	defer cancel()

	script := "printf '%s' " + beginMarker + "; env; printf '%s' " + endMarker

	cmd := exec.CommandContext(ctx, shell, "-l", "-i", "-c", script)
	// The probe reads a value; it must never inherit the terminal. Leaving stdin
	// closed keeps a shell that decides to prompt from blocking on a read that will
	// never be answered — the timeout would catch it, but five seconds of a stalled
	// boot is a worse outcome than an immediate EOF.
	cmd.Stdin = nil
	// Startup chatter belongs on stderr as often as stdout, and none of it is ours
	// to show. Only stdout is captured, and the markers filter that.
	cmd.Stderr = nil

	out, err := cmd.Output()
	// A non-zero exit is not disqualifying on its own. Interactive startup files
	// fail in small ways constantly — a missing plugin, a stale completion — and
	// the shell still assembled a correct PATH before failing. If the markers and
	// a PATH are present, the answer is good regardless of the status.
	if len(out) == 0 && err != nil {
		return "", false
	}
	return parsePATH(string(out))
}

// parsePATH pulls the exported PATH out of a shell probe's stdout. It reads only
// what the markers enclose, and within that only a line that is exactly a PATH
// assignment, so neither a startup banner nor another variable whose value happens
// to contain a newline can be mistaken for the answer.
func parsePATH(out string) (string, bool) {
	begin := strings.Index(out, beginMarker)
	if begin < 0 {
		return "", false
	}
	body := out[begin+len(beginMarker):]
	end := strings.Index(body, endMarker)
	if end < 0 {
		return "", false
	}

	for _, line := range strings.Split(body[:end], "\n") {
		value, found := strings.CutPrefix(strings.TrimSuffix(line, "\r"), "PATH=")
		if !found {
			continue
		}
		if value = strings.TrimSpace(value); value != "" {
			return value, true
		}
	}
	return "", false
}

// mergePATH appends the entries of shellPATH that current does not already carry,
// preserving current's order and therefore its precedence. It reports whether
// anything was added.
//
// Appending rather than prepending is the whole safety argument. A shell's PATH
// and an inherited one frequently hold the same directories in different orders —
// a version manager's shim ahead of a system binary, say — and prepending would
// silently change which of two installed copies runs. Appending cannot: a name
// that already resolved resolves to the same file, and only a name that resolved
// to nothing can newly resolve.
func mergePATH(current, shellPATH string) (string, bool) {
	entries := splitPATH(current)
	seen := make(map[string]bool, len(entries))
	for _, e := range entries {
		seen[e] = true
	}

	var added bool
	for _, e := range splitPATH(shellPATH) {
		if seen[e] {
			continue
		}
		seen[e] = true
		entries = append(entries, e)
		added = true
	}
	if !added {
		return current, false
	}
	return strings.Join(entries, string(os.PathListSeparator)), true
}

// splitPATH breaks a PATH into its non-empty entries. An empty entry is legal and
// means the current directory, which is a resolution rule chartr has no business
// propagating into a list of places to find an agent.
func splitPATH(path string) []string {
	var out []string
	for _, e := range strings.Split(path, string(os.PathListSeparator)) {
		if e != "" {
			out = append(out, e)
		}
	}
	return out
}
