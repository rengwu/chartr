package terminal

import (
	"time"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/terminal/detect"
)

// Agent-grammar publishing thresholds. Vars, not consts, so a test can shrink
// them the way the opener thresholds already are.
var (
	// agentStartupGrace is how long after an agent is first identified the sampler
	// refuses to publish an *absence*-derived idle. Claude emits no title at all for
	// its first several seconds, and a tab must not flicker idle while an agent
	// boots. A positive signal is always believed, grace or not.
	agentStartupGrace = 3 * time.Second
	// agentAbsenceConfirm is how many consecutive samples of "no rule matched" it
	// takes before idle is published. This is the asymmetry that keeps the indicator
	// calm: a positively-matched state publishes at once, while a mere absence of
	// working — which a single dropped title frame produces — has to be held and
	// confirmed before it moves anything.
	agentAbsenceConfirm = 3
)

// publisher holds the hysteresis for one identified agent on one tab. It is the
// pure half of agent sampling: given the engine's verdict for a sample and the
// time, it says what the tab should read and whether that changed — no PTY, no
// clock of its own, so the asymmetry is testable directly.
//
// The asymmetry, copied from herdr, is the whole point. A rule that positively
// matched (working, blocked, or idle) publishes immediately, because the agent
// said so. An absence — no rule matched at all — is only a candidate idle: it must
// repeat agentAbsenceConfirm times before it publishes, and it is refused outright
// during the startup grace. Without it the indicator strobes on every frame an
// agent skips.
type publisher struct {
	grace   time.Duration
	confirm int
	since   time.Time // when this agent was identified
	absent  int       // consecutive samples with no rule match
	current string    // the last published state
}

// newPublisher seats a freshly identified agent. It starts on working rather than
// idle so a just-spawned agent orbits rather than flashing a spurious idle tick
// (the same seat newProc gives a session), and starts the grace clock at now.
func newPublisher(now time.Time) *publisher {
	return &publisher{
		grace:   agentStartupGrace,
		confirm: agentAbsenceConfirm,
		since:   now,
		current: model.TerminalWorking,
	}
}

// update folds one sample's verdict in and reports the state to publish and
// whether it changed.
func (p *publisher) update(res detect.Result, now time.Time) (state string, changed bool) {
	// A veto rule matched: this screen is meaningless (a transcript viewer or model
	// picker showing stale prompt text), so it moves nothing — not the state, and
	// not the absence count either, or a veto would slowly starve into idle.
	if res.Veto {
		return p.current, false
	}

	// A positive match: the agent named its own state, so believe it at once.
	if res.State != "" {
		p.absent = 0
		if res.State == p.current {
			return p.current, false
		}
		p.current = res.State
		return p.current, true
	}

	// No rule matched. That is a candidate idle, never an immediate one.
	p.absent++
	if now.Sub(p.since) < p.grace {
		return p.current, false // still booting; it has not had a chance to speak
	}
	if p.absent < p.confirm {
		return p.current, false // held until confirmed
	}
	if p.current == model.TerminalIdle {
		return p.current, false
	}
	p.current = model.TerminalIdle
	return p.current, true
}
