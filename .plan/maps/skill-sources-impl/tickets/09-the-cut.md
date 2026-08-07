---
type: task
blocked_by: [06, 07, 08]
claimed_by: s651a9e000f26
claimed_at: 2026-08-07T04:54:19Z
---

# The cut

## Question

Delete the layer model. **Strictly deletion and documentation — the moment this
ticket grows anything behavioural it has stopped being the cut.**

Every consumer has moved by now, which is why this lands last rather than first. The
`simplify` map's cut-first precedent supplies the stance and not the order: its cut
removed a *gate*, so the escape hatch was a terminal and `git`; this cut removes the
*engine*, and a tree with the layer model gone and the registry not yet wired cannot
compose a payload, so it cannot spawn the sessions that would do the remaining work.
There is a hard edge besides — ticket 06's migration consumes the very embed this
ticket deletes.

Delete: `prompt.Roots`, `RootsFor`, `Resolve`, `Names`, `Library`, `Materialize` and
the `LayerBuiltin`/`LayerUser`/`LayerWorkspace` tags; `hashFiles`, `ShippedHash`,
`Skill.Hash`, `Skill.Stale`, `ForkedFrom`, `staleWarning`, `LibraryWarnings`; the
`assets/skills` embed and `<configDir>/builtin-skills/` with its `readmeText`; the
`tracker-convention` skill and `glossary.md` if ticket 07 left anything behind;
`prompt.Launch`; `configsurface.go`'s `resolvedSkills`/`resolveSkillDir` and the
`layerSkillPrefix` hatch. Also the stale duplicate at the repo root (`/skills/`,
gitignored, already drifted from the embedded copy).

**Delete `docs/skill-sync.md` and write ADR 0017 in the same commit.** That file
carries a *"decided 2026-07-22, do not re-litigate"* block whose **no runtime
loading** rule this effort reverses, and no ADR records it — checked against all
sixteen — so deleting it before 0017 exists opens a window where a retired decision
has no home. Its *re-author, never wrap* rule survives, relocated to
`chartr-skills/CONTRACT.md`, where it now applies upstream.

**ADR 0017 — *Skills come from registered sources; chartr ships none*.** It carries
three things, and it is the *three* that make it an ADR rather than a sixth
amendment to 0009: the **model** (one ordered list of operator-owned sources,
first-hit resolution, explicit `[roles]` bindings, chartr shipping only two payloads
and a conventions ruleset — reaffirming ADR 0002 and ADR 0005); the **trust posture
and its cost** (a registered source is executable text reaching agents run with
permissions skipped, and the only assertion of trust in a git source's entire
lifetime is the moment its URL is typed); and the **reproducibility retirement**
(two machines no longer resolve identical bytes for the same ticket; what replaces
it is weaker and honest about being weaker — the trailer names the source and pins
it where there is a pin, and `Payload-SHA256` still fixes the exact bytes for the
machine that composed them).

**ADR 0009 gains a second banner beside its existing one**, not a sixth amendment.
All five of its amendments open by saying the mechanism is untouched, and here the
mechanism dies; its execution half is already superseded, so amending the content
half would leave the file deciding nothing while still reading, top to bottom, as
operative config policy. 0017 supersedes the content half; with the existing banner
covering the other, 0009 becomes wholly historical and the new banner says so in one
line.

Edit `CONTEXT.md`: **Skill library** and **Committed skills** die here, and
**Context bundle** loses any glossary or skill-library-manifest clause ticket 07 did
not already take.

## Done when

A grep for every symbol on the deletion list returns nothing, `internal/prompt` no
longer embeds `assets/skills`, and `<configDir>/builtin-skills/` is written by
nothing. ADR 0017 exists and ADR 0009 carries its second banner; `docs/skill-sync.md`
is gone in the same commit. The cockpit still spawns a ticket session and a free
session. `go vet`, `go test`, and the frontend `check`/`build`/`vitest` pass.

## Answer

The layer model is gone. One commit (`8888e17`), deletion and documentation only
— nothing behavioural was added.

