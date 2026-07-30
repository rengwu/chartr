package terminal

import (
	"time"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
)

// The notification clock's shipped defaults. n is the threshold — a run shorter
// than this is not worth interrupting anyone over — and D is the settle delay an
// exit has to survive before it counts, which is what collapses an agent's flicker
// between tool calls into one notification at the real end of the run. Ticket 02
// gives an operator these two through `notify.toml`; these are what stands until
// they do, and what stands when the file is absent or a value is dropped.
const (
	DefaultNotifyAfter  = config.DefaultNotifyAfter  // n
	DefaultNotifySettle = config.DefaultNotifySettle // D
)

// RunFinished is one ended run, reported once. It carries everything both of its
// consumers need — the OS notification (ticket 03) and the dot on the session's
// card (ticket 04) — so that neither re-derives any part of the rule: the tab's
// identity, the ticket it is bound to where it is a session, the state it settled
// into, and how long it worked.
//
// MapSlug and TicketNum are empty on a tab that is not a session. An ad-hoc shell
// running a long build is a run like any other; it simply has no ticket to name.
type RunFinished struct {
	TerminalID string
	SpaceID    string
	MapSlug    string
	TicketNum  int
	// Reason is the state the terminal settled into: model.TerminalIdle,
	// TerminalBlocked, TerminalDead or TerminalExited.
	Reason string
	// Duration is the span from the run's beginning to its end. The settle wait is
	// not counted — the operator is told how long the session worked, not how long
	// it took the clock to be sure it had stopped.
	Duration time.Duration
}

// runClock is the whole of the notification rule for one tab: a fold over the
// (state, timestamp) pairs the publisher emits that produces at most one
// RunFinished per run. It is pure — time arrives as a parameter and it reads no
// clock of its own — so ten minutes of history replay in microseconds and no test
// of it sleeps.
//
// It sits downstream of publisher and consumes *published* states, never raw
// evidence. The publisher already smooths a positive signal differently from a
// bare absence and holds a startup grace; a second notion of "working" derived
// here would disagree with the one the sidebar shows, and the operator would be
// told about a run they never saw.
//
// The rule: a run begins the first time the tab publishes working. It ends at the
// last moment the tab was working before staying out of working continuously for
// settle. Re-entering working before then cancels the pending end and the run
// continues; any other state merely updates the reason that will be reported. On
// end the run's duration is its beginning to its end, and an event is emitted only
// if that duration is at least after.
type runClock struct {
	// The tab's identity, fixed for the clock's life — a tab's id, space and
	// ticket binding never change after it starts.
	id      string
	spaceID string
	session *Session

	after  time.Duration // n
	settle time.Duration // D

	// The run in progress, if any. start is when it began, lastWorking the most
	// recent moment the tab was seen working (the end it will be reported at), and
	// reason the state it has fallen out of working into — cleared whenever working
	// resumes, because a pending end that is cancelled reports nothing.
	running     bool
	start       time.Time
	lastWorking time.Time
	reason      string
}

// newRunClock seats a clock for one tab. after and settle must be positive; a
// non-positive one is belt-and-braces replaced by its default rather than left to
// make the machine fire on every dip (ticket 02 validates the operator's file, so
// reaching here with a zero is a defect, not a configuration).
func newRunClock(id, spaceID string, s *Session, after, settle time.Duration) *runClock {
	if after <= 0 {
		after = DefaultNotifyAfter
	}
	if settle <= 0 {
		settle = DefaultNotifySettle
	}
	return &runClock{id: id, spaceID: spaceID, session: s, after: after, settle: settle}
}

// configureRunClock is the source switch for one terminal. Updating positive
// constants preserves a run already in progress; disabling removes the machine
// entirely, and re-enabling seats a fresh one.
func (t *Terminal) configureRunClock(enabled bool, after, settle time.Duration) {
	t.mu.Lock()
	defer t.mu.Unlock()
	if !enabled {
		t.clock = nil
		return
	}
	if t.clock == nil {
		t.clock = newRunClock(t.ID, t.SpaceID, t.session, after, settle)
		return
	}
	t.clock.after, t.clock.settle = after, settle
}

// updateRunClock folds the state the terminal actually published on this sample.
// It runs after detection and publishing, never from raw evidence.
func (t *Terminal) updateRunClock(now time.Time) *RunFinished {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.clock.update(t.state, now)
}

// update folds one published state in at now and returns the run that ended on
// this sample, or nil. A nil clock folds nothing and returns nothing, which is
// what turning notifications off looks like from here: the clock does not run at
// all rather than each consumer checking a flag (ticket 02).
func (c *runClock) update(state string, now time.Time) *RunFinished {
	if c == nil || state == "" {
		// No state published yet — nothing to fold. An empty state is the tab before
		// its first sample, not an exit from working.
		return nil
	}

	if state == model.TerminalWorking {
		if !c.running {
			c.running, c.start = true, now
		}
		// Working again: any pending end is cancelled, and with it the reason it
		// would have carried.
		c.lastWorking, c.reason = now, ""
		return nil
	}

	// Out of working. Before the first working sample there is no run to end — a
	// tab that never works never fires.
	if !c.running {
		return nil
	}

	// The reason reported is whatever the tab has settled into most recently: a run
	// that lands blocked and then goes idle is reported as idle.
	c.reason = state
	if now.Sub(c.lastWorking) < c.settle {
		return nil // still inside the settle window; the run may yet resume
	}

	// The run is over either way; only whether it is worth reporting is left.
	dur := c.lastWorking.Sub(c.start)
	reason := c.reason
	c.running, c.reason = false, ""
	if dur < c.after {
		return nil
	}

	ev := &RunFinished{TerminalID: c.id, SpaceID: c.spaceID, Reason: reason, Duration: dur}
	if c.session != nil {
		ev.MapSlug, ev.TicketNum = c.session.MapSlug, c.session.TicketNum
	}
	return ev
}
