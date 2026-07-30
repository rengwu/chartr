---
type: task
blocked_by: [01]
claimed_by: s31174e546833
claimed_at: 2026-07-30T05:38:54Z
---

# Drag a space to reorder it

## Question

Make the stored order the operator's to set. A space row is draggable, dropping it
writes the new sequence through one endpoint, and the arrangement survives a
restart — the tracer bullet from the sidebar through the API to the registry and
back into the model snapshot.

**One endpoint, the whole list.** `POST /api/spaces/reorder` takes the complete
ordered list of space ids and applies it. It is not a per-row move: a full-list
write is idempotent, matches ticket 01's dense rewrite, and cannot leave the
sidebar half-moved when the operator drags twice in quick succession. A request
whose list omits a registered space, names an unknown id, or repeats one is a
`400` that changes nothing — a partial list is a client bug, not a partial
reorder.

**The drag affordance is chrome, on primitives.** Read `docs/design-system.md`
first. The handle and the drop indicator are built from vendored primitives and
design tokens (ADR 0012) — no raw colour, no hand-rolled component, no amber.
The drop indicator uses `--primary` / `--ring`, the emphasis roles the chrome
already reserves. Icons are Phosphor. The row must show where it will land before
the drop, not only after it.

**Keyboard reordering shares the write path.** A focused space row moves with a
modifier plus arrow keys and emits the same complete-list request the pointer drag
emits, so there is exactly one write path and the keyboard is not a second
implementation. The binding sits behind the existing `isEditingTarget` guard so a
keystroke aimed at a terminal, a text field or a dialog is never stolen.

**Dragging is inert while the sidebar filter is active.** A drop position within a
filtered subset does not describe a position in the whole list, and mapping one to
the other invents a rule the spec does not make. The handles are disabled while
the filter box is non-empty.

Tests lead. In `internal/server`, following the prior art in `spaces_test.go` —
which already drives ordering by posting to the API and reading the model snapshot
back: register several spaces, post a reorder, assert the snapshot sequence;
assert the order survives a rebuild; assert each `400` case leaves the previous
order intact. In `web`, a vitest over the pure reorder helper (a list, a source
index and a destination index produce the new id sequence), following the existing
pure-helper tests in `web/src/lib`. The drag interaction itself is not
unit-tested; the helper and the server seam carry the behaviour.

Done when: an operator can drag a space to a new position and it stays there after
a restart; the same move is possible from the keyboard; the handles are inert
under an active filter; a malformed reorder request is rejected without changing
anything; `go vet ./...`, `go test ./...` and the frontend `check`, `build` and
`vitest` scripts pass, with no amber in the built CSS.

## Answer

The tracer bullet is through: a space row is draggable, the drop writes the whole
list to `POST /api/spaces/reorder`, and the arrangement is on disk before the
response returns. `registry.Reorder(ids)` takes the complete ordered list and
assigns 0..n-1; the sidebar the cockpit renders is the registry's `List` order,
unchanged from ticket 01.

**Each Done-when clause.**

- *Drag a space and it stays there after a restart.* The card is the drag source
  (`draggable`, the whole card — a far bigger target than the grip), the drop
  resolves to a new id sequence, and `reorderSpaces` posts it. Persistence is
  ticket 01's: `Reorder` ends in `saveLocked`, which densifies and writes
  atomically. `TestReorderSetsTheSidebarAndSurvivesARestart` posts a reorder,
  reads the snapshot, then starts a *second* chartr over the same config dir and
  asserts the same sequence — the only way to tell a stored order from one held
  in this process.
- *The same move from the keyboard.* `⌥↑` / `⌥↓` move the selected space through
  `moveSelected` → `applyOrder`, the exact function the drop calls. There is one
  write path and one endpoint; the keyboard is not a second implementation. The
  binding sits inside `onGlobalKey` behind `isEditingTarget`, but is read *ahead*
  of that function's `metaKey || ctrlKey || altKey` bail-out — it is the one
  binding that wants a modifier. Arrows were chosen partly because macOS leaves
  `e.key` alone under ⌥ (a letter would arrive as `∂`).