**Deleted, symbol by symbol:** `prompt.Roots`, `RootsFor`, `Resolve`, `Names`,
`Library`, `Materialize`, `LayerBuiltin`/`LayerUser`/`LayerWorkspace`;
`hashFiles`, `shortHash`, `ShippedHash`, `MatchesShipped`, `Skill.Hash`,
`Skill.Stale`, `ForkedFrom`, `staleWarning`, `LibraryWarnings`; the
`assets/skills` embed with `embeddedFiles`, `dirFiles`, `treeFiles`,
`newSkill`, `joinSkill` and `readmeText`; `SourceRepo`/`SourceCommit` and the
four method-skill name constants; `prompt.Launch`; `configsurface.go`'s
`resolvedSkills`, `resolveSkillDir` and the `layerSkillPrefix` hatch. The
`tracker-convention` skill and `glossary.md` were already gone — ticket 07 took
the whole directory — and the gitignored `/skills/` duplicate at the repo root
is removed.

`internal/prompt/prompt.go` went from 594 lines to 125: `CoreSkill`, the three
origin constants, `Skill`, `Part`, `Payload` and `splitFrontmatter`. `Skill`
kept `Name`, `Dir`, `Description`, `Source`, `Commit`, `Body` — `Layer` went
with the tags that were its only values.

**The core moved to `assets/core-ticket.md`**, embedded as a string beside
`core-free.md` rather than read out of an `embed.FS` — chartr's own voice with
nowhere to resolve from. Its bytes are unchanged (a `git mv`), so both goldens
still pass untouched.

### Each Done-when clause

- *A grep for every symbol returns nothing* — a case-insensitive grep over
  `*.go`/`*.svelte`/`*.ts` for the whole list plus `ResolvedSkill`,
  `forked_from` and `assets/skills` returns only `layerUserConfig`, which is
  `user.toml` and not a skill layer.
- *`internal/prompt` no longer embeds `assets/skills`* — the directory does not
  exist; the package embeds three files, `conventions.md` and the two cores.
- *`<configDir>/builtin-skills/` is written by nothing* — `Materialize` and its
  `New` call site are gone. The path survives in exactly one place, `firstrun.go`,
  which only ever *reads* it.
- *ADR 0017 exists, ADR 0009 carries its second banner, `docs/skill-sync.md` is
  gone in the same commit* — all three in `8888e17`. 0017 carries the three
  things: the model (reaffirming 0002 and 0005), the trust posture with the
  URL-typing moment named as the only assertion of trust in a git source's
  lifetime, and the reproducibility retirement with what replaces it stated as
  weaker.
- *The cockpit still spawns a ticket session and a free session* — asserted at
  the process boundary: `TestRegisteredAgentDrivesTheSpawn`,
  `TestFirstRunSeedsTheSkillsAndTheBindings`, the seven `TestFreeSession*`, and
  both payload goldens. All pass.
- *All checks* — `go vet ./...`, `go test ./...`, and `check`/`build`/`vitest`
  in `web/` are clean.

### The one place this cut changed behaviour, and why it could not not

**Ticket 06's rename-aside branch is deleted.** That ticket split an existing
`<configDir>/builtin-skills/` two ways on `prompt.MatchesShipped`: byte-identical
to the embedded library → renamed to `builtin-skills.migrated/` and registered
nowhere; diverging anywhere → registered in place. This ticket deletes the embed
that comparison reads, so there is nothing left to compare against and the branch
has no test it could rest on. It now takes the legacy directory's shape exactly:
`sources.HasSkills(builtin)` → register `Migrated built-in skills` in place;
empty or absent → no row.

I am confident this loses nothing real, and the reasoning is worth having on the
map rather than in a commit message. Ticket 07 changed both the embedded set and
the core's bytes, so `MatchesShipped` already answered *diverging* for every
library materialized by any shipped build — its own answer flags this. The
release freeze means no build between 06 and 09 ever shipped. So on every upgrade
path an operator can actually be on, the rename-aside branch was already
unreachable, and what ships is the branch that would have run anyway. It is also
the branch that touches nothing on disk, which is the direction ticket 06 argued
for in its own words: *keeping too much is the right error to make*. `Nothing in
this effort destroys data` is now true by construction — there is no `os.Rename`
left in the migration.

