---
type: task
blocked_by: [06]
---

# The ticket pane

## Question

From looking at a star to reading in one click: selection opens the full ticket — question, Done-when, blockers with their answers inline, session history — in the responsive detail pane, docked right, re-docking to the bottom (capped at half the map panel) when the panel is narrow, while the camera measures the pane's actual size and eases the star into the space it leaves free (planning ticket 08 as amended). The map's own material — destination, notes, decisions, fog — opens the same way from the map's title. Deep links name a star. The pane's actions are status-appropriate; those that need later tickets (spawn, review) may stub to disabled affordances here.

Done when: clicking a star opens the pane with the full ticket content assembled from the derived model; the pane re-docks responsively and the camera seats the star in the free space in both dockings; the map-material pane opens from the title; a deep link selects and seats its star.

## Proposed Answer

Selection opens a responsive detail pane over the star-map island, assembled entirely from the derived model — no second fetch. Layout:

- **`web/src/lib/DetailPane.svelte`** — the pane content. For a ticket: its inlined body (Question and Done-when), then its blockers with their answers pulled *inline* — `sectionOf` reads each blocker's own `Answer` / `Proposed Answer` / `Ruled out` section out of the model's inlined body, degrading a missing or unanswered blocker to a labelled note — then a Session-history section. For the map's title: the map material — Destination lifted to the top, the rest of the body rendered with the duplicate Destination heading stripped. It renders one thing; whether it docks right or bottom is the parent's decision, so this file is only content and its status eyebrow.
- **`web/src/lib/starmap/dock.ts` (+ `dock.test.ts`)** — the dock decision as a pure function. Three strategies (`width`, `aspect`, `hybrid`) are offered so the feel can be compared directly, each optionally wrapped in a dead-band so dragging the card through the boundary doesn't flip-flop the side. The card runs `hybrid` with hysteresis: bottom when the panel is either too narrow (< 600px) or portrait (h > 1.1·w), right otherwise. Ten unit tests pin the three strategies, the dead-band, the hysteresis-off equivalence, and degenerate sizes.
- **`web/src/lib/MapCard.svelte`** — hosts the island and, over it, the pane in a holder that docks right (`min(400px, 58%)`) or re-docks to the bottom (`max-height: 50%` — the half-panel cap). A `ResizeObserver` measures the pane's actual footprint and feeds `insets` to the island, so the camera eases the selected star into the space the pane leaves free — a right pane insets the right edge, a bottom pane the bottom edge — in both dockings. A star selection and the map material are one pane showing one thing: opening either closes the other.
- **`web/src/lib/SpacePane.svelte`** — owns selection and the deep link. `#s=<space>&m=<map>&t=<ticket>` (or `&mat=1` for the map material) is parsed once at boot so the linked star opens and seats on load; a `hashchange` listener re-applies manual edits and back/forward; the live selection is reflected back into the URL via `replaceState` (which never fires the listener, so the two don't loop). A selection belongs to one map and is dropped when the focused map changes.

Against Done-when: **(1)** clicking a star opens the pane with the full ticket assembled from the derived model — body, blockers-with-answers inline, session history; **(2)** the pane re-docks responsively (`dock.ts`, under test) and the measured `insets` seat the star in the free space in both dockings; **(3)** the map material opens from the title (the map name, or the `notes` button when the space has several maps); **(4)** a deep link selects and seats its star, and edits round-trip through the URL. `svelte-check` (0 errors, 0 warnings), `vitest` (26 passing), the Vite build, `go vet`, and `go build` all pass.

Three decisions for review to weigh:

- **Three dock strategies ship; `hybrid` is wired.** `dock.ts` carries `width`/`aspect`/`hybrid` so the re-dock feel can be compared directly rather than guessed; the card uses `hybrid`. Planning ticket 08's amendment named "< ~780px" as the narrow trigger; this realises it as a 600px width gate *plus* a portrait guard, with a 32px / 0.12-ratio dead-band so a drag through the boundary holds the current side. The exact width figure differs from the amendment's approximation and is the one number worth an eye.
- **Session history is a visible stub.** The pane reserves the section and says "No sessions on this ticket yet" — the derived model carries no session history until the session tickets (09+) land. The pane's shape is complete; that data arrives later with no layout change. The same holds for the status-appropriate actions (spawn, review): their tickets have not landed, so the pane shows none rather than disabled affordances.
- **Blocker answers are read from the inlined bodies, never re-fetched.** Because the model inlines each ticket's body, a blocker's answer shows inline with no extra round-trip; the pane assembles the whole reading view from one snapshot.

Scope notes for review: the star-easing camera *feel* is confirmed by design/eye consistent with ticket 06 and was **not** re-run in the live app this session (it needs the control-socket backend) — the headless island seam and the dock decision are the parts under automated test. The space-stage chrome that shipped in the same code commit (the space-header hierarchy, the standardized bars, the map show/hide toggle, and the docked-map min-width floor) is map⇄terminal-layout polish recorded on ticket 08's 2026-07-20 amendment, not this ticket's scope.

Review payload should carry this Done-when and the spec by assembly (spec, Prompts and payload).
