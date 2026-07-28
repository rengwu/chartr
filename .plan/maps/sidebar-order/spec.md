# Sidebar order

## Problem Statement

The operator does not choose the order of the sidebar; the sidebar chooses it for
them. Spaces render pinned-first, then by recency (`registry.List`), so the row
under the cursor moves whenever a space becomes active. An operator who thinks of
their spaces in a fixed arrangement — the one they are shipping, the one they
maintain, the two they only visit — cannot express it. The one lever that exists,
`pinned`, is coarse (a space is top or it is not), and it has no interface at all:
it is reachable only by `POST /api/spaces/{id}/pin`, and nothing in the cockpit
renders a control for it.

## Solution

The operator drags a space where they want it, and it stays there. A stored
order replaces the derived one outright: recency is still recorded — it is a
useful fact, and `last_agent` sits beside it — but it no longer sorts anything.
Nothing re-arranges the sidebar behind the operator's back.

`pinned` is deleted rather than kept beside the new order. Dragging a space to
the top is everything pinning did and more, so keeping both would leave two
authorities racing over one list. The upgrade is invisible: on first read the
order the operator sees today — pinned first, then recency — is frozen into the
stored order, so the sidebar after the upgrade is byte-for-byte the sidebar
before it, and only then does it stop moving.

A newly registered space appends at the end. The stored order is treated as the
operator's, and registration does not get to disturb it.

## User Stories

1. As an operator, I want to drag a space to a new position in the sidebar, so
   that my spaces sit in the arrangement I think of them in.
2. As an operator, I want that arrangement to survive a restart, so that I set it
   once rather than every session.
3. As an operator, I want a space I have just been working in to stay where I put
   it, so that the sidebar is a stable map rather than a recency feed.
4. As an operator, I want my sidebar to look exactly the same immediately after
   upgrading, so that a new feature does not silently rearrange my workspace.
5. As an operator, I want a newly registered space to leave my existing order
   untouched, so that adding a repo is not a rearrangement.
6. As an operator, I want to know where a newly registered space went, so that I
   am not hunting for something I just added.
7. As an operator, I want the drag affordance to be visible on a row, so that I
   can tell the sidebar is rearrangeable without discovering it by accident.
8. As an operator, I want a drag in progress to show me where the row will land,
   so that I am not guessing at the drop position.
9. As an operator, I want to reorder spaces from the keyboard, so that the
   feature is not mouse-only.
10. As an operator, I want the filtered sidebar to leave my order alone, so that
    searching does not become a way to accidentally rearrange things.
11. As an operator, I want `pinned` gone from the config file rather than left
    behind as a dead key, so that the file describes what the app actually does.
12. As an operator, I want a hand-edited or corrupt order in `spaces.toml` to
    degrade into a sensible sidebar rather than an empty or duplicated one, so
    that a typo never costs me my workspace list.
13. As an operator running two chartr instances against the same registry, I want
    the last write to win cleanly rather than corrupt the file, so that the
    ordering never leaves the registry unreadable.

## Implementation Decisions

**The registry owns the order.** `registry.Entry` gains an integer `order`, and
`registry.List` sorts by it alone — the `Pinned` and `LastActive` comparisons come
out. `LastActive` continues to be recorded on activation, because the field is
read elsewhere and is worth having; it simply stops being an ordering input. The
sidebar's own comment (`internal/model`) that spaces arrive pre-ordered and are
rendered in slice order stays true, and the client still never re-sorts.

**The migration is a freeze, not a reset.** On the first load of a registry file
that carries no `order`, the entries are sorted by the *old* rule — pinned first,
then recency, then path as the stable tiebreak — and that sequence is written back
as `order` 0..n-1. The operator's sidebar is therefore unchanged at the moment of
upgrade, and only stops re-sorting afterwards. A registry that already carries
`order` is never re-derived.

**Order is dense and rewritten wholesale.** A reorder rewrites every entry's
`order` to its new index rather than attempting sparse insertion. The registry
already saves atomically (temp file plus rename) and the list is small; a dense
rewrite makes duplicate and missing indices unrepresentable after any successful
write, which is worth more here than avoiding a full write.

**Malformed order degrades, never breaks.** Entries carrying a duplicate or
missing `order` are ordered among themselves by the old rule and appended after
the well-formed ones, then the whole list is re-densified on the next save. This
follows the `terminal.toml` contract — a bad value is dropped and a default
stands, so the app never breaks on a hand-edit.

**A new space appends.** Registration assigns `max(order) + 1`. The registration
flow already selects the space it just registered, which is what keeps story 6
honest without a placement rule that disturbs the existing order.

