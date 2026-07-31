# Scratch space — implementation

## Destination

The [spec](../scratch-space/spec.md) implemented end to end: one built-in
**Scratch** space, present from first run, that holds ad-hoc shells in the
operator's home directory and nothing else. It is never registered, never
`git init`ed, and cannot be removed. It is hidden until it has a shell open and
disappears when the last one ends, taking the selection to the neighbouring row
the way a forgotten space does. It sits in the operator's own sidebar
arrangement, holds that slot even while hidden, and refuses every action that
needs a repository. Done looks like a **New Scratch Shell** control in the
sidebar footer, one click from a shell in `$HOME`, and no `git init` anywhere.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were settled in the
[spec](../scratch-space/spec.md), which is the single source of truth here. Do
not re-litigate a decision; if implementation exposes one as wrong, raise it
rather than quietly deviating. This effort has no planning map: it was charted
from a design conversation straight into the spec.

**Per-session reading order:** the spec, then this map, then your ticket.
Vocabulary comes from `CONTEXT.md` at the repo root — *space*, *session*,
*cockpit*, *chrome*, *control socket* are its words and the tickets use them.
Note the one that matters most here: an **ad-hoc shell is not a session**. A
session is a PTY against a ticket; the Scratch space holds shells and can hold
nothing else. The spec names the settled seams; prefer them to line-level file
paths, which go stale.

**The sequence fans out from the tracer bullet.** `01 → {02, 03}`. Ticket 01
puts the whole path in — registry entry, snapshot, footer control, show/hide —
and 02 and 03 are independent of each other: refusing repo-scoped actions and
remembering a sidebar slot do not touch. 01 carries the reorder tolerance rather
than deferring it to 03, because without it the first hidden Scratch space
breaks dragging for every *registered* space, which is a regression inside 01
rather than a gap 03 fills.

**The frontend rules are binding.** The sidebar and the stage are chrome, so
ADR 0010 and ADR 0012 apply in full and `docs/design-system.md` is required
reading before any UI work: a token for every colour, a vendored primitive for
every component, no raw hex, no amber, Phosphor icons. Nothing in this effort
goes near the terminal or star-map islands — a scratch shell is an ordinary
ad-hoc shell and the terminal socket, scrollback replay and customization seam
are untouched.

**Relevant ADRs.** ADR 0003 (the space is the unit of serialisation) is why the
Scratch space can hold no session: it has no map and nothing to serialise, so
the live-session gate never applies to it. ADR 0009 (execution vs content
config) is why it resolves no committed layers. ADR 0010 is why this is a chrome
and model change and not an island one.

**Per-ticket checks:** `go vet ./...` and `go test ./...`, plus the frontend
`check`, `build` and `vitest` scripts for any ticket touching `web/` — the embed
test compiles against `dist/`. Grep the built CSS for amber before committing.
Verify against a real running binary with an isolated config root
(`XDG_CONFIG_HOME`) rather than the operator's own `spaces.toml`; this effort
writes to the registry file and reads `$HOME`, so an unisolated run touches both.

**Linting this map.** The repo carries the wayfinder linter (`wayfinder.Lint`,
reached through `mapscan.Discover`, which folds its diagnostics into each map's
malformations) but wires no CLI for it. The cockpit surfaces them on the map
itself, so the cheapest check is to open this map in chartr; failing that, a
throwaway `package main` in the module that calls `Discover` and prints the
malformations does it in one run.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the spec — it does not open fog here. -->

## Out of scope

- **Choosing a folder for a scratch shell** — the working directory is `$HOME`,
  full stop; a per-shell chooser is the registration flow minus `git init`,
  which is a different feature.
- **More than one scratch space** — one is a home for loose shells; several is a
  space registry without the registry.
- **Renaming, pinning or configuring it** — it has no settings.
- **Persisting scratch shells across a restart** — shells die with the process
  today and scratch shells are no different; a restart simply leaves the Scratch
  space hidden.
- **Running skills, agents or sessions in it** — deliberately refused, not
  deferred: a Scratch space that could spawn is a space.
- **Deep-linking the Scratch space** — star links name a map and a ticket, and
  it has neither.
- **A shell at an arbitrary path** — this introduces no general "terminal
  anywhere" surface.
