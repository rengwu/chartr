---
type: grilling
blocked_by: []
---

# Telling a planning map from an implementation map

## Question

The two map kinds have different lifecycles. A planning map's ticket is grilled live with a human and resolves directly. An implementation map's ticket runs `implementing → proposed → [agent review] → [human review] → resolved`. So the harness must know which kind it is looking at **before it offers any action** — and getting it wrong means either gating a conversation that needed no gate, or letting code resolve unreviewed.

How does it tell? The candidate signals are inferential: `to-tickets` writes to `.plan/<slug>-impl/` and marks every ticket `type: task`, and the wayfinder skill's other types (`grilling`, `prototype`, `research`) do not appear in an implementation map. Are those signals load-bearing enough to infer from, or should the kind be **explicit** — declared in the map body, or in the committed workspace config?

Push on the awkward cases and say what happens in each:

- A hand-written implementation map that never went through `to-tickets` and follows none of its conventions.
- A planning map containing a `task` ticket — the wayfinder skill explicitly allows one, as the type that *does* rather than decides.
- A map whose Notes override "plan, don't do" and carry execution into the map itself, which the skill also permits.

Then decide the sharper question underneath: is **kind** a property of the *map*, or is it derived **per ticket**? If per ticket, can one map hold both lifecycles at once — and if it can, is that a feature or a thing to forbid?
