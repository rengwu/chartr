# Transcript-backed auto-titles

## Problem Statement

chartr currently decides that an agent-bearing tab deserves an automatically
generated title by observing the tab enter a working state, later become idle,
and show a changed reconstructed screen. None of those observations proves that
the operator submitted a prompt. Agent startup, a resumed session, an automatic
continuation, a spinner, a clock, a token counter, or another cosmetic repaint can
arm the scheduler and cause a model call even though the operator typed nothing.
The screen tail used as generation context also mixes the actual request with TUI
chrome, ANSI-derived reconstruction, status lines, tool output, and whatever else
happens to remain visible.

The cockpit supports more than sessions launched for tickets. An operator may use
a free session or open an empty shell and manually start Claude, Codex, OpenCode,
Pi, Kimi, or Grok. Multiple copies of the same adapter may use different state
roots or accounts in parallel. A solution tied only to chartr's launch path, to a
browser Enter key, or to one provider would therefore miss ordinary tabs and
could associate a tab with the wrong persisted conversation.

The operator needs a mechanical guarantee: no newly persisted human prompt means
no paid title generation. When an agent already provides a useful native session
title, chartr should display it rather than pay to duplicate it.

## Solution

chartr will make the matched persisted agent session the source of truth for
automatic titles. Six transcript adapters—Claude, Codex, OpenCode, Pi, Kimi, and
Grok—will discover the persisted session belonging to a live agent-bearing tab,
tail only new structured records, and normalize them into native-title changes and
completed top-level human turns.

A native title always wins. chartr displays it as soon as the adapter exposes it,
refreshes it without a debounce, and performs no paid title generation while a
native title is available.

When there is no native title, chartr generates the first title after the first
eligible completed human turn. It supplies only that turn's non-empty textual user
prompt and final visible assistant text to the title generator. Later paid
refreshes require both fifteen minutes since the previous paid attempt and three
additional eligible completed human turns. Multiple qualifying turns are
coalesced, and only the latest completed turn is summarized. A scheduled paid
attempt is made at most once for a transcript turn and is never retried without a
new eligible human turn.

Transcript binding is conservative. chartr derives a process identity from the
foreground agent, resolves only adapter-specific state-root environment variables,
and matches provider metadata such as process identity, working directory,
process-start time, session identity, and new transcript writes. Historical turns
establish a cursor but cannot authorize spending. A prompt written after the
foreground process started may count even when it arrived before binding
completed. If a binding is ambiguous, unavailable, disabled, or unreadable,
chartr delays or skips the title. It never falls back to screen-change or
keystroke inference.

The existing machine-wide auto-title toggle controls the complete behavior. Its
settings copy will explain that chartr reads a narrowly matched current transcript
turn and may send the user prompt and final visible response to the same adapter
for titling.

## User Stories

1. As an operator, I want a tab with no newly persisted human prompt to make no
   title-model call, so that idle agents and cosmetic repainting cost nothing.
2. As an operator, I want a newly launched but untouched agent to remain untitled,
   so that startup activity is not mistaken for conversation.
3. As an operator, I want a resumed agent with no new prompt to make no paid title
   call, so that historical work is not charged again.
4. As an operator, I want an existing native session title to appear immediately
   after resume, so that useful persisted metadata is reused at zero additional
   cost.
5. As an operator, I want native provider titles to take precedence over
   chartr-generated titles, so that chartr does not duplicate work the agent has
   already performed.
6. As an operator, I want native-title changes to appear without waiting for a
   debounce, so that a free metadata update is reflected promptly.
7. As an operator, I want the first title generated after the first complete
   meaningful textual turn when no native title exists, so that a new conversation
   becomes identifiable quickly.
8. As an operator, I want later paid title refreshes to require fifteen minutes and
   three additional human turns, so that a conversational burst does not repeatedly
   spend on labels.
9. As an operator, I want several turns inside that refresh window coalesced into
   one update based on the latest completed turn, so that stale intermediate work
   is neither titled nor charged.
10. As an operator, I want a failed title attempt never retried for the same turn,
    so that a rate limit, unsupported model, or malformed response cannot create a
    background spending loop.
11. As an operator, I want incomplete, interrupted, and error-ended turns skipped,
    so that a partial exchange does not become the tab's description.
