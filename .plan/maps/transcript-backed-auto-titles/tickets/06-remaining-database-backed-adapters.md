---
type: task
blocked_by: [03, 04]
undermined_by: []
---

# Remaining database-backed transcript adapters

## Question

Implement the transcript adapters for every provider the research ticket assigns
to the database-backed family. If that research finds no such provider, close this
ticket as ruled out rather than inventing work for it.

These adapters satisfy the same shared contract and are exercised by the same
contract harness as the JSONL ones — one behavioral contract across providers,
with no provider difference visible in the cockpit and no special case in the
scheduler.

What differs is reading. The agent owns the database and is writing to it live, so
queries are read-only, incremental and indexed, and must never block or contend
with the writer. Several open tabs must not produce repeated full-history reads or
expensive scans. Nothing chartr does may modify a provider's store.

As with every adapter, each sniffs the fields it requires and becomes unavailable
on an unrecognized schema rather than guessing, and a store that is missing,
locked or unreadable is an expected unavailable state that produces no title and
no error surfaced to the operator.

## Done when

- Every provider assigned to the database-backed family has an adapter satisfying
  the shared contract, exercised by the shared contract harness.
- Each declares its own state-root environment allowlist and documented default,
  as data beside the existing per-adapter tables.
- Queries are read-only, incremental and indexed, safe against a live writer, and
  several open tabs do not cause repeated full-history reads or expensive scans.
- No chartr operation writes to, migrates or locks a provider's store.
- Native titles are exposed where the provider has them, and block paid generation
  for those tabs exactly as Claude's do.
- An unrecognized schema, or a missing, locked or unreadable store, makes that
  adapter unavailable, yielding no title and no operator-facing error.
- Fixtures are synthetic, contain no copied transcript bodies, credentials, hidden
  reasoning or real tool output, and record the provider format and version they
  represent. Live-write behavior is covered.
- A live tab on each of these agents titles under the same first-turn rule, the
  same fifteen-minute and three-turn refresh gate, and the same bounded prompt and
  final-response context.
- Two tabs of the same provider open in one space bind to their own persisted
  sessions.
