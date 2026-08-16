---
type: research
blocked_by: [02]
undermined_by: []
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
