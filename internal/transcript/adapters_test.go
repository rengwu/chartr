package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// The shared contract says every adapter behaves the same way. These are the
// things it cannot say: the one filter each provider needs that nobody else does,
// and the two facts about binding that only make sense per provider.
//
// Each of them is a rule that, if it were wrong, would cost the operator money or
// show them the wrong conversation — which is why they are pinned here rather than
// left to the contract's general shape.

// A store nobody has written to since the agent started belongs to nobody live.
// Together with two tabs in one directory, this is the whole of what the five
// registry-less providers have to tell apart.
func TestBindingIgnoresAStoreOlderThanTheAgent(t *testing.T) {
	for _, c := range contractCases {
		t.Run(c.name, func(t *testing.T) {
			s := c.store(t)
			s.Start()
			s.Turn("a question from the session that was already here", "its answer")

			// The agent started after everything in the store was written.
			agent := s.Agent()
			agent.Started = time.Now().Add(time.Hour)

			w := watch(agent, c.adapter)
			for range 3 {
				if got := w.Poll(); len(got) != 0 {
					t.Fatalf("an idle store produced %+v", got)
				}
			}
			if w.Session() != "" {
				t.Fatalf("bound %q from a store that predates the agent", w.Session())
			}
		})
	}
}

// Two tabs of one provider in two directories: the case the registry-less five
// *can* tell apart, and must.
func TestTwoDirectoriesEachBindTheirOwnSession(t *testing.T) {
	for _, c := range contractCases {
		t.Run(c.name, func(t *testing.T) {
			a, b := c.store(t), c.store(t)
			a.Start()
			b.Start()

			wa, wb := watch(a.Agent(), c.adapter), watch(b.Agent(), c.adapter)
			wa.Poll()
			wb.Poll()

			a.Turn("the first directory's question", "the first directory's answer")
			b.Turn("the second directory's question", "the second directory's answer")

			gotA, gotB := turnsOf(wa.Poll()), turnsOf(wb.Poll())
			if len(gotA) != 1 || gotA[0].Prompt != "the first directory's question" {
				t.Fatalf("the first tab polled %+v", gotA)
			}
			if len(gotB) != 1 || gotB[0].Prompt != "the second directory's question" {
				t.Fatalf("the second tab polled %+v", gotB)
			}
		})
	}
}

// Pi's file does not exist until its first answer does, and it appears with the
// opening prompt already inside it. Under the settled seat-at-end rule that turn
// is history, so a Pi tab's first turn is never titled and a single-prompt session
// stays untitled — the cheap failure, deliberately chosen over deciding that an
// absent store counts as a bound one.
func TestPiCannotTitleItsFirstTurn(t *testing.T) {
	s := newPiFixture(t)
	w := watch(s.Agent(), pi{})
	if got := w.Poll(); len(got) != 0 {
		t.Fatalf("a session with no file yet produced %+v", got)
	}

	// The provider's first flush: header, opening prompt and its answer, all at
	// once. Nothing existed for chartr to bind to before it.
	s.Start()
	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("Pi's opening turn was offered up to be titled: %+v", got)
	}
	if w.Session() == "" {
		t.Fatal("the session did not bind once its file appeared")
	}

	// Every turn after it is ordinary.
	s.Turn("the second question", "the second answer")
	got := turnsOf(w.Poll())
	if len(got) != 1 || got[0].Prompt != "the second question" {
		t.Fatalf("polled %+v, want the second turn", got)
	}
}

// Kimi's title field holds a title only when it says so. The default kind is the
// prompt itself, capped at 200 characters, and publishing it would put a whole
// prompt in the tab label and block paid generation for the life of the session.
func TestKimiPublishesOnlyARealTitle(t *testing.T) {
	s := newKimiFixture(t)
	s.Start()
	w := watch(s.Agent(), kimi{})
	w.Poll()

	s.replaceable("the operator's entire first prompt, wearing the title field")
	if got := titlesOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a replaceable title was published: %q", got)
	}

	// The kind Kimi's own generation sets, when its feature flag and its
	// provider allow it to run at all.
	s.writeState(map[string]any{
		"id": s.id, "version": 2, "cwd": s.dir,
		"title": "A generated title", "titleKind": "generated", "isCustomTitle": false,
	})
	if got := titlesOf(w.Poll()); len(got) != 1 || got[0] != "A generated title" {
		t.Fatalf("a generated title published %q", got)
	}

	// A record from before titleKind existed: only an operator's own rename
	// counts.
	s.writeState(map[string]any{
		"id": s.id, "version": 2, "cwd": s.dir,
		"title": "A legacy prompt copy", "isCustomTitle": false,
	})
	if got := titlesOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a legacy non-custom title was published: %q", got)
	}
}

