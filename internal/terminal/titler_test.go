package terminal

import (
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/transcript"
)

// These are the auto-title behaviour tests. They drive the real Manager through
// its two injected seams — a normalized transcript source in place of a
// provider's store, and a title generator in place of a paid CLI — and assert
// only what the cockpit shows and what was spent: the title in a manager
// snapshot, and the number, adapter, profile and bounded context of generator
// invocations. Nothing here asserts internal title state, how often a source is
// polled relative to a clock, or how a provider stores a transcript.
//
// The transcript seam's own grammar — which records are a completed top-level
// human turn, and which are machinery — is internal/transcript's contract and is
// tested there. Here a turn is simply an event that arrived, and an incomplete,
// interrupted, error-ended, non-text or historical turn is simply an event that
// did not.

// fakeTranscript is one tab's injected transcript: the events a test hands it,
// returned by the next Poll and then forgotten, exactly as a real watcher hands
// out what appeared since the last call. It counts polls, which is how "the
// toggle stops transcript observation" is proved without asserting a cadence.
type fakeTranscript struct {
	mu     sync.Mutex
	queued []transcript.Event
	polls  int
}

func (f *fakeTranscript) emit(events ...transcript.Event) {
	f.mu.Lock()
	f.queued = append(f.queued, events...)
	f.mu.Unlock()
}

// Poll implements TitleSource.
func (f *fakeTranscript) Poll() []transcript.Event {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.polls++
	out := f.queued
	f.queued = nil
	return out
}

func (f *fakeTranscript) polled() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.polls
}

// nativeTitle and humanTurn are the two normalized events, written the way a
// test reads best.
func nativeTitle(title string) transcript.Event {
	return transcript.Event{Kind: transcript.NativeTitle, Title: title}
}

func humanTurn(prompt, response string) transcript.Event {
	return transcript.Event{Kind: transcript.HumanTurn, Prompt: prompt, Response: response}
}

// fakeGen stands in for the paid generator. It records every request it was
// given, answers with whatever the test configured, and can be held open so a
// test can drive further events while a generation is still running.
type fakeGen struct {
	mu    sync.Mutex
	calls []TitleRequest
	title string
	ok    bool
	hold  chan struct{}
	fired chan struct{}
}

func newFakeGen(title string, ok bool) *fakeGen {
	return &fakeGen{title: title, ok: ok, fired: make(chan struct{}, 32)}
}

// generate is the injected generator. It reports every invocation on fired
// before blocking on any hold, so a test can tell "one generation ran" from "one
// generation is still running".
func (g *fakeGen) generate(req TitleRequest) (string, bool) {
	g.mu.Lock()
	g.calls = append(g.calls, req)
	hold, title, ok := g.hold, g.title, g.ok
	g.mu.Unlock()

	g.fired <- struct{}{}
	if hold != nil {
		<-hold
	}
	return title, ok
}

func (g *fakeGen) count() int {
	g.mu.Lock()
	defer g.mu.Unlock()
	return len(g.calls)
}

func (g *fakeGen) request(i int) TitleRequest {
	g.mu.Lock()
	defer g.mu.Unlock()
	return g.calls[i]
}

// titleRig is a Manager with the two seams injected and hand-driven beats: no
// PTY, no sampler goroutine, no clock. A tab is seated in the manager so the
// assertions can read the same snapshot the cockpit renders from.
type titleRig struct {
	t   *testing.T
	m   *Manager
	gen *fakeGen

	mu       sync.Mutex
	bindings map[string]TitleBinding
	terms    []*Terminal
	binds    int
}

func newTitleRig(t *testing.T) *titleRig {
	t.Helper()
	// A nil onChange keeps the background sampler out of it: this test drives
	// every beat itself.
	m := NewManager(nil, nil)
	gen := newFakeGen("Generated title", true)
	m.SetTitleGenerator(gen.generate)

	r := &titleRig{t: t, m: m, gen: gen, bindings: map[string]TitleBinding{}}
	m.bindTitles = func(term *Terminal) (TitleBinding, bool) {
		r.mu.Lock()
		defer r.mu.Unlock()
		r.binds++
		b, ok := r.bindings[term.ID]
		return b, ok
	}
	return r
}

