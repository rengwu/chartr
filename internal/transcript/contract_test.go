package transcript

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/proc"
)

// This file is the shared adapter contract: one suite every provider's adapter
// is held to, so "Claude works" and "Codex works" mean the same sentence rather
// than two sets of provider-shaped tests that happen to pass.
//
// It asserts only what a consumer can see — the events a Watcher hands out — and
// never how a provider stores anything. A provider joins by implementing
// contractStore (writing its own format's records from provider-neutral
// instructions) and adding one row to contractCases; nothing in the scenarios
// below changes.

// contractStore is one provider's half of the harness: a synthetic store on a
// temp state root, plus the mutations the scenarios need. Every implementation
// writes fixtures it made up — no copied transcript bodies, no credentials, no
// hidden reasoning, no real tool output — and names the format and version those
// fixtures represent, so a provider schema change is a deliberate edit to a
// fixture rather than a silent reinterpretation of the operator's data.
type contractStore interface {
	// Format names the store format and the provider version the fixtures were
	// written against.
	Format() string

	// Agent is the live agent this store belongs to: exactly the facts binding
	// is given, with a state root under the test's temp directory.
	Agent() proc.Agent

	// Start creates the persisted session, as the provider does at launch,
	// carrying no completed turn yet.
	Start()

	// Turn appends one completed top-level human turn: the operator's text, the
	// provider's machinery in between, and a final visible answer.
	Turn(prompt, response string)

	// PartialTurn appends a turn whose final record is cut mid-write, as a
	// reader sees it while the provider is still flushing; Complete finishes
	// that record.
	PartialTurn(prompt, response string)
	Complete()

	// Title records a change to the provider's own session title.
	Title(title string)

	// Ignored appends the record kinds an adapter must read past: reasoning,
	// tool calls and results, system and developer material, subagent traffic,
	// summaries and the rest of the machinery.
	Ignored()

	// SyntheticUser appends a user-shaped record the provider wrote itself — a
	// slash command, a caveat, an interruption notice — which no adapter may
	// promote into a human turn.
	SyntheticUser(text string)

	// Malformed appends a complete record that is not readable at all.
	Malformed()

	// Drift appends a complete record of a kind the adapter interprets, in a
	// shape it does not recognize: the provider's next schema.
	Drift()

	// Replace swaps the store for a shorter one carrying history of its own, as
	// a truncation or a rewrite in place looks from outside.
	Replace()

	// Remove deletes the store, as an operator clearing their history or
	// disabling persistence does.
	Remove()
}

// contractCase is one provider's participation in the suite.
type contractCase struct {
	name    string
	adapter Adapter
	store   func(t *testing.T) contractStore
}

// contractCases is every adapter under contract. Codex, OpenCode, Pi, Kimi and
// Grok land here as their adapters do.
var contractCases = []contractCase{
	{name: "claude", adapter: claude{}, store: newClaudeStore},
}

func TestAdapterContract(t *testing.T) {
	for _, c := range contractCases {
		t.Run(c.name, c.run)
	}
}

