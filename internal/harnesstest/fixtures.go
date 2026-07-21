package harnesstest

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"
)

// NewSpaceRepo creates a temporary git repository to stand in for a space, with
// a deterministic identity so committed history is assertable. It returns the
// repository root. Later tickets register this path as a space and drop maps
// into it.
func NewSpaceRepo(t testing.TB) string {
	t.Helper()
	dir := t.TempDir()
	Git(t, dir, "init", "-q", "-b", "main")
	Git(t, dir, "config", "user.email", "harness-test@example.com")
	Git(t, dir, "config", "user.name", "Harness Test")
	Git(t, dir, "config", "commit.gpgsign", "false")
	return dir
}

// Git runs a git command in dir and returns its trimmed combined output,
// failing the test on error.
func Git(t testing.TB, dir string, args ...string) string {
	t.Helper()
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("harnesstest: git %s: %v\n%s", strings.Join(args, " "), err, out)
	}
	return strings.TrimSpace(string(out))
}

// WriteMap writes a map body to .plan/<slug>/map.md under repo, creating
// directories as needed. It does not commit — a test that wants the map in
// history commits it explicitly, and one testing discovery-by-notice drops it
// while the harness is watching.
func WriteMap(t testing.TB, repo, slug, body string) string {
	t.Helper()
	return WriteFile(t, repo, filepath.Join(".plan", slug, "map.md"), body)
}

// WriteTicket writes a ticket file at .plan/<slug>/tickets/<filename> under
// repo. It does not commit — discovery reads the working tree, so a test drives
// derivation by dropping files exactly as a session or a `git pull` would.
func WriteTicket(t testing.TB, repo, slug, filename, body string) string {
	t.Helper()
	return WriteFile(t, repo, filepath.Join(".plan", slug, "tickets", filename), body)
}

// StubAgent installs a fake agent CLI named `name` on PATH for the rest of the
// test — the "stub agent CLI on PATH" the spawn tests drive against (spec, Testing
// Decisions). The stub is a real executable the harness launches in a PTY: it
// ignores its argv (the adapter's --model flag and any bound args) and appends
// every line it reads on stdin to a record file, then blocks reading more so the
// session stays live. The returned path is that record file, so a test asserts the
// opener arrived at the agent's stdin by reading it back.
//
// It prepends a fresh bin directory to PATH (so the stub shadows any real CLI of
// the same name) via t.Setenv, which forbids parallel tests — the spawn tests are
// sequential. It skips on Windows, where the shell-script stub would not run;
// the process-boundary spawn tests run on the unix CI paths.
func StubAgent(t testing.TB, name string) (recordPath string) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("stub agent CLI uses a POSIX shell script; not supported on Windows")
	}
	binDir := t.TempDir()
	recordPath = filepath.Join(t.TempDir(), name+"-stdin.log")

	// A line-buffered recorder: read a line, append it (reopening the file each
	// iteration flushes it to disk), loop. The read blocks on an open PTY, so the
	// stub stays alive as a live TUI would — exactly what the "lands on a live tab"
	// assertion needs.
	script := fmt.Sprintf("#!/bin/sh\nwhile IFS= read -r line; do printf '%%s\\n' \"$line\" >> %q; done\n", recordPath)
	stub := filepath.Join(binDir, name)
	if err := os.WriteFile(stub, []byte(script), 0o755); err != nil {
		t.Fatalf("harnesstest: writing stub agent %q: %v", name, err)
	}

	t.Setenv("PATH", binDir+string(os.PathListSeparator)+os.Getenv("PATH"))
	return recordPath
}

