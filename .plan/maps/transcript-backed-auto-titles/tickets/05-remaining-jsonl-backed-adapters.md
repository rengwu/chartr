---
type: task
blocked_by: [03, 04]
undermined_by: []
---

# Remaining JSONL-backed transcript adapters

## Question

Implement the transcript adapters for every provider the research ticket assigns
to the append-only JSONL family, so their tabs title under exactly the same rules
Claude's already do.

Each adapter satisfies the existing shared contract and is exercised by the same
contract harness. There is one behavioral contract across providers: a provider
difference must not be visible in the cockpit, and must not leak into the
scheduler as a special case. New per-adapter knowledge — the state-root
environment allowlist, the record shapes, the native-title location — belongs
beside the existing per-adapter data tables for prompt delivery and headless
generation, as data rather than as branches in the caller.

JSONL reading is incremental by byte offset and tolerates an incomplete trailing
record, since the agent is writing while chartr reads. Appends after the cursor,
file rotation and file replacement must not cause a full-history re-read.

Each adapter sniffs the fields it requires and becomes unavailable on an
unrecognized shape rather than parsing on a guess, so a provider upgrade cannot
turn unrelated persisted data into title context. A provider the research ticket
recorded as having neither a usable native title nor a safe one-shot recipe still
gets its adapter, and its tabs simply stay untitled.

## Done when

- Every provider assigned to the JSONL family has an adapter satisfying the shared
  contract, exercised by the shared contract harness rather than by a
  provider-specific test path.
- Each declares its own state-root environment allowlist and documented default,
  as data beside the existing per-adapter tables.
- Reading is incremental by byte offset and survives an incomplete trailing
  record, appends after the cursor, file rotation and file replacement without
  re-reading full history.
- Native titles are exposed where the provider has them, and block paid
  generation for those tabs exactly as Claude's do.
- An unrecognized record shape makes that adapter unavailable, and no other
  adapter or provider is used in its place.
- Fixtures are synthetic, contain no copied transcript bodies, credentials, hidden
  reasoning or real tool output, and record the provider format and version they
  represent.
- A live tab on each of these agents titles under the same first-turn rule, the
  same fifteen-minute and three-turn refresh gate, and the same bounded prompt and
  final-response context, with no provider difference visible in the cockpit.
- Two tabs of the same provider open in one space bind to their own persisted
  sessions, and neither displays or transmits the other's conversation.
