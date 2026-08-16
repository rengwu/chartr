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
