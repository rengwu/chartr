---
type: task
blocked_by: [01]
---

# Scratch offers only what it can do

## Question

Close the gap between what the Scratch space *is* and what the cockpit still
offers to do with it. After ticket 01 it holds shells, but its card still carries
a skills launcher and a new-shell control, its stage still offers a star-map over
a folder with no repository, and every repo-scoped endpoint would act on the
operator's home directory if asked. Make the refusals real at the server and the
controls absent from the chrome.

**Refusal is a server rule, not merely an absent button.** Spawning, launching an
on-ramp skill, ideating, previewing a payload, the death-halt verbs, releasing a
ticket, the tracker-adapter offer and its dismissal, opening a per-space config
layer, setting the remembered agent, and deregistering all resolve a space before
they act. Each refuses the Scratch identity with a bad request rather than acting
on `$HOME`. Opening, closing and acknowledging a terminal accept it — that is the
whole of what it is for. The chrome will not offer the refused ones, but a stale
or hand-written client must not be able to spawn an agent into the home
directory, and deregistering it must be refused rather than silently ignored: a
no-op on an explicit destructive request is worse than a clear answer.

These handlers share one shape — resolve the space named in the path, or answer
not-found. That is the prefactor: give them a single resolver that refuses the
Scratch identity as well, and swap it in, rather than repeating the check at each
call site. The three terminal handlers keep the plain lookup.

**The card loses every control it cannot honour.** No remove control, because it
cannot be removed. No branch chip, because telling the operator something about a
folder that is not a repository is telling them something false. No skills
launcher and no new-shell control, because neither can act. That empties its
action row entirely, so the row is not rendered rather than rendered blank. The
drag handle and the shell rows stay: it is reorderable and its shells are
selectable like any other space's.

**The stage loses its map controls.** The star-map toggle, the map card and the
tracker-adapter banner do not render, and the map's keyboard bindings are inert
while the Scratch space is on show. The star-map's open state is a standing
preference held on the stage rather than per space, so this has to be enforced
where the map renders and not only where it is toggled — otherwise an operator
who leaves the map open on a registered space and switches to Scratch carries it
across. The stage's empty state is unreachable by construction: a Scratch space
with no shells is not on show.

**The settings surface stops listing it.** It enumerates spaces as config scopes,
and the Scratch space has no config to scope. It reads the unfiltered list minus
Scratch, not the sidebar's visibility-filtered one — a Scratch space that
happened to have a shell open must not appear there either.

Read `docs/design-system.md` before the chrome work. Removing controls touches no
colour, but the emptied action row must collapse cleanly rather than leave a gap,
and nothing here may hand-roll a replacement for a primitive it removes.

Tests lead, at the same seam ticket 01 used: drive the API and read the model
snapshot back. Each repo-scoped endpoint refuses the Scratch identity with a bad
request; deregistering it is refused and it is still in the next snapshot; the
three terminal endpoints still accept it. The chrome changes are verified by
driving the real app — the card carries no remove, branch, skills or new-shell
control, the stage carries no map toggle, and `M` does nothing while Scratch is
selected.

## Done when

Every repo-scoped endpoint refuses the Scratch space with a bad request and
touches nothing in the home directory; the three terminal endpoints still accept
it; the Scratch card shows only its drag handle, its name and its shells; its
stage shows only the terminal; `M` and `Esc` do not summon a star-map over it;
switching to it from a registered space with the map open does not carry the map
across; it does not appear in the settings surface's list of spaces even while it
has a shell open. `go vet ./...`, `go test ./...` and the frontend `check`,
`build` and `vitest` scripts pass, with no amber in the built CSS.
