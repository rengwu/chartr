---
type: task
blocked_by: [06, 10, 11]
claimed_by: sd88d7d931ae8
claimed_at: 2026-07-21T05:50:35Z
---

# Session states on the star-map

## Question

The moons grammar, inside the renderer's idiom (never Svelte components — ADR 0010). A session is an amber moon orbiting its star; liveness is its motion — working orbits with a trail, quiet crawls dimmed and blinking, dead freezes grey and greys its orbital apparatus. Proposed docks the moon at the star's rim; agent review adds the smaller violet counter-orbiter — the one new hue. Human review deliberately breaks the orbital grammar: the docked moon whitens, the star warms toward gold, ping rings emanate, and an offscreen star keeps calling through the gold edge chevron. A live change is one flare plus one fading ticker line; approval's ignition reuses the base language. Every state carries a non-color channel — motion or shape — so nothing is color-only.

Done when: island-seam tests script the full lifecycle — implementing, quiet, dead, proposed, agent review, human review, approval igniting a dependent — from pushed models and assert zero star movement throughout; each state renders per the grammar on the fixture maps; the chevron appears exactly when the beckoning star is offscreen; the no-color-only property holds for every session state.

## Proposed Answer

The moons grammar lands as a strictly-additive overlay inside the renderer — no
new Svelte component, no touch to layout, and nothing new stored anywhere. The
session axis is derived per push from the snapshot the cockpit already carries.

**What I built:**

- **`web/src/lib/starmap/session.ts` — the grammar, written down.** `GRAMMAR` is
  a record over the six session states (`implementing`, `quiet`, `dead`,
  `proposed`, `agent-review`, `human-review`) naming, for each, its `hue` *and*
  its non-colour channels: the moon's `motion` (orbit / crawl / still), its
  `moon` carriage (orbiting / docked / frozen), and its `marks` (trail, blink,
  halo, counter-orbit, ping-rings). The renderer **draws from this record** rather
  than re-encoding the grammar in its own branches, so the no-colour-only property
  the test asserts is a property of what actually gets painted.
  `sessionStates(map, terminals)` is the pure derivation: session tabs are matched
  to their map and ticket (ticket 09's `Session`), liveness is the status the
  server already decided (ticket 10 — so an idling HITL grilling deliberately
  shows *nothing*, the non-signal the prototype's fixture 2 makes), and the
  pipeline reads off the ticket's `proposed` status plus the review session on it.
- **`starmap.ts` — the overlay and the two screen-space affordances.** `setModel`
  grew a second argument (the overlay keyed by ticket number) rather than a second
  seam call, so one push is one visual beat. `#drawSession` is the variant-A
  port: the orbit ring (greyed and dashed when dead), the moon at its docked /
  orbiting / frozen position, the trail, the blink, the grey halo, the violet
  counter-orbiter, the gold wash and pings. The vanilla claimed rings stand down
  when an overlay speaks for the claim, so nothing competes with the orbit.
  `#drawChevrons` pins a gold chevron and a `#03 wants you` caption at the edge a
  beckoning star left by; `#drawTicker` writes the one fading line naming what
  just changed. A changed status *or* session state flares its star once.
- **The seam widened by exactly three read-only accessors** — `overlays()`,
  `ticker()`, `beckoning()` — so the grammar, the ticker and the chevron are
  assertable headless, matching how ticket 06 pinned positions and selection.
- **Wiring:** `SpacePane → MapCard → StarMap.svelte` thread the space's
  `terminals` to the island wrapper, which computes the overlay at the seam. The
  chrome gained no state and no colour.
- **`docs/design-system.md`:** the star-map data-viz exemption now names
  `SESSION_HUE` and records that the set is *closed* — a new state earns a new
  motion, not a new hue.

**How each Done-when clause is met** (`web/src/lib/starmap/starmap.test.ts`,
`session.test.ts`; 45 vitest tests pass, 12 of them new):

- *scripts the full lifecycle from pushed models, zero movement throughout* —
  `SESSION_LIFECYCLE` is seven pushed beats over the five-ticket fixture:
  implementing → quiet → dead → proposed → agent review → human review →
  approval igniting #04 onto the frontier. Every beat's `positions()` is compared
  against the starting layout and is identical.
- *each state renders per the grammar* — the same walk asserts `overlays()`
  equals the pushed state per beat and that each names a motion and a non-colour
  signature; approval leaves `overlays()` empty, the base language taking over.
  A second suite runs one real frame against a recording stub 2D context, so
  every state's draw path is executed rather than only described.
- *the chevron appears exactly when the beckoning star is offscreen* — a
  human-review star onscreen raises nothing; panned off the right edge (through
  the island's own mouse path, not a test seam) it raises `[3]`; panned back it
  stands down; and an offscreen *working* session never beckons. The painted
  caption is asserted at frame level too.
- *no-colour-only for every session state* — `session.test.ts` asserts every
  state names a motion and a moon carriage, and that the six non-colour
  signatures are pairwise distinct: the overlay survives greyscale. It also pins
  that the axis spends exactly one new hue.

**Tested:** `vitest` (45), `svelte-check` (0 errors / 0 warnings), the Vite
build, `go vet ./...`, `go test ./...` — all pass; no amber in the built CSS
(the overlay's hues live in the island's exempt TS palette, never the chrome).

**Deliberately left / flagged for review:**

- **How `human-review` is derived.** The model carries no "awaiting a human"
  field — ticket 12 builds the hub. I derived it from the snapshot: a `proposed`
  ticket whose *review* session has exited (the reviewer ran, the verdict is
  written, the brief awaits you), versus `agent-review` while that session is
  live, versus plain `proposed` when no reviewer has ever seated. This is honest
  against today's model and needs no new state, but if ticket 12 introduces an
  explicit signal (a brief on disk, an acknowledgement tick), `stateOf` in
  `session.ts` is the one function to re-point. Flagged as the judgement call in
  this ticket.
- **A claim whose session is gone shows no moon.** The overlay speaks for
  *sessions*; the claim itself is already the base amber star. So an operator
  who closed a live session's tab sees the claimed star, not a dead moon —
  consistent with ticket 10's kill-vs-death distinction.
- **The ticker and chevron are painted by the island, not the chrome.** The
  ticket says "inside the renderer's idiom (never Svelte components)", so both
  are canvas text. That costs them selectability and a11y exposure; if the human
  review hub wants the ticker as chrome, it should move there wholesale rather
  than being mirrored.
- **The chevron treats a star hidden behind the detail pane as offscreen** (it
  uses the pane-aware free rect). A call to action errs toward calling.
- **Feel is unverified by eye**, as with tickets 06 and 07: the constants,
  speeds, radii and alphas are ported verbatim from the canonical prototype
  variant A, and the frame test proves the paths run, but a headless session
  cannot judge the motion. Worth one look in a browser against a live session.
- Not built here (tickets 12 and 14): the review hub's buttons, the "Needs you"
  queue, and the action-station badge. The overlay only *renders* the states
  those tickets act on.
