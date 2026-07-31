---
type: task
blocked_by: [01]
claimed_by: s3817e5ae187c
claimed_at: 2026-07-31T13:37:06Z
---

# Scratch keeps its place in the sidebar

## Question

Make the Scratch space's position in the sidebar the operator's, the way every
registered space's already is. After ticket 01 it can be dragged and moved with
the keyboard, but it appends at the end on every load — so the arrangement is
lost on restart, and it is lost again every time the space hides and comes back.
Give it a remembered slot that it holds whether or not it is on show.

**It persists as a single order value, never as a space row.** The registry
file's `[[space]]` rows go on meaning exactly what they have always meant —
folders the operator registered — which is what keeps "delete the registry and
re-add your folders" a complete recovery. The Scratch space needs no recovering;
it is rebuilt from nothing on every run. Its sidebar slot is the only thing about
it worth persisting, so it persists as a scalar beside the rows.

**It holds a slot in the order whether it is visible or not.** Visibility is a
question the chrome answers at render time; the order is a fact the registry
keeps either way. This is what makes it come back where the operator left it
rather than at the bottom of the list, and it is the reason the slot cannot be
derived from the visible list.

**The load and save round trip runs through the existing densification.** The
registry densifies the whole arrangement — registered spaces and the Scratch
entry together — on every save, and registered rows are written with the orders
that densification gave them. That means the file may carry a gap where Scratch
sits. The existing load path already compacts a gapped-but-unique arrangement
without disturbing its sequence, so the shape is: compact the registered rows,
splice Scratch back in at its recorded index, densify again. Reuse that path
rather than adding a second ordering rule beside it.

**A file written before this feature must load unchanged.** It carries no
recorded index, so the Scratch space appends at the end and the operator's
existing arrangement is byte-for-byte what it was — the same invisible-upgrade
rule the stored order itself was introduced under. An operator who upgrades and
never opens a scratch shell must not be able to tell this shipped.

Tests lead, and this ticket's seam is the registry's own load and save rather
than the API — the on-disk order format is a contract the snapshot cannot
observe, and the registry's existing suite is the prior art. Cover: the Scratch
slot round-trips through a save and a reload; a file whose registered rows carry
a gap where Scratch sat loads back into the same sequence; a pre-upgrade file
with no recorded index loads with the registered arrangement untouched and
Scratch appended last. Then one check at the API seam, following ticket 01's
tests, that the arrangement survives a restart: move the Scratch space, start a
second chartr over the same config root, and assert the same sequence — the only
way to tell a stored order from one held in this process.

## Done when

An operator drags the Scratch space to a position, closes every scratch shell so
it hides, opens a new one, and finds it back in the position they put it; that
position survives a restart; an operator upgrading from a registry file written
before this feature sees their existing arrangement unchanged with the Scratch
space appended last. `go vet ./...` and `go test ./...` pass.

## Answer

The Scratch space's sidebar slot now persists, as one scalar beside the rows. The
whole change is in `internal/registry`; nothing in `web/` moved, because the
chrome already posts the list it can see and the server already splices Scratch
back into it (ticket 01) — the only thing missing was that the slot did not
outlive the process.

**One scalar, never a row.** `file` gained `ScratchOrder *int`
(`scratch_order`), declared before `Spaces` because a scalar cannot follow an
array of tables in TOML, and a pointer for the same reason the rows' own `order`
is read through one: an absent key has to be told apart from a recorded slot of
0. `saveLocked` already skipped the Scratch entry when building the rows; it now
records that entry's order on the way past. `[[space]]` rows still mean exactly
what they meant, so deleting the registry and re-adding folders is still a
complete recovery.

**Load compacts, splices, densifies.** `addScratch` takes the recorded slot and
seats Scratch by it rather than appending. The registered rows arrive already
compacted by the existing `ordered`, so the file's gap is closed before the
splice and reopened by it — a save that wrote rows 0, 2, 3 beside
`scratch_order = 1` loads back to precisely that sequence. No second ordering
rule was added: the splice is expressed on the sorted sequence and the whole list
takes its index as its order, which is the same densification the rest of the
package does.

**Done-when, clause by clause.** *Dragged, hidden, reopened, still in place* —
the slot is registry state and nothing in the terminal lifecycle touches it, so
a hide/show cycle cannot move it; a reorder posted while Scratch is hidden splices
it back at that slot (ticket 01) and the save records it. *Survives a restart* —
the slot is on disk and read as the splice index on the next load. *The upgrade is
invisible* — a file with no `scratch_order` appends Scratch at the end and leaves
the registered arrangement exactly as it was, whether the file already stored an
order or predates that too. `go vet ./...` and `go test ./...` are green.

**Tested at the two seams the ticket names.** In `registry_test.go`, at the file
seam: the slot round-trips through a save and a reload (asserting the file
carries `scratch_order`, still writes three rows for three registered spaces, and
leaves the registered rows the gapped orders 0, 2, 3); a hand-written gapped file
loads back into the same sequence; `scratch_order = 0` is a slot and not an
absence; a file with no recorded slot leaves the arrangement untouched — covered
for both a file that stores an order and the pre-order `preUpgrade` fixture; and
a hand-edited out-of-range slot degrades to appending without costing a space.
In `scratch_test.go`, at the API seam: `TestScratchSlotSurvivesARestart` moves
Scratch between two registered spaces, starts a second chartr over the same config
root, and asserts the snapshot's sequence — the only thing that can tell a stored
order from one this process happens to remember. All five new registry tests and
the restart test fail against the pre-change registry, which is what makes them
tests rather than echoes.

Verified beyond the suite against a real built binary under isolated `HOME`,
`XDG_CONFIG_HOME` and data roots: reordering to *(B, Scratch, A)* wrote
`scratch_order = 1` above two rows carrying orders 0 and 2 and no row for
Scratch; a restart over that config root pushed the same three ids in the same
order; opening a scratch shell, closing the last one, posting a reorder that
omits the now-hidden Scratch, and opening another left it still in slot 1; and
stripping `scratch_order` from the file — a pre-feature registry — started with
the two registered spaces in their arrangement and Scratch appended last.

Deliberately unchanged: the chrome. No `web/` file needed editing, so the
frontend `check`/`build`/vitest and the amber grep were not the gate here; the
Go embed test still compiles against the existing `dist/`. Ticket 02's refusals
are untouched — nothing here goes near a repo-scoped action.
