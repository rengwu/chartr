---
type: task
blocked_by: [01]
undermined_by: []
claimed_by: sa752e79475f4
claimed_at: 2026-08-16T07:10:29Z
---

# Normalized transcript events, the adapter contract, and the Claude adapter

## Question

Auto-titling must learn that a conversation happened from the agent's own
persisted session rather than from the reconstructed screen. Build the subsystem
that makes that possible, and prove it on Claude.

One transcript subsystem owns discovery, binding, incremental reading and
normalization. Everything downstream consumes provider-neutral events and never
sees a provider's storage format.

The normalized event model carries only what title scheduling needs:

- a native session title changed, with its value;
- a completed top-level human turn, carrying that turn's textual user prompt and
  the final visible assistant text.

Nothing else is emitted. Hidden reasoning, system and developer instructions,
tool calls, tool results, intermediate assistant messages, subagent traffic,
sidechains, summaries, slash commands, permission choices and synthetic messages
are not events. Records that are not a top-level human text turn with a final
visible response cannot become one.

Binding is conservative and must be unique. A candidate session is matched from
the adapter, the working directory, and observed transcript writes. Where a
provider exposes a direct process-to-session registry, prefer it. Ambiguous
candidates are re-checked when new transcript writes appear; persistent ambiguity
produces no binding, silently.

Binding seats the cursor at the end of the transcript as it stands at binding
time. Existing history establishes position but is never emitted as an eligible
turn — that is what keeps a resumed session from being charged for work already
done. A session bound before its first write, as chartr-launched tabs are, sees
its opening turn; a prompt already persisted before binding stays behind the
cursor and its tab simply stays untitled.

Transcript formats are versioned external formats even where a provider publishes
no stable schema. Each adapter sniffs the fields it requires and becomes
unavailable on an unknown shape rather than guessing, so a provider upgrade
cannot turn unrelated persisted data into title context.

Claude is the first implementation and the proof of the contract. Its store is
append-only JSONL, so incremental reading maintains byte offsets and tolerates an
incomplete trailing record.

Transcript bodies are ephemeral inputs. Runtime state keeps the session identity,
the native title, and the reading position — never a second copy of the
conversation.

## Done when

- A single transcript subsystem owns discovery, binding, incremental reading and
  normalization, and its consumers see provider-neutral events only.
- The normalized event model distinguishes native-title changes, top-level human
  turns, final visible assistant text, and completion, and carries nothing beyond
  what one-shot title scheduling needs.
- One adapter contract is defined that a provider implementation satisfies, with
  the Claude adapter as its first implementation: discovery, native title,
  top-level user text, final visible assistant text, completion, incremental
  cursors, and unknown-schema failure.
- Discovery reads metadata before message bodies, and provider paths are
  constructed from validated identity rather than from a transcript path found
  inside message content.
- A binding is unique or it does not exist. Ambiguous candidates are re-checked
  when new transcript writes appear; persistent ambiguity yields no binding and no
  error surfaced to the operator.
- The cursor is seated at the end of the transcript at binding time. Historical
  turns never become eligible turns. A session bound before its first write sees
  its opening turn; a prompt persisted before binding stays behind the cursor.
- Resuming a persisted session surfaces its native title while its history stays
  behind the cursor.
- Incremental reading maintains byte offsets, tolerates an incomplete trailing
  record, and handles append after cursor and file truncation or replacement
  without re-reading full history.
- An unrecognized record shape makes the adapter unavailable rather than parsed
  on a guess.
- A shared contract harness exercises the adapter against fixtures covering
  partial final records, append after cursor, truncation or replacement,
  unavailable stores, malformed records, ignored record kinds, synthetic
  user-like records, and schema drift.
- Fixtures are synthetic: no copied personal transcript bodies, credentials,
  hidden reasoning or real tool output. Each records the provider format and
  version it represents so a future schema change is deliberate.
- Binding tests cover a direct process-to-session registry mapping,
  working-directory and observed-writes matching, a session bound before its
  first write, a prompt persisted before binding staying behind the cursor, a
  resume carrying only historical turns, two concurrent same-adapter tabs in one
  space, persistent ambiguity, and a session that changes or disappears.

## Answer

Built as `internal/transcript`: the subsystem, the contract, and Claude as its
first implementation. Nothing is wired into the terminal manager — ticket 03 is
the first caller, exactly as ticket 01 was left.

**The seam.** A consumer holds a `Watcher` for one resolved `proc.Agent` and
calls `Poll` on its own beat. It sees two events and no provider storage format:
`NativeTitle` (the provider's own title for the session, published when it
appears and again whenever it changes) and `HumanTurn` (the operator's own text
and the final visible assistant text answering it). Completion is carried by the
event's existence — a turn that is still running, was interrupted, ended in an
error or produced nothing visible never arrives — so no consumer has to ask
whether a turn is over. Both sides of a turn are bounded to 2000 runes at the
seam; the generator's own, smaller context budget stays ticket 03's business,
and the bound here is structural, so a pasted logfile never sits in a watcher or
crosses the seam whole. `Adapter` (bind) and `Session` (id, poll) are the whole
of what a provider implements; `adapters` is a one-row table beside the adapter
package's other per-agent tables, and an agent with no row is simply not
watchable.

**Unavailable is the only failure.** Nothing in the package returns an error to
anyone. No candidate, several candidates, an unreadable store, a provider
version whose records it does not recognize — all of them are silence, retried
on the next poll, because a poll is exactly the moment an ambiguous match could
have become unique. A watcher whose binding ends drops it and binds again, and
since every binding seats at the end of the transcript, a rebind cannot
resurrect history as new turns.

