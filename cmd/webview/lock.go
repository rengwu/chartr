// Command webview is the best-effort native shell for the cockpit (ADR 0011,
// ADR 0013): the same in-process server the supported `chartr` binary runs,
// wrapped in a real OS window instead of a browser tab.
//
// The package is split by build tag. `main_webview.go` (//go:build webview) is
// the real cgo shell; `main_stub.go` (//go:build !webview) is a tiny main that
// refuses and points at `make webview`. This file carries the single-instance
// lock, which is deliberately tag-free: it is pure Go, it compiles and tests at
// CGO_ENABLED=0, and it is the one piece of shell behaviour a unit test can
// reach without a real window.
package main

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// lockName is the single-instance lock, relative to the data dir. Keying the
// lock to the data dir is what makes distinct --data-dir roots distinct
// instances by construction (spec story 55): there is no global lock to share.
const lockName = ".chartr/shell.lock"

// lockInfo is what a live shell records about itself: enough for a second
// launch to either raise that window or tell the operator exactly where it is.
type lockInfo struct {
	// PID is the live shell's process id. It is also the liveness test — a lock
	// whose process is gone is stale, which is how the invariant survives a
	// crash or a ⌘Q that never runs a deferred cleanup.
	PID int
	// URL is the loopback address the live shell's server is bound to. The
	// refuse-with-message path prints it so the operator can reach the running
	// cockpit in a browser even when raising the window is not possible.
	URL string
}

// lockPath is where the lock for dataDir lives.
func lockPath(dataDir string) string {
	if dataDir == "" {
		dataDir = "."
	}
	return filepath.Join(dataDir, filepath.FromSlash(lockName))
}

// errLocked reports a live shell already holding the lock for this data dir.
type errLocked struct{ info lockInfo }

func (e *errLocked) Error() string {
	return fmt.Sprintf("a shell is already running at %s (pid %d)", e.info.URL, e.info.PID)
}

// alive reports whether pid is a live process. It is a variable over the
// platform probe so the lock tests can hold a lock "live" without spawning
// anything.
var alive = processAlive

// acquireLock claims the single-instance lock for dataDir, recording this
// process and the loopback URL its server is bound to. It returns a release
// func the caller defers.
//
// A lock held by a live process is refused with *errLocked carrying that
// instance's info — the caller decides whether to raise the window or print the
// URL. A lock left behind by a dead process is stale and is taken over: an
// abrupt exit must not lock the operator out of their own cockpit forever.
func acquireLock(dataDir, url string) (release func(), err error) {
	path := lockPath(dataDir)
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return nil, err
	}

	me := lockInfo{PID: os.Getpid(), URL: url}
	// O_EXCL is the claim: two shells launched at once cannot both believe they
	// won. The loser falls through to the liveness check below, where the
	// winner's live pid refuses it.
	f, err := os.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o644)
	if errors.Is(err, os.ErrExist) {
		info, ok, readErr := readLock(path)
		if readErr != nil {
			return nil, readErr
		}
		if ok && alive(info.PID) {
			return nil, &errLocked{info: info}
		}
		// Stale (dead pid) or unparseable: take it over. An abrupt exit must not
		// lock the operator out of their own cockpit forever.
		if err := os.WriteFile(path, []byte(formatLock(me)), 0o644); err != nil {
			return nil, err
		}
		return releaseFunc(path, me), nil
	}
	if err != nil {
		return nil, err
	}
	if _, err := f.WriteString(formatLock(me)); err != nil {
		f.Close()
		os.Remove(path)
		return nil, err
	}
	if err := f.Close(); err != nil {
		os.Remove(path)
		return nil, err
	}
	return releaseFunc(path, me), nil
}

// releaseFunc drops a lock we hold.
func releaseFunc(path string, me lockInfo) func() {
	return func() {
		// Only drop the lock if it is still ours: a stale-takeover race would
		// otherwise have the loser delete the winner's lock. The whole record is
		// the identity — pid alone is not enough, since a takeover can come from
		// the same pid on a different port.
		if info, ok, err := readLock(path); err == nil && ok && info != me {
			return
		}
		os.Remove(path)
	}
}

// readLock reads the lock at path. A missing or unparseable lock reports ok
// false rather than an error — a corrupt lock is indistinguishable from no lock
// for our purposes, and refusing to start over a garbled file would be worse
// than taking it over.
func readLock(path string) (info lockInfo, ok bool, err error) {
	b, err := os.ReadFile(path)
	if errors.Is(err, os.ErrNotExist) {
		return lockInfo{}, false, nil
	}
	if err != nil {
		return lockInfo{}, false, err
	}
	info, ok = parseLock(string(b))
	return info, ok, nil
}

// formatLock renders a lock file: two `key = value` lines, deliberately not a
// TOML dependency for a file with two fields.
func formatLock(info lockInfo) string {
	return fmt.Sprintf("pid = %d\nurl = %q\n", info.PID, info.URL)
}

func parseLock(s string) (lockInfo, bool) {
	var info lockInfo
	for _, line := range strings.Split(s, "\n") {
		key, val, found := strings.Cut(line, "=")
		if !found {
			continue
		}
		val = strings.TrimSpace(val)
		switch strings.TrimSpace(key) {
		case "pid":
			n, err := strconv.Atoi(val)
			if err != nil {
				return lockInfo{}, false
			}
			info.PID = n
		case "url":
			if unquoted, err := strconv.Unquote(val); err == nil {
				val = unquoted
			}
			info.URL = val
		}
	}
	if info.PID <= 0 || info.URL == "" {
		return lockInfo{}, false
	}
	return info, true
}
