---
type: task
blocked_by: [09]
claimed_by: sfccf652981ed
claimed_at: 2026-08-07T06:29:03Z
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

## Answer

The source list has a screen, and the prose describes what shipped. One commit
(`e8e5e33`).

**The screen.** `SourcesSettings.svelte` on the settings route's global scope,
under the agent library and above the per-machine cosmetics. It carries exactly
the six actions the planning tickets designed — register a source, remove one,
toggle enabled, reorder, refresh a git source, restore a role binding — behind
six new routes under `/api/config/sources` and `/api/config/roles`. Everything
else is an open-the-file action on the existing named-layer route: `sources.toml`,
`conventions.md` and `preferences.md` join `user.toml` on `globalLayers()`, so
ADR 0014's boundary held without a second config store and without a route that
takes a path from the client.

Reorder is two arrow buttons rather than drag-and-drop. Position is shown as the
number it is beside them, because position *is* resolution order and the operator
should read it rather than infer it from where a row sits. Removal asks twice on
the second click — a git registration is not cheap to undo.

**The three decisions it is load-bearing for**, each with the copy that carries
it:

- *Ticket 01's orphaned checkout and the "this checkout is chartr's" caveat.*
  Every git row says the checkout is chartr's, that a refresh discards anything
  edited inside it, and that the answer to "I want to edit this" is a `dir`
  source. The registration form says the same thing *before* the clone, beside
  the URL field, because typing the URL is the only assertion of trust in the
  source's whole lifetime (ADR 0017) and nothing asks again. Where `sources.toml`
  is named, the orphaning of `sources/` is named with it.
- *Ticket 03's only recovery for a deleted role binding.* A Role bindings block
  under the source list renders all four with what they resolve to, tells a
  deleted row (`not bound — this role refuses to spawn`) apart from an
  unresolvable one (the ref in `--destructive` with the reason beside it), and
  offers **Restore default** on any row that is not already at its seeded value.
  It writes through `config.SetUserRole`, the same seam the seed writes through.
- *Ticket 07's silent migration.* A migrated source is an ordinary row from the
  first time Settings is opened — no badge, no report, nothing to dismiss. That
  is the whole of what makes the silence acceptable, and
  `TestASourceRegisteredBeforeFirstOpenIsVisible` asserts it end to end from a
  legacy `skills/` directory.

**The free payload's preview** hangs off the section header. It is the same seam
and the same modal: `PayloadPreview.svelte` gained a `free` mode that swaps the
fetch for `GET /api/payload/free`, drops the role selector and drops the "what
will run it" block (a free session picks its agent at the `new shell` caret, so
that block has no answer for it). Everything else — origin badges, warnings, the
composed-document disclosure — is unchanged, which is what makes it the one place
the operator watches their own `preferences.md` land in an assembled document.

**Wire.** `model.Source` and `model.RoleBinding` are mirrors of `sources.State`
and the `[roles]` table rather than re-exports, keeping `model` the leaf package
every other one writes into. Both are walked fresh on every rebuild, like the
PATH probes beside them — a cached skill count would be wrong the moment a source
changed underneath. `gitAvailable` rides along so the registration form can say
the refusal is coming instead of only reporting it afterwards.

### Each Done-when clause

- *Register a folder and a git repo, reorder, toggle off, refresh and see the new
  pin, restore a deleted binding, open each of the four files — without
  hand-editing TOML.* All six actions ship;
  `TestSourcesSectionDrivesTheList` drives register → count → shadowing →
  reorder → toggle → remove against the snapshot, `TestRestoringADeletedRoleBinding`
  drives the binding, and `TestTheSourcesFilesAreOpenableByName` asserts all four
  files resolve by name (and that an unknown name is a 400, not a path).
  **Not exercised end to end: a real `git` registration and refresh**, which
  would need a network or a fixture repo — the gate below is what is asserted
  instead, and `internal/sources` already has the clone and fetch under test.
- *A source registered before the migration ran is visible.* Asserted above,
  including its path and skill count.
- *The free payload previews from this screen.* `TestTheFreePayloadPreviewsFromSettings`
  at the route, and the `free` mode in the modal.
- *`git` absent refuses at the gate, naming why, before a row is written.*
  `TestAGitSourceIsRefusedWithoutGit` empties PATH after startup and asserts the
  400, the message, and that no row appeared.
- *The docs describe the shipped system.* `docs/getting-started.md` gains a
  "Bring your own skills" section (sources, order, the seeded row, bindings, both
  cautions, the free preview), its map-charting step is now the `new shell` free
  session rather than the deleted skill launcher, the paths table lists the five
  config-root files that actually exist, and the dangling `skills/README.md` link
  is gone. `CLAUDE.md`'s maps section names `conventions.md` as the contract
  instead of the deleted `tracker-convention` adapter, and says where the
  vendored seed lives.
- *`CONTEXT.md` reads as one document.* Three entries were false after tickets 05
  and 08: **Settings surface** (it now carries sources, bindings and the free
  preview, and the editable/open-the-file line is worth stating once), **Agent
  library** ("refuses every spawn, ideate included" — `ideate` is gone), and
  **User config** (it carries the `[roles]` table too). Everything else was
  already true; the coherence read changed nothing for style alone.
- *All checks.* `go vet ./...`, `go test ./...`, and `check`/`build`/`vitest` in
  `web/` are clean.

### Design system

Existing primitives only — `Button`, `Badge`, `Input`, `Checkbox`, `Select`, and
the section shape `AgentLibrary` already uses. No `shadcn-svelte add`, no new
token, and no raw colour: the only chromatic token used is `--destructive`, on
the one row that is genuinely broken. Nothing about the palette felt missing.

### Deliberately left out

- **A folder picker for a `dir` source.** The native chooser exists
  (`nativePicker`, `/api/spaces/pick`) but it is scoped to registering a space,
  and widening it is a different ticket's call. The path is typed, as
  `sources.RegisterDir` already expects.
- **Drag-and-drop reordering.** The space sidebar has it and `reorder.ts` is
  shared, but two arrows are less machinery for a list of two or three rows, and
  they are keyboard-reachable without a second interaction model.
- **A shadowed-skill list on the row.** A source whose every skill is shadowed is
  badged; *which* names lost is on the wire (`shadowed`) and unrendered. The
  payload preview's origin badges already answer it per skill, and duplicating
  that here would be a second place to keep true.
- **I did not run the built app by hand.** Every clause above is asserted at the
  process boundary, and the session was asked to be economical. **What that leaves
  unverified is the same thing ticket 09 flagged and this ticket inherits: how
  the settings screen actually looks** — now with two new sections rather than
  three fewer rows. It is one `make build webview` away, and no test covers
  appearance. The map's own "read both payloads in the cockpit's preview" check
  is likewise unrun.

### Flagged

- **`internal/sources/assets/chartr-skills/`'s `to-spec` and `to-tickets` still
  say "glossary"** — flagged by tickets 07 and 09, unchanged here for the same
  reason: fixing it means re-authoring in the skills repo and re-running
  `make vendor-skills`, which is ticket 01's artifact.
- **ADR 0005 still describes the shipped library**, as ticket 09 noted. The spec
  assigned the narrative pass to this ticket, and ADRs are a decision record
  rather than narrative prose — 0017 supersedes the model 0005 describes and
  says so. Amending 0005 would be re-deciding, not documenting; if a banner is
  wanted there it is a one-line follow-up.
