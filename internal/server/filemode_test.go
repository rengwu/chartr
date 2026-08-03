package server_test

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// websocket-origin-fix ticket 05 at the process boundary: what chartr writes about
// the operator is the operator's alone. A payload is the whole prompt a session was
// handed, and the registry is the absolute path of every repository they work in;
// on a shared machine — a build host, a dev box with more than one login — 0644
// under 0755 hands both to every account on it. This is a local exposure only, and
// the cheapest thing on this map to close.
//
// The other half of the ticket is what is deliberately *not* tightened: chartr
// writes into the operator's repository too, and those files are git's and the
// human's. TestTheRunMarkerStaysARepositoryFile pins that line, so a later blanket
// chmod cannot quietly cross it.

// modeUnderUmask is the mode a want-mode create actually lands with on this
// machine. The umask has the last word, and a machine running its tests under a
// tight one must not fail an assertion about chartr's own choice — the subject of
// the untightened files is that they are written *differently*, not the exact bits.
func modeUnderUmask(t *testing.T, want os.FileMode) os.FileMode {
	t.Helper()
	probe := filepath.Join(t.TempDir(), "probe")
	if err := os.WriteFile(probe, nil, want); err != nil {
		t.Fatalf("probing the umask: %v", err)
	}
	info, err := os.Stat(probe)
	if err != nil {
		t.Fatalf("probing the umask: %v", err)
	}
	return info.Mode().Perm()
}

// assertMode reports a mismatch in octal both ways round, the only readable way to
// put a permissions failure.
func assertMode(t *testing.T, path string, want os.FileMode) {
	t.Helper()
	info, err := os.Stat(path)
	if err != nil {
		t.Fatalf("stat %s: %v", path, err)
	}
	if got := info.Mode().Perm(); got != want {
		t.Errorf("%s is %#o, want %#o", path, got, want)
	}
}

// skipOnWindows guards the POSIX permission bits, which mean nothing there.
func skipOnWindows(t *testing.T) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("POSIX file modes")
	}
}

// spawnOneSession is the whole normal path — a registered space, a frontier
// ticket, a stub agent — run for its filesystem effects alone. It returns the
// space's repository and the session id.
func spawnOneSession(t *testing.T, h *chartrtest.Chartr) (repo, sessionID string) {
	t.Helper()
	repo = chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	return repo, mustSpawn(t, h, resp.ID, "widget", 1, "implement").SessionID
}

// A spawned session's payload — both the copy in the space and the archive chartr
// keeps outside it — is readable by its owner and by nobody else, and so are the
// directories holding them.
func TestASessionPayloadIsOwnerOnly(t *testing.T) {
	skipOnWindows(t)

	h := chartrtest.Start(t)
	repo, sid := spawnOneSession(t, h)

	// The gitignored copy the opener points the agent at.
	sessionDir := filepath.Join(repo, ".chartr", "run", sid)
	assertMode(t, filepath.Join(sessionDir, "payload.md"), 0o600)
	assertMode(t, sessionDir, 0o700)

	// The archive under chartr's data root, which outlives the copy above.
	archiveDir := filepath.Join(h.DataDir, "sessions", sid)
	assertMode(t, filepath.Join(archiveDir, "payload.md"), 0o600)
	assertMode(t, archiveDir, 0o700)
	// And the directory gathering every session's archive: 0700 there is what stops
	// another login listing which sessions ever ran.
	assertMode(t, filepath.Join(h.DataDir, "sessions"), 0o700)
}

// The run directory and its `*` marker are the repository's, not chartr's: git
// reads the marker, and a checkout shared with another login has to keep working.
// They stay ordinary repository files while the payload inside them does not.
func TestTheRunMarkerStaysARepositoryFile(t *testing.T) {
	skipOnWindows(t)

	h := chartrtest.Start(t)
	repo, _ := spawnOneSession(t, h)

	runDir := filepath.Join(repo, ".chartr", "run")
	assertMode(t, filepath.Join(runDir, ".gitignore"), modeUnderUmask(t, 0o644))
	assertMode(t, runDir, modeUnderUmask(t, 0o755))
}

// The registry records the absolute path of every repository the operator works
// in. It is written through a temp file and a rename, so the mode has to be right
// on the temp file for it to be right on the file that lands.
func TestTheRegistryIsOwnerOnly(t *testing.T) {
	skipOnWindows(t)

	h := chartrtest.Start(t)
	register(t, h, chartrtest.NewSpaceRepo(t))

	assertMode(t, filepath.Join(h.ConfigDir, "spaces.toml"), 0o600)
}

// The agent library is chartr's own config, and its `env` is the one place in it
// where an operator may reasonably keep a secret.
func TestTheAgentLibraryIsOwnerOnly(t *testing.T) {
	skipOnWindows(t)

	h := chartrtest.Start(t)
	registerAgent(t, h, "claude", map[string]any{
		"adapter": "claude",
		"env":     []string{"ANTHROPIC_AUTH_TOKEN=s3cret"},
	})

	assertMode(t, filepath.Join(h.ConfigDir, "user.toml"), 0o600)
}

// The config root holds both of the files above, and several writers can be the
// one to create it — the registry save, the skill library materializing, an
// operator's own editor. It is created owner-only at startup so its mode is not
// whichever of them happened to run first.
func TestTheConfigRootIsOwnerOnly(t *testing.T) {
	skipOnWindows(t)

	root := filepath.Join(t.TempDir(), "chartr")
	chartrtest.Start(t, chartrtest.WithConfigDir(root))

	assertMode(t, root, 0o700)
}

// os.WriteFile applies its mode only when it creates the file, so an install
// carrying a 0644 registry from an older build would keep it forever if the write
// were the only guard. The rename an atomic write ends with lands the temp file's
// mode on the destination, which is what repairs it on the next save.
func TestAnOlderWorldReadableRegistryIsRepairedOnSave(t *testing.T) {
	skipOnWindows(t)

	configDir := t.TempDir()
	stale := filepath.Join(configDir, "spaces.toml")
	if err := os.WriteFile(stale, []byte("\n"), 0o644); err != nil {
		t.Fatalf("seeding a stale registry: %v", err)
	}
	if err := os.Chmod(stale, 0o644); err != nil {
		t.Fatalf("seeding a stale registry: %v", err)
	}

	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	register(t, h, chartrtest.NewSpaceRepo(t))

	assertMode(t, stale, 0o600)
}
