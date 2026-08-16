---
type: task
blocked_by: [03, 04]
undermined_by: []
claimed_by: scbdfa9e5612a
claimed_at: 2026-08-16T10:36:14Z
---

# Remaining transcript adapters

## Question

Implement the transcript adapters for every provider the research ticket assigns,
so their tabs title under exactly the same rules Claude's already do.

Each adapter satisfies the existing shared contract and is exercised by the same
contract harness. There is one behavioral contract across providers: a provider
difference must not be visible in the cockpit, and must not leak into the
scheduler as a special case. New per-adapter knowledge — the state-root
environment allowlist, the record shapes, the native-title location — belongs
beside the existing per-adapter data tables for prompt delivery and headless
generation, as data rather than as branches in the caller.

Only the reader differs by storage family, and it hides behind the same cursor:

- Append-only JSONL reads incrementally by byte offset and tolerates an
  incomplete trailing record, since the agent is writing while chartr reads.
  Appends after the cursor, and file truncation or replacement, must not cause a
  full-history re-read.
- A database-backed store keeps a cursor over an indexed column and issues small
  read-only queries such as `WHERE rowid > :cursor LIMIT N`. The agent owns the
  database and is writing to it live, so queries must never block or contend
  with the writer, several open tabs must not produce repeated full-history
  reads, and nothing chartr does may modify the store.

Each adapter sniffs the fields it requires and becomes unavailable on an
unrecognized shape rather than parsing on a guess, so a provider upgrade cannot
turn unrelated persisted data into title context. A provider the research ticket
recorded as having neither a usable native title nor a safe one-shot recipe gets
no adapter at all; its tabs stay untitled until the provider gains one.

## Done when

- Every provider assigned by the research ticket has an adapter satisfying the
  shared contract, exercised by the shared contract harness rather than by a
  provider-specific test path.
- Each declares its own state-root environment allowlist and documented default,
  as data beside the existing per-adapter tables.
- JSONL reading is incremental by byte offset and survives an incomplete trailing
  record, appends after the cursor, and file truncation or replacement without
  re-reading full history. Database reading is read-only, incremental and
  indexed, safe against a live writer, and never writes to, migrates or locks
  the store.
- Native titles are exposed where the provider has them, and block paid
  generation for those tabs exactly as Claude's do.
- An unrecognized record shape or schema, or a missing, locked or unreadable
  store, makes that adapter unavailable — no title, no operator-facing error, and
  no other adapter or provider used in its place.
- Fixtures are synthetic, contain no copied transcript bodies, credentials, hidden
  reasoning or real tool output, and record the provider format and version they
  represent. Live-write behavior is covered for database-backed stores.
- A live tab on each of these agents titles under the same first-turn-only rule
  and the same bounded prompt and final-response context, with no provider
  difference visible in the cockpit.
- Two tabs of the same provider open in one space each bind to their own
  persisted session where distinguishable, and stay untitled where not; neither
  displays or transmits the other's conversation.

## Answer

**Current outcome after simplification.** Codex, Pi, Kimi and Grok remain under
the shared JSONL contract. OpenCode's adapter and `internal/sqlite` were removed:
a private SQLite/WAL reader was too much owned infrastructure for a minor,
best-effort title feature. OpenCode remains fully supported as a terminal agent;
its tabs simply receive no transcript-backed automatic title. The implementation
account below is retained as history of what was built and then deliberately
removed.

All five providers have adapters. Codex, OpenCode, Pi, Kimi and Grok join Claude
under the one `Adapter`/`Session` contract and the one harness, so every agent
chartr ships a detection manifest for now takes its title from its own persisted
session. The "no adapter" branch stays unused, as ticket 04 predicted.

**The reader is shared; only the storage family differs.** `tail` is the
byte-offset cursor the five JSONL providers use — it leaves an incomplete
trailing record unconsumed, reads forward from where the last poll stopped, and
re-seats in history mode when a store turns out shorter than the cursor. Claude
moved onto it, so there is one JSONL reader rather than five. `sidecar` is the
second cursor: Kimi's `state.json`, Grok's `summary.json` and Claude's own
process registry are rewritten rather than appended to, so a stat replaces an
offset. Every provider difference lives in its own `fold`.

