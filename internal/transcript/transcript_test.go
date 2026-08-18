package transcript

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/proc"
)

// These prove the subsystem's own behaviour — what a consumer sees around an
// adapter — with a hand-written adapter in place of a provider, so the rules
// below hold for every provider that ever joins the table.

// fakeAdapter binds on demand and hands out scripted polls.
type fakeAdapter struct {
	binds   int
	refuse  bool
	session *fakeSession
}

type fakeSession struct {
	id     string
	polls  int
	script [][]Event
	alive  int // polls before the binding ends; 0 means it never does
}

func (a *fakeAdapter) Bind(proc.Agent) (Session, bool) {
	if a.refuse {
		return nil, false
	}
	a.binds++
	s := *a.session
	return &s, true
}

func (s *fakeSession) ID() string { return s.id }

func (s *fakeSession) Poll() ([]Event, bool) {
	var events []Event
	if s.polls < len(s.script) {
		events = s.script[s.polls]
	}
	s.polls++
	return events, s.alive == 0 || s.polls < s.alive
}

func agentFor(t *testing.T) proc.Agent {
	t.Helper()
	return proc.Agent{Adapter: "claude", PID: 99, PGID: 99, Dir: t.TempDir(), StateRoot: t.TempDir()}
}

// An adapter that cannot bind is asked again on every poll — which is what
// re-checks an ambiguous match when new writes appear — and says nothing in the
// meantime.
func TestAnUnboundWatcherKeepsAsking(t *testing.T) {
	ad := &fakeAdapter{refuse: true}
	w := watch(agentFor(t), ad)

	for i := range 3 {
		if got := w.Poll(); got != nil {
			t.Fatalf("poll %d of an unbound watcher produced %+v", i, got)
		}
	}
	if w.Session() != "" {
		t.Fatalf("an unbound watcher named session %q", w.Session())
	}
	if ad.binds != 0 {
		t.Fatalf("a refusing adapter recorded %d bindings", ad.binds)
	}
}

// A binding that ends is dropped and made again, so a session that changes under
// a live tab is picked up without the consumer knowing anything happened.
func TestAWatcherRebindsWhenABindingEnds(t *testing.T) {
	ad := &fakeAdapter{session: &fakeSession{
		id: "session-1",
		script: [][]Event{{{Kind: TurnFinished, Outcome: OutcomeCompleted,
			Prompt: "a question", Response: "an answer"}}},
		alive: 2,
	}}
	w := watch(agentFor(t), ad)

	if got := turnsOf(w.Poll()); len(got) != 1 {
		t.Fatalf("the first poll produced %+v", got)
	}
	w.Poll() // the poll that ends the binding
	w.Poll() // binds again
	if ad.binds != 2 {
		t.Fatalf("the watcher bound %d times, want a fresh binding after the old one ended", ad.binds)
	}
	if w.Session() != "session-1" {
		t.Fatalf("rebound session is %q", w.Session())
	}
}

// A watcher hands its consumer exactly what the adapter produced, in order.
func TestEventsPassThroughUnchanged(t *testing.T) {
	want := []Event{
		{Kind: NativeTitle, Title: "A persisted title"},
		{Kind: TurnFinished, Outcome: OutcomeCompleted, Prompt: "a question", Response: "an answer"},
	}
	ad := &fakeAdapter{session: &fakeSession{id: "session-1", script: [][]Event{want}}}
	w := watch(agentFor(t), ad)

	got := w.Poll()
	if len(got) != len(want) {
		t.Fatalf("polled %d events, want %d", len(got), len(want))
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("event %d came out as %+v, want %+v", i, got[i], want[i])
		}
	}
}

// An agent with no process behind it is not watchable, whatever its adapter.
func TestAnAgentWithNoProcessIsNotWatchable(t *testing.T) {
	agent := agentFor(t)
	agent.PID = 0
	if _, ok := Watch(agent); ok {
		t.Fatal("watched an agent with no pid")
	}
}

// OpenCode remains a supported terminal agent, but its database-backed session
// store is deliberately outside this small best-effort feature. Not watching it
// is the cheap failure and keeps SQLite infrastructure out of chartr.
func TestOpenCodeIsNotTranscriptWatchable(t *testing.T) {
	agent := proc.Agent{Adapter: "opencode", PID: 99}
	if Supported(agent.Adapter) {
		t.Fatal("opencode unexpectedly reports transcript support")
	}
	if w, ok := Watch(agent); ok {
		t.Fatalf("watching opencode returned %+v, want no watcher", w)
	}
}

// The seam's own bound is rune-safe: a multibyte glyph is never cut in half, and
// the provider's padding is not spent against the budget.
func TestTextIsBoundedWithoutSplittingAGlyph(t *testing.T) {
	got := head(strings.Repeat("ø", textCap+50)+"  ", textCap)
	if n := len([]rune(got)); n != textCap {
		t.Fatalf("bounded text is %d runes, want %d", n, textCap)
	}
	if strings.ContainsRune(got, '�') {
		t.Fatal("bounding split a multibyte glyph")
	}
	if short := head("  a short line  ", textCap); short != "a short line" {
		t.Fatalf("short text came out as %q", short)
	}
}
