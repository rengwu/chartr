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
