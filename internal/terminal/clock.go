package terminal

import (
	"time"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/transcript"
)

// The notification clock's shipped defaults. n is the threshold — a turn shorter
// than this is not worth interrupting anyone over — and D debounces blocked
// signals and settles the state-only fallback. Provider-recorded completion does
// not wait for D.
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
	// Reason is the semantic outcome or attention state: idle/completed, blocked,
	// dead, exited, failed or interrupted.
	Reason string
	// Duration is the span from the run's beginning to its end. The settle wait is
	// not counted — the operator is told how long the session worked, not how long
	// it took the clock to be sure it had stopped.
	Duration time.Duration
}

const (
	RunFailed      = "failed"
	RunInterrupted = "interrupted"
)

// runClock is the notification fold for one tab. Before a provider transcript is
// bound it preserves the old working/idle rule as a compatibility fallback. Once
// bound, TurnStarted and TurnFinished are authoritative, terminal idle is ignored,
// and only positive blocked or process-lifecycle states remain meaningful.
//
// Time always arrives as a parameter, so both paths remain deterministic and
// fast to test.
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
	// semantic is true only after this clock consumed a provider TurnStarted.
	// Binding alone is insufficient: a watcher seated mid-turn deliberately keeps
	// that historical start behind its cursor and must retain the state fallback.
	semantic bool
	// awaitNonWorking prevents a stale working frame immediately after semantic
	// completion from opening a duplicate fallback run.
	awaitNonWorking bool
	// blockedReported deduplicates a permission/question notification within one
	// semantic turn. It resets only when the transcript opens or closes a turn.
	blockedReported bool
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
	if t.clock == nil {
		return nil
	}
	return t.clock.updateSource(t.state, now, t.clock.semantic)
}

// notificationTurnStarted arms the clock from the provider's authoritative turn
// boundary, replacing any provisional working run observed during boot.
func (t *Terminal) notificationTurnStarted(now time.Time) {
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.clock != nil {
		t.clock.turnStarted(now)
	}
}

// notificationTurnFinished closes a semantic turn immediately, without an idle
// confirmation or settle delay.
func (t *Terminal) notificationTurnFinished(now time.Time, outcome transcript.Outcome) *RunFinished {
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.clock == nil {
		return nil
	}
	reason := model.TerminalIdle
	switch outcome {
	case transcript.OutcomeFailed:
		reason = RunFailed
	case transcript.OutcomeInterrupted:
		reason = RunInterrupted
	}
	return t.clock.turnFinished(now, reason)
}

// update folds one published state in at now and returns the run that ended on
// this sample, or nil. A nil clock folds nothing and returns nothing, which is
// what turning notifications off looks like from here: the clock does not run at
// all rather than each consumer checking a flag (ticket 02).
func (c *runClock) update(state string, now time.Time) *RunFinished {
	return c.updateSource(state, now, false)
}

// updateSource preserves the old state fold until a provider TurnStarted has
// actually arrived. During a semantic turn, idle is presentation only; positive
// blocked state and process death remain useful signals.
func (c *runClock) updateSource(state string, now time.Time, semantic bool) *RunFinished {
	if c == nil || state == "" {
		// No state published yet — nothing to fold. An empty state is the tab before
		// its first sample, not an exit from working.
		return nil
	}

	if state == model.TerminalWorking {
		if c.awaitNonWorking {
			return nil
		}
		if !c.running {
			c.running, c.start = true, now
		}
		// Working again: any pending end is cancelled, and with it the reason it
		// would have carried.
		c.lastWorking, c.reason = now, ""
		return nil
	}
	c.awaitNonWorking = false

	if semantic {
		switch state {
		case model.TerminalIdle:
			return nil
		case model.TerminalBlocked:
			if !c.running || c.blockedReported || now.Sub(c.lastWorking) < c.settle {
				return nil
			}
			dur := now.Sub(c.start)
			if dur < c.after {
				return nil
			}
			c.blockedReported = true
			return c.event(model.TerminalBlocked, dur)
		case model.TerminalDead, model.TerminalExited:
			return c.turnFinished(now, state)
		default:
			return nil
		}
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

	return c.event(reason, dur)
}

func (c *runClock) turnStarted(now time.Time) {
	if c == nil {
		return
	}
	// The provider's own submission is the authoritative beginning. Re-seat even
	// if a working status started a provisional run during boot.
	c.running, c.start, c.lastWorking = true, now, now
	c.reason, c.blockedReported, c.semantic, c.awaitNonWorking = "", false, true, false
}

func (c *runClock) turnFinished(now time.Time, reason string) *RunFinished {
	if c == nil || !c.running {
		return nil
	}
	dur := now.Sub(c.start)
	c.running, c.reason, c.blockedReported, c.semantic, c.awaitNonWorking = false, "", false, false, true
	if dur < c.after {
		return nil
	}
	return c.event(reason, dur)
}

func (c *runClock) event(reason string, dur time.Duration) *RunFinished {
	ev := &RunFinished{TerminalID: c.id, SpaceID: c.spaceID, Reason: reason, Duration: dur}
	if c.session != nil {
		ev.MapSlug, ev.TicketNum = c.session.MapSlug, c.session.TicketNum
	}
	return ev
}
