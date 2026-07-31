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
