package terminal

import (
	"strings"

	"github.com/rengwu/chartr/internal/transcript"
)

// This file is the tab-side half of the auto-title feature: what a tab does with
// the normalized transcript events of the agent session running in it, and the
// one paid generation those events may authorize. The spending itself — which
// adapter, which model, the process — is the Manager's injected generator
// (server side); here there is only the rule and the string.
//
// It used to be a hysteresis over the reconstructed screen: a tab seen working,
// then idle, past a startup grace, with a screen hash different from the last
// titled one, and a debounce since the last attempt. All four guards are gone,
// because none of them was evidence that anyone had typed. A spinner, a clock, a
// token counter, a boot splash and a resumed history all repaint a screen, and
// any of them could arm a paid call for a tab whose operator had said nothing —
// with the screen tail, TUI chrome and tool output included, sent along as the
// context. Activity detection still owns what the tab *shows*; it simply no
// longer authorizes spending.
//
// What decides now is the agent's own persisted session (internal/transcript),
// which records what actually happened.
//
// # The cost ladder has two rungs
//
//   - A native provider title always wins. It is published the moment the
//     adapter exposes it, refreshed with no debounce, and blocks every paid
//     attempt while it is there. Observing a title the provider already wrote is
//     free, so it is gated by nothing and asks no questions about who wrote it.
//   - With no native title, the first completed turn a tab sees schedules one
//     generation. That first turn is also the only chance: a session launches at
//     most one paid attempt ever, and a failure, invalid output or a cancelled
//     run consumes it. A generated title is a first-impression label and is
//     never refreshed — it may go stale as the conversation drifts, which is the
//     deliberate price of never charging twice for one session.
//
// The second rung is optional. The operator's native-only flag (autotitle.toml)
// keeps the first and removes the second: tabs whose harness writes its own title
// are titled, tabs whose harness does not stay plain, and nothing is spent.
//
// # What a tab retains
//
// The session's identity and read position (inside the source), the native or
// displayed title, and whether the one paid attempt has been spent. Turn text is
// handed to the generator and dropped; chartr keeps no second copy of a
// conversation the provider already stores.

// titleContextCap bounds the whole context one generation is given, in runes.
// Small on purpose: a title needs the gist of one exchange, not a transcript,
// and a small context is a cheap context.
const titleContextCap = 1500

// MaxTitleRunes is the length half of the cockpit's displayed-title contract —
// a tab label is a marquee, not a paragraph. It applies to every automatic
// title, whether the provider wrote it or a generation produced it, so the
// generator's own output cleaning and a native title agree on one limit.
const MaxTitleRunes = 60

// TitleSource is one tab's normalized transcript: the events its agent's own
// persisted session produced since the last poll, provider-neutral by the time
// they arrive here. In production it is an internal/transcript Watcher; a test
// hands the manager its own.
type TitleSource interface {
	Poll() []transcript.Event
}

// TitleBinding is a live agent tab matched to the persisted session behind it:
// the adapter running it, the allowlisted provider environment that selects the
// account or state root that session belongs to, and the stream of events it
// produces.
type TitleBinding struct {
	// Adapter is the tab's own harness. Titling is same-adapter: a session's
	// conversation is only ever summarised by the vendor already running it.
	Adapter string
	// Env is the `KEY=VALUE` state-root environment a generation must carry to
	// run under the same profile as the live agent, rather than under whichever
	// default chartr itself was started with. Nothing else about the agent's
	// environment is read or retained.
	Env []string
	// Source is the tab's normalized transcript.
	Source TitleSource
}

// TitleRequest is one paid generation as it reaches the generator: the adapter
// to spend on, the profile to spend under, and the bounded context to summarise.
// It is deliberately the whole of what crosses to the server side — no tab, no
// screen, no transcript record.
type TitleRequest struct {
	Adapter string
	Env     []string
	Context string
}

// titleAttempt is the request plus the identity it was authorized for, so a
// title that lands after its tab has moved on to another agent or another
// process is dropped rather than labelling a conversation it never described.
type titleAttempt struct {
	req   TitleRequest
	agent string
	pid   int
}