**OpenCode needed a database reader, and it is `internal/sqlite`, written here.**
The agent owns `opencode.db` and streams into it while chartr reads, so chartr
must not write, migrate, lock or contend with it — not even through the `-shm`
file an ordinary read-only connection creates. So the reader opens the file
`O_RDONLY` and reads pages: header, write-ahead log, `sqlite_schema`, rowid
b-trees. Writing is not implemented, which is a stronger promise than not
writing. Only frames up to the last commit frame are visible and each is checked
against the log's salts and running checksum, so a transaction still being
written is unreadable rather than half-read; a checkpoint that moves the ground
mid-read is caught by `Stable` and the batch is discarded. Its tests read
databases the real `sqlite3` wrote with the writer still holding them open — a
reader tested against a writer of its own making would only prove the two agree.
The alternatives were a large pure-Go driver or shelling out per poll; the human
chose this one. It skips where `sqlite3` is not installed, which is the one place
in the suite that is conditional.

**New per-adapter knowledge went to the existing data tables.**
`internal/adapter/stateroot.go` gains rows for codex, opencode, pi and grok, and
the per-variable suffix ticket 04 asked for — `XDG_DATA_HOME` + `/opencode`,
`PI_CODING_AGENT_DIR` + `/sessions`. `StateRootEnv` unwinds that suffix, so a
generation lands on the same root the tab resolved rather than one segment below
it. `titlegen.go` gains the four measured headless recipes; each contributes only
its CLI's own default model, because no cheap model id for those four has been
run first-hand and inventing one is the guess that table exists to avoid.

**What each adapter had to know, and nowhere else.** Codex sniffs `history_mode`
to choose between two incompatible record families that one build writes both of,
reads only the `event_msg` stream and never the model wire, and binds only a
`thread_source: user` / `originator: codex-tui` rollout — which excludes
subagents and chartr's own `codex exec` generations by the same predicate.
OpenCode closes a turn on a `step-finish` part with `reason: "stop"`, re-reads
the turn's rows at closing time because they are rewritten in place while a
response streams, and refuses the `New session - <ISO>` placeholder title. Pi
refuses anything but session format 3 and takes its title from the latest
`session_info`. Kimi normalises a turn identity written as a string in the loop
events and an integer in `turn.ended`, and publishes a title only when
`titleKind` is `generated` or `custom`. Grok concatenates chunked messages and
ignores its own headless runs for free, since those write no `updates.jsonl`.

**The harness grew three things.** `Title` now reports whether the provider has a
native title at all — Codex does not, and the same scenario proves both halves,
so "has a title" never quietly becomes "the test was skipped". A native title
change must republish with no debounce. And two tabs of one provider in one space
must each see their own session or neither see anything: only Claude's registry
can tell two apart, so the other five bind nothing there, and no tab ever carries
the other's text. Both ignore-scenarios now append a real turn afterwards,
because a binding that *died* on a record it should have read past produces no
events either.

**Verified against this host's real stores**, by running the adapters over them
read-only: 111 of 157 Codex rollouts (the other 46 refused at the head as
subagents, `codex exec` runs, and one pre-`history_mode` file), all 362 Pi
sessions, all 83 Kimi wire logs, all 5 Grok update logs and all 36 OpenCode
sessions. That found two fields the fixtures had invented wrongly — OpenCode's
`summary`, which is a boolean on an assistant message and a diff object on a user
message, and Grok's `content`, which is an object on a message chunk and a list
on a `tool_call_update`. Both are now read raw and typed only where a turn is
read out of them. Kimi's turn identity matched on every real turn; the three of
34 completed turns not emitted had no human-origin text prompt, none an identity
mismatch.

**Verified against live processes**, by starting each agent's TUI in a pty in a
fresh directory: all five resolve their state root correctly, including the two
that name a parent.

**One ticket-04 finding is corrected, and it changes behaviour.** All five
providers create their store *lazily, at first submit* — not at session start.
Each TUI was left at its prompt for forty seconds with its trust dialog accepted
and none wrote a session a binding could find; OpenCode inserted no `session`
row, which settles the open question its section records. So no tab on these five
can bind before its first submission, and that first turn arrives already behind
the cursor. **The second turn is the first one that can be titled.** Combined
with the human's ruling that a store absent at bind time still seats at the end,
this is the settled behaviour: it costs a title on single-turn sessions, and
OpenCode's and Grok's native titles land free during that same first turn anyway.
The asset carries the correction inline.

**Not verified.** No live agent was driven through a complete turn, so no paid
generation ran end to end on any of the five: the turn-level behaviour rests on
the adapters reading this host's real stores plus the manager tests ticket 03
left. That needs a real model call per provider and is the one thing left to
check on a machine where spending is intended. The four default-only generation
rungs are likewise unexercised. Windows is out of scope, and the cross-platform
build still compiles with no process reader.

Verified: `make test`, `make vet`, `make check`, `go test -race` on transcript,
sqlite and terminal, and `GOOS=windows` / `GOOS=linux` builds.