// OpenCode names a new session with a placeholder, and a native title blocks paid
// generation for as long as it stands — so publishing the placeholder would pin
// the tab to a timestamp forever.
func TestOpenCodeRefusesThePlaceholderTitle(t *testing.T) {
	s := newOpencodeFixture(t)
	s.Start() // inserts the session with its placeholder title
	w := watch(s.Agent(), opencode{})
	if got := titlesOf(w.Poll()); len(got) != 0 {
		t.Fatalf("the placeholder title was published: %q", got)
	}
	// The same shape, on a child session, is a placeholder too.
	s.Title("Child session - 2026-08-16T10:36:14.000Z")
	if got := titlesOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a child placeholder was published: %q", got)
	}
	s.Title("The title OpenCode generated")
	if got := titlesOf(w.Poll()); len(got) != 1 || got[0] != "The title OpenCode generated" {
		t.Fatalf("a real title published %q", got)
	}
}

// The one thing a database store does that an append-only log cannot: a row is
// rewritten in place while the answer streams. A reader that trusted what it first
// saw would title the tab from half a sentence.
func TestOpenCodeReadsAStreamedAnswerAtItsFinalValue(t *testing.T) {
	s := newOpencodeFixture(t)
	s.Start()
	w := watch(s.Agent(), opencode{})
	w.Poll()

	s.beginTurn("the question", "the ans")
	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a turn still streaming produced %+v", got)
	}
	s.growPart("the answer, once it had finished streaming")
	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a turn with no closing step produced %+v", got)
	}
	s.finishTurn()

	got := turnsOf(w.Poll())
	if len(got) != 1 {
		t.Fatalf("the finished turn produced %+v", got)
	}
	if got[0].Response != "the answer, once it had finished streaming" {
		t.Fatalf("the answer came out as %q, want its final value", got[0].Response)
	}
}

// Codex's two record families are selected by a field that is not a version, and
// one build writes both. A rollout that names neither is not a store to guess at.
func TestCodexRefusesAnUnknownHistoryMode(t *testing.T) {
	for _, mode := range []string{"", "some-future-mode"} {
		t.Run("history_mode="+mode, func(t *testing.T) {
			s := newCodexFixture(t, codexPaginated)
			s.mode = mode
			s.Start()
			s.mode = codexPaginated
			s.Turn("a question", "an answer")

			w := watch(s.Agent(), codex{})
			for range 3 {
				if got := w.Poll(); len(got) != 0 {
					t.Fatalf("a rollout with history_mode %q produced %+v", mode, got)
				}
			}
			if w.Session() != "" {
				t.Fatalf("bound %q from a rollout this adapter cannot read", w.Session())
			}
		})
	}
}

// A subagent's rollout and a `codex exec` run — which is how chartr's own title
// generation runs — are ordinary files in the same tree with the same working
// directory. Both are excluded by the head record, before a line of conversation
// is read.
func TestCodexBindsOnlyTheInteractiveThread(t *testing.T) {
	for _, kind := range []struct {
		name  string
		patch func(map[string]any)
	}{
		{"a subagent's thread", func(m map[string]any) { m["thread_source"] = "subagent" }},
		{"a codex exec run", func(m map[string]any) { m["originator"] = "codex_exec" }},
	} {
		t.Run(kind.name, func(t *testing.T) {
			mine := newCodexFixture(t, codexPaginated)
			mine.Start()

			// A second rollout in the same directory, written since the agent
			// started, that a tab must not bind to.
			other, _ := mine.Peer()
			theirs := other.(*codexStore)
			theirs.Start()
			theirs.patchHead(kind.patch)

			w := watch(mine.Agent(), codex{})
			w.Poll()
			if w.Session() != mine.id {
				t.Fatalf("bound %q, want this tab's own thread %q", w.Session(), mine.id)
			}
			mine.Turn("my question", "my answer")
			theirs.Turn("the other thread's question", "the other thread's answer")

			got := turnsOf(w.Poll())
			if len(got) != 1 || got[0].Prompt != "my question" {
				t.Fatalf("polled %+v, want only this thread's turn", got)
			}
		})
	}
}

