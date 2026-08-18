// Package transcript is where chartr learns that a conversation actually
// happened.
//
// The cockpit used to infer it from the reconstructed screen: a tab went busy,
// went idle, and looked different, so surely the operator had said something.
// None of that is evidence. A spinner, a clock, a token counter, a boot splash
// and a resumed history all repaint a screen, and every one of them could arm a
// paid title generation for a tab whose operator had typed nothing. The agent
// itself, meanwhile, writes down exactly what happened, in its own store, for
// free.
//
// So this package makes the agent's own persisted session the source of truth,
// and gives the rest of chartr three facts and nothing else:
//
//   - NativeTitle — the provider already titled this conversation. Displaying it
//     costs nothing, and a title chartr did not pay for is the best kind.
//   - TurnStarted — a top-level human turn began. Conversation notifications use
//     it to measure the run without consulting a spinner or terminal title.
//   - TurnFinished — that turn reached a provider-recorded ending. A successful,
//     text-bearing finish is the only thing that may authorize spending on a
//     title; notifications also care about non-text and unsuccessful endings.
//
// Nothing else becomes an event. Hidden reasoning, system and developer
// instructions, tool calls and their results, intermediate assistant messages,
// subagent traffic, sidechains, summaries, slash commands, permission choices
// and the provider's own synthetic user-shaped records are all read past. A
// lifecycle event may omit conversation text, but only a successful top-level
// human text turn with a final visible response may carry both text fields.
//
// # The shape of the subsystem
//
// One subsystem owns discovery, binding, incremental reading and normalization.
// A consumer holds a Watcher, calls Poll on its own beat, and sees provider
// -neutral events; it never learns whether the provider behind them keeps JSONL,
// a database, or something else entirely. Adding a provider is an Adapter, never
// a branch in a caller.
//
// # False negatives are the cheap failure
//
// Every uncertain condition in here resolves to silence: no binding, no events,
// no error the operator sees. A tab that stays untitled costs nothing, while a
// tab bound to the wrong conversation spends the operator's money on it and
// shows them someone else's work. So an ambiguous match is no match, an
// unreadable store is no store, and a record shape the adapter does not
// recognize ends the binding rather than being parsed on a guess.
//
// # Transcript bodies are ephemeral
//
// An event is passed to its consumer and dropped. What a binding retains between
// polls is the session's identity, its native title, its read position, and at
// most one in-flight prompt awaiting its answer — never a second copy of the
// conversation. chartr does not archive what the provider already stores.
package transcript

import (
	"strings"

	"github.com/rengwu/chartr/internal/proc"
)

// Kind is what one normalized event says happened.
type Kind int

const (
	// NativeTitle is the provider's own title for the bound session, as it now
	// stands. It is emitted when a binding first observes a title and again
	// whenever the value changes — a free metadata update, with no debounce and
	// no generation behind it.
	NativeTitle Kind = iota + 1
	// TurnStarted is one top-level submission by the operator. Prompt is present
	// when it was text-only, and empty for an attachment-bearing turn.
	TurnStarted
	// TurnFinished closes the preceding TurnStarted. Outcome says how; Prompt and
	// Response are present only when the successful turn is suitable for titling.
	TurnFinished
)

// Outcome is the provider-neutral way a top-level turn ended.
type Outcome string

const (
	OutcomeCompleted   Outcome = "completed"
	OutcomeFailed      Outcome = "failed"
	OutcomeInterrupted Outcome = "interrupted"
)

// Event is one normalized fact from a bound session. Fields unused by its Kind
// are empty.
type Event struct {
	Kind Kind
	// Outcome is set on TurnFinished.
	Outcome Outcome

	// Title is the provider's session title, on a NativeTitle event. Non-empty:
	// a title cleared back to nothing is not published, since blanking a tab's
	// label tells the operator less than leaving the last good one up.
	Title string

	// Prompt is the operator's own text for this turn, and Response the final
	// visible assistant text answering it. Either may be empty on lifecycle-only
	// events; auto-title requires both while notifications do not.
	//
	// Both are bounded (see textCap). They are the only conversation content
	// this package ever hands out.
	Prompt, Response string
}

// Completed reports whether a finish is a normal provider completion.
func (e Event) Completed() bool {
	return e.Kind == TurnFinished && e.Outcome == OutcomeCompleted
}

// textCap bounds each side of a turn as it crosses the seam, in runes.
//
// A generator's context budget is smaller than this and shaping it is the
// consumer's business — it is the consumer that knows what it is about to spend
// on. The cap here is the structural one: an adapter holds a prompt in memory
// while it waits for the answer to it, and a pasted logfile of a prompt must not
// sit in a watcher for the life of a tab or cross this seam whole. Heads, not
// tails, on both sides: an operator states their request first, and an assistant
// answers before it elaborates.
const textCap = 2000

