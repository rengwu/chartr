package transcript

import "testing"

// Binding is the part of this package that can be wrong in the expensive
// direction. Reading a transcript badly costs a title; binding a tab to the
// wrong transcript shows the operator someone else's conversation and pays a
// model to summarise it. So these tests are the matrix of situations discovery
// has to tell apart, driven through the public Watcher exactly as the cockpit
// will drive it.

// The registry is the direct process-to-session mapping, and it is preferred
// precisely because it answers the question the filesystem cannot: two sessions
// in one directory, both written to seconds ago, and only one of them this
// process's.
func TestBindingPrefersTheProcessRegistry(t *testing.T) {
	mine := newClaudeFixture(t)
	mine.Start()
	mine.Turn("an old question of mine", "an old answer of mine")

	other := mine.peer(4343, "99999999-8888-4777-8666-555555555555")
	other.Start()
	other.Turn("the other tab's question", "the other tab's answer")

	w, ok := Watch(mine.Agent())
	if !ok {
		t.Fatal("claude is not watchable")
	}
	w.Poll()
	if w.Session() != mine.id {
		t.Fatalf("bound session %q, want this process's own %q", w.Session(), mine.id)
	}

	mine.Turn("my new question", "my new answer")
	other.Turn("the other tab's new question", "the other tab's new answer")
	got := turnsOf(w.Poll())
	if len(got) != 1 || got[0].Prompt != "my new question" {
		t.Fatalf("polled %+v, want only this tab's own turn", got)
	}
}

// Without a registry there is still an answer, as long as it is unique: a
// session file written since the agent started, whose own records name the
// directory the agent is working in.
func TestBindingMatchesByDirectoryAndObservedWrites(t *testing.T) {
	s := newClaudeFixture(t)
	s.startBare()
	s.Turn("an old question", "an old answer")

	w, _ := Watch(s.Agent())
	w.Poll()
	if w.Session() != s.id {
		t.Fatalf("bound %q, want the session written in this directory (%q)", w.Session(), s.id)
	}

	s.Turn("the new question", "the new answer")
	if got := turnsOf(w.Poll()); len(got) != 1 || got[0].Prompt != "the new question" {
		t.Fatalf("polled %+v, want the one new turn", got)
	}
}

// A session nobody has written to since this agent started is nobody's live
// session. Together with a registry entry left behind by a previous holder of
// the pid, this is the pid-reuse case: neither route may answer.
func TestBindingRefusesAStaleRegistryAndAnIdleStore(t *testing.T) {
	s := newClaudeFixture(t)
	s.startBare()
	s.Turn("a question from the process that had this pid before", "an answer to it")
	s.idle()
	s.stale(s.id)

	w, _ := Watch(s.Agent())
	if got := w.Poll(); len(got) != 0 {
		t.Fatalf("a stale registry entry produced %+v", got)
	}
	if w.Session() != "" {
		t.Fatalf("bound %q from a session that predates the process", w.Session())
	}
}

// A tab chartr launched is bound during its agent's startup, before the session
// has been written to at all. The end of an unwritten transcript is offset zero,
// so that tab — unlike a resumed one — sees its own opening turn.
func TestBindingBeforeTheFirstWriteSeesTheOpeningTurn(t *testing.T) {
	s := newClaudeFixture(t)
	s.announce()

	w, _ := Watch(s.Agent())
	if got := w.Poll(); len(got) != 0 {
		t.Fatalf("a session with nothing written produced %+v", got)
	}
	if w.Session() != s.id {
		t.Fatalf("bound %q before the first write, want %q", w.Session(), s.id)
	}

	s.startBare()
	s.Turn("the opening prompt", "the opening answer")
	got := turnsOf(w.Poll())
	if len(got) != 1 || got[0].Prompt != "the opening prompt" {
		t.Fatalf("polled %+v, want the opening turn", got)
	}
}

// The other half of that rule: a prompt already persisted when chartr arrives
// stays behind the cursor, and the answer that lands afterwards does not rescue
// it. This is what leaves a manually started agent's launch-command prompt
// untitled instead of comparing timestamps to decide whether it counts.
func TestAPromptPersistedBeforeBindingStaysBehindTheCursor(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	s.prompt("the prompt that was already there")

	w, _ := Watch(s.Agent())
	w.Poll()

	s.answer("the answer that landed afterwards")
	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a turn that began before binding produced %+v", got)
	}

	s.Turn("a question asked after binding", "its answer")
	if got := turnsOf(w.Poll()); len(got) != 1 || got[0].Prompt != "a question asked after binding" {
		t.Fatalf("polled %+v, want the first turn that began after binding", got)
	}
}

