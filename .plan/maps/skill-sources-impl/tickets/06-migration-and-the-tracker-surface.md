---
type: task
blocked_by: [02]
claimed_by: s8452a699f9cc
claimed_at: 2026-08-06T16:29:29Z
---

# Migration, and the tracker-adapter surface

## Question

Make an upgrade from the layer model lose nothing, and delete the one feature this
effort's boundary refuses to keep.

**Read the release freeze in the map's Notes before starting.** Migration fires on
the absence of `sources.toml`, exactly once per machine, and it is silent — so once
this ticket lands, **no release may be cut until ticket 09 lands.** Develop against
`XDG_CONFIG_HOME=$(mktemp -d)`; your real config root gives you one migration and
testing this by hand will spend it.

**Who owns the bytes decides all four fates.** The two directories under chartr's
config root get an active migration. The two inside the operator's repo get a
stated-and-left-alone fate, because the ground rule means chartr may only *stop
touching* them — never delete or rewrite.

`<configDir>/skills/` auto-registers as an ordinary `dir` source named **`Legacy
skills`**, only if it exists and the discovery walk finds at least one skill; empty
or absent contributes no row. That an auto-registered fork of `implement` stops
driving `task` tickets is real but bounded to free-session bare-name lookups —
qualified bindings already closed that door for every source, migrated or not.

`<configDir>/builtin-skills/` is compared once against the shipped copy being
retired, **using the byte-comparison ticket 09 deletes** — this is the one call site
that gets to use it before it goes, which is cheaper than reinventing a differ.
Byte-identical, empty or absent: **rename it aside to `builtin-skills.migrated/`,
unregistered**, for the operator to remove or ignore. This is deliberately a rename
and not a delete — it is the only irreversible operation on the map, it runs once,
it is silent, an operator's edited fork lives nowhere else, and it rests entirely on
a comparison being right. A stray directory nobody cleans up and a wrong delete of
someone's only copy are not comparable. Diverging anywhere: leave it exactly where
it is and register it as **`Migrated built-in skills`**. **`Legacy skills` first,
`Migrated built-in skills` second**, both before the default row — the old order was
workspace › user › built-in, and the surviving relative order carries forward.

`<space>/.chartr/skills/` — chartr simply stops resolving it. The directory is
untouched and **nothing is said**. It goes inert exactly as silently as it goes
unread.

`docs/agents/issue-tracker.md` — stop writing new ones, stop refreshing existing
ones, and **delete the whole offer surface**: the install and dismiss handlers, the
classify and install call sites, the offer on the Go model and its TypeScript
mirror, `TrackerDismissed`/`SetTrackerDismissed` in the space registry,
`TrackerAdapterBanner.svelte` and its wiring in `MapCard.svelte`, the
`internal/tracker` package and its embedded template. This is a full deletion rather
than a shrink because the offer's entire reason to exist is reaching an agent chartr
did not launch, which this effort refuses everywhere else. **Existing files are
declared harmless and left alone** — their content stays true, since nothing here
moves or reshapes `.plan/maps/`; the file merely stops being refreshed. The
asymmetry with `.chartr/skills/` is deliberate: that one is a behaviour regression
an operator can be surprised by, this one is not.

**The first-run sequence is one function, in this order:** scan both config-root
skill directories and diff the built-in copy; rename the built-in copy aside if
untouched; write `sources.toml` with the default toggle plus whichever migrated
rows apply; reconcile the default source from the seed (ticket 05); seed `[roles]`
if absent (ticket 05); materialize `conventions.md` (ticket 03). Each ticket adds
its step **in place** rather than inventing its own startup hook — this ticket owns
steps one to three and the function they live in. **Nothing is reported.** Every
first-run write this effort produces is quiet; the migrated rows are discoverable
the moment the operator opens Settings, and nothing pushes them, or the fact that a
migrated fork no longer drives its old role, at the operator on the run it happens.

## Done when

Three migration cases pass at the process boundary against a pre-populated old
root: a `<configDir>/skills/` holding one skill becomes a `Legacy skills` row; a
`builtin-skills/` byte-identical to shipped is renamed aside and **not** registered;
a `builtin-skills/` with one edited byte survives in place and becomes a `Migrated
built-in skills` row, ordered after the legacy row. Nothing is written into a space
repo and nothing is reported. The tracker-adapter offer is gone from the model, the
API, the registry and the UI, and an existing `docs/agents/issue-tracker.md` is left
untouched. `go vet`, `go test`, and the frontend `check`/`build`/`vitest` pass.

## Answer

The first-run sequence is now one function — `firstRun` in
`internal/server/firstrun.go` — and `New` calls it in place of the four calls
tickets 03 and 05 had left inline. Its order is the ticket's: migrate, load the
list, reconcile the seed, seed `[roles]`, materialize `conventions.md`. Each step
carries its own idempotence; only the migration half is once-per-machine.

**The migration** (`migrateSkillLayers`) fires on the absence of
`sources.toml` — exposed as the new `sources.FilePath(configDir)` — and always
leaves that file behind, even with nothing to migrate, so it cannot fire twice.
`<configDir>/skills/` becomes a `Legacy skills` `dir` row when the bounded
discovery walk finds at least one skill (`sources.HasSkills`, the same `discover`
a registered source is read with, so an empty directory answers no rather than
becoming a permanently `empty` row). `<configDir>/builtin-skills/` goes through
`prompt.MatchesShipped` — the byte comparison ticket 09 deletes, borrowed here for
its one call site — and is either renamed aside to `builtin-skills.migrated/`
unregistered, or left exactly where it is as `Migrated built-in skills`. `Legacy
skills` registers first, so the surviving relative order is legacy › built-in ›
default. `<space>/.chartr/skills/` gets no code at all; it goes inert by ticket
07 simply not resolving it.