// tab seats a live agent tab whose transcript the test writes, and returns that
// transcript. adapterName is what the tab's agent resolved to; env is the
// allowlisted provider environment its profile is selected with.
func (r *titleRig) tab(id, adapterName string, env ...string) (*Terminal, *fakeTranscript) {
	r.t.Helper()
	term := &Terminal{
		ID:            id,
		SpaceID:       "s1",
		Title:         id,
		launchedAgent: adapterName,
		agent:         adapterName,
		shellPID:      4000 + len(r.terms),
		alive:         true,
		state:         model.TerminalIdle,
		subs:          map[*subscriber]struct{}{},
		done:          make(chan struct{}),
		grid:          newGrid(gridDefaultCols, gridDefaultRows),
	}
	r.t.Cleanup(term.grid.close)

	src := &fakeTranscript{}
	r.mu.Lock()
	r.bindings[id] = TitleBinding{Adapter: adapterName, Env: env, Source: src}
	r.terms = append(r.terms, term)
	r.mu.Unlock()

	r.m.mu.Lock()
	r.m.terms[id] = term
	r.m.order = append(r.m.order, id)
	r.m.mu.Unlock()
	return term, src
}

// unwatchable seats a tab whose adapter offers neither a native title nor a safe
// one-shot recipe: nothing binds it, so nothing observes or spends.
func (r *titleRig) unwatchable(id, adapterName string) *Terminal {
	term, _ := r.tab(id, adapterName)
	r.mu.Lock()
	delete(r.bindings, id)
	r.mu.Unlock()
	return term
}

// beat runs one transcript beat over the seated tabs and reports how many paid
// generations it launched.
func (r *titleRig) beat() int {
	r.t.Helper()
	r.mu.Lock()
	terms := append([]*Terminal(nil), r.terms...)
	r.mu.Unlock()
	_, launched := r.m.titleTick(terms)
	return launched
}

// beats runs n beats and reports the total number of generations launched.
func (r *titleRig) beats(n int) int {
	r.t.Helper()
	total := 0
	for range n {
		total += r.beat()
	}
	return total
}

// title reads a tab's automatic title out of the manager snapshot — the same
// value the browser renders.
func (r *titleRig) title(id string) string {
	r.t.Helper()
	for _, info := range r.m.ForSpace("s1") {
		if info.ID == id {
			return info.AutoTitle
		}
	}
	r.t.Fatalf("no terminal %q in the space snapshot", id)
	return ""
}

// awaitTitle waits for a tab's snapshot title to reach want. Generations run off
// the beat, so a landed title is observed rather than assumed.
func (r *titleRig) awaitTitle(id, want string) {
	r.t.Helper()
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if got := r.title(id); got == want {
			return
		}
		time.Sleep(5 * time.Millisecond)
	}
	r.t.Fatalf("title of %s = %q, want %q", id, r.title(id), want)
}

// awaitGenerations waits for n generator invocations to have started.
func (r *titleRig) awaitGenerations(n int) {
	r.t.Helper()
	for i := 0; i < n; i++ {
		select {
		case <-r.gen.fired:
		case <-time.After(2 * time.Second):
			r.t.Fatalf("waited for %d generations, saw %d", n, r.gen.count())
		}
	}
}

// setAgent moves a tab to a different foreground agent in a different process —
// what an operator quitting one agent and starting another in the same shell
// does.
func (r *titleRig) setAgent(term *Terminal, adapterName string, env ...string) *fakeTranscript {
	src := &fakeTranscript{}
	r.mu.Lock()
	r.bindings[term.ID] = TitleBinding{Adapter: adapterName, Env: env, Source: src}
	r.mu.Unlock()

	term.mu.Lock()
	term.agent, term.launchedAgent = adapterName, adapterName
	term.shellPID += 1000
	term.mu.Unlock()
	return src
}

// An untouched boot spends nothing. The tab is live, its agent is up and its
// screen is painting — the four screen-derived guards this replaced would have
// armed on exactly this — but the operator has submitted nothing, so the
// transcript reports nothing and no generation is scheduled.
func TestUntouchedBootSpendsNothing(t *testing.T) {
	r := newTitleRig(t)
	term, src := r.tab("t1", "claude")
	term.grid.write([]byte("Welcome to the agent\r\n✳ Thinking… (12s · 340 tokens)\r\n"))

	if launched := r.beats(5); launched != 0 {
		t.Fatalf("an untouched boot launched %d generations, want 0", launched)
	}
	if got := r.title("t1"); got != "" {
		t.Fatalf("an untouched boot titled the tab %q, want no title", got)
	}
	if r.gen.count() != 0 {
		t.Fatalf("generator ran %d times on an untouched boot", r.gen.count())
	}
	if src.polled() == 0 {
		t.Fatal("the tab's transcript was never observed")
	}
}

