---
type: grilling
blocked_by: [01, 02, 03, 04]
---

# Sequencing the work

## Question

Tickets 01–04 settle four mostly-independent decisions. This ticket turns them into an order — one implementation map or several, what lands first, and what "done" means for the effort as a whole. It inherits every answer above; if any of them is shaky, say which and how it changes the sequence.

The naive order is "cut first, then build" — ticket 01's deletion shrinks everything downstream (less UI for ticket 03 to explain, less payload machinery for ticket 02 to repackage, a smaller app for ticket 04 to wrap). The counter-pressures: the webview shell is the operator's oldest want and is almost entirely additive — it could land first without touching the cut; the skills repackaging (02) and the lifecycle cut (01) interact through the payload composer and the review prompt, so doing them in the wrong order means migrating code that is about to be deleted; and every intermediate state must leave a *working* harness, because this repo drives itself — the cockpit is how the work gets done.

Settle:

- **The order.** Which ticket's work lands first, and why — with the dependencies named explicitly rather than gestured at. Where can implementation maps run in parallel (different file surfaces), and where would they collide (composer, config, cockpit screens)?
- **The map shape.** One implementation map for the whole effort, or one per decision? The repo's convention (`to-tickets` graduating a planning map) suggests one — argue whether that holds when the four decisions are this independent.
- **Self-hosting continuity.** At every point in the sequence, the harness must still drive this repo's own maps. Which intermediate state is the most dangerous (lifecycle half-cut? payload composer mid-repackaging?), and what is the escape hatch if the cockpit breaks underneath its own work?
- **Done.** What the operator can *do* at the end that they cannot do today — stated as a short list of concrete capabilities, not virtues. Include what happens to the dead weight that is not part of any ticket (`Probe.svelte`, stale `sessions/` archives, the no-op `make webview` target, tracked `.DS_Store` files, the stray root `node_modules`): swept by a named ticket, or left to rot?
