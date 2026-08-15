//go:build !darwin && !linux

package proc

import (
	"errors"
	"runtime"
	"testing"
)

// The process-backed tests in resolve_unix_test.go need a reader, and this
// platform has none. They are skipped by name here rather than silently absent,
// so a run on Windows or a BSD says what it did not check and why — and the two
// facts that must still hold are asserted rather than assumed: the package
// compiles, and every resolution is unavailable.

func TestProcessReaderIsUnavailableHere(t *testing.T) {
	t.Logf("skipping the process-backed tests: %s has no foreground-process or "+
		"process-environment lookup, so transcript binding is unavailable by "+
		"design (initial platform support is macOS and Linux)", runtime.GOOS)

	if _, err := Lookup(1, []string{"CLAUDE_CONFIG_DIR"}); !errors.Is(err, ErrUnsupported) {
		t.Fatalf("Lookup = %v, want ErrUnsupported", err)
	}
	if got, ok := Foreground(1, func([]string) string { return "claude" }); ok {
		t.Fatalf("Foreground resolved %+v, want unavailable", got)
	}
	if got, ok := Launched("claude", 1); ok {
		t.Fatalf("Launched resolved %+v, want unavailable", got)
	}
}