func (c contractCase) run(t *testing.T) {
	// A fixture that does not say what it is a fixture *of* cannot be checked
	// against a provider's next release.
	if s := c.store(t); !strings.Contains(s.Format(), ".") {
		t.Fatalf("fixtures declare format %q, which names no provider version", s.Format())
	}

	t.Run("history behind the cursor is never a turn", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		s.Turn("an old question", "an old answer")
		s.Turn("another old question", "another old answer")

		w := watch(s.Agent(), c.adapter)
		if turns := turnsOf(w.Poll()); len(turns) != 0 {
			t.Fatalf("binding emitted %d historical turn(s): %+v", len(turns), turns)
		}
	})

	t.Run("a native title survives the cursor", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		s.Turn("an old question", "an old answer")
		s.Title("Persisted session name")

		w := watch(s.Agent(), c.adapter)
		events := w.Poll()
		if got := titlesOf(events); len(got) != 1 || got[0] != "Persisted session name" {
			t.Fatalf("resume published titles %q, want the persisted one", got)
		}
		if turns := turnsOf(events); len(turns) != 0 {
			t.Fatalf("resume emitted %d historical turn(s)", len(turns))
		}
	})

	t.Run("a turn completed after binding is one event", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		s.Turn("an old question", "an old answer")

		w := watch(s.Agent(), c.adapter)
		w.Poll()
		s.Turn("the new question", "the new answer")

		got := turnsOf(w.Poll())
		if len(got) != 1 {
			t.Fatalf("appended one turn, got %d events: %+v", len(got), got)
		}
		if got[0].Prompt != "the new question" || got[0].Response != "the new answer" {
			t.Fatalf("turn carried %q / %q", got[0].Prompt, got[0].Response)
		}
	})

	t.Run("an unfinished turn waits for its final record", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.PartialTurn("the question", "the answer")
		if got := turnsOf(w.Poll()); len(got) != 0 {
			t.Fatalf("a half-written record produced %d turn(s)", len(got))
		}
		s.Complete()
		got := turnsOf(w.Poll())
		if len(got) != 1 || got[0].Prompt != "the question" || got[0].Response != "the answer" {
			t.Fatalf("completing the record produced %+v, want the whole turn once", got)
		}
	})

	t.Run("a replaced store keeps its history behind the cursor", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		s.Turn("an old question", "an old answer")
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.Replace()
		if got := turnsOf(w.Poll()); len(got) != 0 {
			t.Fatalf("a replaced store emitted %d historical turn(s): %+v", len(got), got)
		}
		s.Turn("the question after the rewrite", "the answer after the rewrite")
		got := turnsOf(w.Poll())
		if len(got) != 1 || got[0].Prompt != "the question after the rewrite" {
			t.Fatalf("after a rewrite the next real turn came out as %+v", got)
		}
	})

	t.Run("machinery is not conversation", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.Ignored()
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("provider machinery produced %d event(s): %+v", len(got), got)
		}
	})

	t.Run("user-shaped records the provider wrote itself are not turns", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.SyntheticUser("/compact")
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("the provider's own user-role records produced %d event(s): %+v", len(got), got)
		}
	})

	t.Run("an absent store is silent", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		s.Turn("a question", "an answer")
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.Remove()
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("a removed store produced %+v", got)
		}
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("a removed store produced %+v on the second poll", got)
		}
	})

	t.Run("a store that was never there is silent", func(t *testing.T) {
		s := c.store(t)
		w := watch(s.Agent(), c.adapter)
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("an agent with no persisted session produced %+v", got)
		}
	})

	t.Run("a malformed record ends the reading", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.Malformed()
		s.Turn("a question after the damage", "an answer after the damage")
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("reading past a malformed record produced %+v", got)
		}
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("rebinding onto a malformed store produced %+v", got)
		}
	})

	t.Run("schema drift fails closed", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		s.Drift()
		s.Turn("a question in the new schema", "an answer in the new schema")
		if got := w.Poll(); len(got) != 0 {
			t.Fatalf("an unrecognized record shape produced %+v", got)
		}
	})

	t.Run("turn text is bounded", func(t *testing.T) {
		s := c.store(t)
		s.Start()
		w := watch(s.Agent(), c.adapter)
		w.Poll()

		long := strings.Repeat("ø", textCap*3)
		s.Turn(long, long)
		got := turnsOf(w.Poll())
		if len(got) != 1 {
			t.Fatalf("a long turn produced %d events", len(got))
		}
		if n := len([]rune(got[0].Prompt)); n > textCap {
			t.Fatalf("prompt crossed the seam at %d runes, cap is %d", n, textCap)
		}
		if n := len([]rune(got[0].Response)); n > textCap {
			t.Fatalf("response crossed the seam at %d runes, cap is %d", n, textCap)
		}
	})
}

// turnsOf and titlesOf are how the scenarios read a poll: by kind, never by
// index, so an adapter that also publishes a title is not failed for it.
func turnsOf(events []Event) []Event {
	var out []Event
	for _, e := range events {
		if e.Kind == HumanTurn {
			out = append(out, e)
		}
	}
	return out
}

func titlesOf(events []Event) []string {
	var out []string
	for _, e := range events {
		if e.Kind == NativeTitle {
			out = append(out, e.Title)
		}
	}
	return out
}