**Ordering that had to change.** `firstRun` runs *before* `prompt.Materialize`,
not after. Materialize tops the built-in library back up file by file, so an
upgraded root compared after it would read as untouched and get renamed aside with
the operator's fork inside it. This is the one structural consequence worth
knowing about: on the run the migration fires, the aside-rename is immediately
followed by Materialize writing a fresh shipped copy back to `builtin-skills/`.
That is correct for now — role resolution still goes through the layer model until
ticket 07 — and it disappears with Materialize in ticket 09. It is called out in a
comment at both sites.

**The tracker-adapter offer is fully deleted**, per the ticket's list: the
`internal/tracker` package and its tests, `assets/issue-tracker.md` and
`prompt.TrackerAdapter`, both routes and both handlers, the classify call site in
`modelSpace`, `model.Space.TrackerAdapter` and `TrackerAdapterOffer` with their
TypeScript mirrors, `TrackerDismissed`/`SetTrackerDismissed` in the space
registry, `TrackerAdapterBanner.svelte`, and its wiring. The ticket named
`MapCard.svelte`; **`SpacePane.svelte` also carried the banner** (the
stale/foreign chrome states, with MapCard holding only the `absent` empty-picker
case) and both are gone. Two stale prose comments went with them
(`internal/server/gate.go`, `SpacePane.svelte`). `prompt.TrackerSkill` —
`tracker-convention` — is a different thing and is untouched.

### Each Done-when clause

- *A `<configDir>/skills/` holding one skill becomes a `Legacy skills` row* —
  `TestMigrationRegistersTheLegacySkillsDirectory`, asserting name, kind, path and
  enabled; `TestMigrationSkipsAnEmptyLegacySkillsDirectory` holds the other half
  and checks the list file is written anyway.
- *A `builtin-skills/` byte-identical to shipped is renamed aside and not
  registered* — `TestMigrationRenamesAnUntouchedBuiltinLibraryAside`, which also
  asserts the renamed directory kept its contents.
- *One edited byte survives in place and becomes a `Migrated built-in skills` row,
  ordered after the legacy row* — `TestMigrationKeepsAnEditedBuiltinLibraryInPlace`,
  which asserts the order, the in-place path, the absence of an aside directory,
  and that the edit itself survived.
- *Nothing is written into a space repo and nothing is reported* —
  `TestMigrationWritesNothingIntoASpaceAndReportsNothing`: an in-repo
  `.chartr/skills/` is untouched and unregistered, no adapter appears, and the log
  never names the migration.
- *An existing `docs/agents/issue-tracker.md` is left untouched* —
  `TestAnExistingTrackerAdapterIsLeftUntouched`, byte-comparing it across a
  register and a snapshot.
- Every case runs at the process boundary: a pre-populated temp config root, a
  real `server.New` via `chartrtest.Start`, and the list read back off disk.
  `TestMigrationRunsOnlyOnce` covers the fire-once rule by removing a migrated row
  and starting again.
- `go vet ./...`, `go test ./...`, and `web`'s `check`, `build` and `vitest` all
  pass.

### Judgment calls worth knowing

- **`MatchesShipped` compares the whole tree, README included, as a set.** Extra
  files, missing files or a differing README all read as diverging. That is the
  conservative direction — an upgrading operator whose `builtin-skills/` was
  materialized by an *older* build will diverge from this build's embedded copy
  and keep their directory as a registered row rather than have it moved. Given
  the operation is the map's only irreversible one, keeping too much is the right
  error to make.
- **An existing `builtin-skills.migrated/`** (an operator who ran a pre-release
  build twice, say) is never merged into or replaced: the rename is skipped and
  the directory is left where it is, unregistered. Silent, like everything else
  here.
- **Three small additions to ticket 02's seam**: `sources.FilePath`,
  `sources.HasSkills`, and `(*Registry).Save`. `Save` exists only because the
  migration must guarantee the *file* even when it registers no rows — every other
  mutation on the seam already saves.
- **Three registry tests** used `SetTrackerDismissed` purely as "force a save";
  they now use `SetLastAgent`, which is the same lever.

### Deliberately left out

No deletions from ticket 09's list beyond the tracker surface — `hashFiles`,
`ShippedHash`, `Skill.Stale`, `LibraryWarnings` and `Materialize` all still stand,
and `MatchesShipped` is written to go with them. No settings surface for the
migrated rows (ticket 10); they are only discoverable in `sources.toml` until
then. `CONTEXT.md` line 103 still lists "tracker adapter" in the *avoid* list of
the tracker-convention entry — harmless, and the vocabulary pass belongs to its
own ticket. `docs/skill-sync.md` needed nothing: every mention there is the
`tracker-convention` skill, not the adapter.

**Release note:** the freeze is now in force. This commit migrates on the absence
of `sources.toml`, silently and once per machine, on a build where sources do not
yet drive role resolution. **No release until ticket 09 lands.**