// The first completed turn without a native title produces exactly one
// generation, and its title is what the cockpit shows.
func TestFirstCompletedTurnGeneratesExactlyOne(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(humanTurn("add a reorder handler to the sidebar", "Added the handler and a test."))
	if launched := r.beat(); launched != 1 {
		t.Fatalf("the first completed turn launched %d generations, want 1", launched)
	}
	r.awaitTitle("t1", "Generated title")

	if launched := r.beats(3); launched != 0 {
		t.Fatalf("idle beats after a title launched %d more generations", launched)
	}
	if r.gen.count() != 1 {
		t.Fatalf("generator ran %d times, want exactly 1", r.gen.count())
	}
}

// chartr's own opening prompt is an ordinary turn: a session tab whose first
// exchange is the injected opener takes the same single generation, with no
// launch-only title path behind it.
func TestOpenerTurnIsAnOrdinaryTurn(t *testing.T) {
	r := newTitleRig(t)
	term, src := r.tab("t1", "claude")
	term.session = &Session{MapSlug: "titles", TicketNum: 3, Role: "implement", Agent: "claude"}

	src.emit(humanTurn("Work this chartr ticket: transcript-driven titles", "Reading the ticket now."))
	if launched := r.beat(); launched != 1 {
		t.Fatalf("an opener turn launched %d generations, want 1", launched)
	}
	r.awaitTitle("t1", "Generated title")
}

// A native provider title is published as soon as it is exposed and blocks every
// paid attempt while it is there — a completed turn afterwards spends nothing.
func TestNativeTitlePublishesAndBlocksGeneration(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(nativeTitle("Read handoff ticket in chartr folder"))
	if launched := r.beat(); launched != 0 {
		t.Fatalf("a native title launched %d generations, want 0", launched)
	}
	if got := r.title("t1"); got != "Read handoff ticket in chartr folder" {
		t.Fatalf("native title = %q, want it published immediately", got)
	}

	src.emit(humanTurn("now do the other half", "Done."))
	if launched := r.beats(3); launched != 0 {
		t.Fatalf("a turn under a native title launched %d generations, want 0", launched)
	}
	if r.gen.count() != 0 {
		t.Fatalf("generator ran %d times while a native title was available", r.gen.count())
	}
}

// A native title changing is a free metadata update: it lands on the next beat,
// with no debounce between it and the one before.
func TestNativeTitleRefreshesWithoutDebounce(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(nativeTitle("First native title"))
	r.beat()
	if got := r.title("t1"); got != "First native title" {
		t.Fatalf("title = %q, want the first native title", got)
	}

	src.emit(nativeTitle("Second native title"))
	r.beat()
	if got := r.title("t1"); got != "Second native title" {
		t.Fatalf("title = %q, want the refreshed native title with no debounce", got)
	}
	if r.gen.count() != 0 {
		t.Fatalf("a native refresh spent %d generations", r.gen.count())
	}
}

// A native title is held to the cockpit's existing display contract: one line,
// and no longer than a generated one.
func TestNativeTitleNormalizedToOneLineAndLength(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(nativeTitle("  Rewrite   the\nwhole scheduler  " + strings.Repeat(" and more", 20)))
	r.beat()

	got := r.title("t1")
	if strings.ContainsAny(got, "\r\n") {
		t.Fatalf("native title %q is not a single line", got)
	}
	if n := len([]rune(got)); n > MaxTitleRunes {
		t.Fatalf("native title is %d runes, want at most %d", n, MaxTitleRunes)
	}
	if !strings.HasPrefix(got, "Rewrite the whole scheduler") {
		t.Fatalf("native title = %q, want the flattened provider title", got)
	}
}

// A generated title is a first impression and is never refreshed: later turns on
// the same session produce no further generations and do not change the label.
func TestGeneratedTitleIsNeverRefreshed(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(humanTurn("first request", "first answer"))
	r.beat()
	r.awaitTitle("t1", "Generated title")

	r.gen.mu.Lock()
	r.gen.title = "Second title"
	r.gen.mu.Unlock()

	src.emit(humanTurn("a completely different second request", "second answer"))
	src.emit(humanTurn("and a third", "third answer"))
	if launched := r.beats(3); launched != 0 {
		t.Fatalf("later turns launched %d generations, want 0", launched)
	}
	if got := r.title("t1"); got != "Generated title" {
		t.Fatalf("title = %q, want the first-impression title kept", got)
	}
}

