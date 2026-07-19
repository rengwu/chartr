---
type: prototype
blocked_by: [02]
claimed_by: claude-fable-5
claimed_at: 2026-07-19T06:53:57Z
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

Prototype: [09-starmap-harness-states.html](../assets/09-starmap-harness-states.html) — open in a browser; three overlay variants on one ported star-map, cycled with ←/→ or `?variant=`; `?` explains how each answers the ticket. The base six states keep the shipped viewer's exact palette and motion; each variant proposes a different *orthogonal axis* for the harness states: **A Moons** (the session is a body — an amber mote orbits the star; liveness is its motion: working orbits, quiet crawls, dead freezes grey; proposed docks it at the rim; agent review adds a violet counter-orbiter; human review breaks the orbital language with a gold beacon), **B Rings** (the pipeline stage is geometry, extending claimed's ring language — rotating dashed ring implementing, sealed solid proposed, violet counter-ring agent review, gold reticle human review; nothing orbits, the calmest), **C Signals** (nothing added — state rides on the star's own light: breathing, glint, amber↔violet oscillation; human review pings *and dims the rest of the map*). Shared: layout computed once and never again (▸ plays the whole lifecycle — implementing → quiet → proposed → agent review → human review → approval igniting the dependent → a death — and no star ever moves), one flare + one fading ticker line per live change, an edge-of-screen gold chevron when the human-review star is offscreen, and a live legend of all twelve states. Fixture `1` is expensif/export-csv-impl (implementation), `2` wayfinder/viewer-hardening (planning) — real graphs, scripted session overlay; the planning fixture's idle HITL session deliberately shows no quiet badge (ticket 02).
