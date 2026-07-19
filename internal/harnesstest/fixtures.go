package harnesstest

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
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
