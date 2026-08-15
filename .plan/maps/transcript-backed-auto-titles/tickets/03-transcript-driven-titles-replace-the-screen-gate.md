---
type: task
blocked_by: [02]
undermined_by: []
---

# Transcript-driven titles replace the screen-derived gate

## Question

This is the behavior change. Everything before it is plumbing; everything after
it is more providers.

Today a tab earns a title by being seen working, then idle, after a startup
grace, with a screen hash that differs from the last titled screen, and a
debounce since the last attempt. None of those four guards proves the operator
submitted anything: agent startup, a resumed session, an automatic continuation,
a spinner, a clock or a token counter can arm them and cause a paid model call
even though nobody typed. The context sent for titling is the tail of the
reconstructed screen, which mixes the actual request with TUI chrome, ANSI-derived
reconstruction, status lines and tool output.

Replace all of it with the normalized transcript events. Terminal activity
detection keeps owning the tab's visible working, idle and blocked state — it
simply stops being authorization to spend.

The cost ladder has three rungs:

A native provider title always wins. Display it as soon as the adapter exposes
it, refresh it with no debounce, and perform no paid generation at all while one
is available. Observing an already-persisted title is always the cheap path, so
it participates in neither the debounce nor the turn counter. Do not distinguish
a user-assigned native name from a provider-generated one.

With no native title, the first eligible completed turn schedules the first
generation immediately. An eligible turn has non-empty top-level human text and
non-empty final visible assistant text, and is not synthetic, a sidechain, a
subagent turn, a tool result, or historical context.

Once a generated title exists, another paid generation is due only when both
conditions hold: at least fifteen minutes since the previous paid attempt, and at
least three further eligible turns observed. It is an AND gate. Turns that
accumulate before it opens are coalesced — the most recent completed turn
replaces earlier pending context while the counter still advances — so a
conversational burst produces one update describing the latest work, not a run of
charges for stale intermediate work.

A transcript turn identity may launch at most one scheduled paid attempt. Failure,
invalid output, cancellation, or an exhausted same-adapter candidate ladder
consumes that attempt; a later eligible turn is required before another, and the
time gate still applies. Falling through candidates inside one attempt is not a
scheduler retry. This closes the current behavior where a failed generation
re-arms against the same screen and retries after the debounce.

What reaches the generator changes with it. It receives a bounded representation
of the current user prompt followed by the final visible assistant text, within
the existing small title-context budget, preserving useful text from both sides.
It never receives system or developer instructions, hidden reasoning, tool calls,
tool results, intermediate assistant messages, prior turns, or raw transcript
records.

Generation stays same-adapter and additionally becomes same-profile. The
generating subprocess receives only the allowlisted provider environment needed
to select the foreground session's resolved account or state root, instead of
inheriting whatever default profile chartr itself was started under, so a custom
account's conversation is never sent through a different account. An adapter with
neither a usable native title nor a safe one-shot generation recipe leaves its
tabs untitled; chartr never crosses vendors to fill the gap.

The existing machine-wide toggle remains the only control, and now gates the
whole feature: off means neither transcript-body observation nor generation. A
title already visible may remain displayed, matching current toggle behavior. The
settings surface stays read-only and gains concise disclosure that the enabled
feature reads the matched current user prompt and final visible response and may
send those two texts to the active adapter for a short title.

The browser contract does not change: the server-authoritative terminal snapshot
and its existing auto-title field stay as they are, with no browser-side
transcript state and no provider-specific UI.

## Done when

- The screen-derived worked gate, the startup grace, the unchanged-screen guard
  and the screen-tail title context are gone. Activity detection still drives the
  tab's visible state and no longer authorizes any spending.
- A native title is displayed as soon as it is exposed, refreshed without a
  debounce, normalized to the existing single-line and maximum-length contract,
  and blocks all paid generation while present.
- With no native title, the first eligible completed turn produces exactly one
  generation.
- A later generation requires both fifteen minutes since the previous paid
  attempt and three further eligible turns; several turns inside that window
  coalesce into one update built from the latest completed turn.
- One transcript turn identity launches at most one scheduled paid attempt, and a
  failed, invalid or cancelled attempt is never retried for that turn.
- The generator receives only the bounded current user prompt and final visible
  assistant text, within the existing title-context budget.
- Generation is same-adapter and same-profile: the subprocess carries only the
  allowlisted provider environment for the tab's resolved account or state root.
- A tab whose adapter offers neither a native title nor a safe one-shot recipe
  stays untitled, and no other vendor is used.
- Turning the toggle off stops transcript-body observation and generation; an
  already-displayed title may remain.
- The settings surface is still read-only and states what material titling reads
  and may transmit.
- The browser contract is unchanged — no new browser-side transcript state and no
  provider-specific UI.
- Runtime state retains session and turn identities, cursors, the native or
  displayed title, timing, counters and attempt bookkeeping, and no second copy
  of the transcript.
- Scheduling tests prove: an untouched boot produces zero generations; a complete
  first turn without a native title produces one; a native title produces zero and
  publishes immediately; a native refresh bypasses the debounce; fewer than three
  later turns do not refresh; three turns before fifteen minutes do not refresh;
  fifteen minutes before three turns does not refresh; satisfying both generates
  once from the latest turn; and a failed turn is never retried.
- Privacy tests place recognizable sentinels in system instructions, reasoning,
  tool calls, tool results, earlier turns, the user text and the final assistant
  text, and prove only the latter two reach the generator and that the total
  context is bounded.
- Lifecycle tests cover turning auto-title off, incomplete and interrupted turns,
  error endings, non-text-only prompts, opener turns, process exit, an agent
  change inside an empty shell, and a native title appearing while a paid turn is
  pending.
- Tests assert externally visible behavior — the title in a terminal manager
  snapshot, and the number, adapter and profile, turn identity and bounded context
  of generator invocations — driven through the manager with an injected
  normalized transcript source and an injected title generator. They do not assert
  scheduler fields, parser helper calls, polling cadence or storage details.
- The existing generator output-cleaning tests remain the contract for single-line
  titles and maximum title length, and the existing terminal activity and
  recording tests remain responsible for visible working, idle and blocked state.
