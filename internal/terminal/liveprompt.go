package terminal

import (
	"errors"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// Live delivery of one prompt preset into a running agent (prompt-presets spec,
// "Live delivery"). It is deliberately the smallest thing that can be true: one
// pending item per tab, held in memory, submitted the next time the tab is
// *observed* idle by the sampler that already runs. There is no persistence, no
// retry, no history, and no acknowledgement from the provider — idle is inferred
// from terminal evidence, and the operator is told as much.
//
// The target is any tab with a live agent in front of it, launched by chartr or
// started by the operator: the question a delivery has to answer is whether a TUI
// is listening, and the sampler answers exactly that for both.

// ErrNotPromptTarget marks a tab with no live agent in front of it: an ordinary
// shell sitting at its prompt, or a tab whose process is gone. What a delivery
// needs is a TUI listening rather than a shell that would run the preset as a
// command, and that is a question about what holds the tab *now* — chartr having
// started the binary is one way to know it, not the only one, so an agent the
// operator ran themselves is as good a target as one chartr launched.
var ErrNotPromptTarget = errors.New("terminal: only a tab with a live agent in front of it can be sent a preset")

// ErrPromptPending marks a second activation while one preset is still waiting.
// One pending item is the whole queue: two would need an order, a display for
// it, and an answer for what a cancel means, none of which the operator asked
// for.
var ErrPromptPending = errors.New("terminal: a preset is already queued for this tab")

// pendingPrompt is the one preset a busy tab is holding: the catalog id the pane
// names it by, and the body snapshotted when the operator sent it. The body is
// copied rather than re-read at delivery because a later edit or deletion must
// not rewrite an action the operator already took (spec, "Persistence and failure
// behavior").
type pendingPrompt struct {
	id   string
	body string
	// agent and pid are the conversation the operator aimed at, so a delivery can
	// prove it is still the one in front of the tab. They matter because eligibility
	// is no longer immutable: an ad-hoc shell's agent is whatever holds the PTY's
	// foreground, and the operator can quit it — leaving a shell that reads idle and
	// would run the preset as a command (see submitDuePrompt).
	agent string
	pid   int
}

// SendPrompt delivers one preset to one of spaceID's tabs. An idle target is
// typed at once; a working, running, or blocked one holds the preset until the
// next idle the sampler observes, which is what queued reports. A tab that is
// not this space's, or has no live agent in front of it, or is already holding a
// preset, is refused and receives nothing.
func (m *Manager) SendPrompt(spaceID, termID, promptID, body string) (queued bool, err error) {
	t, ok := m.promptTarget(spaceID, termID)
	if !ok {
		return false, ErrNoTerminal
	}
	queued, err = t.sendPrompt(promptID, body)
	if err != nil {
		return false, err
	}
	if queued {
		// The pending item is on the pushed model, so the pane sees it appear with no
		// refresh. An immediate send changes nothing there.
		m.notify()
	}
	return queued, nil
}

// CancelPrompt drops the preset a tab is holding before it is delivered. It
// reports whether there was one to drop: cancelling a tab that holds nothing is
// an ordinary no-op — the pane can post it against a row the sampler cleared a
// moment earlier — and only an id that names no tab of this space is an error.
func (m *Manager) CancelPrompt(spaceID, termID string) (bool, error) {
	t, ok := m.promptTarget(spaceID, termID)
	if !ok {
		return false, ErrNoTerminal
	}
	cleared := t.cancelPrompt()
	if cleared {
		m.notify()
	}
	return cleared, nil
}

// promptTarget resolves one of spaceID's tabs. A terminal in another space is
// not found rather than refused: the pane acts on the space it is scoped to, so
// a foreign id can only be a stale client, and there is nothing there for it.
func (m *Manager) promptTarget(spaceID, termID string) (*Terminal, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	t := m.terms[termID]
	if t == nil || t.SpaceID != spaceID {
		return nil, false
	}
	return t, true
}

// promptSeatLocked names the agent conversation a preset would be delivered
// into: the adapter in front of the tab and the process it is. It is the whole
// of eligibility, and it is the identification the tab's own status and title
// already run on — a launched tab knows its adapter from the launch, an ad-hoc
// shell's is whatever the sampler identified in the PTY's foreground. Not ok
// means there is no agent listening, and typing there would hand the preset to a
// shell. The caller holds t.mu.
func (t *Terminal) promptSeatLocked() (agent string, pid int, ok bool) {
	if !t.alive {
		return "", 0, false
	}
	if agent = t.titleAgentLocked(); agent == "" {
		return "", 0, false
	}
	return agent, t.titlePIDLocked(), true
}

// sendPrompt applies the state gate for one tab under its own lock, so the
// decision and the record of it cannot be split by a sample landing in between.
// A tab that reads idle is submitted to off the lock; anything else stores the
// snapshot.
func (t *Terminal) sendPrompt(id, body string) (queued bool, err error) {
	t.mu.Lock()
	agent, pid, ok := t.promptSeatLocked()
	switch {
	case !ok:
		t.mu.Unlock()
		return false, ErrNotPromptTarget
	case t.pending != nil:
		t.mu.Unlock()
		return false, ErrPromptPending
	case t.state != model.TerminalIdle:
		t.pending = &pendingPrompt{id: id, body: body, agent: agent, pid: pid}
		t.mu.Unlock()
		return true, nil
	}
	t.mu.Unlock()

	// Off the caller's goroutine for the same reason the opener is: the submission
	// holds the input lock across its beat, and an HTTP response must not wait on
	// it. A tab that goes busy in this instant is the accepted race — the operator
	// asked for it while it read idle, and best-effort is the whole contract.
	go t.submitPrompt(body)
	return false, nil
}

// cancelPrompt drops any pending preset, reporting whether there was one.
func (t *Terminal) cancelPrompt() bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.pending == nil {
		return false
	}
	t.pending = nil
	return true
}

