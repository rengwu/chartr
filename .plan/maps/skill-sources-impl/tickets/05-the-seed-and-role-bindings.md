---
type: task
blocked_by: [01, 02]
claimed_by: sa7b6dffd9349
claimed_at: 2026-08-06T16:06:19Z
---

# The seed, and role bindings that resolve into it

## Question

Make a fresh install able to spawn a role-typed ticket session offline. Two halves
that the spec kept apart and this map merged, because either alone delivers a
directory or a table rather than a capability: **the seed lands, and the four role
bindings resolve into it.**

**The seed.** Vendor ticket 01's repo as a checked-in copy of a pinned ref under
`internal/sources/assets/chartr-skills/` — with the sources package, not
`internal/prompt`, which this effort is gutting. One `make vendor-skills` target
clones at a ref, replaces the vendored directory wholesale, and rewrites the pin
constant; the procedure lives as that target's comment. **Not a build step that
clones** (a release's contents must be a function of the commit, and a fetching
build would make the test suite need network) and **not a submodule** (`go install`
and source tarballs carry none, so the binary would ship with an empty default
source and a first run would have no role skills at all).

`<configDir>/sources/chartr-skills/` is in one of two states, read off the
filesystem: **no `.git` means chartr's bytes**, reconciled against the embedded seed
at every startup — compare file set and bytes, replace the whole directory when
anything differs, temp-then-rename, deliberately wholesale so a skill deleted
upstream actually disappears. **`.git` present means the operator's pin**, and
chartr never writes there again. The seed records nothing about itself on disk; its
identity is the compiled constant, because an on-disk marker is a second source of
truth whose only distinctive behaviour is going stale. A refresh converts seeded to
pinned exactly once, then refreshes like any other git source. **Deleting the
directory is the reset** — one rule covering reset, repair and a directory an
operator wrecked. Two scalars persist beside the default toggle, the fetched commit
and its timestamp, written only by a refresh and absent while seeded, so the row
reads either *"shipped with this build"* or *"fetched ⟨date⟩ — ⟨sha⟩"* without
inspecting the filesystem at load. This makes the default row `git`-kinded; nothing
else about it moves.

Retire the provenance machinery this replaces: `hashFiles`, `ShippedHash`,
`Skill.Hash`, `forked_from`, `Skill.Stale`, `staleWarning`, `LibraryWarnings` and
`Materialize` are on ticket 09's deletion list — **do not delete them here**, ticket
06 consumes the byte-comparison one last time. `splitFrontmatter` survives, demoted
to stripping the body.

**The bindings.** A flat `[roles]` scalar table in `user.toml`, beside `[agents.*]`
— four `role = "Source/skill"` lines, **always qualified, never bare**. A bare name
resolved through source order would reintroduce exactly the implicit capture the
table exists to prevent. Seed all four **once**, on the first startup that finds no
table at all, immediately after the reconciliation above so the default source is
guaranteed present — the presence test is the table, not the file, so an operator
upgrading with an existing `user.toml` seeds like a new install. **Never
auto-refill:** a deleted row is a legitimate way to make a role refuse until
rebound, and recovery is an explicit single-row settings action in ticket 10,
written through the comment-preserving TOML surgery `SetUserAgent` already uses.

**An unresolvable binding refuses the spawn — no terminal, no claim commit.** This
needs no new code path: `prompt.Compose` already returns an error and
`launchSession` already aborts on it before `writeClaimCommit` runs. Resolving
through the registry instead of `prompt.Resolve(role, roots)` is a swap inside that
call. The error names the role, the recorded binding string, and **which of the
three unresolvable shapes it hit** — disabled source, removed source, or skill
missing from that source. Re-key the claim trailer to `Skill: <name>=<source>` for a
dir source and `<name>=<source>@<commit>` where the source carries a pin.

`RoleForTicketType` and `config.Roles` are **kept unchanged** — role is the key the
spawn gate offers all four values under, it drives the quiet hint, it validates
input at the spawn boundary, and the trailer records it on its own line. Update
`CONTEXT.md`'s **Role** entry to drop "it selects a skill".

## Done when