`TestMigrationRenamesAnUntouchedBuiltinLibraryAside` and
`TestMigrationKeepsAnEditedBuiltinLibraryInPlace` are replaced by
`TestMigrationRegistersTheBuiltinLibraryInPlace` (order after the legacy row,
the in-place path, no aside directory, the operator's bytes surviving) and
`TestMigrationSkipsAnEmptyBuiltinLibrary`. Both `prompt.Materialize` call sites
in that file went with the function; the fixtures now write skill directories by
hand.

### What moved beyond the list, because the list forced it

- **`model.ResolvedSkill` and `Model.Skills`/`Space.Skills`** are deleted. They
  were fed by nothing but `resolvedSkills`. The TypeScript mirror goes with them,
  `Layer` narrows to `'user'`, and `ConfigLayer.holds` loses `'skills'`. No
  component rendered any of it — three model fixtures in tests were the only
  readers.
- **The three skill config layers** (`builtin-skills`, `user-skills`,
  `workspace-skills`) leave `globalLayers`, and `spaceLayers` is deleted: it held
  nothing else, so a space now carries an empty `layers` array. `resolveLayerPath`
  and `resolveGlobalLayerPath` have collapsed into one resolution; the space-scoped
  route survives because the client still asks through a space id. Every retired
  name is now refused with a 400, which the open test asserts by name.
- **The claim trailer's layer form is deleted**, not just unused: with `Layer`
  and `Hash` gone there is no second branch. The core reads `core=chartr` through
  the existing sourced form. That is a change to the bytes ticket 07 recorded
  (`core=built-in:<shippedHash>`), forced by deleting `Skill.Hash`, and ADR 0017's
  reproducibility section is where the reasoning now lives.
- **`prompt_test.go` is deleted whole.** Every test in it — shadowing, the
  no-`SKILL.md` case, fork drift, the nine-skill library, materialize-preserves-
  edits, `Launch` — asserted the deleted model. Its one still-used helper
  (`contains`) moved to `payload_test.go`; `TestNeitherCoreNorRoleIsShadowableByASkillLayer`
  in the server suite still guards the property that outlives it.
- **`README.md` loses its "Bundled skills" roadmap item**, which linked
  `docs/skill-sync.md`. Deleting the doc without it would leave a dangling link,
  and the item describes a mechanism this effort replaced. The rest of the
  narrative pass is ticket 10's.

### Flagged

- **`internal/sources/assets/chartr-skills/`'s `to-spec` and `to-tickets` still
  say "glossary"**, as ticket 07 flagged. Unchanged here — fixing them means
  re-authoring in the skills repo and re-running `make vendor-skills`, which is
  ticket 01's artifact.
- **`docs/getting-started.md` and ADR 0005 still describe the shipped library.**
  The spec assigns the narrative pass to ticket 10; only the vocabulary that this
  ticket makes false was edited (`CONTEXT.md`'s **Skill library** and **Committed
  skills** entries are deleted; **Conventions** loses "tracker adapter" from its
  avoid list). **Context bundle** needed nothing — ticket 07 had already taken its
  glossary and manifest clauses.
- **The release freeze lifts with this commit.** Ticket 06's migration and
  sources-driven role resolution are now in the same tree.

### Deliberately left out

No settings surface for anything deleted here — the sources section is ticket 10,
and until it lands the migrated rows are discoverable only in `sources.toml`.
I did not run the built app by hand: the two Done-when clauses that could regress
(both session kinds spawning) are asserted at the process boundary against a real
PTY, a real registry and a real seeded config root, which is a stronger signal
than a click-through, and the session was asked to be economical. **What that
leaves unverified is the settings screen's appearance** now that the three skill
rows are gone from it — it is one `make build webview` away and no test covers
how it looks with fewer rows.
