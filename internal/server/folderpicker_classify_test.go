package server

// The chooser's failure modes, tested at the one place they are told apart.
// This file is `package server` rather than the external folderpicker_test.go
// beside it because the seam worth pinning is internal: classifyPickError is
// what decides whether the cockpit says "no folder chosen" or "the chooser
// broke, here is what it said", and getting that wrong is silent — the operator
// sees a plausible answer either way.
//
// The ExitErrors are real ones, produced by running `sh` with the exit status
// and stderr under test, because the field that matters (ExitError.Stderr) is
// only populated by the same cmd.Output path production uses. Constructing one
// by hand would test a struct literal rather than the contract.

import (
	"errors"
	"fmt"
	"os/exec"
	"strings"
	"testing"
)

// exitErrorFrom runs a shell that prints stderr and exits with code, and returns
// the *exec.ExitError that cmd.Output produced.
func exitErrorFrom(t *testing.T, code int, stderr string) error {
	t.Helper()
	script := fmt.Sprintf("printf %%s %q >&2; exit %d", stderr, code)
	_, err := exec.Command("sh", "-c", script).Output()
	if err == nil {
		t.Fatalf("sh -c %q unexpectedly succeeded", script)
	}
	var exitErr *exec.ExitError
	if !errors.As(err, &exitErr) {
		t.Fatalf("wanted an *exec.ExitError from sh, got %T: %v", err, err)
	}
	return err
}

// Dismissal is the outcome the picker exists to handle gracefully, and it must
// stay an outcome: a 200 saying no folder was chosen, not an error in red.
func TestClassifyPickErrorDismissal(t *testing.T) {
	got := classifyPickError(exitErrorFrom(t, dismissCode, ""), false)
	if !errors.Is(got, errPickCancelled) {
		t.Fatalf("exit %d must read as a cancellation, got %v", dismissCode, got)
	}
}

// A deadline is nobody answering the dialog. The kill that ends it is ours, so
// whatever the child's exit looked like, this is a cancellation.
func TestClassifyPickErrorTimeoutIsCancellation(t *testing.T) {
	got := classifyPickError(exitErrorFrom(t, 127, "irrelevant"), true)
	if !errors.Is(got, errPickCancelled) {
		t.Fatalf("a timed-out chooser must read as a cancellation, got %v", got)
	}
}

// The regression this file exists for. 127 with a loader message on stderr is
// precisely how the AppImage's zenity failed while the cockpit reported the
// operator had changed their mind (#1/#2); it must now surface as a fault
// carrying the chooser's own words.
func TestClassifyPickErrorLoaderFailureSurfaces(t *testing.T) {
	const msg = "zenity: error while loading shared libraries: libgstplay-1.0.so.0: undefined symbol"
	got := classifyPickError(exitErrorFrom(t, 127, msg), false)
	if errors.Is(got, errPickCancelled) {
		t.Fatal("a chooser that died in the loader must not read as a cancellation")
	}
	if !strings.Contains(got.Error(), "127") {
		t.Errorf("the exit status is the first thing to diagnose on, missing from %q", got)
	}
	if !strings.Contains(got.Error(), "libgstplay") {
		t.Errorf("the chooser's own stderr is the actionable half, missing from %q", got)
	}
}

// An argv the chooser did not understand is our bug, not the operator's choice.
func TestClassifyPickErrorBadArgvSurfaces(t *testing.T) {
	got := classifyPickError(exitErrorFrom(t, 255, "This option is not available."), false)
	if errors.Is(got, errPickCancelled) {
		t.Fatal("exit 255 is a rejected argv, not a dismissal")
	}
	if !strings.Contains(got.Error(), "255") {
		t.Errorf("exit status missing from %q", got)
	}
}

// Only the first line reaches the operator, and only so much of it: an error
// surface sized for a sentence must not become a log viewer.
func TestClassifyPickErrorStderrIsBounded(t *testing.T) {
	long := strings.Repeat("x", stderrExcerpt+50)
	got := classifyPickError(exitErrorFrom(t, 2, "first line\n"+long), false)
	if strings.Contains(got.Error(), long) {
		t.Error("stderr beyond the first line must not be echoed")
	}
	if !strings.Contains(got.Error(), "first line") {
		t.Errorf("the first line of stderr is what says why, missing from %q", got)
	}
}

func TestClassifyPickErrorSilentFailureStillReports(t *testing.T) {
	got := classifyPickError(exitErrorFrom(t, 3, ""), false)
	if errors.Is(got, errPickCancelled) {
		t.Fatal("a chooser that exited 3 saying nothing is still a fault")
	}
	if strings.HasSuffix(got.Error(), ": ") {
		t.Errorf("no stderr must leave no dangling separator, got %q", got)
	}
}

// A chooser that never started is not an exit status at all. nativePicker
// resolved it on PATH moments earlier, so this is a fault and must say so
// without inventing a code.
func TestClassifyPickErrorNonExitError(t *testing.T) {
	got := classifyPickError(errors.New("fork/exec: permission denied"), false)
	if errors.Is(got, errPickCancelled) {
		t.Fatal("a chooser that could not be started is not a cancellation")
	}
	if !strings.Contains(got.Error(), "raising the folder chooser") {
		t.Errorf("got %q", got)
	}
}
