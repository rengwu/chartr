# sidebar order — implementation

## Destination

The [spec](../sidebar-order/spec.md) implemented end to end: the operator drags a
space to where they want it and it stays there, across restarts and across every
activation. Recency is still recorded but sorts nothing, and `pinned` is gone from
the registry, the wire model and the HTTP surface. The upgrade is invisible — the
sidebar on first launch after it is exactly the sidebar before it, and only then
does it stop rearranging itself. Done looks like one ordering authority, a
`POST /api/spaces/reorder` that takes the whole list, and no `pin` route.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were settled in the
[spec](../sidebar-order/spec.md), which is the single source of truth here. Do not
re-litigate a decision; if implementation exposes one as wrong, raise it rather
than quietly deviating. This effort has no planning map: it was charted from a
design conversation straight into the spec.

**Per-session reading order:** the spec, then this map, then your ticket.
Vocabulary comes from `CONTEXT.md` at the repo root — *space*, *session*,
*cockpit* are its words and the tickets use them. The spec names the settled seams;
prefer them to line-level file paths, which go stale.

**The sequence is expand-then-contract, and the order is the point.** `01 → 02 →
03`. The stored order is added *beside* the derived one and the migration freezes
today's sequence into it (01); the cockpit learns to write it (02); only then is
`pinned` deleted (03). Reversing any pair means either a visible reshuffle on
upgrade or a window where the sidebar has no ordering authority at all.

**The frontend rules are binding.** The sidebar is chrome, so ADR 0010 and
ADR 0012 apply in full and `docs/design-system.md` is required reading before any
UI work: tokens for every colour, a vendored primitive for every component, no
raw hex, no amber, Phosphor icons. The drop indicator uses `--primary` / `--ring`.
Nothing here goes near the terminal or star-map islands.

**Per-ticket checks:** `go vet ./...` and `go test ./...`, plus the frontend
`check`, `build` and `vitest` scripts for any ticket touching `web/` — the embed
test compiles against `dist/`. Grep the built CSS for amber before committing.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

- **01 — Spaces carry an explicit order.** `registry.Entry` gains `order` and
  `List` sorts by it alone; one load-time rule both freezes a pre-upgrade file
  into today's pinned-then-recency sequence and degrades a malformed one
  (duplicate/missing orders sort by the old rule and append behind), and every
  save densifies to 0..n-1. Registration appends at `max(order)+1`; activation no
  longer moves a row. `pinned` still written, orders nothing.
  [ticket](tickets/01-spaces-carry-an-explicit-order.md)
- **02 — Drag a space to reorder it.** `POST /api/spaces/reorder` takes the
  complete ordered list and `registry.Reorder` applies it as 0..n-1; it validates
  the whole list before touching an entry, so an omitted, unknown or repeated id
  is a `400` that cannot half-move the sidebar. The card is the drag source with a
  Phosphor grip and a `--primary`/`--ring` drop indicator drawn before the drop;
  `⌥↑`/`⌥↓` on the selected space emit the same whole-list write, behind
  `isEditingTarget`; both are inert while the filter box is non-empty. Nothing is
  applied optimistically — the arrangement returns as a snapshot.
  [ticket](tickets/02-drag-a-space-to-reorder-it.md)

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the spec — it does not open fog here. -->

## Out of scope

- **Tab reordering within a space** — terminals are an in-memory map rebuilt at
  every start and nothing rehydrates them, so a stored tab order would have no
  tabs to apply to after a restart.
- **Session rehydration on startup** — named only because tab reordering depends
  on it; a separate effort with its own questions.
- **Grouping, folding or nesting spaces** — one flat ordered list.
- **Syncing the order between machines** — the registry is per-machine and
  uncommitted.
- **Re-sorting on any signal** — a signal may flag a row, never move it.
