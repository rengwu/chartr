---
type: task
blocked_by: [02]
undermined_by: []
claimed_by: s7455d8faf998
claimed_at: 2026-08-16T10:09:41Z
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

The cost ladder has two rungs:

A native provider title always wins. Display it as soon as the adapter exposes
it, refresh it with no debounce, and perform no paid generation at all while one
is available. Observing an already-persisted title is always the cheap path, so
it participates in no paid-generation gating. Do not distinguish a user-assigned
native name from a provider-generated one.

With no native title, the first eligible completed turn schedules one generation
immediately. An eligible turn has non-empty top-level human text and non-empty
final visible assistant text, and is not synthetic, a sidechain, a subagent turn,
a tool result, or historical context.

That first turn is also the only chance. A session launches at most one scheduled
paid attempt ever: failure, invalid output, cancellation, or an exhausted
same-adapter candidate ladder consumes it, and no later turn re-arms generation.
Falling through candidates inside one attempt is not a second attempt. A
generated title is never refreshed — it is a first-impression label, and the
deliberate cost is that it can go stale as the conversation drifts. This closes
the current behavior where a failed generation re-arms against the same screen
and retries after the debounce.

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
- A session launches at most one paid attempt ever; a failed, invalid, cancelled
  or ladder-exhausted attempt leaves the tab untitled, and later turns do not
  re-arm it. A generated title is never refreshed.
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
- Runtime state retains the session identity, the cursor, the native or displayed
  title, and whether the one paid attempt has been spent — and no second copy of
  the transcript.
- Scheduling tests prove: an untouched boot produces zero generations; a complete
  first turn without a native title produces exactly one; a native title produces
  zero and publishes immediately; a native refresh bypasses the debounce; later
  turns after a generated title produce no further generations; and a failed,
  invalid or cancelled attempt is never retried, even after later turns.
- Privacy tests place recognizable sentinels in system instructions, reasoning,
  tool calls, tool results, earlier turns, the user text and the final assistant
  text, and prove only the latter two reach the generator and that the total
  context is bounded.
- Lifecycle tests cover turning auto-title off, incomplete and interrupted turns,
  error endings, non-text-only prompts, opener turns, process exit, an agent
  change inside an empty shell, and a native title appearing while a paid turn is
  pending.
- Tests assert externally visible behavior — the title in a terminal manager
  snapshot, and the number, adapter and profile, and bounded context of generator
  invocations — driven through the manager with an injected normalized transcript
  source and an injected title generator. They do not assert scheduler fields,
  parser helper calls, polling cadence or storage details.
- The existing generator output-cleaning tests remain the contract for single-line
  titles and maximum title length, and the existing terminal activity and
  recording tests remain responsible for visible working, idle and blocked state.

## Answer

The behaviour change is in: `internal/transcript` is wired through the terminal
manager to the generator, and the four screen-derived guards are deleted.

**What is gone.** `titleSched` and everything it held — the worked-gate, the
45-second startup grace, the screen-hash unchanged-guard and the 7-minute
debounce — with the screen-tail context and the hash that keyed it. Activity
detection is untouched and still owns the tab's visible working, idle and
blocked state; it simply no longer says anything about spending. Nothing in the
title path reads the grid, the OSC title or the scrollback any more.

**What decides now.** A tab holds one binding to its agent's own persisted
session and folds that session's normalized events on a beat (the sampler's slow
tick — the cadence is nobody's contract). A native title is published the moment
it appears and again on every change, normalized to the cockpit's one-line
`MaxTitleRunes` contract — which the generator's own output cleaning now shares
rather than duplicating — and blocks every paid attempt while it stands. With no
native title, the first completed turn schedules exactly one generation. The
attempt is marked spent when it is *scheduled*, not when it succeeds: a decline,
an unusable answer, a cancelled run and an exhausted ladder all consume it, and
no later turn re-arms it. A generated title is never refreshed, while a native
title appearing later still replaces it for free — including one that lands while
the generation is still running, which the generation then does not overwrite.

**What crosses to the generator.** `TitleRequest{Adapter, Env, Context}` and
nothing else: no tab, no screen, no transcript record. `Context` is the turn's
prompt followed by its final visible answer inside the existing 1500-rune
budget, *shared* rather than split down the middle — whichever side is short is
carried whole and hands its remainder to the other, so a pasted prompt cannot eat
the answer and a long answer cannot eat the question. `Env` is
`adapter.StateRootEnv` for the root the live process itself resolved, appended
last to the host environment; that is what makes generation same-profile as well
as same-adapter, so a second Claude account's conversation is summarised under
that account rather than under whichever profile chartr was started in.

**The identity a title belongs to.** Title state is keyed by (adapter, pid): the
adapter chartr launched with the tab's own process, or whatever holds an ad-hoc
shell's foreground group. A change in either is a different conversation, so the
binding, the spent attempt and the displayed title all go with the session that
ended — which is exactly what an agent change inside an empty shell is. A tab
whose process exits keeps the title it earned and stops being observed. The
toggle off drops the binding while keeping the native title, the spent bit and
whatever is displayed, so re-enabling re-binds at the *end* of the transcript
instead of reading the backlog that accumulated while the feature was off.

**Two judgement calls**, recorded because the specification does not settle them.
A launched tab takes its adapter from the launch rather than from foreground
identification, so an agent chartr ships no *activity* manifest for is still
titleable once it has a transcript adapter (ticket 05). And an agent change
clears the displayed title: leaving the old label up would show one
conversation's title on another's tab, and a missing title is the cheaper
failure.

**Tests.** `internal/terminal/titler_test.go` drives the real manager through the
two injected seams — a normalized transcript source and a title generator — and
asserts only the manager snapshot's title and the number, adapter, profile and
bounded context of generator invocations. It covers the untouched boot, the one
generation a first turn produces, the opener turn, native publication and
debounce-free refresh, native normalization, the never-refreshed generated title,
the spent attempt under decline/invalid/cancelled, two tabs on two profiles, an
unwatchable adapter, the privacy boundary (sentinels in the screen's system
preamble, reasoning, tool call and tool result, in the OSC title and in a second
turn of the same batch — only the titled turn's two texts reach the generator),
the bound, the toggle, incomplete turns, process exit, the agent change, and a
native title overtaking a pending generation. A server test runs a stub `claude`
on PATH and proves the tab's state-root variable reaches the subprocess. The
existing `cleanTitle` tests are unchanged and remain the single-line and
maximum-length contract; the activity and recording tests are untouched.

**Excluded.** No new transcript adapters — the five remaining providers have no
row, so their tabs stay untitled by the same rule as any unwatchable adapter
(ticket 05). No ADR, no new setting, and no browser change beyond the settings
disclosure copy and two field comments: the snapshot's `autoTitle` field is
exactly as it was.

**Verified.** `make test`, `make vet`, `make check`, `go test -race` on the
terminal and server packages, `GOOS=windows`/`GOOS=linux` builds, and the web
unit tests (191). Not verified live against a real paid generation — that needs a
real agent CLI completing a turn on this host.
