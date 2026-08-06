---
type: task
blocked_by: [09]
---

# The sources settings section, and the documentation pass

## Question

Give the source list a screen, and write the narrative prose that describes the
finished system. Both are terminal work and the docs describe the state this ticket
completes, which is why they ride together rather than trailing each other.

**The screen is load-bearing for three resolved decisions**, not a convenience.
Ticket 01's orphaned-checkout warning and its "this checkout is chartr's" caveat
live here; ticket 03's only recovery for a deleted role binding is a control here;
and **ticket 07's entire acceptance of a silent migration rests on "it shows up the
moment the operator opens Settings."** Ship without it and that silence has nothing
to be discoverable *in*.

The editable/open-the-file line is drawn on ADR 0014's existing boundary and is
already decided: **editable** is the set of actions the planning tickets designed —
register a source, remove one, toggle enabled, reorder, refresh a git source, and
restore a role binding to its default. **Open-the-file** is everything else:
`sources.toml`, `user.toml`, `conventions.md`, `preferences.md`, each openable in
the operator's editor. Never a second config store. Layout and placement are yours
to choose when you work this.

A row renders name, kind, enabled, position, status (`ok` / `unavailable` /
`empty`), and skill count. A git row additionally renders url, commit, fetched time
and a refresh action, and **must say that the checkout is chartr's and that a
refresh discards local edits inside it** — this will bite someone otherwise, and the
answer to "I want to edit this" is a `dir` source. Where the file is named, say that
git checkouts under `sources/` are orphaned if `sources.toml` is lost and that
chartr does not collect them. A source shadowed by a higher one is marked as
shadowed; the default row reads either *"shipped with this build"* or *"fetched
⟨date⟩ — ⟨sha⟩"*. An `empty` source reads `0 skills` with a remove action beside it.
Registering a `git` source when `git` is absent from `PATH` is refused at the gate,
naming why, before a row is written. **The free payload's preview hangs here** —
same seam, same modal, four parts, no ticket or role selector. It is the only place
the operator sees their own `preferences.md` land in an assembled document.

Existing primitives only: **no `shadcn-svelte add` step and no new token should be
needed.** If you reach for one, the palette is missing a role and that is worth
flagging rather than working around.

**The documentation pass.** `docs/getting-started.md` describes the skill library
and `CLAUDE.md` describes the maps convention; both are narrative prose about an end
state, so every intermediate version would have been wrong regardless of when it was
written. Rewrite them against what shipped. Then do a **final coherence read of
`CONTEXT.md`** — tickets 03, 05, 08 and 09 each edited it in place, which keeps it
honest but not necessarily elegant. `docs/design-system.md` is untouched.

## Done when

An operator can register a folder and a git repo of skills, reorder them, toggle
one off, refresh a git source and see its new pin, restore a deleted role binding,
and open each of the four config files in their editor — all without hand-editing
TOML. A source registered before the migration ran is visible in the list. The free
payload previews from this screen. `docs/getting-started.md` and `CLAUDE.md`
describe the shipped system and `CONTEXT.md` reads as one document. `go vet`,
`go test`, and the frontend `check`/`build`/`vitest` pass.