// Grok's headless mode — which is how chartr's own title generation runs — writes
// a chat history and no update log. Keying on the update log excludes it for free,
// which is the cleanest version of that hazard among the five.
func TestGrokIgnoresAHeadlessRun(t *testing.T) {
	mine := newGrokFixture(t)
	mine.Start()

	// A headless run in the same directory: a session directory, a summary, a
	// chat history, and no updates.jsonl.
	other, _ := mine.Peer()
	headless := other.(*grokStore)
	headless.writeSummary(nil)
	headless.aside("chat_history.jsonl",
		headless.line(map[string]any{"role": "user", "content": "the prompt chartr generated a title from"}),
		headless.line(map[string]any{"role": "assistant", "content": "A Title"}),
	)

	w := watch(mine.Agent(), grok{})
	w.Poll()
	if w.Session() != mine.id {
		t.Fatalf("bound %q, want this tab's own session %q", w.Session(), mine.id)
	}
	mine.Turn("my question", "my answer")
	got := turnsOf(w.Poll())
	if len(got) != 1 || got[0].Prompt != "my question" {
		t.Fatalf("polled %+v, want only this tab's turn", got)
	}
}

// Grok streams both halves of a turn in chunks, and the chunks have to be put back
// together. A reader that took the first one would title the tab from a fragment.
func TestGrokConcatenatesChunks(t *testing.T) {
	s := newGrokFixture(t)
	s.Start()
	w := watch(s.Agent(), grok{})
	w.Poll()

	prompt := "a question long enough that the provider sends it in more than one piece"
	answer := "an answer long enough that the provider sends it in more than one piece"
	s.Turn(prompt, answer)

	got := turnsOf(w.Poll())
	if len(got) != 1 {
		t.Fatalf("a chunked turn produced %+v", got)
	}
	if got[0].Prompt != prompt || got[0].Response != answer {
		t.Fatalf("chunks reassembled as %q / %q", got[0].Prompt, got[0].Response)
	}
}

// Every provider chartr can watch must also be able to spend, or a tab with no
// native title would have no route to a title at all. This is the pairing the
// specification requires of a supported adapter.
func TestEveryWatchableProviderCanAlsoGenerate(t *testing.T) {
	for name := range adapters {
		if !Supported(name) {
			t.Errorf("%s is in the table but not supported", name)
		}
	}
	for _, c := range contractCases {
		if strings.HasPrefix(c.name, "codex") {
			continue // both cases are the one codex adapter
		}
		if !Supported(c.name) {
			t.Errorf("%s is under contract but has no row in the adapter table", c.name)
		}
	}
}

// patchHead rewrites a codex rollout's head record, so a fixture can produce the
// kinds of thread that must never be a binding candidate.
func (s *codexStore) patchHead(patch func(map[string]any)) {
	s.t.Helper()
	data, err := os.ReadFile(s.file)
	if err != nil {
		s.t.Fatalf("read fixture: %v", err)
	}
	lines := strings.SplitN(string(data), "\n", 2)
	var head map[string]any
	if json.Unmarshal([]byte(lines[0]), &head) != nil {
		s.t.Fatalf("head record is not readable: %s", lines[0])
	}
	payload, _ := head["payload"].(map[string]any)
	patch(payload)
	patched, err := json.Marshal(head)
	if err != nil {
		s.t.Fatalf("marshal head: %v", err)
	}
	rest := ""
	if len(lines) > 1 {
		rest = lines[1]
	}
	if err := os.WriteFile(s.file, []byte(string(patched)+"\n"+rest), 0o600); err != nil {
		s.t.Fatalf("write fixture: %v", err)
	}
}

// The fixtures must sit where a real store's do, since discovery walks the
// provider's own directory layout rather than being told a path.
func TestFixturesSitWhereTheProviderPutsThem(t *testing.T) {
	for _, c := range []struct {
		name  string
		store func(*testing.T) contractStore
		want  []string
	}{
		{"claude", newClaudeStore, []string{claudeProjects, claudeSessions}},
		{"codex", newCodexPaginatedStore, []string{codexSessions}},
		{"pi", newPiStore, nil},
		{"kimi", newKimiStore, []string{kimiSessions, kimiIndex}},
		{"grok", newGrokStore, []string{grokSessions}},
	} {
		t.Run(c.name, func(t *testing.T) {
			s := c.store(t)
			s.Start()
			root := s.Agent().StateRoot
			for _, want := range c.want {
				if _, err := os.Stat(filepath.Join(root, want)); err != nil {
					t.Errorf("the fixture wrote no %s under the state root: %v", want, err)
				}
			}
		})
	}
}