// submitDuePrompt is the sampler's half: called once per sample, after the state
// for that sample has been published, it submits the pending preset if the tab
// now reads idle. Taking the item under the lock is what makes "exactly once"
// true — a second pass finds nothing left. It reports whether the row changed,
// so the sampler pushes the cleared one.
//
// There is no separate transition to detect. A preset is only ever queued on a
// tab that was not idle, so the first idle that finds one here *is* the
// transition; and if the tab went idle between the operator's click and the
// store, the very next sample delivers it, which is the same promise.
//
// The seat is re-checked because idle alone stopped being enough the moment an
// agent the operator started counted as a target: quitting that agent leaves a
// *shell* at its prompt, which reads idle and would run the preset as a command.
// A seat that is gone or has become another conversation drops the item instead
// — the same rule as an exit, which the operator already sees as the pending
// preset disappearing with what it was aimed at.
func (t *Terminal) submitDuePrompt() bool {
	t.mu.Lock()
	if t.pending == nil {
		t.mu.Unlock()
		return false
	}
	agent, pid, ok := t.promptSeatLocked()
	if !ok || agent != t.pending.agent || pid != t.pending.pid {
		t.pending = nil
		t.mu.Unlock()
		return true
	}
	if t.state != model.TerminalIdle {
		t.mu.Unlock()
		return false
	}
	body := t.pending.body
	t.pending = nil
	t.mu.Unlock()

	go t.submitPrompt(body)
	return true
}

// submitPrompt types one line into a live TUI and presses return. It is the same
// two-write shape typeOpener needs and for the same reasons — return is CR, not
// LF, and the submit key must arrive in its own read (see typeOpener) — and both
// go through here so there is one implementation of it.
//
// The pair is held under the input lock, so no other write can land between the
// text and its return: an operator keystroke racing a delivery arrives after the
// submission, never inside it. A write error only means the agent has exited,
// which the read loop is reaping in parallel.
func (t *Terminal) submitPrompt(body string) {
	line := strings.TrimRight(body, "\r\n")
	if line == "" {
		return
	}

	t.writeMu.Lock()
	defer t.writeMu.Unlock()
	if _, err := t.pty.Write([]byte(line)); err != nil {
		return
	}
	time.Sleep(openerSubmit)
	_, _ = t.pty.Write([]byte("\r"))
}