- *Handles inert under an active filter.* `reorderable = filter.trim() === ""`
  gates `draggable` and `moveSelected` both. The grip dims rather than
  disappearing and its tooltip says why; a handle that vanishes reads as a layout
  bug.
- *A malformed request is rejected without changing anything.* `Reorder`
  validates the whole list — length against the registry, every id known, no
  repeat — *before* touching an entry, so a refusal is structurally incapable of
  a partial write. `ErrBadReorder` is the sentinel the handler maps to `400`
  (anything else is a save failure and a `500`).
  `TestReorderRejectsAListThatIsNotTheWholeRegistry` drives five cases — omitting
  a space, an unknown id, a repeat, empty, and one longer than the registry —
  and re-asserts the previous sequence after each, from a deliberately
  non-default arrangement so a silent reset could not pass.
- *The checks.* `go vet ./...`, `go test ./...`, and the frontend `check`,
  `build` and `vitest` all pass. The built CSS carries chroma at hue 27.3 and
  22.2 only — the two `--destructive` reds; every other colour is hue ~107.

**The drag affordance.** Tokens and utilities only, no new component and no raw
colour: a Phosphor `DotsSixVertical` grip at `text-muted-foreground`, the dragged
card at `opacity-50`, and a drop indicator on `--primary` with a `--ring` halo
(`bg-primary ring-2 ring-ring/30`). The indicator is absolutely positioned in the
gap between cards, so showing where the row will land never nudges the list the
operator is aiming at — and it is drawn on `dragover`, before the drop, per
story 8.

**Tests.** Two in `internal/server` following `spaces_test.go`'s prior art (post
to the API, read the sequence off the model snapshot — never the sort function's
internals), and `web/src/lib/reorder.test.ts` over the pure helper, following the
existing pure-helper tests. The helper is two functions rather than one: `reorder`
(list, source, destination → new sequence) and `dropIndex`, which converts "released
on the *edge* of row N" into that destination. Splitting them puts the one piece of
real arithmetic under test — a downward drag inserts at N+1 but *lands* at N,
the row itself having left the list — instead of leaving it inline in a component
that nothing drives.

**Three things worth flagging.**

1. *No browser smoke of the gesture itself.* The Chrome extension is not
   connected in this session, so the pointer drag has never been performed by a
   human or a machine. What was verified against a real running binary (isolated
   via `XDG_CONFIG_HOME`, so no operator registry was touched) is the endpoint
   round trip: a reorder lands in `spaces.toml`, and a partial or unknown-id list
   returns `400` with the file unchanged. The helper and the server seam carry
   the behaviour, as the ticket intended — but the drop-indicator geometry and
   the ⌥-arrow binding have been reasoned about, not seen.
2. *Drag state is held by id, not index.* A snapshot can land mid-drag (a space
   registered elsewhere, a session opening), and an index captured on grab would
   then name a different row by the time it is dropped. Both ends re-resolve at
   drop time and bail if either row has left.
3. *The keyboard moves the **selected** space, not the focused one.* The ticket
   says "a focused space row"; selection is what the sidebar highlights, what
   `[`/`]` already move, and what a global binding can address without the row
   having keyboard focus. With nothing clicked, `selected` falls back to the
   first space, which is also the row the sidebar highlights — so the two read
   the same. Flagging it as a wording gap, not a deviation worth reopening.

**Deliberately left out.** No optimistic reordering — the new arrangement arrives
as a fresh snapshot over the control socket like every other action's result, and
a local list that could disagree with the registry is exactly the second
authority this effort exists to remove. Nothing about `pinned`: `Entry.Pinned`,
`SetPin`, the pin route and `Space.pinned` are ticket 03's. The two stale
"pinned first, then by recency" comments ticket 01 left in `App.svelte` are
corrected here, as it asked.