// A session launches at most one paid attempt ever. Whatever consumed it —
// a declining ladder, output that cleaned to nothing, a cancelled run — the tab
// stays untitled and no later turn re-arms it.
func TestSpentAttemptIsNeverRetried(t *testing.T) {
	cases := []struct {
		name  string
		title string
		ok    bool
	}{
		{"declined", "", false},
		{"invalid output", "   ", true},
		{"cancelled", "", false},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			r := newTitleRig(t)
			r.gen.title, r.gen.ok = tc.title, tc.ok
			_, src := r.tab("t1", "claude")

			src.emit(humanTurn("the request", "the answer"))
			if launched := r.beat(); launched != 1 {
				t.Fatalf("the first turn launched %d generations, want 1", launched)
			}
			r.awaitGenerations(1)

			src.emit(humanTurn("another request", "another answer"))
			src.emit(humanTurn("and another", "and another answer"))
			if launched := r.beats(4); launched != 0 {
				t.Fatalf("a spent attempt was retried %d times", launched)
			}
			if got := r.title("t1"); got != "" {
				t.Fatalf("title = %q, want the tab left untitled", got)
			}
			if r.gen.count() != 1 {
				t.Fatalf("generator ran %d times, want exactly the one spent attempt", r.gen.count())
			}
		})
	}
}

// Generation is same-adapter and same-profile: each tab's request carries its own
// adapter and only the allowlisted provider environment that selects its resolved
// state root, so a custom account's conversation is never sent through another.
func TestGenerationCarriesTheTabsAdapterAndProfile(t *testing.T) {
	r := newTitleRig(t)
	_, first := r.tab("t1", "claude", "CLAUDE_CONFIG_DIR=/home/op/.claude")
	_, second := r.tab("t2", "claude", "CLAUDE_CONFIG_DIR=/home/op/.claude-work")

	first.emit(humanTurn("personal work", "personal answer"))
	second.emit(humanTurn("work account work", "work answer"))
	if launched := r.beat(); launched != 2 {
		t.Fatalf("two first turns launched %d generations, want 2", launched)
	}
	r.awaitGenerations(2)

	profiles := map[string]bool{}
	for i := range 2 {
		req := r.gen.request(i)
		if req.Adapter != "claude" {
			t.Fatalf("request %d ran on adapter %q, want the tab's own", i, req.Adapter)
		}
		if len(req.Env) != 1 {
			t.Fatalf("request %d carried %d environment entries, want only the state-root one", i, len(req.Env))
		}
		profiles[req.Env[0]] = true
	}
	if !profiles["CLAUDE_CONFIG_DIR=/home/op/.claude"] || !profiles["CLAUDE_CONFIG_DIR=/home/op/.claude-work"] {
		t.Fatalf("profiles = %v, want each tab's own resolved state root", profiles)
	}
}

// A tab whose adapter offers neither a native title nor a safe one-shot recipe
// binds to nothing: it is never observed, never spent on, and never titled — and
// no other vendor is reached for.
func TestUnwatchableAdapterStaysUntitled(t *testing.T) {
	r := newTitleRig(t)
	r.unwatchable("t1", "somenewagent")

	if launched := r.beats(4); launched != 0 {
		t.Fatalf("an unwatchable adapter launched %d generations", launched)
	}
	if got := r.title("t1"); got != "" {
		t.Fatalf("title = %q, want an unwatchable tab left untitled", got)
	}
	if r.gen.count() != 0 {
		t.Fatalf("generator ran %d times for an unwatchable adapter", r.gen.count())
	}
}