Against a throwaway config root with no network: the seed materializes, `[roles]`
is seeded with four qualified rows, and a `grilling` ticket composes a payload
containing the seed's `grill` body. Putting a `.git` in the seeded directory stops
chartr writing there; deleting the directory re-materializes it next startup. An
unresolvable binding aborts before any claim commit, with an error naming the role,
the binding string and which shape it hit — asserted at the process boundary on the
response and on the absence of a commit in the space's history. `make
vendor-skills` reproduces the vendored directory. `go vet` and `go test` pass.

## Answer

A fresh install spawns a role-typed session offline. Both halves landed, as two
commits: `76161c1` vendors the seed and gives the default row a refresh,
`c580565` binds the four roles and swaps role resolution onto the registry.

**The seed.** `internal/sources/assets/chartr-skills/` is a checked-in copy of
ticket 01's repo at `ece028df`, embedded by `internal/sources/seed.go` — with the
sources package, not `internal/prompt`. `Reconcile(configDir)` runs at every
startup: `.git` present means the operator's pin and chartr returns without
touching it; otherwise the whole file set and its bytes are compared against the
embedded copy and the directory is **replaced wholesale** through a temp dir and a
rename. `SeedCommit` is the only record of the seed's identity, and nothing is
written on disk about it. `make vendor-skills` clones at a ref, replaces the
directory wholesale and rewrites that constant; the procedure lives as the
target's comment.

**`Refresh` now takes the default row** rather than refusing it, which closes the
gap ticket 02's answer left open. While seeded there is nothing to fetch into, so
the first refresh clones `SeedURL` over chartr's bytes and the `.git` it leaves is
what converts the source to pinned for good; every later refresh takes the
ordinary fetch path. `RegisterGit`'s clone is extracted to `cloneInto` and shared,
and the two scalars ticket 02 already persisted are written here for the first
time.

**The bindings.** `internal/config/roles.go` reads and writes a flat `[roles]`
table in `user.toml`. `SeedRoleBindings` writes four qualified rows if — and only
if — there is no table at all, and `SetUserRole` rebinds one row through the same
comment-preserving surgery `SetUserAgent` uses, refusing a bare name at the writer
with the qualified form shown. Startup runs `sources.Load` → `sources.Reconcile` →
`seedRoleBindings`, in that order, so a seeded row never points at a directory
that is not yet there. Bindings are re-read fresh at every composition, so a hand
edit reaches the next spawn with no restart.

**The swap.** `ComposeInput` gains `Sources` and `Bindings`; the role half
resolves through `resolveRoleSkill` (`internal/prompt/rolebinding.go`) and the
core still resolves through the layers, which is ticket 07's to move. An
unresolvable binding needed no new code path, exactly as the ticket said: `Compose`
returns the error and `launchSession` aborts on it before `writeClaimCommit`. The
claim trailer is re-keyed by `skillTrailer` to `<name>=<source>[@<commit>]` for a
sourced skill, keeping the layer form for the core alone.

### Each Done-when clause

- *Seed materializes; `[roles]` seeded with four qualified rows; a `grilling`
  ticket composes the seed's `grill` body* — `TestFirstRunSeedsTheSkillsAndTheBindings`
  asserts all three against a clean config root, reading the marker sentence out
  of the materialized `SKILL.md` and finding it in the composed part, whose
  provenance tag is `chartr-skills`. No network anywhere in it.
- *A `.git` stops chartr writing; deleting re-materializes next startup* —
  `TestSeedResetsOnDeleteAndStopsAtAGitDirectory` at the process boundary, plus
  `TestReconcileNeverOverwritesAPinnedCheckout` and `TestReconcileReplacesWholesale`
  at the package seam (the latter also covers an edited skill being restored and a
  file the seed does not carry being removed).
- *An unresolvable binding aborts before any claim commit, naming the role, the
  binding and the shape* — `TestUnresolvableBindingRefusesTheSpawnWithoutClaiming`
  (removed source, missing skill) and `TestSpawnRefusedWhenTheBoundSourceIsDisabled`
  (disabled, which needs the list in that state before the server loads it, so the
  sources file is written first). All three assert on the HTTP response *and* on
  `git rev-parse HEAD` still failing in the space, plus the ticket still deriving
  open and on the frontier.
- *`make vendor-skills` reproduces the vendored directory* — run against the local
  checkout (`SKILLS_REPO=~/Desktop/Projects/chartr-skills`) after the directory was
  first populated by `git archive`; it produced a byte-identical tree and left
  `SeedCommit` on the same ref.
- *`go vet` and `go test` pass* — both clean. `web`'s `check`, `build` and
  `vitest` also run clean; no frontend file changed.

### Existing tests this moved, and why

Four tests asserted the layer model over a *role* skill, which is exactly what
this ticket replaces. `TestPayloadComposesWithProvenanceAndBundle` and
`TestSkillShadowingMatrix` now expect `chartr-skills` for the role part;
`TestBehindDefaultSurfaced` and `TestMaterializedLibraryEditsCompose` were
retargeted onto `core`, the one skill still resolved through the layers, so the
fork-staleness and materialized-edit behaviours stay covered until ticket 09
deletes them outright. The spawn trailer assertion moved to `Skill:
implement=chartr-skills`. `TestUnreadablePromptDeliveryWarnsAndFallsBack` writes
`user.toml` wholesale and so had to carry the seeded `[roles]` table — a real
consequence of never auto-refilling, and worth having in a test.

### Flagged

- **`SeedURL` points at a remote that does not exist yet.** Ticket 01 left the
  skills repo local-only, so `https://github.com/rengwu/chartr-skills.git` is the
  intended upstream, not a live one. Nothing on the offline path touches it: the
  seed is embedded, and only a refresh of the default row would fetch. The
  seeded→pinned conversion is tested against a local repository (a clone from a
  path is the same code path), but it has never run against the real remote, and
  it will fail until that repo is pushed. `make vendor-skills` has the same
  dependency and the same `SKILLS_REPO` escape hatch.
- **`Compose` keeps a layer fallback when `Sources` is nil**, for this package's
  own tests, which compose with roots alone. Both server call sites pass the
  registry. It is a hedge, and it dies with the layer model in ticket 09.
- **The `[roles]` table is written unaligned**, matching every other key chartr
  writes rather than the aligned block the spec and this ticket show. A column
  that only holds while chartr is the writer is one an operator's own edit
  silently breaks.

### Deliberately left out

No deletions from ticket 09's list — `hashFiles`, `ShippedHash`, `Skill.Hash`,
`forked_from`, `Skill.Stale`, `staleWarning`, `LibraryWarnings` and `Materialize`
all still stand, and ticket 06 still needs the byte comparison. No settings
surface for rebinding a role (`SetUserRole` exists and is tested; wiring it to a
route and a row is ticket 10's). No sources file written at startup and no
migration — ticket 06's. `Contract` is still not a payload part — ticket 07's.
Nothing in the frontend moved.

**Release note:** this commit is releasable. The freeze the map's Notes describe
begins at ticket 06.
