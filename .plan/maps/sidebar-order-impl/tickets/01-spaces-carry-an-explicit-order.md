---
type: task
blocked_by: []
claimed_by: s8f542e9753dd
claimed_at: 2026-07-29T19:06:09Z
---

# Spaces carry an explicit order, frozen from today's

## Question

Give the registry a stored order and make it the only thing `List` sorts by,
without any operator-visible change. This is the expand half: after this ticket
the sidebar looks and behaves exactly as it does today, but its order is a fact on
disk rather than a derivation. Ticket 02 gives the operator a way to change it;
ticket 03 removes what it replaces.

**The field.** `registry.Entry` gains an integer order. `registry.List` sorts by
it alone — the `Pinned` and `LastActive` comparisons come out of the comparator.
`LastActive` is still recorded on activation and `Pinned` is still read and
written for now; neither orders anything after this ticket. The comment on `List`
that describes pinned-then-recency is part of the change, not decoration: it
becomes the description of a stored order that nothing but an explicit reorder
moves.

**The migration is a freeze.** On loading a registry file where no entry carries
an order, sort the entries by the *old* rule — pinned first, then most-recently-
active, then path as the stable tiebreak — and assign 0..n-1 in that sequence,
persisting it on the next save. This is what makes the upgrade invisible, and it
is the load-bearing behaviour of this ticket. A file that already carries orders
is never re-derived; a partially-ordered file is handled by the degradation rule
below rather than by re-freezing.

**Order is dense, and rewritten wholesale.** Any write that changes the sequence
rewrites every entry's order to its new index. The registry already saves
atomically via temp-file-plus-rename, so a dense rewrite costs one save and makes
duplicate or missing indices unrepresentable after any successful write.

**Malformed input degrades, never breaks.** Entries with a duplicate or missing
order are sorted among themselves by the old rule and appended after the
well-formed ones, then the whole list is re-densified on the next save. This is
the `terminal.toml` contract applied to the registry: a hand-edit or a truncated
file costs the operator their arrangement, never their list of spaces.

**Registration appends.** A newly registered space takes `max(order) + 1`. No
other placement rule; the stored order belongs to the operator and registration
does not disturb it.

Tests lead, and the migration test is the one that matters. In
`internal/registry`: a file carrying mixed `pinned` and `last_active` values and
no order loads to exactly the sequence the old `List` produced — assert against
the old rule's output, not a hand-copied list — and that sequence is stable across
a save-and-reload cycle. Add the degradation cases: duplicate orders, orders on
some entries only, orders with gaps, each asserting a total order that loses no
entry. In `internal/server`, assert through the HTTP seam that activating a space
no longer moves it, and that a newly registered space appears last in the model
snapshot.

Done when: `registry.List` sorts by the stored order alone; a pre-upgrade
registry file produces a byte-identical sidebar sequence on first load and a
persisted order afterwards; activation no longer re-sorts; a new space lands last;
a malformed order file still yields every space exactly once; `go vet ./...` and
`go test ./...` pass. No frontend change in this ticket.

## Answer

The sidebar's order is now a fact on disk. `registry.Entry` carries an integer
`Order` (TOML key `order`), `registry.List` sorts by it alone, and nothing an
operator can do today changes what they see — the expand half is in place with no
visible change.

**Each Done-when clause.**

- *`List` sorts by the stored order alone.* The comparator is `sortByOrder`:
  `Order`, then `Path` purely to make the sort total (the entries come out of a
  map with no iteration order of its own, and orders are unique after any load or
  save). `Pinned` and `LastActive` are out of it. The doc comment is rewritten as
  the ticket asked: pin and recency are still recorded and sort nothing.
- *A pre-upgrade file freezes, then persists.* `Load` resolves the sequence
  through one function, `ordered(entries, carried)`. Entries with a unique order
  keep their sequence; entries with a duplicate or missing one are sorted among
  themselves by the old rule — pinned, then recency, then path — and appended
  behind. A file carrying no order at all is just the case where every entry is
  malformed, so the freeze and the degradation rule are the same code path rather
  than two rules that could disagree. The result is densified to 0..n-1 in memory;
  `Load` itself does not write, per the ticket ("persisting it on the next save"),
  and `saveLocked` densifies before every write so the freeze lands with the first
  mutation.
- *Activation no longer re-sorts.* `Register` on an already-registered path — the
  only activation seam there is; nothing else touches `LastActive` — still
  refreshes recency and now leaves the row where it sits.
- *A new space lands last.* `Register` gives a new entry `nextOrderLocked()` =
  `max(order)+1`.
- *A malformed file yields every space exactly once.* The partition in `ordered`
  is total: every entry lands in exactly one of the two buckets and both are
  concatenated, so nothing is dropped or duplicated whatever the file says.
- `go vet ./...` and `go test ./...` pass (registry also under `-race`). No
  frontend change.

**Tests.** New `internal/registry/registry_test.go` (the package had none): the
migration test asserts the frozen sequence against an independent restatement of
the old comparator — `oldRule`, lifted from the pre-change `List` — not a
hand-copied list, plus a guard that the result is *not* the file's row order so
the assertion cannot pass vacuously; then it forces a save and reads the orders
back out of `spaces.toml` itself, which is the only way to tell a persisted freeze
from one re-derived on the next load. Degradation cases: duplicate orders, orders
on some entries only, gaps (which densify on save), and `order = 0` on one entry
while others carry none — the last is why presence is detected rather than
inferred from the zero value. In `internal/server`, `TestNewSpaceLandsLast` and
`TestActivatingASpaceDoesNotReorder` drive the HTTP seam and read the sequence off
the model snapshot, following the prior art the pin test set.

**Two things worth flagging.**

1. *`TestPinOrdersAhead` is gone*, replaced by `TestPinNoLongerReorders`. It
   asserted exactly the behaviour this ticket removes, so it could not survive; the
   replacement pins the vestigial state and dies with the route in ticket 03.
2. *Detecting a missing `order`.* A missing key and `order = 0` decode into the
   same `int`, so `Load` unmarshals the document a second time as
   `[]map[string]any` purely to learn which rows carried the key. It is the plain
   option — `*int` on `Entry` would push "maybe absent" onto every consumer for a
   fact that stops being true the moment `Load` returns, and `toml.MetaData`
   reports array-of-table keys without their index, so it cannot answer
   *which* entry. Both decodes see rows in file order, and the second cannot fail
   because the first already parsed.

**Deliberately left out.** No `Reorder` method and no `POST /api/spaces/reorder` —
that is ticket 02, and there is no operator-facing way to change the order yet. No
frontend change at all, which leaves two now-stale comments in `web/src/App.svelte`
(lines ~103 and ~352) describing "pinned first, then by recency"; ticket 02 rewrites
that code and should correct them. `Entry.Pinned`, `SetPin`, the pin route and
`Space.pinned` all stay for ticket 03. The file's row order in `spaces.toml` is
still sorted by path for stable diffs — the order is explicit in each row now, so
row order carries no meaning.
