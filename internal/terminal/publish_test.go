package terminal

import (
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/terminal/detect"
)

// The asymmetry that keeps the indicator calm, asserted on the pure half. A rule
// that positively matched publishes at once; a bare absence of any match is only a
// candidate idle, held until it repeats.
func TestPublisherHysteresis(t *testing.T) {
	now := time.Now()

	t.Run("a positive idle publishes at once", func(t *testing.T) {
		p := seated(now)
		state, changed := p.update(detect.Result{State: model.TerminalIdle}, now)
		if !changed || state != model.TerminalIdle {
			t.Fatalf("positive idle = (%q, %v), want (%q, true) on the first sample",
				state, changed, model.TerminalIdle)
		}
	})

	t.Run("a bare working→idle is held, then confirmed", func(t *testing.T) {
		p := seated(now)
		// The agent stops matching any rule — a title it dropped, not an idle it
		// announced. Every sample short of the confirmation holds working.
		for i := 1; i < p.confirm; i++ {
			state, changed := p.update(detect.Result{}, now)
			if changed || state != model.TerminalWorking {
				t.Fatalf("absence sample %d = (%q, %v), want it held at %q",
					i, state, changed, model.TerminalWorking)
			}
		}
		// The confirming sample is the one that publishes.
		state, changed := p.update(detect.Result{}, now)
		if !changed || state != model.TerminalIdle {
			t.Fatalf("confirming absence = (%q, %v), want (%q, true)", state, changed, model.TerminalIdle)
		}
		// And it settles: idle does not re-publish itself.
		if _, changed := p.update(detect.Result{}, now); changed {
			t.Error("a settled idle published again")
		}
	})

	t.Run("one working frame resets the absence count", func(t *testing.T) {
		p := seated(now)
		p.update(detect.Result{}, now) // a dropped frame
		if _, changed := p.update(detect.Result{State: model.TerminalWorking}, now); changed {
			t.Error("working re-published while already working")
		}
		// The count restarted, so the next absence is sample one of confirm again.
		for i := 1; i < p.confirm; i++ {
			if _, changed := p.update(detect.Result{}, now); changed {
				t.Fatalf("absence sample %d published; the count did not reset", i)
			}
		}
	})

	t.Run("blocked publishes at once and holds", func(t *testing.T) {
		p := seated(now)
		state, changed := p.update(detect.Result{State: model.TerminalBlocked}, now)
		if !changed || state != model.TerminalBlocked {
			t.Fatalf("blocked = (%q, %v), want (%q, true)", state, changed, model.TerminalBlocked)
		}
		if _, changed := p.update(detect.Result{State: model.TerminalBlocked}, now); changed {
			t.Error("blocked re-published while already blocked")
		}
	})

	t.Run("the startup grace refuses an absence-derived idle", func(t *testing.T) {
		// Claude emits no title for its first several seconds; a tab must not flicker
		// idle while an agent boots. A freshly seated publisher — grace clock running.
		p := newPublisher(now)
		for i := 0; i < p.confirm*3; i++ {
			if state, changed := p.update(detect.Result{}, now.Add(p.grace/2)); changed {
				t.Fatalf("absence published %q inside the startup grace", state)
			}
		}
		// Past the grace, the same absence confirms as usual.
		var published bool
		for i := 0; i < p.confirm; i++ {
			if _, changed := p.update(detect.Result{}, now.Add(p.grace+time.Second)); changed {
				published = true
			}
		}
		if !published {
			t.Error("absence never published idle after the startup grace elapsed")
		}
	})

	t.Run("a positive signal is believed even inside the grace", func(t *testing.T) {
		p := newPublisher(now) // grace clock running
		state, changed := p.update(detect.Result{State: model.TerminalIdle}, now)
		if !changed || state != model.TerminalIdle {
			t.Fatalf("positive idle inside the grace = (%q, %v), want it believed", state, changed)
		}
	})

	t.Run("a veto moves nothing, and does not starve into idle", func(t *testing.T) {
		p := seated(now)
		for i := 0; i < p.confirm*3; i++ {
			state, changed := p.update(detect.Result{Veto: true}, now.Add(time.Hour))
			if changed || state != model.TerminalWorking {
				t.Fatalf("veto sample %d = (%q, %v), want it to move nothing", i, state, changed)
			}
		}
	})
}

// seated returns a publisher for a just-identified agent, with the grace already
// elapsed unless a test says otherwise — the steady state most cases care about.
func seated(now time.Time) *publisher {
	p := newPublisher(now)
	p.since = now.Add(-p.grace) // past the startup grace
	return p
}
