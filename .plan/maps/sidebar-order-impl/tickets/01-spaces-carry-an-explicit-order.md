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
