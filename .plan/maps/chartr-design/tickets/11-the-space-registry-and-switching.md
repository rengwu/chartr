---
type: prototype
blocked_by: [08]
assets: [.plan/chartr-design/assets/11-space-registry-and-switching.html]
---

# The space registry and switching

## Question

A space is a git repository, registered once and switched between (ADR 0003). Design the registry and the switching.

- **Registering.** Point chartr at a path — then what? Must it already be a git repository, and must it already have a `.plan/`? A repo with no maps is legal; ticket 07 charts one. What does chartr write, and *where*: this is the first chartr-owned state that is not a space's own committed config, so it needs a home and a story for what happens when it is lost.
- **Discovery.** wayfinder-maps' picker already lists every map under a project's `.plan/`, and that code is liftable. chartr needs it plus a layer above: many repos, each with many maps.
- **Switching.** What switching means when the space you left has a session still cooking. The switcher has to carry state — *this one is running, that one wants you* — which starts to overlap with the attention question the fog is holding. Decide whether the switcher is where attention lives, or whether attention deserves its own surface.
- **Ordering and scale.** Most-recent, manual, favourites? Is the realistic number of spaces five or fifty? The answer changes the design completely.
- **Removal.** De-registering a space that has live work, and whether chartr ever touches the repository on the way out. It should probably touch nothing — but say so deliberately.

Link the prototype as an asset.

## Answer

**The registry and switcher are variant A ("Harbour"): the branded Spaces sidebar carries registration, discovery and ordering, and cross-space attention lives at two altitudes — *ambient* on the sidebar rows, *actionable* in a summonable queue.** [The asset](../assets/11-space-registry-and-switching.html) holds A canonical plus C "Beacon" (sidebar-only), the rejected pole. Copy was run through impeccable's *clarify* (design rationale kept out of the live strings) and the design through *critique* (degraded single-context, 33/40) — the two constraints it surfaced are folded in at the end.

