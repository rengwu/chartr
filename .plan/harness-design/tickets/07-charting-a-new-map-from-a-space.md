---
type: grilling
blocked_by: [03, 04]
claimed_by: claude-opus-4-8
claimed_at: 2026-07-19T08:12:39Z
---

# Charting a new map from a space

## Question

Every other action in the cockpit assumes a map already exists. This one does not: a space is registered, it has no `.plan/` at all, and the human wants to start wayfinding. The cockpit has to get them from an empty space to a charted map.

Charting is itself a wayfinder session — name the destination, grill the frontier breadth-first, write the map and its first tickets — and it is emphatically HITL: the skill is explicit that an agent answering its own grilling questions has broken the method. Decide what the cockpit actually offers.

Settle:

- **The shape of a charting session.** It is spawned with the charting prompt injected and **no ticket bound to it** — every other session binds to exactly one. What does a ticketless session mean for a state model that so far assumes session ↔ ticket?
- **Who names the effort**, and where the slug comes from.
- **What the star-map shows** while a map is being charted and does not yet exist — and immediately after, when it exists with nothing resolved. A map of all-open stars is the one view the viewer was never really designed for.
- **Adopting a space whose `.plan/` was charted outside the harness** — probably the *common* case, since wayfinder maps exist today without one. Does that path differ from charting fresh, and does the harness need to verify a map it did not create (the adapter's `lint` exists for exactly this)?
