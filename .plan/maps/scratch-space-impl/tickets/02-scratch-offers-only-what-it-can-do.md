---
type: task
blocked_by: [01]
claimed_by: s03fcfb72b49a
claimed_at: 2026-07-31T14:37:45Z
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

## Answer

Closed the gap on both sides: the refusals are real at the server, and the
controls are gone from the chrome.

**One resolver, swapped in.** `repoSpace` (server/spaces.go) is the single
doorstep for every action that needs a repository: it answers not-found for an
unknown id and refuses the Scratch identity with a `400` naming the reason. It
replaced the repeated `reg.Get` + not-found block at spawn, launch, ideate,
payload preview, `haltTarget` (resume/respawn/release-session), ticket release,
tracker-adapter install, and per-space config open — and it is now the doorstep
at three handlers that previously did no lookup at all: deregister, set-agent,
and tracker dismiss. The three terminal handlers keep the plain lookup, as
specified. Two shapes changed as a consequence, both deliberate and worth a
reader knowing:

- **Deregistering an unknown id was a `204` no-op and is now a `404`**, because
  the resolver is the shape the ticket asked these handlers to share. The same
  applies to set-agent and tracker-dismiss. That is a strictly better answer,
  but it is a wire change beyond Scratch, so it is flagged rather than buried.
- **`handleSetSpaceAgent` now resolves the space before decoding its body**, so
  a Scratch request gets the Scratch refusal rather than "agent is required".

`registry.Deregister`'s own Scratch guard stands untouched underneath — the HTTP
refusal is the answer, the registry invariant is the floor.

**The card lost every control it cannot honour.** The forget button and the
whole action row — branch chip, skills launcher, new-shell control — are behind
`{#if !space.scratch}`. The row is not rendered rather than rendered blank, so
the card's `gap-2` leaves no strip under the shells. The drag handle and the
shell rows stay.

**The stage lost its map controls.** `mapless` gates the star-map toggle, the
tracker banner and the map's `M`/`Esc` bindings. Because the map's open state is
a standing preference on the stage rather than per space, the card itself is
gated by `mapOpen` (`mapShown && !mapless`) where it renders — which is what
stops an operator carrying an open map across from a registered space. Three
other readers of `mapShown` follow from a *rendered* card and now read `mapOpen`
too: the docked split's frozen terminal width, the first-dock measurement, and
the deep-link reflection. The first of those was a real bug in waiting — without
it a Scratch shell would have sat in 60% of the stage with nothing beside it.

**The settings surface stops listing it.** `configurableSpaces` in
`spacevisibility.ts` sits beside `visibleSpaces` as a second pure predicate, and
`App.svelte` feeds it the *unfiltered* snapshot, so a Scratch space with a shell
open is visible in the sidebar and still absent as a config scope.

**Tested** at the seam ticket 01 used. `TestRepoScopedEndpointsRefuseScratch`
drives all thirteen refused endpoints and asserts each answers `400` *with a
message naming Scratch* — several of them would 400 anyway on a missing agent or
map, so the status alone would not have proved the guard. It registers a real
stub agent so the launch endpoints clear their own doorstep first, asserts
Scratch is still in the next snapshot after the refused deregistration, and
asserts the isolated home is still empty afterwards.
`TestTerminalEndpointsAcceptScratch` covers open/seen/close. Three vitest cases
cover `configurableSpaces`, including the Scratch-with-a-shell case that tells it
apart from `visibleSpaces`.

**Verified against a real built binary** under isolated `HOME`,
`XDG_CONFIG_HOME` and data roots: all thirteen endpoints refused with the Scratch
message; a registered space still accepted tracker-dismiss and set-agent (`204`)
and an unknown id still answered `404`; the terminal endpoints accepted Scratch;
the control socket showed Scratch still present after the refused
deregistration; and the isolated home was completely empty — no `.git`, no
`.plan/`, no adapter, no payload. Green: `go vet ./...`, `go test ./... -count=1`,
frontend `check` (0 errors), `build`, and 211 vitests; no amber in the built CSS.

**Left out, deliberately.** No controllable browser was attached to this session,
so the chrome half was verified by build and type-check plus the pure-module
test, not by a visual click-through — the same limitation ticket 01 recorded.
The three DOM claims resting on that (no remove/branch/skills/new-shell control
on the card, no map toggle on the stage, `M` inert) are simple template guards,
but they are unclicked and a human should look. Also: the stage's empty state is
left alone, being unreachable for Scratch by construction, and the stage header's
gear is left in place — it opens the global settings route, not a space scope.

**Unrelated, found in passing:** `web/src/lib/SpacePane.svelte` carries a literal
NUL byte in the `{#key}` separator (`${activeTerm.id}\0${terminalPrefsKey}`). It
predates this ticket, it works, and it is out of scope here — but it makes the
file binary to `grep`/`rg`, which will cost the next reader time.
