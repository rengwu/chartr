//go:build !windows

package terminal

import (
	"errors"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// Live preset delivery at the terminal-manager seam (prompt-presets, ticket 03).
// The tab under test is a real PTY running the raw-mode stub from opener_test.go,
// which records every byte it is sent — so "the agent received the preset" is
// asserted from the agent's side, exactly as the typed opener is.
//
// The state the delivery is gated on is set by hand rather than acted out by a
// stub agent. The grammar that produces it is `sample`'s business and is covered
// where it lives; what this file is about is what the queue does with each state
// once it reads it.

// promptTab launches a free tab — a shell with the stub preloaded — and returns
// it with the log of everything that reached the agent's stdin. The seat is then
// made by hand, as it is for an operator's own agent below: a free tab is an
// ad-hoc shell, so its agent is the sampler's to identify, and `stub` is not a
// known adapter, so nothing in the sampler will move the tab's state or its seat
// underneath a test that sets them.
func promptTab(t *testing.T) (*Manager, *Terminal, string) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("the raw-mode stub requires POSIX stty")
	}
	useTestShell(t)
	shrinkOpenerTiming(t)

	log := filepath.Join(t.TempDir(), "keystrokes.log")
	agent, args, env := rawModeAgent(t, log)

	m := NewManager(nil, nil) // nil onChange: no background sampler, the test drives delivery
	t.Cleanup(m.Shutdown)

	term, err := m.OpenFree("s1", t.TempDir(), "f1", agent, args, env, "", "stub")
	if err != nil {
		t.Fatalf("opening the free tab: %v", err)
	}
	// The stub types nothing until it is in raw mode, and until it holds the
	// terminal the shell that started it would run a delivery as a command; every
	// assertion below is about bytes chartr sent to the agent, so wait for the
	// agent to be the one reading them.
	if !term.awaitForeground(5 * time.Second) {
		t.Fatal("the preloaded stub never took the terminal")
	}
	if !term.awaitReady(openerSettle, 5*time.Second) {
		t.Fatal("the stub agent never came up")
	}
	seatAdHocAgent(t, term, "stub", 4242)
	return m, term, log
}

// seatAdHocAgent makes the tab look like what an operator's own `claude` looks
// like to chartr: nothing was launched here, and the sampler has identified a
// known agent holding the PTY's foreground. Both fields are the sampler's to
// write in production; setting them by hand is the same bargain setState makes —
// the grammar that produces them is covered where it lives.
func seatAdHocAgent(t *testing.T, term *Terminal, agent string, pgrp int) {
	t.Helper()
	term.mu.Lock()
	term.launchedAgent = ""
	term.agent = agent
	term.lastPgrp = pgrp
	term.mu.Unlock()
}

func setState(t *testing.T, term *Terminal, state string) {
	t.Helper()
	term.mu.Lock()
	term.state = state
	term.mu.Unlock()
}

func pending(t *testing.T, m *Manager, id string) string {
	t.Helper()
	info, ok := m.Lookup(id)
	if !ok {
		t.Fatalf("no terminal %q", id)
	}
	return info.PendingPrompt
}

// An idle agent is told at once: the snapshotted body, then a carriage return in
// a write of its own, and nothing is left pending behind it.
func TestIdleAgentReceivesThePresetImmediately(t *testing.T) {
	m, term, log := promptTab(t)
	setState(t, term, model.TerminalIdle)

	const body = "Answer in as few words as the question allows."
	queued, err := m.SendPrompt("s1", term.ID, "brief", body)
	if err != nil {
		t.Fatalf("sending the preset: %v", err)
	}
	if queued {
		t.Fatal("an idle agent queued the preset instead of receiving it")
	}

	got := waitForFile(t, log, body+"\r", 5*time.Second)
	if strings.Contains(got, "\n") {
		t.Errorf("the preset was submitted with a linefeed, which leaves it unsent in a real TUI: %q", got)
	}
	if p := pending(t, m, term.ID); p != "" {
		t.Errorf("pending preset = %q after an immediate send, want none", p)
	}
}

// Working, running and blocked are all the same answer: no bytes, one visible
// pending preset. The tab is never typed into while it is busy or sitting on a
// permission prompt.
func TestBusyAgentHoldsThePresetPending(t *testing.T) {
	for _, state := range []string{model.TerminalWorking, model.TerminalRunning, model.TerminalBlocked} {
		t.Run(state, func(t *testing.T) {
			m, term, log := promptTab(t)
			setState(t, term, state)

			queued, err := m.SendPrompt("s1", term.ID, "brief", "Keep it short.")
			if err != nil {
				t.Fatalf("sending the preset: %v", err)
			}
			if !queued {
				t.Fatal("a busy agent was sent the preset rather than queueing it")
			}
			if p := pending(t, m, term.ID); p != "brief" {
				t.Errorf("pending preset = %q, want %q", p, "brief")
			}

			time.Sleep(200 * time.Millisecond) // well past the shrunk submit beat
			if b, _ := os.ReadFile(log); len(b) > 0 {
				t.Errorf("a %s agent was typed into: %q", state, b)
			}
		})
	}
}

