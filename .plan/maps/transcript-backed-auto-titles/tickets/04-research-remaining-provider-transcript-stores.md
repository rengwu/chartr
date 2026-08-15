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
assistant text, completion, stable turn identities, incremental cursors, and
failure on an unknown shape. This ticket answers what each provider offers
against it.

Two questions decide how the remaining work is cut. First, storage family:
append-only JSONL adapters maintain byte offsets and tolerate an incomplete
trailing record, while database-backed adapters need read-only, incremental,
indexed queries that are safe against a live writer. Second, capability: a
provider that exposes neither a usable native session title nor a safe headless
one-shot generation recipe leaves its tabs permanently untitled. That is an
accepted outcome under this specification, but it must be recorded deliberately
rather than discovered during implementation.

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
- What discovery metadata is available for binding — session identity, working
  directory, process-start information — and whether the provider exposes a
  direct process-to-session registry that binding should prefer.
- Whether a safe headless one-shot generation recipe exists for the no-native-title
  case, or whether the provider can contribute none.
- The specific installed provider version each observation was made against.

And overall:

- Each of the five providers is assigned to the JSONL-backed adapter ticket or the
  database-backed adapter ticket. If no provider is database-backed, the answer
  says so explicitly so that ticket can be ruled out.
- Any provider offering neither a usable native title nor a safe one-shot recipe
  is named as permanently untitled, so the gap is a recorded decision rather than
  a bug found later.