// The privacy boundary. Everything the tab holds that is not the titled turn —
// the reconstructed screen with its system preamble, hidden reasoning, tool call
// and tool result, the OSC title the agent painted, and a second turn that
// arrived in the same batch — is recognizable by a sentinel, and none of it may
// reach the generator. Only the operator's own prompt and the final visible
// answer to it do.
func TestGeneratorContextCarriesOnlyTheTitledTurn(t *testing.T) {
	r := newTitleRig(t)
	term, src := r.tab("t1", "claude")

	term.grid.write([]byte(
		"SYSTEM_SENTINEL you are a coding agent\r\n" +
			"REASONING_SENTINEL the user probably wants\r\n" +
			"TOOLCALL_SENTINEL Bash(ls -la)\r\n" +
			"TOOLRESULT_SENTINEL total 48 drwxr-xr-x\r\n"))
	term.mu.Lock()
	term.oscTitle = "OSCTITLE_SENTINEL"
	term.mu.Unlock()

	src.emit(
		humanTurn("PROMPT_SENTINEL rename the publish helper", "ANSWER_SENTINEL renamed it in three files."),
		humanTurn("LATERPROMPT_SENTINEL now delete it", "LATERANSWER_SENTINEL deleted."),
	)
	if launched := r.beat(); launched != 1 {
		t.Fatalf("two turns in one batch launched %d generations, want 1", launched)
	}
	r.awaitGenerations(1)

	got := r.gen.request(0).Context
	for _, want := range []string{"PROMPT_SENTINEL", "ANSWER_SENTINEL"} {
		if !strings.Contains(got, want) {
			t.Fatalf("context %q is missing %s — both sides of the turn must reach the generator", got, want)
		}
	}
	for _, forbidden := range []string{
		"SYSTEM_SENTINEL", "REASONING_SENTINEL", "TOOLCALL_SENTINEL", "TOOLRESULT_SENTINEL",
		"OSCTITLE_SENTINEL", "LATERPROMPT_SENTINEL", "LATERANSWER_SENTINEL",
	} {
		if strings.Contains(got, forbidden) {
			t.Fatalf("context leaked %s: %q", forbidden, got)
		}
	}
}

// The context stays inside the title-context budget however large the turn was,
// and still preserves useful text from both sides rather than spending the whole
// budget on a pasted prompt.
func TestGeneratorContextIsBounded(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(humanTurn(
		"PROMPT_HEAD "+strings.Repeat("p", 4000),
		"ANSWER_HEAD "+strings.Repeat("a", 4000)))
	r.beat()
	r.awaitGenerations(1)

	got := r.gen.request(0).Context
	if n := len([]rune(got)); n > titleContextCap {
		t.Fatalf("context is %d runes, want at most %d", n, titleContextCap)
	}
	if !strings.Contains(got, "PROMPT_HEAD") || !strings.Contains(got, "ANSWER_HEAD") {
		t.Fatalf("context %q dropped one side of the turn", got)
	}
}

// The machine-wide toggle gates the whole feature: with it off nothing is
// observed and nothing is spent, while a title already on screen stays there.
// Turning it back on resumes both.
func TestToggleOffStopsObservationAndGeneration(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	src.emit(nativeTitle("A native title"))
	r.beat()
	if got := r.title("t1"); got != "A native title" {
		t.Fatalf("title = %q, want the native title published", got)
	}

	r.m.ConfigureAutoTitle(false)
	before := src.polled()
	src.emit(humanTurn("a request while titling is off", "an answer"))
	if launched := r.beats(3); launched != 0 {
		t.Fatalf("titling was off and launched %d generations", launched)
	}
	if src.polled() != before {
		t.Fatalf("the transcript was observed %d times while titling was off", src.polled()-before)
	}
	if got := r.title("t1"); got != "A native title" {
		t.Fatalf("title = %q, want an already-displayed title to remain", got)
	}

	// Back on, the tab is observed again — from a fresh binding, so nothing that
	// happened while the feature was off is charged for.
	r.m.ConfigureAutoTitle(true)
	r.beat()
	if src.polled() == before {
		t.Fatal("the transcript was not observed again after the toggle came back on")
	}
}

// Turns that never complete — still running, interrupted, ended in an error, or
// prompts that were not text-only — never arrive as events at all, so nothing is
// scheduled for them.
func TestIncompleteTurnsSpendNothing(t *testing.T) {
	r := newTitleRig(t)
	_, src := r.tab("t1", "claude")

	// The transcript reports nothing for any of them; a beat over an empty poll
	// must therefore change nothing and spend nothing.
	src.emit()
	if launched := r.beats(5); launched != 0 {
		t.Fatalf("incomplete turns launched %d generations, want 0", launched)
	}
	if got := r.title("t1"); got != "" {
		t.Fatalf("title = %q, want no title for turns that never completed", got)
	}
}

// A process that exits keeps the title it earned and stops being observed: a
// dead tab is not a conversation, and nothing more can be spent on it.
func TestProcessExitKeepsTheTitleAndStopsObserving(t *testing.T) {
	r := newTitleRig(t)
	term, src := r.tab("t1", "claude")

	src.emit(nativeTitle("Its own title"))
	r.beat()
	if got := r.title("t1"); got != "Its own title" {
		t.Fatalf("title = %q, want the native title", got)
	}

	term.mu.Lock()
	term.alive = false
	term.mu.Unlock()

	before := src.polled()
	src.emit(humanTurn("a turn after death", "an answer"))
	if launched := r.beats(3); launched != 0 {
		t.Fatalf("a dead tab launched %d generations", launched)
	}
	if src.polled() != before {
		t.Fatal("a dead tab was still observed")
	}
	if got := r.title("t1"); got != "Its own title" {
		t.Fatalf("title = %q, want the earned title kept after exit", got)
	}
}

