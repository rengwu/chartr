package terminal

import (
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// The notification rule, whole, as a table over (state, timestamp) sequences —
// the same shape the rule engine is tested in. Time is a parameter of the fold,
// so ten minutes of history run in microseconds and nothing here sleeps.
func TestRunClockRule(t *testing.T) {
	// Round numbers, unrelated to the shipped defaults, so a case reads as its own
	// arithmetic: work for at least n = 30s, and an exit counts once the tab has
	// been out of working for D = 10s.
	const (
		n = 30 * time.Second
		d = 10 * time.Second
	)
	const (
		s  = time.Second
		wk = model.TerminalWorking
	)

	for _, tc := range []struct {
		name  string
		steps []step
		want  []fired
	}{
		{
			// The base case: one run, one notification, and the samples that follow
			// the event produce nothing — "at most one per run" is the contract.
			name: "a run past n fires exactly once",
			steps: []step{
				{0, wk}, {40 * s, wk},
				{41 * s, model.TerminalIdle}, {51 * s, model.TerminalIdle}, {90 * s, model.TerminalIdle},
			},
			want: []fired{{at: 51 * s, reason: model.TerminalIdle, duration: 40 * s}},
		},
		{
			// Work the operator was watching anyway. The run ends exactly as the long
			// one does; it is only the report that is suppressed.
			name: "a run under n never fires",
			steps: []step{
				{0, wk}, {10 * s, wk},
				{11 * s, model.TerminalIdle}, {21 * s, model.TerminalIdle}, {60 * s, model.TerminalIdle},
			},
		},
		{
			// The flicker case, which is why D exists: an agent dipping out of working
			// between tool calls is one run, reported once, and its duration spans the
			// dips rather than counting only the last stretch.
			name: "dips shorter than D are one run spanning them",
			steps: []step{
				{0, wk}, {10 * s, wk},
				{15 * s, model.TerminalIdle},
				{20 * s, wk}, {30 * s, wk},
				{35 * s, model.TerminalIdle},
				{40 * s, wk}, {50 * s, wk},
				{55 * s, model.TerminalIdle}, {61 * s, model.TerminalIdle},
			},
			want: []fired{{at: 61 * s, reason: model.TerminalIdle, duration: 50 * s}},
		},
		{
			// The other side of the same coin: past D the run is genuinely over, and
			// working again is a new run with its own beginning, its own duration and
			// its own notification.
			name: "a gap longer than D followed by more work is two runs",
			steps: []step{
				{0, wk}, {40 * s, wk},
				{45 * s, model.TerminalIdle}, {51 * s, model.TerminalIdle}, {80 * s, model.TerminalIdle},
				{100 * s, wk}, {140 * s, wk},
				{145 * s, model.TerminalIdle}, {152 * s, model.TerminalIdle},
			},
			want: []fired{
				{at: 51 * s, reason: model.TerminalIdle, duration: 40 * s},
				{at: 152 * s, reason: model.TerminalIdle, duration: 40 * s},
			},
		},
		{
			name: "a run that lands idle is reported as idle",
			steps: []step{
				{0, wk}, {40 * s, wk}, {45 * s, model.TerminalIdle}, {55 * s, model.TerminalIdle},
			},
			want: []fired{{at: 55 * s, reason: model.TerminalIdle, duration: 40 * s}},
		},
		{
			// The ending that most deserves interrupting an operator: the session is
			// waiting on them and will wait forever.
			name: "a run that stops blocked is reported as blocked",
			steps: []step{
				{0, wk}, {40 * s, wk}, {45 * s, model.TerminalBlocked}, {55 * s, model.TerminalBlocked},
			},
			want: []fired{{at: 55 * s, reason: model.TerminalBlocked, duration: 40 * s}},
		},
		{
			name: "a session that dies mid-run is reported as dead",
			steps: []step{
				{0, wk}, {40 * s, wk}, {45 * s, model.TerminalDead}, {55 * s, model.TerminalDead},
			},
			want: []fired{{at: 55 * s, reason: model.TerminalDead, duration: 40 * s}},
		},
		{
			name: "an ad-hoc shell whose process exits is reported as exited",
			steps: []step{
				{0, wk}, {40 * s, wk}, {45 * s, model.TerminalExited}, {55 * s, model.TerminalExited},
			},
			want: []fired{{at: 55 * s, reason: model.TerminalExited, duration: 40 * s}},
		},
		{
			// A permission prompt the operator cleared themselves, or an agent that
			// paused and carried on: the block never became an ending, so it is never
			// reported. The reason is whatever the run actually settled into.
			name: "a block the run resumes from before D does not fire",
			steps: []step{
				{0, wk}, {40 * s, wk},
				{45 * s, model.TerminalBlocked},
				{50 * s, wk}, {60 * s, wk},
				{65 * s, model.TerminalIdle}, {76 * s, model.TerminalIdle},
			},
			want: []fired{{at: 76 * s, reason: model.TerminalIdle, duration: 60 * s}},
		},
		{
			// Nothing to report about work that never happened — an idle tab sitting
			// open all day, an agent that blocked on its very first prompt.
			name: "a terminal that never reaches working never fires",
			steps: []step{
				{0, model.TerminalIdle}, {60 * s, model.TerminalIdle},
				{120 * s, model.TerminalBlocked}, {600 * s, model.TerminalExited},
			},
		},
		{
			// A tab sampled before it has published anything at all. An empty state is
			// the absence of a reading, not an exit from working, so it neither begins
			// a run nor ends one — the run here ends on the first *published* state
			// after the working stretch, by which point D has long since elapsed.
			name: "an unpublished state moves nothing",
			steps: []step{
				{0, ""}, {10 * s, ""},
				{20 * s, wk}, {60 * s, wk},
				{65 * s, ""}, {100 * s, ""},
				{101 * s, model.TerminalIdle}, {111 * s, model.TerminalIdle},
			},
			want: []fired{{at: 101 * s, reason: model.TerminalIdle, duration: 40 * s}},
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			got := run(newRunClock("t1", "space", nil, n, d), tc.steps)
			assertFired(t, got, tc.want)
		})
	}
}