12. As an operator, I want non-text-only turns skipped for paid generation, so that
    chartr never guesses from images, audio, attachments, or opaque structured
    content.
13. As an operator, I want generation context limited to my current prompt and the
    final visible assistant response, so that hidden reasoning, system instructions,
    tool calls, tool results, and earlier history are not sent for titling.
14. As an operator, I want a chartr-injected opening prompt treated by the same
    transcript rule as any other top-level user turn, so that there is no separate
    fragile launch-only title path.
15. As an operator, I want a prompt supplied on an agent's launch command to count
    when the persisted event was written after that process started, so that an
    initial prompted launch can receive a title.
16. As an operator, I want slash commands, permission choices, synthetic messages,
    subagent traffic, summaries, and tool-result records excluded unless they form
    an eligible top-level human text turn with a final visible response, so that
    agent machinery cannot authorize spending.
17. As an operator, I want ticket-bound sessions, free sessions, and manually
    launched agents inside empty shells to follow the same title rules, so that the
    feature does not depend on how the tab was opened.
18. As an operator, I want Claude, Codex, OpenCode, Pi, Kimi, and Grok supported
    through one behavioral contract, so that provider differences do not leak into
    the cockpit.
19. As an operator, I want two same-provider agents in the same space matched to
    their own persisted sessions, so that one tab never displays or transmits the
    other tab's conversation.
20. As an operator, I want multiple Claude state roots and accounts to work in
    parallel, so that aliases using different configuration directories remain
    isolated.
21. As an operator, I want chartr to derive a state root from the running agent's
    environment rather than scan similarly named directories, so that custom roots
    work without another configuration surface.
22. As an operator, I want ambiguous session matching to delay or skip silently, so
    that a missing title remains a cheap, harmless failure.
23. As an operator, I want transcript parsing to fail closed when a provider changes
    its schema, so that an upgrade cannot turn unrelated persisted data into title
    context.
24. As an operator, I want active transcripts tailed incrementally, so that several
    open tabs do not cause repeated full-history reads or expensive database scans.
25. As an operator, I want the auto-title toggle to stop transcript observation and
    paid generation, so that the existing control remains the single feature switch.
26. As an operator, I want settings to explain the transcript material used for
    titling, so that enabling the feature has a clear privacy boundary.
27. As an operator, I want a title generated through the same adapter and resolved
    provider profile as the live agent, so that a custom account's conversation is
    not sent through another account or vendor.
28. As an operator, I want title bodies and process environments omitted from logs,
    so that diagnostics do not create another copy of sensitive material.

## Implementation Decisions

- Persisted transcript events replace the current screen-derived worked gate,
  unchanged-screen guard, startup grace, and screen-tail title context. Terminal
  activity detection remains responsible for the tab's visible state, but it is
  not authorization for title-model spending.
- A single transcript subsystem owns discovery, binding, incremental reading, and
  normalization. The terminal manager consumes provider-neutral events rather than
  provider storage formats.
- The normalized event model distinguishes native-title changes, top-level human
  turns, final visible assistant text, completion, and stable transcript turn
  identity. It carries only what title scheduling needs.
- Claude, Codex, OpenCode, Pi, Kimi, and Grok each implement the same transcript
  adapter contract. JSONL-backed providers maintain byte offsets and tolerate an
  incomplete trailing record. Database-backed providers use read-only, incremental,
  indexed queries compatible with a live writer.
- Adapter formats are treated as versioned external formats even when a provider
  does not publish a stable schema. Each adapter sniffs the fields it requires and
  becomes unavailable on an unknown shape instead of guessing.
- Discovery reads metadata before message bodies. A candidate is matched from the
  adapter, foreground process or process group, working directory, process-start
  time, provider session identity, and observed writes. Where a provider exposes a
  direct process-to-session registry, that mapping is preferred.
- A binding must be unique. Ambiguous candidates remain pending and may be retried
  cheaply as new metadata arrives. Persistent ambiguity produces no title and no
  paid call.
- Binding seats a cursor at the live process boundary. Existing transcript history
  cannot authorize generation. A new event whose timestamp is at or after process
  start remains eligible even if it was persisted before discovery completed,
  covering an initial prompt supplied on the launch command.
- Resuming an existing persisted session may surface its native title immediately,
  but the historical conversation remains behind the spending cursor.