// Resuming a persisted session is the case the whole cursor rule exists for: the
// title comes back for free, and none of the work already done is offered up to
// be charged for again.
func TestResumingSurfacesTheTitleAndNoHistory(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	s.Turn("the first question of yesterday", "its answer")
	s.Turn("the second question of yesterday", "its answer")
	s.Title("Yesterday's work")

	w, _ := Watch(s.Agent())
	events := w.Poll()
	if got := titlesOf(events); len(got) != 1 || got[0] != "Yesterday's work" {
		t.Fatalf("resume published %q, want the persisted title", got)
	}
	if got := turnsOf(events); len(got) != 0 {
		t.Fatalf("resume offered %+v to be titled", got)
	}
	if got := w.Poll(); len(got) != 0 {
		t.Fatalf("a quiet resumed session kept producing %+v", got)
	}
}

// Two agents of one adapter in one space, each with its own conversation. Each
// tab must see its own and only its own.
func TestTwoTabsInOneSpaceEachSeeTheirOwnSession(t *testing.T) {
	a := newClaudeFixture(t)
	a.Start()
	b := a.peer(4343, "99999999-8888-4777-8666-555555555555")
	b.Start()

	wa, _ := Watch(a.Agent())
	wb, _ := Watch(b.Agent())
	wa.Poll()
	wb.Poll()

	a.Turn("the first tab's question", "the first tab's answer")
	b.Turn("the second tab's question", "the second tab's answer")

	gotA := turnsOf(wa.Poll())
	gotB := turnsOf(wb.Poll())
	if len(gotA) != 1 || gotA[0].Prompt != "the first tab's question" {
		t.Fatalf("the first tab polled %+v", gotA)
	}
	if len(gotB) != 1 || gotB[0].Prompt != "the second tab's question" {
		t.Fatalf("the second tab polled %+v", gotB)
	}
}

// Where they cannot be told apart, both stay untitled — silently, and for as
// long as the ambiguity lasts. The re-check is the poll itself: the moment the
// candidates change, the question is asked again.
func TestPersistentAmbiguityBindsNothingUntilItResolves(t *testing.T) {
	a := newClaudeFixture(t)
	a.startBare()
	a.Turn("one candidate's question", "one candidate's answer")

	b := a.peer(4343, "99999999-8888-4777-8666-555555555555")
	b.startBare()
	b.Turn("the other candidate's question", "the other candidate's answer")

	w, _ := Watch(a.Agent())
	for i := range 3 {
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("poll %d of an ambiguous match produced %+v", i, got)
		}
		if w.Session() != "" {
			t.Fatalf("poll %d bound %q while two candidates matched", i, w.Session())
		}
	}

	b.Remove()
	w.Poll()
	if w.Session() != a.id {
		t.Fatalf("bound %q once the ambiguity cleared, want %q", w.Session(), a.id)
	}
	a.Turn("a question once the ambiguity cleared", "its answer")
	if got := turnsOf(w.Poll()); len(got) != 1 {
		t.Fatalf("polled %+v after the ambiguity cleared", got)
	}
}

// A process that moves to another session — /clear, or a resume into a new
// conversation — is no longer driving the one this binding names. The old
// binding ends and the new one is seated at the new session's end, so nothing
// already in it is offered up.
func TestASessionThatChangesIsReboundAtItsEnd(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	s.Turn("a question before clearing", "its answer")

	w, _ := Watch(s.Agent())
	w.Poll()
	if w.Session() != s.id {
		t.Fatalf("bound %q, want %q", w.Session(), s.id)
	}
	was := s.id

	s.clearTo("77777777-6666-4555-8444-333333333333")
	s.startBare()
	if got := w.Poll(); len(got) != 0 {
		t.Fatalf("the poll that noticed the change produced %+v", got)
	}
	w.Poll()
	if w.Session() == was || w.Session() != s.id {
		t.Fatalf("bound %q after the session changed, want %q", w.Session(), s.id)
	}

	s.Turn("a question after clearing", "its answer")
	if got := turnsOf(w.Poll()); len(got) != 1 || got[0].Prompt != "a question after clearing" {
		t.Fatalf("polled %+v after the session changed", got)
	}
}

// A store that disappears — persistence turned off, history cleaned up — is an
// expected steady state, not an error. It goes quiet and stays quiet.
func TestASessionThatDisappearsGoesQuiet(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	s.Turn("a question", "an answer")

	w, _ := Watch(s.Agent())
	w.Poll()
	s.Remove()

	for i := range 3 {
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("poll %d of a removed store produced %+v", i, got)
		}
	}
}

// An agent chartr holds no transcript knowledge of is not watchable at all,
// which is how the four unmeasured providers behave until their adapters land.
func TestUnknownAdaptersAreNotWatchable(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	agent := s.Agent()
	agent.Adapter = "codex"

	if Supported(agent.Adapter) {
		t.Fatalf("%s reports transcript support with no adapter behind it", agent.Adapter)
	}
	if w, ok := Watch(agent); ok {
		t.Fatalf("watching %s returned %+v, want no watcher", agent.Adapter, w)
	}
}
