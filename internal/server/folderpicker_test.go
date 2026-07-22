package server_test

import (
	"os/exec"
	"runtime"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The native folder picker at the process boundary. The dialog itself is
// deliberately not exercised: raising it would put a real modal window on
// whoever is running the tests and wait for a human to answer it. What is
// testable — and what the frontend actually depends on — is the capability flag
// the snapshot carries, which is what steers New Space between the operator's
// own chooser and the typed-path fallback.

// A machine that can raise a chooser says so in every snapshot, and one that
// cannot says that instead. Getting this backwards is the one failure that
// strands an operator: a false positive raises a dialog that never appears, and
// a false negative hides the picker behind the paste-a-path form forever.
func TestSnapshotReportsFolderPickerAvailability(t *testing.T) {
	h := chartrtest.Start(t)

	want := hasNativeChooser()
	if got := h.Snapshot(ctx(t)).NativePicker; got != want {
		t.Fatalf("snapshot nativePicker = %v, want %v on %s", got, want, runtime.GOOS)
	}
}

// hasNativeChooser answers "can this machine raise a folder dialog" from the
// outside — the same question the server answers, asked independently rather
// than by calling into it, so the test pins the contract instead of restating
// the implementation.
func hasNativeChooser() bool {
	switch runtime.GOOS {
	case "darwin":
		// osascript ships with macOS; there is no configuration in which it is
		// absent, so the picker is always available there.
		return true
	case "linux":
		for _, bin := range []string{"zenity", "kdialog"} {
			if _, err := exec.LookPath(bin); err == nil {
				return true
			}
		}
	}
	// Windows is out of scope for this build: an operator there gets the
	// typed-path form, not a half-tested chooser.
	return false
}
