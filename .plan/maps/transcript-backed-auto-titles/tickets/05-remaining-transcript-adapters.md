---
type: task
blocked_by: [03, 04]
undermined_by: []
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