**Reordering is one endpoint.** `POST /api/spaces/reorder` takes the complete
ordered list of space ids and applies it; it is not a per-row move. A full-list
write matches the dense rewrite above, is idempotent, and cannot leave the list
half-moved if the operator drags twice quickly. Ids not present in the request,
or unknown to the registry, are a `400` — a partial list is a client bug, not a
partial reorder.

**`pinned` is removed end to end.** `Entry.Pinned`, `Registry.SetPin`, the
`POST /api/spaces/{id}/pin` route and handler, and `Space.pinned` on the wire
model all go. The TOML key stops being written on the next save and is ignored on
read, so an old file loads without complaint. `CONTEXT.md` has no entry for pin;
nothing in the glossary needs revising.

**The drag lives in the chrome, on primitives.** The sidebar rows are chrome
(ADR 0010), so the drag affordance is built on the vendored primitives and design
tokens per ADR 0012 and `docs/design-system.md` — no raw colour, no hand-rolled
component, no amber. The drop indicator uses `--primary` / `--ring`, the emphasis
roles the chrome already reserves for this. Icons are Phosphor.

**Keyboard reordering shares the endpoint.** A focused row moves with a modifier
plus arrow keys, emitting the same complete-list request the pointer drag emits,
so there is one write path and the keyboard is not a second implementation.
`isEditingTarget` already guards global bindings from stealing keys aimed at a
terminal or a text field, and the binding sits behind it.

**Dragging is disabled while a filter is active.** The filtered sidebar is a view
over a subset, so a drop position within it does not describe a position in the
whole list. Rather than invent a mapping, the handles are inert while the filter
box is non-empty.

## Testing Decisions

A good test here observes the order the operator would see, through the seam the
cockpit itself reads — not the sort function's internals. The sidebar's order is
already tested this way: `internal/server/spaces_test.go` drives pin ordering by
posting to the HTTP API and reading the resulting model snapshot back, and that
is the prior art every ordering test here follows.

- **`internal/server`** is the primary seam. Register spaces, post a reorder, read
  the model snapshot, assert the sequence. Assert that activating a space does not
  change it, that a newly registered space lands last, that a reorder naming an
  unknown or missing id is rejected with `400` and leaves the order untouched, and
  that the `pin` route is gone (a post to it returns `404`).
- **`internal/registry`** carries the migration test, because the freeze is a
  property of loading a file and has no HTTP surface: a file with mixed
  `pinned`/`last_active` and no `order` loads to exactly the sequence the old
  `List` produced, and the sequence is stable across a save/reload cycle. Add the
  degradation cases — duplicate `order`, missing `order`, an `order` on some
  entries only — asserting a total order with no entry lost.
- **`web`** gets vitest coverage of the pure reorder helper (given a list, a
  source index and a destination index, produce the new id sequence), following
  the existing pure-helper tests in `web/src/lib`. The drag interaction itself is
  not unit-tested; the helper and the server seam carry the behaviour.
- The full suite bar is unchanged: `go vet ./...`, `go test ./...`, and the
  frontend `check`, `build` and `vitest` scripts, with no amber in the built CSS.

## Out of Scope

- **Tab reordering within a space.** Terminals are an in-memory map rebuilt on
  every start (`internal/terminal/manager.go`) and nothing rehydrates them, so a
  stored tab order would have no tabs to apply to after a restart. Ordering that
  survives only until the process exits is not what was wanted. This returns if
  and when held sessions are restored on startup, which is a separate effort with
  its own questions (scrollback, claims, what a restored dead session means).
- **Session rehydration on startup.** Named here only because the item above
  depends on it. Not begun here.
- **Grouping, folding, or nesting spaces.** One flat ordered list.
- **A per-space colour, icon, or label.** Ordering only.
- **Syncing the order between machines.** The registry is per-machine and
  uncommitted, and stays that way.
- **Re-sorting on any signal.** An actionable signal may flag a row; it never
  moves one. This is the existing rule (`attention.ts`, story 8) and the stored
  order makes it absolute.

## Further Notes

The knowingly-accepted cost of appending new spaces at the end is that a fresh
registration can land below the fold in a long sidebar. Registration selects the
space it creates, so the operator is looking at it either way; the trade was made
deliberately in favour of never disturbing a set order.

Deleting `pinned` is cheap only because it never grew an interface. Had a pin
control shipped, this would be a deprecation with a migration story for the
operator as well as the file. It is worth removing now, while the cost is three
symbols and a route.
