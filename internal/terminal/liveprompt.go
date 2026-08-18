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

// ErrNotPromptTarget marks a tab that is not a live agent chartr launched: an
// ordinary shell, an agent the operator started themselves, or a tab whose
// process is gone. Only a tab chartr put an agent into can be typed at, because
// only there does chartr know a TUI is listening rather than a shell that would
// run the preset as a command.
var ErrNotPromptTarget = errors.New("terminal: only a live agent chartr launched can be sent a preset")

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
}

// SendPrompt delivers one preset to one of spaceID's tabs. An idle target is
// typed at once; a working, running, or blocked one holds the preset until the
// next idle the sampler observes, which is what queued reports. A tab that is
// not this space's, or not a live chartr-launched agent, or already holding a
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

// sendPrompt applies the state gate for one tab under its own lock, so the
// decision and the record of it cannot be split by a sample landing in between.
// A tab that reads idle is submitted to off the lock; anything else stores the
// snapshot.
func (t *Terminal) sendPrompt(id, body string) (queued bool, err error) {
	t.mu.Lock()
	switch {
	case t.launchedAgent == "" || !t.alive:
		t.mu.Unlock()
		return false, ErrNotPromptTarget
	case t.pending != nil:
		t.mu.Unlock()
		return false, ErrPromptPending
	case t.state != model.TerminalIdle:
		t.pending = &pendingPrompt{id: id, body: body}
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
// true — a second pass finds nothing left. It reports whether it submitted, so
// the sampler pushes the cleared row.
//
// There is no separate transition to detect. A preset is only ever queued on a
// tab that was not idle, so the first idle that finds one here *is* the
// transition; and if the tab went idle between the operator's click and the
// store, the very next sample delivers it, which is the same promise.
func (t *Terminal) submitDuePrompt() bool {
	t.mu.Lock()
	if t.pending == nil || !t.alive || t.state != model.TerminalIdle {
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