- State-root discovery reads only an allowlist of environment variables defined by
  the active adapter. An unset variable resolves to that provider's documented
  default. The raw process environment is discarded immediately and is never
  logged, serialized, or exposed to the browser.
- Multiple configurations of one adapter are independent. In particular, two
  Claude processes with different configuration-directory values resolve separate
  registries and transcript trees even when they share an executable, working
  directory, and adapter name.
- A resolved state root is normalized before use. Session identifiers, process
  start information, and working directory are validated before any transcript
  body is read. chartr constructs provider paths from validated identity rather
  than accepting an arbitrary transcript path from message content.
- A native title is the preferred title whenever the active adapter can expose a
  non-empty usable value. Native titles are normalized to the cockpit's existing
  single-line length contract, published immediately when changed, and block all
  paid title generation while present. chartr does not distinguish user-assigned
  native names from provider-generated native names.
- Native-title publication does not participate in the paid debounce or turn
  counter. Observing and displaying an already-persisted title is always the cheap
  path.
- In the absence of a native title, the first eligible completed turn may schedule
  the first Chartr title immediately. An eligible turn has non-empty top-level human
  text and non-empty final visible assistant text and is not synthetic, a sidechain,
  a subagent turn, a tool result, or historical context.
- After a Chartr title exists, another paid generation is due only when both
  conditions hold: at least fifteen minutes have elapsed since the preceding paid
  attempt, and at least three further eligible completed human turns have been
  observed. The conditions form an AND gate.
- Eligible turns that accumulate before the AND gate opens are coalesced. The most
  recent completed turn replaces earlier pending context, while the scheduler keeps
  the count needed to open the turn gate.
- A transcript turn identity may launch at most one scheduled paid attempt. Failure,
  invalid output, cancellation, or an exhausted same-adapter candidate ladder
  consumes that attempt. A later eligible turn is required before any further
  attempt, and the time gate continues to apply.
- The title generator receives a bounded representation of the current user prompt
  followed by the final visible assistant text. The bound stays within the existing
  small title-context budget and preserves useful text from both sides. It never
  receives system or developer instructions, hidden reasoning, tool calls, tool
  results, intermediate assistant messages, prior turns, or raw transcript records.
- Transcript bodies are ephemeral inputs. Runtime state retains session and turn
  identities, cursors, native or displayed title, timing, counters, and attempt
  bookkeeping, but does not retain a second transcript history.
- Paid generation remains same-adapter and also becomes same-profile. The generated
  subprocess receives only the allowlisted provider environment needed to select
  the foreground session's resolved account or state root. It does not inherit a
  different default profile merely because chartr itself was started outside that
  profile.
- Every supported adapter supplies native-title discovery when its store offers one
  and a safe one-shot generation recipe for the no-native-title case. If an
  installed provider version offers neither a usable native title nor a safe
  generation command, that tab remains untitled; chartr never crosses vendors to
  fill the gap.
- The existing bounded same-adapter candidate ladder remains one scheduled paid
  attempt. Falling through candidates inside that attempt is not a later scheduler
  retry, while relaunching the scheduler for the same turn is forbidden.
- The existing machine-wide auto-title toggle remains the only control. When off,
  chartr performs neither transcript-body observation nor title generation. A title
  already visible may remain displayed, matching the current toggle behavior.
- The settings surface remains read-only and gains concise disclosure that the
  enabled feature reads the matched current user prompt and final visible response
  and may submit those two texts to the active adapter for a short title.
- The server-authoritative terminal snapshot and existing auto-title field remain
  the browser contract. No new browser-side transcript state or provider-specific
  UI is introduced.
- Initial platform support is macOS and Linux. The transcript subsystem degrades to
  unavailable wherever foreground process identity or allowlisted environment
  lookup is unsupported.

## Testing Decisions

- Tests assert externally visible behavior: the title in a terminal manager
  snapshot and the number, adapter/profile, turn identity, and bounded context of
  generator invocations. They do not assert scheduler fields, parser helper calls,
  polling cadence, or storage implementation details.
- The primary behavioral seam is the terminal manager with an injected normalized
  transcript source and injected title generator. This is the highest existing seam
  that can prove both what the cockpit displays and whether it spent a generation.