// StubDyingAgent installs a fake agent CLI that emits a unique marker to its PTY
// and then exits — the stub the death-halt tests drive against (ticket 10). Unlike
// StubAgent, which blocks so the session stays live, this one dies on cue, so a
// test can assert the harness detects the death, pins the dead session with its
// scrollback (the marker) intact, and takes no action of its own. It returns the
// marker so a test asserts scrollback survival by finding it after the death.
//
// Like StubAgent it prepends a fresh bin directory to PATH via t.Setenv (so the
// stub shadows any real CLI of the name and parallel tests are forbidden) and
// skips on Windows, where the POSIX-shell stub would not run.
func StubDyingAgent(t testing.TB, name string) (marker string) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("stub agent CLI uses a POSIX shell script; not supported on Windows")
	}
	binDir := t.TempDir()
	marker = "SESSION-OUTPUT-" + name

	// Print the marker to the PTY (so it lands in scrollback), then exit — the read
	// loop sees EOF and the harness detects the death.
	script := fmt.Sprintf("#!/bin/sh\nprintf '%%s\\n' %q\nexit 0\n", marker)
	stub := filepath.Join(binDir, name)
	if err := os.WriteFile(stub, []byte(script), 0o755); err != nil {
		t.Fatalf("harnesstest: writing dying stub agent %q: %v", name, err)
	}

	t.Setenv("PATH", binDir+string(os.PathListSeparator)+os.Getenv("PATH"))
	return marker
}

// StubProposingAgent installs a fake implement agent that walks a ticket to
// `proposed`: it appends a `## Proposed Answer` (with the given prose under it) to
// the ticket file, commits that one file under the operator's git identity, and
// exits — the "reads its payload, writes a `## Proposed Answer`, commits, then
// dies" stub the spec's Testing Decisions describe. The harness detects the death,
// pins the dead session, and derives `proposed` from the committed file, so a
// review can then seat on it (ticket 11). ticketRel is the ticket's path relative
// to the space root; the stub runs with the space as its working directory.
//
// Like the other stubs it prepends a fresh bin dir to PATH via t.Setenv (so it
// shadows any real CLI of the name and parallel tests are forbidden) and skips on
// Windows, where the POSIX-shell stub would not run.
func StubProposingAgent(t testing.TB, name, ticketRel, proposed string) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("stub agent CLI uses a POSIX shell script; not supported on Windows")
	}
	binDir := t.TempDir()

	// Append the proposed answer to the ticket and commit only that file, then exit.
	// The heredoc is quoted so the prose is written verbatim, never shell-expanded.
	script := fmt.Sprintf("#!/bin/sh\ncat >> %q <<'WFPROPOSED'\n\n## Proposed Answer\n\n%s\nWFPROPOSED\ngit add -- %q\ngit commit -q -m 'Propose answer' -- %q\nexit 0\n",
		ticketRel, proposed, ticketRel, ticketRel)
	stub := filepath.Join(binDir, name)
	if err := os.WriteFile(stub, []byte(script), 0o755); err != nil {
		t.Fatalf("harnesstest: writing proposing stub agent %q: %v", name, err)
	}

	t.Setenv("PATH", binDir+string(os.PathListSeparator)+os.Getenv("PATH"))
}

// WaitForFileContains polls path until it contains want or the deadline passes,
// returning its contents. It fails the test on timeout — a test asserting the
// opener reached the stub's stdin names the marker it expects rather than guessing
// how fast the PTY delivers the line.
func WaitForFileContains(t testing.TB, path, want string, within time.Duration) string {
	t.Helper()
	deadline := time.Now().Add(within)
	for {
		b, _ := os.ReadFile(path)
		if strings.Contains(string(b), want) {
			return string(b)
		}
		if time.Now().After(deadline) {
			t.Fatalf("harnesstest: %s never contained %q within %s; got %q", path, want, within, b)
		}
		time.Sleep(20 * time.Millisecond)
	}
}

// WriteFile writes body to relPath under repo, creating parent directories. It
// returns the absolute path written.
func WriteFile(t testing.TB, repo, relPath, body string) string {
	t.Helper()
	abs := filepath.Join(repo, relPath)
	if err := os.MkdirAll(filepath.Dir(abs), 0o755); err != nil {
		t.Fatalf("harnesstest: mkdir for %s: %v", relPath, err)
	}
	if err := os.WriteFile(abs, []byte(body), 0o644); err != nil {
		t.Fatalf("harnesstest: write %s: %v", relPath, err)
	}
	return abs
}
