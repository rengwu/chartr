package transcript

import (
	"strings"
	"testing"
)

// The shared contract says what every adapter must do. These are the rules
// Claude's own store makes possible to get wrong: what counts as the operator
// speaking, what counts as an answer, and what must never leave the store at
// all.

// A title Claude wrote itself is free to display and free to refresh. It is
// published when it appears and when it changes, never twice for the same value,
// and always on one line.
func TestNativeTitlesArePublishedOnEveryChangeAndNoMore(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.Title("Reworking the binder")
	if got := titlesOf(w.Poll()); len(got) != 1 || got[0] != "Reworking the binder" {
		t.Fatalf("a new title published %q", got)
	}
	s.Title("Reworking the binder")
	if got := titlesOf(w.Poll()); len(got) != 0 {
		t.Fatalf("an unchanged title published %q again", got)
	}
	s.Title("Reworking\n   the binder,   again")
	got := titlesOf(w.Poll())
	if len(got) != 1 || got[0] != "Reworking the binder, again" {
		t.Fatalf("a changed title published %q, want it flattened onto one line", got)
	}
}

// Claude records a tool's output in the user role, so "the user said something"
// is not a question about the role. A turn requires the marks Claude puts on
// text a human actually submitted — which is also what chartr's own opening
// prompt carries, since the provider records an argument it was launched with
// exactly as it records typing.
func TestATurnIsWhatClaudeAttributedToAHuman(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.Turn("Read the file /work/.chartr/run/s1/payload.md in full", "Reading it now.")
	got := turnsOf(w.Poll())
	if len(got) != 1 {
		t.Fatalf("a launched tab's opening turn produced %d events: %+v", len(got), got)
	}
	if got[0].Prompt != "Read the file /work/.chartr/run/s1/payload.md in full" {
		t.Fatalf("turn carried prompt %q", got[0].Prompt)
	}
	if got[0].Response != "Reading it now." {
		t.Fatalf("turn carried response %q", got[0].Response)
	}
}

// An answer that stopped to call a tool has not answered. The prompt stays
// pending across polls until something visible finishes the turn.
func TestAnAnswerThatStopsForAToolHasNotFinished(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.prompt("what does this function do?")
	s.append(s.line(s.assistant([]map[string]any{
		{"type": "text", "text": "Let me look."},
	}, "tool_use", false)))
	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a turn still running produced %+v", got)
	}

	s.answer("It resolves the state root.")
	got := turnsOf(w.Poll())
	if len(got) != 1 || got[0].Prompt != "what does this function do?" ||
		got[0].Response != "It resolves the state root." {
		t.Fatalf("finishing the turn produced %+v", got)
	}
}

// Claude ends a turn across more than one record: the hidden reasoning it
// finished on is written as its own record, marked finished, and the visible
// answer follows in the next one. Only the record with something visible in it
// completes the turn — treating the reasoning record as the end would lose the
// turn entirely, which is the common shape of a long working turn.
func TestAFinishedTurnSplitAcrossRecordsIsStillOneTurn(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.prompt("a question that took a while")
	s.append(s.line(s.assistant([]map[string]any{
		{"type": "thinking", "thinking": "the last of the reasoning"},
	}, "end_turn", false)))
	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a reasoning-only record completed the turn: %+v", got)
	}
	s.answer("and here is what I found")

	got := turnsOf(w.Poll())
	if len(got) != 1 || got[0].Prompt != "a question that took a while" ||
		got[0].Response != "and here is what I found" {
		t.Fatalf("the split ending produced %+v", got)
	}
}

// An interrupted turn simply never completes: the notice Claude writes in the
// user role ends the pending prompt, and whatever the assistant says next
// answers nothing.
func TestAnInterruptedTurnIsNeverTitled(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.prompt("a question the operator changed their mind about")
	s.append(s.line(s.envelope(map[string]any{
		"type": "user",
		"message": map[string]any{"role": "user", "content": []map[string]any{
			{"type": "text", "text": "[Request interrupted by user]"},
		}},
	})))
	s.answer("half an answer")

	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("an interrupted turn produced %+v", got)
	}
}

// A prompt that is not text alone is not a prompt chartr will summarise: there
// is no honest title to be had from an image, an attachment or an opaque block
// by reading the sentence next to it.
func TestAPromptThatIsNotTextAloneIsNotATurn(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.append(s.line(s.envelope(map[string]any{
		"type": "user",
		"message": map[string]any{"role": "user", "content": []map[string]any{
			{"type": "image", "source": map[string]any{"type": "base64", "media_type": "image/png", "data": "aW52ZW50ZWQ="}},
			{"type": "text", "text": "what is wrong with this screenshot?"},
		}},
		"origin":       map[string]any{"kind": "human"},
		"promptSource": "typed",
	})))
	s.answer("The legend is cut off.")

	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a turn carrying an image produced %+v", got)
	}
}

