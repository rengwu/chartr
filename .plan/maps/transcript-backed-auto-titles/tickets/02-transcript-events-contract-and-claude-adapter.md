---
type: task
blocked_by: [01]
undermined_by: []
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
- a completed top-level human turn, carrying that turn's textual user prompt, the
  final visible assistant text, and a stable transcript turn identity.

Nothing else is emitted. Hidden reasoning, system and developer instructions,
tool calls, tool results, intermediate assistant messages, subagent traffic,
sidechains, summaries, slash commands, permission choices and synthetic messages
are not events. Records that are not a top-level human text turn with a final
visible response cannot become one.

Binding is conservative and must be unique. A candidate session is matched from
the adapter, the foreground process or process group, the working directory, the
process-start time, the provider session identity, and observed transcript
writes. Where a provider exposes a direct process-to-session registry, prefer it.
Ambiguous candidates stay pending and retry cheaply as new metadata arrives;
persistent ambiguity produces no binding, silently.

Binding seats a cursor at the live process boundary. Existing history establishes
position but is never emitted as an eligible turn — that is what keeps a resumed
session from being charged for work already done. The one carve-out: an event
timestamped at or after process start stays eligible even if it was persisted
before discovery finished, which is how a prompt supplied on an agent's launch
command still counts.

Transcript formats are versioned external formats even where a provider publishes
no stable schema. Each adapter sniffs the fields it requires and becomes
unavailable on an unknown shape rather than guessing, so a provider upgrade
cannot turn unrelated persisted data into title context.

Claude is the first implementation and the proof of the contract. Its store is
append-only JSONL, so incremental reading maintains byte offsets and tolerates an
incomplete trailing record.

Transcript bodies are ephemeral inputs. Runtime state keeps session and turn
identities, cursors, the native title, and reading position — never a second copy
of the conversation.

## Done when

- A single transcript subsystem owns discovery, binding, incremental reading and
  normalization, and its consumers see provider-neutral events only.
- The normalized event model distinguishes native-title changes, top-level human
  turns, final visible assistant text, completion, and stable transcript turn
  identity, and carries nothing beyond what title scheduling needs.
- One adapter contract is defined that a provider implementation satisfies, with
  the Claude adapter as its first implementation: discovery, native title,
  top-level user text, final visible assistant text, completion, stable turn IDs,
  incremental cursors, and unknown-schema failure.
- Discovery reads metadata before message bodies, and provider paths are
  constructed from validated identity rather than from a transcript path found
  inside message content.
- A binding is unique or it does not exist. Ambiguous candidates remain pending
  and retry cheaply; persistent ambiguity yields no binding and no error surfaced
  to the operator.
- The cursor is seated at the process-start boundary. Historical turns never
  become eligible turns. An event at or after process start is eligible even when
  persisted before discovery completed.
- Resuming a persisted session surfaces its native title while its history stays
  behind the cursor.
- Incremental reading maintains byte offsets, tolerates an incomplete trailing
  record, and handles append after cursor, file rotation and file replacement
  without re-reading full history.
- An unrecognized record shape makes the adapter unavailable rather than parsed
  on a guess.
- A shared contract harness exercises the adapter against fixtures covering
  partial final records, append after cursor, rotation or replacement,
  unavailable stores, malformed records, ignored record kinds, synthetic
  user-like records, and schema drift.
- Fixtures are synthetic: no copied personal transcript bodies, credentials,
  hidden reasoning or real tool output. Each records the provider format and
  version it represents so a future schema change is deliberate.
- Binding tests cover a direct process-to-session mapping, working-directory and
  process-start matching, an initial prompted launch written before binding, a
  resume carrying only historical turns, two concurrent same-adapter tabs in one
  space, persistent ambiguity, process-identity reuse, and a session that changes
  or disappears.
