---
type: task
blocked_by: [01, 02]
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