**Claude's store, measured first-hand** (2.1.22x–2.1.23x, this host): per-session
append-only JSONL at `projects/<encoded-cwd>/<session-uuid>.jsonl` under the
resolved state root, plus `sessions/<pid>.json` — a live process-to-session
registry carrying pid, session uuid, working directory and session start. That
registry is the direct mapping the specification says to prefer, and it is what
lets two Claude tabs in one directory each find their own conversation instead
of both finding neither. It is trusted only when the entry is for this pid, in
this process's directory, and names a session that began after the process did;
the last is the pid-reuse guard, since an entry left by a previous holder of the
pid describes a session older than the process reading it. Without a usable
entry, discovery falls back to the specification's other route: session files
written since the agent started whose own head records name this working
directory, one candidate or none. The private encoding of the project directory
is deliberately not reimplemented — a session uuid is unique across the tree, so
the file is found by that validated uuid under any project directory and
confirmed by the directory recorded inside it. No path ever comes from record
content.

**The native title is the `ai-title` record**, not the registry's `name` field.
Both are "native" in some sense, but `name` is a session handle Claude derives
from the directory (`chartr-5b`, `macdirstat-c7`) while `ai-title` is a title of
the conversation ("Read handoff ticket in chartr folder"). Since a native title
blocks paid generation for the life of the session, taking the handle would have
pinned every Claude tab to a directory slug forever. This is the one judgment
call in the ticket; it is recorded here because the specification does not
distinguish them. The `ai-title` records live in the transcript stream, so a
change is picked up by the same incremental read as everything else, and a
resumed session's title is read out of history at binding time while its turns
stay behind the cursor.

**What counts as a turn.** Claude records tool results in the user role, so the
role decides nothing. A turn opens on a user record Claude itself marked as
typed by a human (`promptSource: "typed"`, `origin.kind: "human"`) that is not a
sidechain, not meta, carries no tool result, and is text only — which excludes
slash-command envelopes, caveats, interruption notices, image prompts and
subagent traffic, and includes chartr's own opening prompt, since the provider
records an argument it was launched with exactly as it records typing. It closes
on the next assistant record that both finished (`stop_reason: "end_turn"`) and
has visible text. Reasoning-only finished records do not close it: Claude ends a
long turn by writing the reasoning it stopped on as its own finished record and
the answer in the next one, so treating the first as the end would lose most
working turns. Tool results and meta records do not interrupt a pending turn;
any other user-role record does.

**Cursors.** Binding reads the transcript as it stands, keeps only the title,
and seats a byte offset at the end of the last complete record; a turn already
under way at that moment is dropped whole, which is what leaves a manually
started agent's launch-command prompt untitled rather than rescued by comparing
timestamps. A session known to the registry but not yet written to binds at
offset zero — the end of an empty transcript — which is how a chartr-launched
tab sees its own opening turn. Appends are read from the offset; an incomplete
trailing record is left unconsumed until its newline lands; a file shorter than
the cursor is re-seated at its new end. A store that disappears, or a registry
that starts naming a different session, ends the binding.

**Failing closed.** A complete line that is not JSON, a record with no `type`,
or a user/assistant record whose message envelope is not the shape the adapter
reads turns out of all end the binding — including on the re-bind, so a drifted
store stays quiet rather than flapping. Unknown record *kinds* are ordinary and
ignored; Claude adds them regularly and skipping a kind is not a guess about its
contents.

**Tests.** The shared harness (`contract_test.go`) drives a `Watcher` through
eleven scenarios — history behind the cursor, native title on resume, append
after cursor, partial final record, replacement, machinery, provider-authored
user-shaped records, absent and never-present stores, malformed records, schema
drift, bounded text — against a `contractStore` a provider supplies; Claude's is
the first, and the five remaining providers join by implementing it and adding a
row. Binding tests cover the registry mapping preferred over an ambiguous
directory, directory-and-writes matching, a stale registry with an idle store,
binding before the first write, a prompt persisted before binding, a resume, two
tabs in one space, persistent ambiguity clearing, a session that changes, and
one that disappears. Claude-specific tests cover title changes and flattening,
tool stops, split turn endings, interruptions, image prompts, meta records, and
a privacy-boundary test that puts sentinels in reasoning, tool input, tool
result, system material, subagent traffic and an earlier turn and proves only
the prompt and the final answer cross the seam. Fixtures are synthetic Go
records — no copied bodies, credentials, reasoning or real tool output — and
name the format and version they represent.

**Verified.** `make test`, `make vet`, and `GOOS=windows`/`GOOS=linux` builds.
Beyond the suite, the normalizer was run over all 333 real transcripts on this
host: 769 turns, zero unreadable files across four provider versions, and exact
per-file agreement with an independently written reference; and three live
Claude processes each bound to their own session through the registry and
surfaced their own native title with no historical turn emitted.

**Excluded.** No terminal-manager or titler wiring, no scheduling, no settings
copy (ticket 03). No adapter for the five remaining providers and no rows in the
table for them (tickets 04 and 05). Two things worth carrying into ticket 03:
binding should be attempted from the moment a tab's agent is known and on every
tick, because the registry-less route cannot see an opening turn — the first
record naming a working directory is often the prompt itself — and because a
Claude session that has not yet been written to is exactly what binds at offset
zero; and Claude usually writes an `ai-title` within the first turn, so for this
provider the free path will normally win the race against the paid one.
