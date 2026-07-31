---
type: task
blocked_by: []
claimed_by: sb95345b5f876
claimed_at: 2026-07-31T12:15:11Z
---

# A scratch shell, opened from the sidebar

## Question

Make the whole path work: the operator clicks **New Scratch Shell** in the
sidebar footer and gets a shell in their home directory, in a Scratch space that
was not there a moment ago and will not be there once they close it. The tracer
bullet runs from the footer control through the HTTP surface to the space
registry and back out on the control socket.

**The Scratch space is one synthetic registry entry, not a second concept.** The
registry holds it in memory at all times, alongside the registered ones, carrying
a fixed identity that cannot collide with a registered space's (those are derived
from the absolute path) and a flag every consumer can ask about. Its working
directory is the operator's home directory, resolved at load and never stored —
it follows the machine, not the file. It is never registered, never `git init`ed,
and no `[[space]]` row is written for it. Because it is an ordinary entry, the
registry's listing and lookup work on it unchanged, and the terminal manager —
which only ever used the space id as a grouping tag and the path as a working
directory — needs no change at all. In this ticket it simply appends at the end
of the sidebar order on every load; remembering where the operator put it is
ticket 03.

**The snapshot always carries it, flagged; the chrome decides whether to render
it.** It is not filtered server-side. Sending it unconditionally is what keeps
the ordering server-authoritative — the one authority the sidebar has — and puts
the visibility question where the selection it interacts with already lives. Its
derived form is deliberately thin: its open shells and nothing else. No maps
discovered, no branch or dirty state read, no skill library or config layers
resolved, no tracker-adapter offer classified, no remembered agent reported.
Nothing in this effort may cause a discovery or a git read against `$HOME`.

**The visibility rule is one predicate, extracted so it is testable.** The
Scratch space is shown when it has at least one open shell. There is no "or it is
currently selected" clause: the selection already falls back to the first
remaining space when the selected one leaves the list — the path a forgotten
space takes — so a Scratch space that empties is handled by machinery that
already exists. Everything downstream of the sidebar reads the filtered list, so
the first-run screen, the filter, the keyboard space-cycling and the reorder
write all follow without further change. The chrome has no component tests and
this does not introduce them; the predicate goes in a plain module beside the
existing pure ones (the attention derivation, the reorder resolution, the
unseen-dot rule) with a vitest file.

**Reordering must tolerate a list that omits the Scratch entry.** Today a reorder
must name every entry exactly once, and that contract is what makes the write
idempotent and unable to leave the sidebar half-moved. The chrome posts what it
can see, and while Scratch is hidden it cannot see it — so without this, the very
first hidden Scratch space breaks dragging for every registered space. The
registry now accepts a list that omits *the Scratch entry only* and splices it
back at its current slot. Every registered space must still be named exactly
once; omitting one of those stays a client bug and is still refused whole,
changing nothing.

**The footer control is the lesser of two actions.** It sits beneath **New
Space** with less visual weight, because two equally-weighted full-width buttons
read as a choice rather than a primary and an alternative. It raises no folder
chooser — a shell that made you pick a folder first is a space you wanted. Read
`docs/design-system.md` before touching it: a vendored primitive, tokens for
every colour, a Phosphor icon, no amber.

Tests lead. In `internal/server`, following the prior art in the existing space
and terminal suites — drive the API and read the model snapshot back, never the
registry's internals: the snapshot carries a flagged Scratch space from first
run; asking it for a terminal opens a shell whose working directory is the home
directory; its derived space is thin (no maps, no branch, no skills, no layers,
no tracker offer), which is what proves nothing is being discovered or classified
against `$HOME`; a reorder that omits Scratch succeeds and leaves it in its slot,
while one that omits a registered space is still refused whole and changes
nothing. In `web`, a vitest over the new pure predicate: a Scratch space with no
shells is filtered out, one with shells is kept, and registered spaces are never
filtered regardless of their shells.

## Done when

An operator with no spaces registered sees no Scratch space; clicking **New
Scratch Shell** opens a working shell in their home directory and the Scratch
space appears in the sidebar holding it; a second shell opens beside the first;
closing the last one makes the Scratch space disappear and leaves the operator on
a neighbouring space rather than a blank stage; no `git init` has run anywhere and
no `[[space]]` row has been written for it; dragging a registered space still
works while the Scratch space is hidden. `go vet ./...`, `go test ./...` and the
frontend `check`, `build` and `vitest` scripts pass, with no amber in the built
CSS.