// head returns the first n runes of s, rune-safe so a multibyte glyph at the cut
// is never split, and trims surrounding whitespace so a provider's own padding
// does not eat the budget.
func head(s string, n int) string {
	r := []rune(s)
	if len(r) > n {
		s = string(r[:n])
	}
	return strings.TrimSpace(s)
}

// Adapter is one provider's transcript knowledge: how to find the persisted
// session a live agent belongs to, and how to read it forward. It is the whole
// contract a provider implements, and the shared harness in the tests is what
// says an implementation satisfies it.
type Adapter interface {
	// Bind matches agent to exactly one of its provider's persisted sessions and
	// returns it seated at the end of the transcript as it stands at this
	// moment.
	//
	// It reports false for every condition that is not a unique, readable
	// match — no candidate, several candidates, an unreadable or absent store, a
	// shape the adapter does not recognize — and says nothing about which.
	// Callers retry: a match that is ambiguous now may be unique once the
	// candidates are written to again.
	Bind(agent proc.Agent) (Session, bool)
}

// Session is one bound conversation, read incrementally.
type Session interface {
	// ID is the provider's own identifier for the bound session. It exists for
	// identity and diagnostics — nothing in the event model is keyed by it.
	ID() string

	// Poll returns the events that appeared since the previous call, in order.
	//
	// It reports false when the binding is over: the store disappeared, it was
	// replaced by a different session, or it drifted into a shape the adapter
	// will not parse. The caller drops the session and may bind again — which,
	// because a fresh binding seats at the end of the transcript, can never
	// resurrect history as new turns.
	Poll() ([]Event, bool)
}

// adapters is the provider table: one entry per provider whose store chartr has
// measured first-hand. Pure data, like the adapter package's delivery, preflight
// and generation tables — a provider is a row here and a file beside claude.go,
// never a branch in the watcher.
//
// An agent whose adapter has no row is unwatchable, which costs its tabs a title
// and nothing else.
var adapters = map[string]Adapter{
	"claude": claude{},
	"codex":  codex{},
	"pi":     pi{},
	"kimi":   kimi{},
	"grok":   grok{},
}

// Supported reports whether chartr can watch an adapter's transcripts at all. A
// caller uses it to skip the work of resolving a process it could not use the
// answer for.
func Supported(adapterName string) bool {
	_, ok := adapters[adapterName]
	return ok
}

// Watcher is one tab's view of its agent's persisted session: unbound until a
// unique candidate exists, then a cursor moving forward through it.
//
// It is not safe for concurrent use; it belongs to whatever samples the tab.
type Watcher struct {
	agent   proc.Agent
	adapter Adapter
	session Session
}

// Watch returns a watcher for a resolved foreground agent, or false when this
// adapter has no transcript knowledge. It performs no I/O — binding happens on
// the first Poll — so it is cheap to call the moment a tab's agent is known.
func Watch(agent proc.Agent) (*Watcher, bool) {
	ad, ok := adapters[agent.Adapter]
	if !ok || agent.PID <= 0 {
		return nil, false
	}
	return watch(agent, ad), true
}

// watch is the seam the tests bind their own adapter through.
func watch(agent proc.Agent, ad Adapter) *Watcher {
	return &Watcher{agent: agent, adapter: ad}
}

// Poll advances the watcher and returns whatever the session did since the last
// call. Nil means nothing to report, which is the ordinary answer and covers
// every unavailable condition too: no binding yet, an ambiguous one, a store
// that is not there, a provider whose persistence the operator turned off.
//
// While unbound, each Poll is another attempt to bind. That is what re-checks an
// ambiguous match when new transcript writes appear: a caller polling on its own
// beat is asking the question again at exactly the moments the answer could have
// changed, and persistent ambiguity simply keeps answering nothing.
func (w *Watcher) Poll() []Event {
	if w == nil || w.adapter == nil {
		return nil
	}
	if w.session == nil {
		s, ok := w.adapter.Bind(w.agent)
		if !ok {
			return nil
		}
		w.session = s
	}
	events, ok := w.session.Poll()
	if !ok {
		// The binding is over. Dropping it lets the next poll bind again — to
		// the session that replaced this one, if any — and a fresh binding seats
		// at the end of the transcript, so nothing behind that cursor can be
		// charged for twice.
		w.session = nil
		return nil
	}
	return events
}

// Session is the provider's identifier for the bound session, or "" while the
// watcher has no binding.
func (w *Watcher) Session() string {
	if w == nil || w.session == nil {
		return ""
	}
	return w.session.ID()
}