- One shared transcript-adapter contract is exercised against sanitized fixtures
  for all six providers. The contract covers discovery metadata, native titles,
  top-level user text, final visible assistant text, completion, stable turn IDs,
  incremental cursors, and unknown-schema failure.
- Fixture contents are synthetic and contain no copied personal transcript bodies,
  credentials, hidden reasoning, or real tool output. Fixtures record the provider
  format/version they represent so future schema changes are deliberate.
- Manager tests extend the existing pure title-scheduler and manager-injection prior
  art, but drive complete normalized turn events rather than internal booleans or
  screen hashes.
- Adapter contract tests cover partial final JSONL records, append after cursor,
  file rotation or replacement, live database writes, unavailable stores, malformed
  records, ignored record kinds, synthetic user-like records, and schema drift.
- Binding tests cover a direct PID mapping, cwd and process-start matching, an
  initial prompted launch written before binding, a resume with only historical
  turns, two concurrent same-adapter tabs in one space, persistent ambiguity, PID
  reuse, and a session that changes or disappears.
- Environment-resolution tests cover default roots, two simultaneous custom Claude
  roots, relative or user-relative values after normalization, inaccessible process
  environments, and the guarantee that non-allowlisted variables never leave the
  process reader.
- Scheduling tests prove: untouched boot produces zero calls; a complete first turn
  without a native title produces one; a native title produces zero and publishes
  immediately; native refresh bypasses the debounce; fewer than three later turns
  do not refresh; three turns before fifteen minutes do not refresh; fifteen minutes
  before three turns does not refresh; satisfying both generates once from the
  latest turn; and a failed turn is never retried.
- Privacy-boundary tests put recognizable sentinels in system instructions,
  reasoning, tools, earlier turns, user text, and final assistant text, then prove
  that only the latter two reach the generator and that the total context is
  bounded.
- Lifecycle tests cover turning auto-title off, incomplete and interrupted turns,
  error endings, non-text-only prompts, opener turns, process exit, agent changes in
  an empty shell, and a native title appearing while a paid turn is pending.
- Existing generator output-cleaning tests remain the contract for single-line
  titles and maximum title length. Existing terminal activity and recording tests
  remain responsible for visible working, idle, and blocked state; transcript tests
  do not duplicate that grammar.
- Platform-specific process tests run on macOS and Linux where supported and skip
  explicitly elsewhere. The cross-platform build must continue to compile with an
  unavailable transcript-process resolver rather than acquiring an implicit Unix
  dependency.

## Out of Scope

- Windows foreground-process and process-environment discovery for transcript
  binding.
- Supporting agent CLIs other than Claude, Codex, OpenCode, Pi, Kimi, and Grok.
- Guessing from PTY bytes, browser key events, reconstructed screen changes, OSC
  titles, clocks, spinners, or status counters when no transcript event exists.
- Paid generation from historical turns merely because an agent session was
  resumed, rediscovered, or reattached.
- Cross-vendor title generation or sending one provider's transcript through a
  different provider.
- Reading hidden reasoning, system instructions, developer instructions, tool calls,
  tool results, subagent transcripts, sidechains, summaries, or full conversation
  history for title context.
- Generating paid titles for incomplete turns, non-text-only prompts, or adapters
  with no safe native-title or one-shot generation capability.
- Editing, clearing, or writing a provider's native session title or transcript.
- A new state-root registration setting, directory picker, provider-specific title
  settings, manual title editor, or browser-side transcript protocol.
- Persisting a duplicate transcript archive inside chartr.
- Guaranteeing compatibility with an unknown future private transcript schema; such
  a version fails closed until its adapter fixture and parser are updated.

## Further Notes

- Provider transcript stores are operational dependencies, not chartr-owned
  schemas. Their adapters should remain small, isolated, and replaceable, consistent
  with the agent-agnostic adapter decision.
- Transcript persistence can be disabled or cleaned up by an agent. That is an
  expected unavailable state, not an error the cockpit needs to surface. No title is
  a valid steady state.
- Native titles may be stable session identities rather than summaries of the latest
  work. The explicit cost decision is still to trust them indefinitely while they
  are available.
- The specification deliberately prefers false negatives over false positives:
  missing a title is harmless, while a false positive may spend money or expose the
  wrong transcript turn.
- The fifteen-minute and three-turn thresholds govern paid refreshes only. They do
  not delay free native-title reads or the first Chartr title when no native title
  exists.