// titleState is one tab's transcript-driven title state. It is small on purpose:
// the identity the binding belongs to, the binding itself, the native title in
// force, and the one bit that says the session's single paid attempt is gone.
type titleState struct {
	// agent and pid are the conversation this state describes: which adapter,
	// and which process running it. A change in either is a different session,
	// and everything below it is dropped.
	agent string
	pid   int

	adapter string
	env     []string
	src     TitleSource

	// native is the provider's own title as last published. Non-empty blocks all
	// paid generation, for as long as it stands.
	native string
	// spent records that this session's one scheduled attempt has been used —
	// whether it produced a title, declined, or was cancelled.
	spent bool
}

// titleBeat advances one tab's transcript by one beat and reports the paid
// generation it authorized, if any, and whether the tab's displayed title
// changed. It runs on the sampler's goroutine, and is the only writer of the
// tab's title state; a landed generation (titleGenerated) is the one thing that
// touches it from elsewhere.
//
// on is the machine-wide toggle. Off means the tab is not observed at all: the
// binding is dropped, no transcript body is read, and nothing is spent — while a
// title already on screen stays there, which is what the toggle has always done.
//
// nativeOnly is the other machine-wide flag, and it cuts the ladder in half rather
// than switching it off: the transcript is still read and native titles are still
// published — that half was always free — but no completed turn ever authorizes a
// generation. It leaves the session's one attempt unspent, so an operator who turns
// the flag back off gets a title from the next completed turn rather than a tab
// permanently barred from one.
func (t *Terminal) titleBeat(on, nativeOnly bool, bind func(*Terminal) (TitleBinding, bool)) (attempt titleAttempt, spend, changed bool) {
	t.mu.Lock()
	agent, alive, pid := t.titleAgentLocked(), t.alive, t.titlePIDLocked()
	if agent != t.titles.agent || pid != t.titles.pid {
		// A different agent, or the same one in a new process: a different
		// conversation. Its binding, its spent attempt and the title it earned all
		// belong to the session that ended.
		changed = t.resetTitlesLocked(agent, pid)
	}
	if !on || !alive || agent == "" {
		// Nothing to observe: a tab with no agent in front, one whose process is
		// over, or the whole feature switched off. The displayed title stays,
		// and the old binding is discarded.
		t.titles.src = nil
		t.mu.Unlock()
		return titleAttempt{}, false, changed
	}
	src := t.titles.src
	t.mu.Unlock()

	if src == nil {
		// Binding is attempted from the moment a tab's agent is known and on every
		// beat after: a session that has not been written to yet is exactly what
		// binds at the start of its own transcript, and a match that is ambiguous
		// now may be unique once the candidates are written to again.
		b, ok := bind(t)
		if !ok {
			return titleAttempt{}, false, changed
		}
		src = b.Source
		t.mu.Lock()
		t.titles.src, t.titles.adapter, t.titles.env = b.Source, b.Adapter, b.Env
		t.mu.Unlock()
	}

	events := src.Poll()

	t.mu.Lock()
	defer t.mu.Unlock()
	var turn *transcript.Event
	for i, ev := range events {
		switch ev.Kind {
		case transcript.NativeTitle:
			// Free and current: display it at once, however many landed in one
			// batch, and there is no debounce to wait out.
			if title := normalizeTitle(ev.Title); title != "" {
				t.titles.native = title
				if t.autoTitle != title {
					t.autoTitle, changed = title, true
				}
			}
		case transcript.HumanTurn:
			// The *first* completed turn is the one candidate. Later ones in the
			// same batch are later turns like any other, and a session gets one
			// attempt however fast its turns arrive.
			if turn == nil {
				turn = &events[i]
			}
		}
	}
	if turn == nil || nativeOnly || t.titles.native != "" || t.titles.spent {
		return titleAttempt{}, false, changed
	}
	// The attempt is spent at the moment it is scheduled, not when it succeeds:
	// failure, invalid output and cancellation all consume it, and no later turn
	// re-arms generation.
	t.titles.spent = true
	return titleAttempt{
		req: TitleRequest{
			Adapter: t.titles.adapter,
			Env:     t.titles.env,
			Context: titleContext(turn.Prompt, turn.Response),
		},
		agent: agent,
		pid:   pid,
	}, true, changed
}