// One event carries everything both consumers need, so that neither re-derives any
// part of the rule: the tab, its space, the ticket it is bound to, what it settled
// into and how long it worked.
func TestRunClockEventCarriesTheTabsIdentity(t *testing.T) {
	const n, d = 30 * time.Second, 10 * time.Second
	steps := []step{
		{0, model.TerminalWorking}, {40 * time.Second, model.TerminalWorking},
		{45 * time.Second, model.TerminalBlocked}, {55 * time.Second, model.TerminalBlocked},
	}

	t.Run("a session names its map and ticket", func(t *testing.T) {
		sess := &Session{MapSlug: "session-notifications-impl", TicketNum: 1, Role: "implement"}
		got := run(newRunClock("term-7", "chartr", sess, n, d), steps)
		if len(got) != 1 {
			t.Fatalf("fired %d events, want exactly 1: %v", len(got), got)
		}
		want := RunFinished{
			TerminalID: "term-7",
			SpaceID:    "chartr",
			MapSlug:    "session-notifications-impl",
			TicketNum:  1,
			Reason:     model.TerminalBlocked,
			Duration:   40 * time.Second,
		}
		if got[0].event != want {
			t.Errorf("event = %+v, want %+v", got[0].event, want)
		}
	})

	t.Run("an ad-hoc shell is a run with no ticket", func(t *testing.T) {
		// A long build in a plain terminal is a run like any other; it simply has no
		// ticket to name, and must not invent one.
		got := run(newRunClock("term-8", "chartr", nil, n, d), steps)
		if len(got) != 1 {
			t.Fatalf("fired %d events, want exactly 1: %v", len(got), got)
		}
		if got[0].event.MapSlug != "" || got[0].event.TicketNum != 0 {
			t.Errorf("event = %+v, want no map or ticket on a tab that is not a session", got[0].event)
		}
	})
}

// Turning notifications off is the absence of a clock, not a flag every consumer
// has to remember to check (ticket 02): a nil clock folds nothing and emits
// nothing, however long the tab works.
func TestNilRunClockEmitsNothing(t *testing.T) {
	var c *runClock
	got := run(c, []step{
		{0, model.TerminalWorking}, {10 * time.Minute, model.TerminalWorking},
		{11 * time.Minute, model.TerminalIdle}, {20 * time.Minute, model.TerminalIdle},
	})
	if len(got) != 0 {
		t.Errorf("a nil clock fired %v; want nothing at all", got)
	}
}