**The central fork — is the switcher where attention lives, or does attention deserve its own surface? Both, at two altitudes.** *Ambient* attention rides the sidebar rows: a liveness dot, a wants-you flag, and [ticket 09](./09-rendering-chartr-states-on-the-star-map.md)'s gold beacon pinging on a row whose map has a review waiting — passive awareness that never demands. *Actionable* attention is a **summonable cross-space queue** ("Needs you"), popped from a ⚑ pip in the Spaces header: only **gate-level** signals — a human review at the gate, a session that died — ranked reviews-first, one click to jump to that space with its map summoned and the ticket in focus. It is the **global counterpart to [ticket 08](./08-the-cockpit-layout.md)'s per-map "Next up" action station** — same summonable-overlay treatment, never a fourth always-on panel — and it **realises the "global everywhere-queue" that ticket 08 explicitly left as a lead for this ticket.** Rejected: **queue-as-a-resident-rail** (steals width the terminal wants, a monitoring posture — 08's variant F) and **sidebar-only** (no single "what needs me across everything" digest when you surface from an hour heads-down). The overlap with the fog stays honest: per-map attention stays in 08's action station; this queue is strictly the *union of gate actionables across spaces*, nothing more. **The queue never auto-surfaces** — strictly pull, echoing [ticket 10](./10-the-human-review-hub.md)'s *suggestion, never a shove*; the ambient beacon and the pip's badge are the only push.

**Registering — a space must be a git repository; a `.plan/` is not required.** The working tree is the unit of serialisation ([ADR 0003](../../../docs/adr/0003-serialise-per-space-no-worktrees.md)), so a git repo is the one hard requirement; a mapless repo is legal ([ticket 07](./07-charting-a-new-map-from-a-space.md)). Point chartr at any folder — if it is not a repo, **Register runs `git init`, announced, never silent**, so an **empty folder is no obstacle** (the ticket's "must it already be a git repository?" answered: no, it must *become* one, visibly). **Where the state lives:** the registry — the registered paths plus each space's local pin/recency — is written to **`~/.config/chartr/`**, the [user-config](../../../CONTEXT.md) layer keyed by space. This is the first chartr-owned state that is *not* a space's committed config, and it is deliberately a **rebuildable index, not a source of truth**: everything authoritative — maps, committed [workspace config](./05-mapping-roles-to-agents.md), git history — lives in the repos. **Losing the registry costs you re-adding folders, never work** — chartr keeps nothing precious out of tree, which is the whole story for loss.

**Discovery — two layers, by notice not refresh.** The registry (spaces) sits over wayfinder-maps' **liftable per-`.plan/` picker** (maps), reused unchanged under each space row. New maps surface by the **notice** pattern ([ADR 0007](../../../docs/adr/0007-map-kind-declared-not-inferred.md)) — a `git pull` or an external terminal's new map appears with no refresh button. Finished maps sort last (✓); an **unclassified map gets a one-click confirm-kind inline** — pre-filled from the heuristic but **never applied on the guess alone** (a mis-read implementation map would resolve code unreviewed, the design's quietest failure; ADR 0007 honoured). Discovery reads **wherever wayfinder writes**: the map folder is moving to `.plan/maps/<slug>/`, so discovery follows the convention and never hard-codes the old path.

**Switching — attention redirection, never a lifecycle event.** Selecting a space swaps the terminal column and the star-map; **the space you leave keeps cooking** in its own PTY. Switching never pauses, kills, or checkpoints a session — one session runs per space (ADR 0003), and the multiplexer's whole job is to let the others run unwatched. The switcher *carries* "this one is running, that one wants you" through the ambient row state, so leaving a space loses none of it.

**Ordering and scale — flat, for 5–20.** Pinned spaces first (manual order), then the rest by **recency** (most-recently-active on top); **no sections beyond that** — a handful-to-twenty list stays legible flat. An actionable signal **flags a row but never re-sorts it** — layout stays stable so muscle memory holds, the sidebar echo of ticket 09's *overlays change, positions don't*. The **filter box is always present**, which is also the scaling rule: past ~15 spaces the filter — not sub-sections — keeps the list navigable. Pin is the only manual ordering; drag-reorder within pins is a later nicety, not a requirement.

**Removal — forget, not destroy.** De-registering **touches nothing in the repository** — not `.plan/`, not the working tree, not git, not committed workspace config; it forgets the registry entry and its local pin/recency, nothing else. Re-register any time and the repo picks up exactly as it sits (commits in git, a dirty tree still dirty). With a **live session**, removal reclaims the **chartr-owned session process** but leaves every byte it wrote untouched — the clean split the whole design rests on: chartr owns the *process*, the operator owns the *tree*. Trust-at-the-gate, said deliberately as the ticket asked.

**First-run onboarding — the empty registry *is* the first-run screen.** With no spaces registered there is nothing to switch between or map, so the cockpit is a single "register your first space" affordance (and removing the last space returns to it). From there onboarding hands straight to the existing on-ramps: a map → one-click spawn on a frontier ticket ([ticket 08](./08-the-cockpit-layout.md)); no map → ad-hoc shells or the "ideate" starter ([ticket 07](./07-charting-a-new-map-from-a-space.md)). This ticket owns only the first empty screen and the wiring into those, which **clears *First-run onboarding* from the map's Not yet specified.**

**Two constraints for the spec (from the critique), binding on the real client:**

- **Keyboard-first navigation.** Switching spaces and opening the queue must have keys, not mouse-only paths — this is a cockpit the operator lives in all day. Ticket 08 owns `M`/`Esc` for the map; the registry adds the space-switch and queue-open bindings.
- **No colour-only state.** Liveness and attention must carry a non-colour channel — motion, shape, or label — consistent with ticket 09's motion-*is*-liveness grammar. The prototype's dots lean too hard on hue; the client must not.

The prototype is throwaway; the decisions above are the deliverable.