// The next idle the sampler observes submits the pending preset — once. A second
// look at a still-idle tab has nothing left to send.
func TestNextIdleSubmitsThePendingPresetOnce(t *testing.T) {
	m, term, log := promptTab(t)
	setState(t, term, model.TerminalWorking)

	const body = "Follow the repository's commit convention."
	if _, err := m.SendPrompt("s1", term.ID, "commits", body); err != nil {
		t.Fatalf("sending the preset: %v", err)
	}

	// Still working: the sampler's pass sends nothing.
	if term.submitDuePrompt() {
		t.Fatal("a working tab submitted its pending preset")
	}

	setState(t, term, model.TerminalIdle)
	if !term.submitDuePrompt() {
		t.Fatal("the first idle pass did not submit the pending preset")
	}
	if term.submitDuePrompt() {
		t.Fatal("a second idle pass submitted the preset again")
	}

	got := waitForFile(t, log, body+"\r", 5*time.Second)
	if strings.Count(got, body) != 1 {
		t.Errorf("the preset arrived %d times, want exactly one: %q", strings.Count(got, body), got)
	}
	if p := pending(t, m, term.ID); p != "" {
		t.Errorf("pending preset = %q after delivery, want none", p)
	}
}

// Cancelling drops the item before it is ever sent: the next idle submits
// nothing, and the tab is free to be sent another preset.
func TestCancelledPresetIsNeverSubmitted(t *testing.T) {
	m, term, log := promptTab(t)
	setState(t, term, model.TerminalWorking)

	if _, err := m.SendPrompt("s1", term.ID, "brief", "Keep it short."); err != nil {
		t.Fatalf("sending the preset: %v", err)
	}
	cleared, err := m.CancelPrompt("s1", term.ID)
	if err != nil {
		t.Fatalf("cancelling: %v", err)
	}
	if !cleared {
		t.Fatal("cancel reported nothing to clear while a preset was pending")
	}
	if p := pending(t, m, term.ID); p != "" {
		t.Errorf("pending preset = %q after cancelling, want none", p)
	}

	setState(t, term, model.TerminalIdle)
	if term.submitDuePrompt() {
		t.Fatal("a cancelled preset was submitted on the next idle")
	}
	time.Sleep(200 * time.Millisecond)
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Errorf("a cancelled preset reached the agent: %q", b)
	}

	// Cancelling again is an ordinary no-op, not an error.
	if cleared, err := m.CancelPrompt("s1", term.ID); err != nil || cleared {
		t.Errorf("second cancel = (%v, %v), want (false, nil)", cleared, err)
	}
}

// One pending item, no more: a second activation while one is queued is refused
// and leaves the first exactly where it was.
func TestSecondPresetIsRefusedWhileOneIsPending(t *testing.T) {
	m, term, _ := promptTab(t)
	setState(t, term, model.TerminalWorking)

	if _, err := m.SendPrompt("s1", term.ID, "brief", "Keep it short."); err != nil {
		t.Fatalf("sending the first preset: %v", err)
	}
	if _, err := m.SendPrompt("s1", term.ID, "commits", "Follow the convention."); !errors.Is(err, ErrPromptPending) {
		t.Fatalf("second send = %v, want ErrPromptPending", err)
	}
	if p := pending(t, m, term.ID); p != "brief" {
		t.Errorf("pending preset = %q, want the first one %q", p, "brief")
	}
}

// An agent the operator started themselves is typed into exactly as a launched
// one is: chartr did not start this binary, and the tab is a target because the
// sampler found an agent in front of it.
func TestOperatorStartedAgentIsADeliveryTarget(t *testing.T) {
	m, term, log := promptTab(t)
	seatAdHocAgent(t, term, "claude", 4242)
	setState(t, term, model.TerminalIdle)

	if info, ok := m.Lookup(term.ID); !ok || !info.PromptTarget {
		t.Fatalf("PromptTarget = %v on a tab with an agent in front, want true", info.PromptTarget)
	}

	const body = "Answer in as few words as the question allows."
	queued, err := m.SendPrompt("s1", term.ID, "brief", body)
	if err != nil {
		t.Fatalf("sending the preset: %v", err)
	}
	if queued {
		t.Fatal("an idle agent queued the preset instead of receiving it")
	}
	waitForFile(t, log, body+"\r", 5*time.Second)
}

// The preset is aimed at a conversation, not at a PTY. When the operator quits
// their own agent, the shell behind it sits at its prompt and reads idle — where
// a delivery would be run as a command — so the pending item is dropped with the
// agent it was queued for rather than typed into what replaced it.
func TestPendingPresetIsDroppedWhenTheAgentLeaves(t *testing.T) {
	m, term, log := promptTab(t)
	seatAdHocAgent(t, term, "claude", 4242)
	setState(t, term, model.TerminalWorking)

	if _, err := m.SendPrompt("s1", term.ID, "brief", "Keep it short."); err != nil {
		t.Fatalf("sending the preset: %v", err)
	}

	// The operator quits the agent: the foreground is the shell again, which the
	// shell grammar reads as idle.
	seatAdHocAgent(t, term, "", 0)
	setState(t, term, model.TerminalIdle)

	if !term.submitDuePrompt() {
		t.Error("dropping the pending preset did not report the row as changed")
	}
	if p := pending(t, m, term.ID); p != "" {
		t.Errorf("pending preset = %q after the agent left, want none", p)
	}
	time.Sleep(200 * time.Millisecond) // well past the shrunk submit beat
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Errorf("the preset was typed at the shell the agent left behind: %q", b)
	}
}

