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
a free session or open an empty shell and manually start Claude, Codex, Pi, Kimi,
or Grok. Multiple copies of the same adapter may use different state
roots or accounts in parallel. A solution tied only to chartr's launch path, to a
browser Enter key, or to one provider would therefore miss ordinary tabs and
could associate a tab with the wrong persisted conversation.

The operator needs a mechanical guarantee: no newly persisted human prompt means
no paid title generation. When an agent already provides a useful native session
title, chartr should display it rather than pay to duplicate it.

## Solution

chartr will make the matched persisted agent session the source of truth for
automatic titles. Five transcript adapters—Claude, Codex, Pi, Kimi, and Grok—will
discover the persisted session belonging to a live agent-bearing tab,
tail only new structured records, and normalize them into native-title changes and
completed top-level human turns.

A native title always wins. chartr displays it as soon as the adapter exposes it,
refreshes it without a debounce, and performs no paid title generation while a
native title is available.

When there is no native title, chartr generates a title from the first eligible
completed human turn observed after binding, and never generates again for that
session. It supplies only that turn's non-empty textual user prompt and final
visible assistant text to the title generator. There is exactly one paid attempt
per session: failure, invalid output, cancellation, or an exhausted candidate
ladder leaves the tab untitled, and no later turn re-arms generation. A generated
title is never refreshed; it is a first-impression label that may go stale as the
conversation drifts.

Transcript binding is conservative. chartr derives a process identity from the
foreground agent, resolves only adapter-specific state-root environment
variables, and matches a session from the working directory and observed
transcript writes, preferring a direct provider process-to-session registry where
one exists. Binding seats the cursor at the end of the transcript as it stands at
that moment: historical turns can never authorize spending, and only turns
completed after binding count. A session bound before its first write — as
chartr-launched tabs are — sees its opening turn; a prompt already persisted
before binding stays behind the cursor. If a binding is ambiguous, unavailable,
disabled, or unreadable, chartr skips the title. It never falls back to
screen-change or keystroke inference.

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
8. As an operator, I want a failed title attempt to leave the tab untitled with no
   retry, so that a rate limit, unsupported model, or malformed response cannot
   create a background spending loop.
9. As an operator, I want incomplete, interrupted, and error-ended turns skipped,
   so that a partial exchange does not become the tab's description.
10. As an operator, I want non-text-only turns skipped for paid generation, so that
    chartr never guesses from images, audio, attachments, or opaque structured
    content.
11. As an operator, I want generation context limited to my current prompt and the
    final visible assistant response, so that hidden reasoning, system instructions,
    tool calls, tool results, and earlier history are not sent for titling.
12. As an operator, I want a chartr-injected opening prompt treated by the same
    transcript rule as any other top-level user turn, so that there is no separate
    fragile launch-only title path.
13. As an operator, I want prompts persisted before binding to stay behind the
    cursor rather than be rescued by timestamp comparison, so that a manually
    started agent's launch-command prompt simply stays untitled.
14. As an operator, I want slash commands, permission choices, synthetic messages,
    subagent traffic, summaries, and tool-result records excluded unless they form
    an eligible top-level human text turn with a final visible response, so that
    agent machinery cannot authorize spending.
15. As an operator, I want ticket-bound sessions, free sessions, and manually
    launched agents inside empty shells to follow the same title rules, so that the
    feature does not depend on how the tab was opened.
16. As an operator, I want Claude, Codex, Pi, Kimi, and Grok supported
    through one behavioral contract, so that provider differences do not leak into
    the cockpit.
17. As an operator, I want two same-provider agents in the same space each matched
    to their own persisted session where they can be distinguished, and both left
    untitled where they cannot, so that one tab never displays or transmits the
    other tab's conversation.
18. As an operator, I want multiple Claude state roots and accounts to work in
    parallel, so that aliases using different configuration directories remain
    isolated.
19. As an operator, I want chartr to derive a state root from the running agent's
    environment rather than scan similarly named directories, so that custom roots
    work without another configuration surface.
20. As an operator, I want ambiguous session matching to delay or skip silently, so
    that a missing title remains a cheap, harmless failure.
21. As an operator, I want transcript parsing to fail closed when a provider changes
    its schema, so that an upgrade cannot turn unrelated persisted data into title
    context.
22. As an operator, I want active transcripts tailed incrementally, so that several
    open tabs do not cause repeated full-history reads or expensive database scans.
23. As an operator, I want the auto-title toggle to stop transcript observation and
    paid generation, so that the existing control remains the single feature switch.
24. As an operator, I want settings to explain the transcript material used for
    titling, so that enabling the feature has a clear privacy boundary.
25. As an operator, I want a title generated through the same adapter and resolved
    provider profile as the live agent, so that a custom account's conversation is
    not sent through another account or vendor.
26. As an operator, I want title bodies and process environments omitted from logs,
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
  turns, final visible assistant text, and completion. It carries only what
  one-shot title scheduling needs.
- Claude, Codex, Pi, Kimi, and Grok each implement the same transcript
  adapter contract. JSONL-backed providers maintain byte offsets and tolerate an
  incomplete trailing record.
- Adapter formats are treated as versioned external formats even when a provider
  does not publish a stable schema. Each adapter sniffs the fields it requires and
  becomes unavailable on an unknown shape instead of guessing.
- Discovery reads metadata before message bodies. A candidate is matched from the
  adapter, the working directory, and observed transcript writes. Where a provider
  exposes a direct process-to-session registry, that mapping is preferred.