// titleGenerated stores the title the one paid attempt produced and reports
// whether it changed what the tab shows — the caller pushes a fresh model only
// when it did.
//
// It is refused in three cases, all of them the same judgement: a title that
// cleans to nothing is not a title, a native title that appeared while the
// generation ran is free and current and wins, and a tab that has moved to
// another agent or another process is no longer the conversation this title
// describes.
func (t *Terminal) titleGenerated(at titleAttempt, title string) bool {
	title = normalizeTitle(title)
	t.mu.Lock()
	defer t.mu.Unlock()
	if title == "" || t.titles.native != "" || title == t.autoTitle {
		return false
	}
	if t.titles.agent != at.agent || t.titles.pid != at.pid {
		return false
	}
	t.autoTitle = title
	return true
}

// resetTitlesLocked drops everything a title state knew about the conversation
// that just ended, and reports whether the tab's displayed title went with it.
// The caller holds t.mu.
func (t *Terminal) resetTitlesLocked(agent string, pid int) bool {
	t.titles = titleState{agent: agent, pid: pid}
	if t.autoTitle == "" {
		return false
	}
	t.autoTitle = ""
	return true
}

// titleAgentLocked names the adapter whose session this tab's title comes from.
// A tab chartr launched knows its adapter from the launch itself — that is the
// stronger fact, and it holds even for an agent chartr ships no activity
// manifest for, whose foreground identification is empty. Everything else is
// whatever identification found in the tab's foreground. The caller holds t.mu.
func (t *Terminal) titleAgentLocked() string {
	if t.launchedAgent != "" {
		return t.launchedAgent
	}
	return t.agent
}

// titlePIDLocked identifies the process a tab's title state belongs to. A tab
// chartr launched an agent in *is* that process; an ad-hoc shell's agent is
// whatever holds the PTY's foreground, which changes the moment the operator
// quits one agent and starts another. The caller holds t.mu.
func (t *Terminal) titlePIDLocked() int {
	if t.launchedAgent != "" {
		return t.shellPID
	}
	return t.lastPgrp
}

// titleContext renders one completed turn as the generator's context: the
// operator's own prompt, then the final visible answer to it, and nothing else.
//
// The budget is shared rather than split down the middle. Whichever side is
// short is carried whole and hands its remainder to the other, so a one-line
// question about a long answer keeps the answer, and a pasted prompt cannot eat
// the response that explains what was done with it. Heads, not tails, on both
// sides: an operator states their request first, and an assistant answers before
// it elaborates.
func titleContext(prompt, response string) string {
	const promptLabel, responseLabel = "User prompt:\n", "\n\nFinal response:\n"

	prompt, response = strings.TrimSpace(prompt), strings.TrimSpace(response)
	budget := titleContextCap - len([]rune(promptLabel)) - len([]rune(responseLabel))
	p, r := shareBudget(len([]rune(prompt)), len([]rune(response)), budget)
	return promptLabel + headRunes(prompt, p) + responseLabel + headRunes(response, r)
}

// shareBudget divides total between two lengths, giving each side its own length
// where both fit and otherwise letting the shorter side spend what it needs
// before the longer one takes the rest. An even split is the fallback when both
// sides are over half.
func shareBudget(a, b, total int) (int, int) {
	if total <= 0 {
		return 0, 0
	}
	if a+b <= total {
		return a, b
	}
	half := total / 2
	switch {
	case a <= half:
		return a, total - a
	case b <= total-half:
		return total - b, b
	}
	return half, total - half
}

// headRunes returns the first n runes of s, rune-safe so a multibyte glyph at
// the cut is never split.
func headRunes(s string, n int) string {
	r := []rune(s)
	if len(r) <= n {
		return s
	}
	return strings.TrimSpace(string(r[:n]))
}

// normalizeTitle holds any automatic title to the cockpit's display contract:
// one line, whitespace collapsed, no longer than MaxTitleRunes. A provider's own
// title goes through it for the same reason a generated one does — a tab label
// is a short muted marquee, and a title that arrives with newlines or a
// paragraph in it is trimmed rather than trusted.
func normalizeTitle(s string) string {
	s = strings.Join(strings.Fields(s), " ")
	if r := []rune(s); len(r) > MaxTitleRunes {
		s = strings.TrimSpace(string(r[:MaxTitleRunes]))
	}
	return s
}