// A preset queued for one session is not handed to the next one: a second agent
// in the same tab is a different conversation, and the operator aimed at the
// first.
func TestPendingPresetDoesNotFollowTheTabIntoAnotherSession(t *testing.T) {
	m, term, log := promptTab(t)
	seatAdHocAgent(t, term, "claude", 4242)
	setState(t, term, model.TerminalWorking)

	if _, err := m.SendPrompt("s1", term.ID, "brief", "Keep it short."); err != nil {
		t.Fatalf("sending the preset: %v", err)
	}

	seatAdHocAgent(t, term, "claude", 5353) // quit, and started another one
	setState(t, term, model.TerminalIdle)

	if !term.submitDuePrompt() {
		t.Error("dropping the pending preset did not report the row as changed")
	}
	if p := pending(t, m, term.ID); p != "" {
		t.Errorf("pending preset = %q after a new session took the tab, want none", p)
	}
	time.Sleep(200 * time.Millisecond)
	if b, _ := os.ReadFile(log); len(b) > 0 {
		t.Errorf("a preset queued for the previous session reached the new one: %q", b)
	}
}

// The narrow target, from the manager's side: a shell sitting at its own prompt
// has no agent listening, and a tab in another space is not this space's to type
// into.
func TestIneligibleTargetsAreRefused(t *testing.T) {
	useTestShell(t)
	m := NewManager(nil, nil)
	t.Cleanup(m.Shutdown)

	shell, err := m.Open("s1", t.TempDir())
	if err != nil {
		t.Fatalf("opening the shell: %v", err)
	}
	if _, err := m.SendPrompt("s1", shell.ID, "brief", "Keep it short."); !errors.Is(err, ErrNotPromptTarget) {
		t.Errorf("sending to a shell with nothing in front = %v, want ErrNotPromptTarget", err)
	}
	if info, ok := m.Lookup(shell.ID); !ok || info.PromptTarget {
		t.Errorf("shell PromptTarget = %v, want false", info.PromptTarget)
	}
	if _, err := m.SendPrompt("s2", shell.ID, "brief", "Keep it short."); !errors.Is(err, ErrNoTerminal) {
		t.Errorf("sending to another space's tab = %v, want ErrNoTerminal", err)
	}
	if _, err := m.SendPrompt("s1", "nope", "brief", "Keep it short."); !errors.Is(err, ErrNoTerminal) {
		t.Errorf("sending to an unknown tab = %v, want ErrNoTerminal", err)
	}
}

// A tab whose process is gone is no target at all, and the pending item it was
// holding dies with it rather than waiting for an idle that can never come.
func TestExitDropsTheTabAndItsPendingPreset(t *testing.T) {
	m, term, _ := promptTab(t)
	setState(t, term, model.TerminalWorking)

	if _, err := m.SendPrompt("s1", term.ID, "brief", "Keep it short."); err != nil {
		t.Fatalf("sending the preset: %v", err)
	}

	term.close()
	deadline := time.Now().Add(5 * time.Second)
	for {
		term.mu.Lock()
		alive := term.alive
		term.mu.Unlock()
		if !alive {
			break
		}
		if time.Now().After(deadline) {
			t.Fatal("the stub agent never died")
		}
		time.Sleep(20 * time.Millisecond)
	}

	info := term.info()
	if info.PendingPrompt != "" {
		t.Errorf("pending preset = %q on a dead tab, want none", info.PendingPrompt)
	}
	if info.PromptTarget {
		t.Error("a dead tab still reads as a live delivery target")
	}
	if _, err := term.sendPrompt("brief", "Keep it short."); !errors.Is(err, ErrNotPromptTarget) {
		t.Errorf("sending to a dead tab = %v, want ErrNotPromptTarget", err)
	}
}

// The submission's two writes are one unit: an operator keystroke racing the
// delivery lands after the carriage return, never between the text and it.
func TestSubmissionSerializesAgainstOtherInput(t *testing.T) {
	_, term, log := promptTab(t)

	// A submit beat long enough that an interleaving write would certainly win the
	// race if the two writes were not held together.
	openerSubmit = 300 * time.Millisecond

	const body = "Stay on the main branch."
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		term.submitPrompt(body)
	}()

	time.Sleep(100 * time.Millisecond) // the text is written; the return is not yet
	if _, err := term.Write([]byte("x")); err != nil {
		t.Fatalf("writing an operator keystroke: %v", err)
	}
	wg.Wait()

	got := waitForFile(t, log, body+"\rx", 5*time.Second)
	if strings.Contains(got, body+"x") {
		t.Errorf("an input write landed between the preset and its return: %q", got)
	}
}