// Several turns between two polls come out as several events, in the order they
// were written.
func TestEveryTurnBetweenTwoPollsIsAnEvent(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.Turn("the first question", "the first answer")
	s.Turn("the second question", "the second answer")
	got := turnsOf(w.Poll())
	if len(got) != 2 {
		t.Fatalf("two turns produced %d events: %+v", len(got), got)
	}
	if got[0].Prompt != "the first question" || got[1].Prompt != "the second question" {
		t.Fatalf("turns came out as %q then %q", got[0].Prompt, got[1].Prompt)
	}
}

// The privacy boundary, asserted at the seam it is drawn at. Every kind of
// material the specification excludes from title context is written into this
// turn carrying its own marker; only the operator's text and the final visible
// answer may come out the other side.
func TestNothingButThePromptAndTheAnswerLeavesTheStore(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	s.Turn("SENTINEL-EARLIER-PROMPT", "SENTINEL-EARLIER-ANSWER")

	w, _ := Watch(s.Agent())
	w.Poll()

	s.prompt("SENTINEL-PROMPT")
	s.append(
		s.line(s.envelope(map[string]any{
			"type": "system", "subtype": "local_command", "content": "SENTINEL-SYSTEM",
		})),
		s.line(s.assistant([]map[string]any{
			{"type": "thinking", "thinking": "SENTINEL-REASONING"},
		}, "tool_use", false)),
		s.line(s.assistant([]map[string]any{
			{"type": "tool_use", "id": s.next(), "name": "Bash",
				"input": map[string]any{"command": "SENTINEL-TOOL-INPUT"}},
		}, "tool_use", false)),
		s.line(s.envelope(map[string]any{
			"type":          "user",
			"toolUseResult": map[string]any{"stdout": "SENTINEL-TOOL-RESULT"},
			"message": map[string]any{"role": "user", "content": []map[string]any{
				{"type": "tool_result", "tool_use_id": s.next(), "content": "SENTINEL-TOOL-RESULT"},
			}},
		})),
		s.line(s.envelope(map[string]any{
			"type":         "user",
			"isSidechain":  true,
			"message":      map[string]any{"role": "user", "content": "SENTINEL-SUBAGENT-PROMPT"},
			"origin":       map[string]any{"kind": "human"},
			"promptSource": "typed",
		})),
		s.line(s.assistant([]map[string]any{
			{"type": "text", "text": "SENTINEL-SUBAGENT-ANSWER"},
		}, "end_turn", true)),
		s.line(s.envelope(map[string]any{
			"type": "user", "isMeta": true,
			"message": map[string]any{"role": "user", "content": "SENTINEL-META"},
		})),
	)
	s.answer("SENTINEL-ANSWER")

	events := w.Poll()
	turns := turnsOf(events)
	if len(turns) != 1 {
		t.Fatalf("the turn produced %d events: %+v", len(turns), turns)
	}
	if turns[0].Prompt != "SENTINEL-PROMPT" || turns[0].Response != "SENTINEL-ANSWER" {
		t.Fatalf("the turn carried %q / %q", turns[0].Prompt, turns[0].Response)
	}
	forbidden := []string{
		"SENTINEL-EARLIER-PROMPT", "SENTINEL-EARLIER-ANSWER", "SENTINEL-SYSTEM",
		"SENTINEL-REASONING", "SENTINEL-TOOL-INPUT", "SENTINEL-TOOL-RESULT",
		"SENTINEL-SUBAGENT-PROMPT", "SENTINEL-SUBAGENT-ANSWER", "SENTINEL-META",
	}
	crossed := strings.Join([]string{turns[0].Prompt, turns[0].Response, titleText(events)}, "\n")
	for _, mark := range forbidden {
		if strings.Contains(crossed, mark) {
			t.Fatalf("%s crossed the seam", mark)
		}
	}
}

// A meta record — a caveat, an image placeholder, a context readout — is Claude
// writing to itself in the user role, and cannot re-arm a turn either.
func TestAMetaRecordIsNotAPrompt(t *testing.T) {
	s := newClaudeFixture(t)
	s.Start()
	w, _ := Watch(s.Agent())
	w.Poll()

	s.append(s.line(s.envelope(map[string]any{
		"type": "user", "isMeta": true,
		"message":      map[string]any{"role": "user", "content": "## Context Usage\n\n**Tokens:** 12"},
		"origin":       map[string]any{"kind": "human"},
		"promptSource": "typed",
	})))
	s.answer("an answer to a readout")

	if got := turnsOf(w.Poll()); len(got) != 0 {
		t.Fatalf("a meta record produced %+v", got)
	}
}

func titleText(events []Event) string { return strings.Join(titlesOf(events), "\n") }