- A binding must be unique. Ambiguous candidates are re-checked when new
  transcript writes appear. Persistent ambiguity produces no title and no paid
  call.
- Binding seats a cursor at the end of the transcript as it stands at binding
  time. Existing transcript history cannot authorize generation. A session bound
  before its first write — as chartr-launched tabs are — sees its opening turn; a
  prompt already persisted before binding stays behind the cursor.
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
- A resolved state root is normalized before use. Session identifiers and working
  directory are validated before any transcript body is read. chartr constructs
  provider paths from validated identity rather than accepting an arbitrary
  transcript path from message content.
- A native title is the preferred title whenever the active adapter can expose a
  non-empty usable value. Native titles are normalized to the cockpit's existing
  single-line length contract, published immediately when changed, and block all
  paid title generation while present. chartr does not distinguish user-assigned
  native names from provider-generated native names.
- Native-title publication does not participate in any paid-generation gating.
  Observing and displaying an already-persisted title is always the cheap path.
- In the absence of a native title, the first eligible completed turn may schedule
  the first Chartr title immediately. An eligible turn has non-empty top-level human
  text and non-empty final visible assistant text and is not synthetic, a sidechain,
  a subagent turn, a tool result, or historical context.
- The first eligible completed turn schedules exactly one paid attempt per
  session. Failure, invalid output, cancellation, or an exhausted same-adapter
  candidate ladder consumes that attempt; the tab stays untitled and no later
  turn re-arms generation. Falling through candidates inside the attempt is not
  a second attempt.
- The title generator receives a bounded representation of the current user prompt
  followed by the final visible assistant text. The bound stays within the existing
  small title-context budget and preserves useful text from both sides. It never
  receives system or developer instructions, hidden reasoning, tool calls, tool
  results, intermediate assistant messages, prior turns, or raw transcript records.
- Transcript bodies are ephemeral inputs. Runtime state retains the session
  identity, the cursor, the native or displayed title, and whether the one paid
  attempt has been spent, but does not retain a second transcript history.
- Paid generation remains same-adapter and also becomes same-profile. The generated
  subprocess receives only the allowlisted provider environment needed to select
  the foreground session's resolved account or state root. It does not inherit a
  different default profile merely because chartr itself was started outside that
  profile.
- Every supported adapter supplies native-title discovery when its store offers one
  and a safe one-shot generation recipe for the no-native-title case. If an
  installed provider version offers neither a usable native title nor a safe
  generation command, that provider gets no adapter until it gains one; chartr
  never crosses vendors to fill the gap.
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
  snapshot and the number, adapter/profile, and bounded context of generator
  invocations. They do not assert scheduler fields, parser helper calls,
  polling cadence, or storage implementation details.
- The primary behavioral seam is the terminal manager with an injected normalized
  transcript source and injected title generator. This is the highest existing seam
  that can prove both what the cockpit displays and whether it spent a generation.
- One shared transcript-adapter contract is exercised against sanitized fixtures
  for all five providers. The contract covers discovery metadata, native titles,
  top-level user text, final visible assistant text, completion, incremental
  cursors, and unknown-schema failure.
- Fixture contents are synthetic and contain no copied personal transcript bodies,
  credentials, hidden reasoning, or real tool output. Fixtures record the provider
  format/version they represent so future schema changes are deliberate.
- Manager tests extend the existing pure title-scheduler and manager-injection prior
  art, but drive complete normalized turn events rather than internal booleans or
  screen hashes.
- Adapter contract tests cover partial final JSONL records, append after cursor,
  file truncation or replacement, live database writes, unavailable stores,
  malformed records, ignored record kinds, synthetic user-like records, and schema
  drift.
- Binding tests cover a direct process-to-session registry mapping,
  working-directory and observed-writes matching, a session bound before its first
  write, a prompt persisted before binding staying behind the cursor, a resume with
  only historical turns, two concurrent same-adapter tabs in one space, persistent
  ambiguity, and a session that changes or disappears.
- Environment-resolution tests cover default roots, two simultaneous custom Claude
  roots, relative or user-relative values after normalization, inaccessible process
  environments, and the guarantee that non-allowlisted variables never leave the
  process reader.
- Scheduling tests prove: untouched boot produces zero calls; a complete first turn
  without a native title produces exactly one; a native title produces zero and
  publishes immediately; native refresh bypasses the debounce; later turns after a
  generated title produce no further calls; and a failed, invalid, or cancelled
  attempt is never retried, even after later turns.
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
- Supporting transcript-backed titles for OpenCode or agent CLIs other than
  Claude, Codex, Pi, Kimi, and Grok. OpenCode remains supported everywhere else;
  its SQLite store is intentionally too much infrastructure for this feature.
- Guessing from PTY bytes, browser key events, reconstructed screen changes, OSC
  titles, clocks, spinners, or status counters when no transcript event exists.
- Paid generation from historical turns merely because an agent session was
  resumed, rediscovered, or reattached.
- Paid title refreshes: once a title is generated it is never updated, even as the
  conversation drifts to other work.
- Titling a prompt persisted before binding completed, such as a launch-command
  prompt in a manually started agent.
- Following transcript file rotation; truncation and replacement are handled,
  rotation is not.
- Building adapters for providers that expose neither a usable native title nor a
  safe one-shot generation recipe; such a provider gains an adapter only when it
  gains one of those.
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
- A generated title is a first-impression label. The deliberate cost of generating
  exactly once is that a title can go stale as the conversation drifts; a native
  title that appears later still replaces it for free.
