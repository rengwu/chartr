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
