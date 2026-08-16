// Package env reconciles the environment chartr was launched with against
// the one its operator actually works in. The gap is invisible until an
// agent that plainly exists cannot be found.
//
// The cockpit resolves every adapter binary against the server process's
// PATH: the registration probe calls exec.LookPath, and the spawn hands a
// bare name to go-pty, which lets exec.Command run the same lookup. Setting
// a child's Env doesn't redirect that — Go resolves the binary out of the
// parent's PATH first — so probe and spawn agree by construction. That's
// what lets registration refuse a binary before the claim is
// written, instead of stranding a claim on a launch that was never going to
// start (ADR 0008).
//
// What neither can see is a PATH the process never inherited, and a desktop
// launch inherits almost nothing. A window opened from Finder or the Dock
// (ADR 0013) starts under launchd with `/usr/bin:/bin:/usr/sbin:/sbin` — no
// `/opt/homebrew/bin` (Homebrew's `claude` unfindable on Apple Silicon), no
// `~/.local/bin`, nothing a version manager adds. Even from a terminal the
// inheritance is partial: `~/.zshrc`/`~/.bashrc` are read by interactive
// shells only, which is where installers are told to put their PATH line.
//
// So chartr asks. Once, at startup, it runs the operator's login shell the
// way their terminal does and adopts the PATH that comes back — the same
// fix VS Code and every other GUI-launched dev tool arrived at.
package env

import (
	"context"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"time"
)

// probeTimeout bounds the shell probe. An interactive shell runs the
// operator's full startup — plugin managers, version managers, prompt
// frameworks — so the budget is generous. It exists to guarantee
// termination, not police a slow `.zshrc`: a shell that blocks on a prompt
// must not hold the cockpit's boot open, and giving up only costs the PATH
// we already had.
const probeTimeout = 5 * time.Second

// disableVar lets an operator switch the probe off — a locked-down
// environment, a non-idempotent shell startup, a launcher that already
// exports the right PATH. Turning it off costs only the augmentation;
// chartr keeps the PATH it inherited.
const disableVar = "CHARTR_NO_PATH_PROBE"

// markers delimit the payload. A login shell is entitled to write to
// stdout — a MOTD, a version notice, a prompt framework's preamble — and
// none of that is ours to suppress. Bracketing the part we asked for lets
// the noise be discarded instead of parsed.
const (
	beginMarker = "__chartr_env_begin__"
	endMarker   = "__chartr_env_end__"
)

// HydratePATH augments this process's PATH with the operator's login-shell
// PATH, so a binary they can run in their terminal is a binary chartr can
// find.
//
// It's deliberately additive: entries already on the inherited PATH keep
// their position and precedence, the shell only contributes what was
// missing. The probe can therefore only widen what's findable, which is
// what makes it safe to run unconditionally at startup.
//
// Every failure is silent and total: unset SHELL, a shell that won't start,
// a timeout, no markers, an empty result. The inherited PATH stands
// untouched and the operator meets the ordinary "not on your PATH"
// message — a diagnostic here would be noise on the happy path and
// misdirection on the unhappy one.
//
// Callers run it once, before server.New, on the main goroutine: it
// mutates process-global state every later lookup reads.
func HydratePATH() {
	// Windows keeps PATH in the registry and hands the same value to every
	// launcher — no gap to close.
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

// loginPATH runs shell as a login *and* interactive shell and reads back the
// PATH it exports.
//
// Both flags are load-bearing. `-l` reads the login files (`~/.zprofile`,
// `~/.profile`); `-i` reads the interactive ones (`~/.zshrc`, `~/.bashrc`) —
// the half that matters most, since an installer's `export PATH=…` almost
// always lands in `.zshrc`. Login alone reproduces the exact blind spot
// this function exists to remove.
//
// The payload is `env`, not a shell-specific `$PATH` expansion: in fish a
// bare `$PATH` is a list and comes back space-separated, while the
// exported variable `env` reads is always colon-joined. `printf` is a
// builtin everywhere $SHELL could plausibly be, so the markers cost no
// extra process.
func loginPATH(shell string) (string, bool) {
	ctx, cancel := context.WithTimeout(context.Background(), probeTimeout)
	defer cancel()

	script := "printf '%s' " + beginMarker + "; env; printf '%s' " + endMarker

	cmd := exec.CommandContext(ctx, shell, "-l", "-i", "-c", script)
	// The probe stands in for the operator's own terminal, so it runs under
	// the restored host environment: under the AppImage an interactive
	// startup file may itself spawn tools the bundle's loader paths would
	// break.
	cmd.Env = HostEnviron()
	// Never inherit the terminal: a shell that decides to prompt must not
	// block on a read nobody will answer. The timeout would catch it, but
	// an immediate EOF beats five seconds of a stalled boot.
	cmd.Stdin = nil
	// Startup chatter belongs on stderr as often as stdout, none of it
	// ours to show. Only stdout is captured; markers filter that.
	cmd.Stderr = nil

	out, err := cmd.Output()
	// A non-zero exit isn't disqualifying: interactive startup files fail
	// in small ways constantly (a missing plugin, stale completion) while
	// still assembling a correct PATH. Markers + a PATH is a good answer
	// regardless of exit status.
	if len(out) == 0 && err != nil {
		return "", false
	}
	return parsePATH(string(out))
}

// parsePATH pulls the exported PATH out of a shell probe's stdout. It reads
// only what the markers enclose, and within that only a line that's exactly
// a PATH assignment, so a startup banner or another multi-line variable
// can't be mistaken for the answer.
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

// mergePATH appends shellPATH entries current doesn't already carry,
// preserving current's order and precedence. Reports whether anything was
// added.
//
// Appending, not prepending, is the safety argument. A shell's PATH and an
// inherited one often hold the same directories in different orders (a
// version manager's shim ahead of a system binary); prepending would
// silently change which installed copy runs. Appending can't — a name that
// already resolved keeps resolving to the same file.
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

// splitPATH breaks a PATH into its non-empty entries. An empty entry
// legally means the current directory — a resolution rule chartr has no
// business propagating into a list of places to find an agent.
func splitPATH(path string) []string {
	var out []string
	for _, e := range strings.Split(path, string(os.PathListSeparator)) {
		if e != "" {
			out = append(out, e)
		}
	}
	return out
}
