# Glossary

The shared vocabulary of the wayfinder method and this harness. Use these words
as defined; they are how the map, the tickets, and this session mean the same
thing.

- **Map** — a wayfinder effort: a `map.md` and its tickets. Always either a
  *planning map* (tickets resolve decisions, worked live with a human) or an
  *implementation map* (tickets deliver code against a settled spec).
- **Ticket** — one question or one unit of work, sized to a single session. Its
  status is derived from its file, never stored in it.
- **Frontier** — the open, unblocked tickets: the edge of the known. A blocker
  counts as cleared the moment its answer is written.
- **Blocker** — a ticket this one depends on. Its `## Answer` is a premise you
  build on; it is handed to you in your context.
- **Question / Done-when** — a ticket's two halves: what it asks, and the concrete
  condition that makes it done. On an implementation ticket the Done-when is the
  contract the work is judged against.
- **`## Answer`** — a resolved ticket's conclusion, written by the session that
  did the work. On disk, an Answer means the ticket is settled.
- **`## Ruled out`** — a ticket closed as out of scope: a decision not to travel
  this route, recorded so it is not rediscovered.
- **Session** — an agent working one ticket. Context is assembled fresh at spawn
  and never accumulated between sessions.
- **Role** — what a session is spawned to do: grill, prototype, research, or
  implement.
