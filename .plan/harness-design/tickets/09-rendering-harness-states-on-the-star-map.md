---
type: prototype
blocked_by: [02]
---

# Rendering harness states on the star-map

## Question

wayfinder-maps' star-map renders six states — resolved, frontier, claimed, blocked, out-of-scope, undermined — through colour, size, glow and pulse, on the principle that *status is the whole star while type rides in the label*. The harness introduces states that viewer never had: **implementing**, **proposed**, **agent review**, **human review**, plus whatever liveness vocabulary ticket 02 settles (working, stuck, dead).

Decide how they read, and prove it by drawing it. The constraints worth respecting rather than discovering:

- **Deterministic layout must survive.** Positions seed from the ticket data so a ticket stays where you learned it, and the design record is emphatic that this spatial memory is the point. Live state must never move a star.
- The palette is **already spent** on six states. Adding five more invites a christmas tree. Is the new axis colour at all — or something orthogonal, like motion, an orbiting mark, or a ring?
- **Human review is a call to action**, not a status. It probably should not read like the states the harness is merely reporting; it is the one star that wants you.
- The star-map becomes **live** for the first time (`fsnotify` → re-derive → push). What does a ticket *changing state under you* look like, given that resolving already perturbs its neighbourhood — and does a live map break the calm the design record was chasing?

Prototype against a real fixture rather than invented data — `../wayfinder/.plan/viewer-hardening` for a planning map, `../expensif/.plan/export-csv-impl` for an implementation map — and link it as an asset.