// A misconstructed clock falls back to the shipped defaults rather than firing on
// every dip — ticket 02 validates the operator's file, so a zero reaching here is a
// defect, and the defect must not turn into a burst of notifications.
func TestRunClockDefaultsNonPositiveConstants(t *testing.T) {
	c := newRunClock("t1", "space", nil, 0, -time.Second)
	if c.after != DefaultNotifyAfter || c.settle != DefaultNotifySettle {
		t.Fatalf("clock = (after %s, settle %s), want the defaults (%s, %s)",
			c.after, c.settle, DefaultNotifyAfter, DefaultNotifySettle)
	}
}

// The clock over real bytes: the recorded Claude session, replayed through the
// scanner, the grid, the rule engine and the publishing hysteresis, then folded by
// the clock exactly as the sampler would fold it. The capture is one working turn
// broken by a permission dialog and a pause, ending back at the prompt — so it is
// one run, reported once, as idle.
//
// The parameters are the recording's scale rather than the shipped defaults (its
// whole life is 89 seconds), and the tail keeps sampling past the last recorded
// byte because real sampling does not stop when an agent stops writing.
func TestRunClockOnRecordedClaudeTurn(t *testing.T) {
	const (
		after  = 30 * time.Second
		settle = 25 * time.Second
		tail   = 30 * time.Second
	)
	origin := time.Time{}
	c := newRunClock("term-1", "chartr", &Session{MapSlug: "agent-state-detection", TicketNum: 2}, after, settle)

	var got []fired
	replayPublished(t, "claude", "rec-claude.jsonl", tail, func(now time.Duration, tr transition, _ bool) {
		if ev := c.update(tr.state, origin.Add(now)); ev != nil {
			got = append(got, fired{at: now, reason: ev.Reason, duration: ev.Duration, event: *ev})
		}
	})

	if len(got) != 1 {
		t.Fatalf("the recorded turn fired %d events, want exactly 1: %v", len(got), got)
	}
	ev := got[0]
	if ev.reason != model.TerminalIdle {
		t.Errorf("reported %q; the capture ends back at Claude's prompt, so the run settled %q",
			ev.reason, model.TerminalIdle)
	}
	if ev.event.MapSlug != "agent-state-detection" || ev.event.TicketNum != 2 {
		t.Errorf("event = %+v, want it to carry the tab's ticket binding", ev.event)
	}
	// The turn's working stretch runs from the tab's first published working to the
	// last, around the dialog and the pause in the middle — a minute of the 89-second
	// capture, not the two-and-a-half-second stretch after the dialog cleared.
	if ev.duration < 55*time.Second || ev.duration > 65*time.Second {
		t.Errorf("duration = %s, want ~61s: the run spans the dips, not just its last stretch", ev.duration)
	}
	// And the settle wait is not counted: the operator is told how long the session
	// worked, not how long the clock took to be sure it had stopped.
	if ev.duration > ev.at-settle+time.Second {
		t.Errorf("duration %s counts part of the %s settle wait (fired at %s)", ev.duration, settle, ev.at)
	}
}

// step is one sample handed to the clock: the published state, and when.
type step struct {
	at    time.Duration
	state string
}

// fired is one event the clock produced and the moment it produced it. reason and
// duration are pulled out because they are what every case is about; event carries
// the whole thing for the cases that assert identity too.
type fired struct {
	at       time.Duration
	reason   string
	duration time.Duration
	event    RunFinished
}

// run folds a sequence of samples through c, from a fixed origin, and returns
// every event it emitted.
func run(c *runClock, steps []step) []fired {
	origin := time.Time{}
	var out []fired
	for _, s := range steps {
		if ev := c.update(s.state, origin.Add(s.at)); ev != nil {
			out = append(out, fired{at: s.at, reason: ev.Reason, duration: ev.Duration, event: *ev})
		}
	}
	return out
}

func assertFired(t *testing.T, got, want []fired) {
	t.Helper()
	if len(got) != len(want) {
		t.Fatalf("fired %d events, want %d\n got: %v\nwant: %v", len(got), len(want), got, want)
	}
	for i := range want {
		if got[i].at != want[i].at || got[i].reason != want[i].reason || got[i].duration != want[i].duration {
			t.Errorf("event %d = (at %s, %q, ran %s), want (at %s, %q, ran %s)",
				i, got[i].at, got[i].reason, got[i].duration,
				want[i].at, want[i].reason, want[i].duration)
		}
	}
}