// An operator who quits one agent and starts another in the same shell is on a
// different conversation: the old title goes, and the new agent gets its own
// single attempt rather than inheriting a spent one.
func TestAgentChangeInAnEmptyShellStartsOver(t *testing.T) {
	r := newTitleRig(t)
	term, src := r.tab("t1", "claude")

	src.emit(humanTurn("the claude request", "the claude answer"))
	r.beat()
	r.awaitTitle("t1", "Generated title")

	r.gen.mu.Lock()
	r.gen.title = "Codex title"
	r.gen.mu.Unlock()
	next := r.setAgent(term, "codex", "CODEX_HOME=/home/op/.codex")

	if launched := r.beat(); launched != 0 {
		t.Fatalf("seating a new agent launched %d generations before any turn", launched)
	}
	if got := r.title("t1"); got != "" {
		t.Fatalf("title = %q, want the previous agent's title dropped", got)
	}

	next.emit(humanTurn("the codex request", "the codex answer"))
	if launched := r.beat(); launched != 1 {
		t.Fatalf("the new agent's first turn launched %d generations, want 1", launched)
	}
	r.awaitTitle("t1", "Codex title")
	if req := r.gen.request(1); req.Adapter != "codex" || len(req.Env) != 1 || req.Env[0] != "CODEX_HOME=/home/op/.codex" {
		t.Fatalf("second request = %+v, want the new agent's own adapter and profile", req)
	}
}

// A native title that appears while a paid generation is still running wins: it
// is current and free, and the generation it overtook does not overwrite it.
func TestNativeTitleWinsOverAPendingGeneration(t *testing.T) {
	r := newTitleRig(t)
	hold := make(chan struct{})
	r.gen.mu.Lock()
	r.gen.hold = hold
	r.gen.mu.Unlock()
	_, src := r.tab("t1", "claude")

	src.emit(humanTurn("the request", "the answer"))
	if launched := r.beat(); launched != 1 {
		t.Fatalf("the first turn launched %d generations, want 1", launched)
	}
	r.awaitGenerations(1)

	src.emit(nativeTitle("The provider's own title"))
	r.beat()
	if got := r.title("t1"); got != "The provider's own title" {
		t.Fatalf("title = %q, want the native title published while the generation ran", got)
	}

	close(hold)
	// Give the generation every chance to land on top of the native title.
	deadline := time.Now().Add(200 * time.Millisecond)
	for time.Now().Before(deadline) {
		if got := r.title("t1"); got != "The provider's own title" {
			t.Fatalf("title = %q, want the native title to win over the pending generation", got)
		}
		time.Sleep(5 * time.Millisecond)
	}
}

// The context builder's own contract: whatever the split, both sides survive and
// the total stays inside the budget.
func TestTitleContextKeepsBothSidesWithinBudget(t *testing.T) {
	cases := []struct {
		name               string
		prompt, response   string
		wantHas            []string
		wantAtMostFraction bool
	}{
		{
			name:     "short turn is carried whole",
			prompt:   "rename the publish helper",
			response: "renamed it in three files",
			wantHas:  []string{"rename the publish helper", "renamed it in three files"},
		},
		{
			name:     "a pasted prompt cannot eat the answer",
			prompt:   strings.Repeat("p", 6000),
			response: "ANSWER_HEAD the fix was a missing newline",
			wantHas:  []string{"ANSWER_HEAD the fix was a missing newline"},
		},
		{
			name:     "a long answer cannot eat the prompt",
			prompt:   "PROMPT_HEAD why is the sidebar flickering",
			response: strings.Repeat("a", 6000),
			wantHas:  []string{"PROMPT_HEAD why is the sidebar flickering"},
		},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := titleContext(tc.prompt, tc.response)
			if n := len([]rune(got)); n > titleContextCap {
				t.Fatalf("context is %d runes, want at most %d", n, titleContextCap)
			}
			for _, want := range tc.wantHas {
				if !strings.Contains(got, want) {
					t.Fatalf("context %q is missing %q", got, want)
				}
			}
		})
	}
}
