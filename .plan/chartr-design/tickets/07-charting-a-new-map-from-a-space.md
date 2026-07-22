---
type: grilling
blocked_by: [03, 04]
---

# Charting a new map from a space

## Question

Every other action in the cockpit assumes a map already exists. This one does not: a space is registered, it has no `.plan/` at all, and the human wants to start wayfinding. The cockpit has to get them from an empty space to a charted map.

Charting is itself a wayfinder session — name the destination, grill the frontier breadth-first, write the map and its first tickets — and it is emphatically HITL: the skill is explicit that an agent answering its own grilling questions has broken the method. Decide what the cockpit actually offers.

Settle:

- **The shape of a charting session.** It is spawned with the charting prompt injected and **no ticket bound to it** — every other session binds to exactly one. What does a ticketless session mean for a state model that so far assumes session ↔ ticket?
- **Who names the effort**, and where the slug comes from.
- **What the star-map shows** while a map is being charted and does not yet exist — and immediately after, when it exists with nothing resolved. A map of all-open stars is the one view the viewer was never really designed for.
- **Adopting a space whose `.plan/` was charted outside the chartr** — probably the *common* case, since wayfinder maps exist today without one. Does that path differ from charting fresh, and does the chartr need to verify a map it did not create (the adapter's `lint` exists for exactly this)?

## Answer

**Charting is not a chartr capability — it's something users do *in* the chartr with their own tools.** The reframe that settles this ticket: the chartr is a **space-based terminal multiplexer first** — tmux/herdr with wayfinder superpowers — and charting is the wayfinder flow Matt Pocock describes (`/grill-with-docs <idea>` → "this is bigger than I thought" → `/wayfinder make a map` → carry on), run by the user in an ordinary shell. The chartr does not own it, inject for it, or drive it. So the ticket's own premise — *"spawned with the charting prompt injected"* — is **overridden**: the chartr injects nothing to chart, and there is **no "charter" role**; the five roles ([grill/prototype/research/implement/review](./05-mapping-roles-to-agents.md)) stay closed.

**The shape of a charting session — it isn't a session.** `session ↔ ticket` stays a **hard invariant**, not a soft rule with a charting asterisk. Rejected: binding a session to *the map* (breaks `finished`-from-ticket per [ADR 0004](../../../docs/adr/0004-derived-ticket-state-and-proposed-answer.md), leaves ticket 09's moon with no star to orbit) and binding to a *synthetic ticket* (theatre that fabricates state the design otherwise derives). Both bend the model so charting *looks* normal and make every consumer branch on it. Instead, charting rides the two on-ramps that are **not** sessions:

- **Ad-hoc shells run wild.** A plain or agent shell in the space, **zero chartr injection**, the user's own skills and setup. [Ticket 08](./08-the-cockpit-layout.md) already seats ad-hoc shells in the terminal column, so this is existing ground. This is the "just a multiplexer" baseline — the chartr is usable with no map at all.
- **The "ideate" on-ramp — the one opinionated nudge.** A small affordance ("new idea" / "chat about your idea") spawns an open-ended session with a **chartr-provided starter prompt** that prods the user on what's on their mind and, if the idea proves big, *suggests* escalating (`/wayfinder make a map of this`). This is a QoL nudge toward the intended flow, and the concrete face of the ticketless bootstrapping mode. Three constraints hold it in line: it is a **button, not a sixth role**; its starter prompt is **on-disk hackable markdown** like every other prompt ([ticket 04](./04-the-prompt-library-the-chartr-injects.md)'s standing preference), filed explicitly as a *non-role* on-ramp prompt so 04's role set stays closed; and **escalation is advisory** — the agent suggests, the user runs the skill. Never a chartr state transition; the chartr does not track "ideation → charting."

Both are **ticketless and live**: like a planning session they are worked with the human and subject to no review gate, and they **end when the human ends them** — the chartr never derives `finished` for them. The only machinery shared with a real session is the adapter's `spawn(cwd, model, promptText)` primitive.

**Who names the effort.** The `/wayfinder` skill writes the map folder and **owns the slug**; the chartr reads the directory name and **never prompts for a name**. (Forward note: `/wayfinder` will move from `.plan/<slug>/` to `.plan/maps/<slug>/`. The chartr reads wherever wayfinder writes — map location is wayfinder's storage shape, out of scope here — so discovery must follow the convention, not hard-code the old path.)

**What the star-map shows.** The star-map is a *summoned overlay*, not the primary surface ([ticket 08](./08-the-cockpit-layout.md)), which makes the empty cases trivial:

- **Empty space, or an idea being grilled with no map yet → no star-map at all.** Nothing to summon; the cockpit is just terminals. The multiplexer stands alone.
- **A map just born, nothing resolved (all-open) → render it as-is, no special view.** The `blocked_by` edges still split it into an unblocked frontier vs. tickets waiting on prerequisites, so an all-open map reads as an *early* map, not a broken one, and [ticket 08](./08-the-cockpit-layout.md)'s Next-up still works. The "view the map was never designed for" turns out not to need designing for.

**Adopting a map — the one and only path.** The chartr watches the space's `.plan/` and **notices** when a map appears — [ADR 0007](../../../docs/adr/0007-map-kind-declared-not-inferred.md)'s notice-don't-drive pattern — whether it came from a chartr-hosted shell, an external terminal, or a `git pull`. So the ticket's "chart fresh vs. adopt from outside" distinction **dissolves**: there is no separate "charted inside" path to differ from. **Notice → auto-detect kind → one-click confirm** is how *every* map enters the chartr.

- **Classification honours [ADR 0007](../../../docs/adr/0007-map-kind-declared-not-inferred.md).** Kind is auto-*detected* from convention but **pre-filled for a single human confirm**, never applied on the heuristic alone — because a mis-detected implementation-map-read-as-planning would resolve code **unreviewed, silently**, the highest-cost, quietest failure the design has. **Provenance strengthens the pre-fill**: a map produced by `/to-tickets` off a known planning map is a near-certain guess; a hand-dropped `.plan/` is softer. Same one tick either way — and that tick is the entire difference between review-on and review-silently-off.
- **No blocking lint at adoption.** The chartr does not gate adoption on the adapter's `lint`; it renders the map as-is and **surfaces** malformations where they bite (a dangling `blocked_by`, an unparseable ticket) rather than refusing the map — consistent with [ticket 06](./06-who-commits-and-how-work-gets-abandoned.md)'s surface-don't-enforce and the trust-at-the-gate stance. `lint` stays an available diagnostic, not an admission test.
