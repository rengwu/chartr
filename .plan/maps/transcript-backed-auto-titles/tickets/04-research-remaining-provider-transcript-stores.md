---
type: research
blocked_by: [02]
undermined_by: []
assets: [provider-transcript-stores.md]
claimed_by: sa1aa599501bd
claimed_at: 2026-08-16T07:42:23Z
---

# Research: transcript stores for Codex, OpenCode, Pi, Kimi and Grok

## Question

Five agents beyond Claude must reach the same transcript-adapter contract. Before
any of them is written, establish what each provider actually persists and how it
can be matched to a live process, so the adapters are implemented against
observed formats rather than guesses.

The adapter contract already exists and defines what an implementation must
supply: discovery metadata, native titles, top-level user text, final visible
assistant text, completion, incremental cursors, and failure on an unknown shape.
This ticket answers what each provider offers against it.

Two questions decide how the remaining work is cut. First, storage family:
append-only JSONL adapters maintain byte offsets and tolerate an incomplete
trailing record, while database-backed adapters need read-only, incremental,
indexed queries that are safe against a live writer. Second, capability: a
provider that exposes neither a usable native session title nor a safe headless
one-shot generation recipe gets no adapter — its tabs stay untitled until the
provider gains one of those. That is an accepted outcome under this
specification, but it must be recorded deliberately rather than discovered during
implementation.

Record findings as observations of specific installed versions. These are
operational dependencies with no guarantee of a stable schema, so note the
version each observation came from.

Do not copy real transcript bodies, credentials, hidden reasoning or real tool
output into the answer or into any fixture material.

## Done when

For each of Codex, OpenCode, Pi, Kimi and Grok, the answer records:

- Where sessions are persisted, and which environment variables select that
  location — the allowlist the adapter will declare — with the documented default
  when unset.
- The storage family: append-only JSONL or database-backed.
- The record shapes needed to recognize a top-level human text turn, its final
  visible assistant text, turn completion, and a stable turn identity, plus the
  record kinds that must be ignored — synthetic messages, slash commands,
  permission choices, subagent traffic, sidechains, summaries and tool results.
- Which fields an adapter can sniff to detect an unknown schema and fail closed.
- Whether a native session title exists, where it lives, and whether it updates
  over a session's life.
- What discovery metadata is available for binding — session identity and working
  directory — and whether the provider exposes a direct process-to-session
  registry that binding should prefer.
- Whether a safe headless one-shot generation recipe exists for the no-native-title
  case, or whether the provider can contribute none.
- The specific installed provider version each observation was made against.

And overall:

- Each of the five providers is assigned to ticket 05 — with its storage family
  recorded, since that decides the reader shape — or named as getting no adapter
  at all.
- Any provider offering neither a usable native title nor a safe one-shot recipe
  is named as getting no adapter until it gains one, so the gap is a recorded
  decision rather than code that can never do anything.

## Answer

All five providers were measured first-hand on this host and **all five are
assigned to ticket 05**. None turned out to lack both a usable native title and
a safe headless recipe, so no provider is left permanently untitled and the
"no adapter" branch this ticket was asked to consider goes unused.

The full findings — per-provider paths, env allowlists, record shapes, ignore
lists, sniff fields, title semantics, discovery metadata and generation recipes,
each tied to the version it was observed on — are in
[assets/provider-transcript-stores.md](../assets/provider-transcript-stores.md).
What follows is the part that changes how the remaining work is cut.

**The split is four to one.** Codex, Pi, Kimi and Grok are append-only JSONL and
take the byte-offset reader Claude already has. **OpenCode alone is
database-backed** — SQLite under WAL, read through `session`/`message`/`part`
with incremental `rowid` queries. Kimi and Grok are JSONL *plus* a rewritten JSON
sidecar (`state.json`, `summary.json`) that holds the title, so their readers
tail a log and poll one small file; that is a variation on the JSONL reader, not
a third family.

**Native titles.** OpenCode and Grok have real ones and will usually win the
free race, exactly as Claude does. Both need a filter, and both filters are
load-bearing: OpenCode's title starts as the placeholder
`New session - <ISO8601>` (its own `Session.isDefaultTitle` regex, reproduced in
the findings), and Kimi's `state.json` title is only a title when
`titleKind ∈ {generated, custom}` — the default `replaceable` is the prompt
text capped at 200 characters. Publishing either unfiltered would pin a tab to a
timestamp or a whole prompt *and* block paid generation for the life of the
session. Codex has no usable native title at all: its `threads.title` is
byte-identical to the first user message on all 155 rows here, up to 53k
characters. Pi's `session_info.name` exists but is user-set only, and no session
in a 362-file corpus had one.

**One-shot recipes exist for all five** (`codex exec`, `opencode run`,
`pi -p`, `kimi -p`, `grok -p`), and Codex's is already shipped in
`titlegen.go`. Only `pi --no-session` writes nothing; the other four persist a
session in the directory chartr binds against. That is bounded rather than
dangerous — generation only runs for an already-bound tab — and Codex and Grok
exclude their own generations for free through filters they need anyway.

**Two things ticket 05 must change rather than just implement.** First,
`internal/adapter/stateroot.go` returns an allowlisted variable's value as the
root verbatim; OpenCode (`XDG_DATA_HOME` + `/opencode`) and Pi
(`PI_CODING_AGENT_DIR` + `/sessions`) name a *parent*, so `stateRootSpec` needs
a per-variable suffix. Second, Codex's `session_meta.history_mode` selects
between two incompatible record families (`paginated` uses `item_completed`
items, `legacy` uses `user_message`/`agent_message` events) and 0.147.0 writes
both, so it must be sniffed like a version even though it is not one.

**None of the five exposes a process-to-session registry.** Claude's
`sessions/<pid>.json` remains unique. Codex's `thread-writer-locks/` names
threads rather than holders, and Grok's `active_sessions.json` is a stale
artifact of the previous build — the string does not appear in 1.0.0. All five
bind through the specification's other route: working directory plus sessions
written since the agent started, unique or nothing.

**One decision is deliberately left to a human.** Pi buffers its session in
memory and creates the file only once the first assistant message exists,
writing header, opening prompt and answer together. Under the seat-at-end rule a
Pi tab can therefore never title its first turn, and a single-prompt session
stays untitled. The findings record one option — seat at offset zero when chartr
observed the store's *absence* at bind time, the analogue of Claude's
registry-known-but-unwritten case — but flag it as a reading of the
specification rather than an implementation detail. OpenCode may have the same
shape; whether its `session` row is inserted at TUI start or at first submit was
not established first-hand and is noted as a live check for ticket 05.

**Excluded.** No adapter code, no table rows, no fixtures — this ticket is the
measurement ticket 05 builds against. Windows was not investigated, per the
map's scope. Nothing was verified against a live running process of any of the
five: every observation is of stores on disk plus the installed programs
themselves, so the claims about what a tab sees *at the moment it binds* are
inferences from write ordering and are marked as such. The weakest observation
is Grok's: 1.0.0 is installed, but this host's only substantive Grok
conversations were written by the immediately preceding build; every field named
is present as a symbol in the 1.0.0 executable and matches its embedded
documentation, but ticket 05 should re-check against a live 1.0.0 session before
trusting the parser. Kimi's `generated` title path and Codex's `threads.name`
rename were both read in the binaries but never observed populated on this host.
