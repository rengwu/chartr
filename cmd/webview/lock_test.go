package main

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
)

// The shell's window is best-effort tier, verified by building the tagged binary
// in CI (ADR 0011). What is testable here — and what the one-window invariant
// actually rests on — is the lock: who holds it, when it is stale, and that two
// data dirs never share one.

// holdLock makes the lock believe every recorded pid is live, which is what a
// running shell looks like from the outside.
func holdLock(t *testing.T, live bool) {
	t.Helper()
	prev := alive
	alive = func(int) bool { return live }
	t.Cleanup(func() { alive = prev })
}

func TestAcquireLockWritesLoopbackURL(t *testing.T) {
	holdLock(t, true)
	dir := t.TempDir()

	release, err := acquireLock(dir, "http://127.0.0.1:54321")
	if err != nil {
		t.Fatalf("acquireLock: %v", err)
	}

	info, ok, err := readLock(lockPath(dir))
	if err != nil || !ok {
		t.Fatalf("readLock: ok=%v err=%v", ok, err)
	}
	if info.URL != "http://127.0.0.1:54321" {
		t.Errorf("URL = %q, want the loopback address the server bound", info.URL)
	}
	if info.PID != os.Getpid() {
		t.Errorf("PID = %d, want this process %d", info.PID, os.Getpid())
	}

	release()
	if _, err := os.Stat(lockPath(dir)); !errors.Is(err, os.ErrNotExist) {
		t.Errorf("lock survived release: %v", err)
	}
}

func TestAcquireLockRefusesLiveInstance(t *testing.T) {
	holdLock(t, true)
	dir := t.TempDir()

	release, err := acquireLock(dir, "http://127.0.0.1:54321")
	if err != nil {
		t.Fatalf("first acquireLock: %v", err)
	}
	defer release()

	_, err = acquireLock(dir, "http://127.0.0.1:65000")
	var locked *errLocked
	if !errors.As(err, &locked) {
		t.Fatalf("second acquireLock err = %v, want *errLocked", err)
	}
	// The refusal must carry enough for the operator to reach the running
	// cockpit when raising its window is not possible.
	if locked.info.URL != "http://127.0.0.1:54321" {
		t.Errorf("refusal URL = %q, want the live instance's", locked.info.URL)
	}
	if locked.info.PID != os.Getpid() {
		t.Errorf("refusal PID = %d, want the live instance's %d", locked.info.PID, os.Getpid())
	}
}

func TestAcquireLockTakesOverStaleLock(t *testing.T) {
	dir := t.TempDir()

	holdLock(t, true)
	// The "crash": the holder acquires and never releases.
	if _, err := acquireLock(dir, "http://127.0.0.1:54321"); err != nil {
		t.Fatalf("first acquireLock: %v", err)
	}

	// The holder is now gone. A lock left behind by a dead process must not lock
	// the operator out of their own cockpit.
	holdLock(t, false)
	release2, err := acquireLock(dir, "http://127.0.0.1:65000")
	if err != nil {
		t.Fatalf("acquireLock over stale lock: %v", err)
	}
	defer release2()

	info, ok, err := readLock(lockPath(dir))
	if err != nil || !ok {
		t.Fatalf("readLock: ok=%v err=%v", ok, err)
	}
	if info.URL != "http://127.0.0.1:65000" {
		t.Errorf("URL = %q, want the new instance's", info.URL)
	}
}

func TestAcquireLockTakesOverCorruptLock(t *testing.T) {
	holdLock(t, true)
	dir := t.TempDir()

	path := lockPath(dir)
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, []byte("half a line, no pid"), 0o644); err != nil {
		t.Fatal(err)
	}

	release, err := acquireLock(dir, "http://127.0.0.1:54321")
	if err != nil {
		t.Fatalf("acquireLock over corrupt lock: %v", err)
	}
	defer release()

	if info, ok, _ := readLock(path); !ok || info.PID != os.Getpid() {
		t.Errorf("corrupt lock was not taken over: ok=%v info=%+v", ok, info)
	}
}

func TestLockIsKeyedToDataDir(t *testing.T) {
	holdLock(t, true)
	a, b := t.TempDir(), t.TempDir()

	releaseA, err := acquireLock(a, "http://127.0.0.1:1111")
	if err != nil {
		t.Fatalf("acquireLock(a): %v", err)
	}
	defer releaseA()

	// Distinct --data-dir roots are distinct instances by construction: the lock
	// lives under the data dir, so there is no global lock to contend for.
	releaseB, err := acquireLock(b, "http://127.0.0.1:2222")
	if err != nil {
		t.Fatalf("acquireLock(b) refused a distinct data dir: %v", err)
	}
	defer releaseB()
}

func TestReleaseLeavesAnotherInstancesLock(t *testing.T) {
	holdLock(t, false)
	dir := t.TempDir()

	release, err := acquireLock(dir, "http://127.0.0.1:1111")
	if err != nil {
		t.Fatalf("acquireLock: %v", err)
	}
	// Someone else took the (apparently stale) lock over while we held it.
	if _, err := acquireLock(dir, "http://127.0.0.1:2222"); err != nil {
		t.Fatalf("takeover: %v", err)
	}

	// Our release must not delete the winner's lock.
	release()
	info, ok, err := readLock(lockPath(dir))
	if err != nil {
		t.Fatal(err)
	}
	if !ok || info.URL != "http://127.0.0.1:2222" {
		t.Errorf("release clobbered the current holder: ok=%v info=%+v", ok, info)
	}
}

func TestLockPathIsUnderTheDataDir(t *testing.T) {
	if got, want := lockPath("/tmp/space"), filepath.Join("/tmp/space", ".wayfinder-harness", "shell.lock"); got != want {
		t.Errorf("lockPath = %q, want %q", got, want)
	}
	if got, want := lockPath(""), filepath.Join(".", ".wayfinder-harness", "shell.lock"); got != want {
		t.Errorf("lockPath(\"\") = %q, want %q", got, want)
	}
}
