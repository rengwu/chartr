package terminal

import (
	"testing"
	"time"
)

// worked seats a scheduler that has already seen a turn run and is past any startup
// grace — the ordinary state in which the guards below are what decide due.
func worked(debounce time.Duration) titleSched {
	return titleSched{debounce: debounce, worked: true}
}

func TestTitleSchedNotDueUntilATurnHasRun(t *testing.T) {
	// A freshly launched, never-worked tab sitting idle at its prompt is NOT due —
	// there is no conversation to summarise yet. This is the boot-screen case.
	s := titleSched{debounce: time.Minute}
	if s.due(true, "abc", time.Now()) {
		t.Fatal("a tab that never worked must not be titled")
	}
	// Once it has been seen working (a turn ran), the next idle is due.
	s.observe(true)
	if !s.due(true, "abc", time.Now()) {
		t.Fatal("after a working turn, idle should be due")
	}
}

func TestTitleSchedStartupGrace(t *testing.T) {
	now := time.Now()
	s := titleSched{debounce: time.Minute, startupGrace: 30 * time.Second, firstAt: now, worked: true}
	// Inside the startup window, even a worked idle tab holds — the opening screen
	// is off-limits.
	if s.due(true, "abc", now.Add(10*time.Second)) {
		t.Fatal("within the startup grace, nothing should be due")
	}
	// Past it, the first title fires.
	if !s.due(true, "abc", now.Add(31*time.Second)) {
		t.Fatal("past the startup grace, a worked idle tab should be due")
	}
}

func TestTitleSchedNotDueWhenNotIdle(t *testing.T) {
	s := worked(time.Minute)
	if s.due(false, "abc", time.Now()) {
		t.Fatal("a working tab must not be due for a title")
	}
}

func TestTitleSchedNotDueOnBlankScreen(t *testing.T) {
	s := worked(time.Minute)
	if s.due(true, "", time.Now()) {
		t.Fatal("an empty screen hash is never due")
	}
}

func TestTitleSchedFiresOnceThenHoldsInFlight(t *testing.T) {
	s := worked(time.Minute)
	now := time.Now()
	if !s.due(true, "abc", now) {
		t.Fatal("first worked idle with content should be due")
	}
	// Armed: a second call before the generation resolves is refused, even a beat
	// later — one generation at a time.
	if s.due(true, "abc", now.Add(time.Second)) {
		t.Fatal("must not fire while a generation is in flight")
	}
}

func TestTitleSchedNeedsAFreshTurnAfterSuccess(t *testing.T) {
	s := worked(time.Minute)
	now := time.Now()
	if !s.due(true, "abc", now) {
		t.Fatal("first should be due")
	}
	s.succeeded("abc")
	// A changed screen, debounce long past — but no new turn has run, so the
	// worked-gate holds. Titles track turns, not repaints.
	if s.due(true, "def", now.Add(time.Hour)) {
		t.Fatal("a changed screen alone must not regenerate; a new turn is required")
	}
	// A new working turn re-arms it.
	s.observe(true)
	if !s.due(true, "def", now.Add(time.Hour)) {
		t.Fatal("after a fresh turn, a changed screen should regenerate")
	}
}

func TestTitleSchedUnchangedGuard(t *testing.T) {
	s := worked(time.Minute)
	now := time.Now()
	if !s.due(true, "abc", now) {
		t.Fatal("first should be due")
	}
	s.succeeded("abc")
	s.observe(true) // a turn ran, so only the unchanged-guard is under test now
	// Same screen, debounce long past: still not due — the guard is the screen.
	if s.due(true, "abc", now.Add(time.Hour)) {
		t.Fatal("unchanged screen must not regenerate")
	}
	if !s.due(true, "def", now.Add(time.Hour)) {
		t.Fatal("changed screen after a turn should be due")
	}
}

func TestTitleSchedDebounce(t *testing.T) {
	s := worked(time.Minute)
	now := time.Now()
	if !s.due(true, "abc", now) {
		t.Fatal("first should be due")
	}
	s.succeeded("abc")
	s.observe(true)
	// A changed screen inside the debounce window is held.
	if s.due(true, "def", now.Add(30*time.Second)) {
		t.Fatal("within debounce must hold even on a changed screen")
	}
	// Past the debounce it fires.
	if !s.due(true, "def", now.Add(90*time.Second)) {
		t.Fatal("past debounce a changed screen should fire")
	}
}

func TestTitleSchedFailureRetriesSameScreenAfterDebounce(t *testing.T) {
	s := worked(time.Minute)
	now := time.Now()
	if !s.due(true, "abc", now) {
		t.Fatal("first should be due")
	}
	// Generation failed: neither the guard nor the worked-gate advanced, so the
	// same screen is due again once the debounce passes — a transient failure is a
	// pause, not a verdict.
	s.failed()
	if s.due(true, "abc", now.Add(30*time.Second)) {
		t.Fatal("failed retry must still respect the debounce")
	}
	if !s.due(true, "abc", now.Add(90*time.Second)) {
		t.Fatal("after debounce, a failed screen should retry")
	}
}
