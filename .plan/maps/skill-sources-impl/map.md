# Skill sources — implementation

## Destination

chartr ships no skills. The operator registers an ordered list of skill sources —
local folders and pinned git repos — manages it from a screen, and binds each
ticket type to one source-qualified skill. chartr composes two payloads: a ticket
session gets its core plus the bound role body; a free session, launched from the
space card's `new shell` control with no ticket, is told what chartr is and what
skills exist and nothing about how to behave. One generated `conventions.md` under
the config root carries chartr's whole file-format contract so a generic skill from
anyone's repo writes maps chartr can read, and a user-owned `preferences.md` is
appended after it, unranked. Upgrading from the layer model loses nothing. Done end
to end is the [spec](../skill-sources/spec.md)'s five capabilities.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were made on the [planning
map](../skill-sources/map.md) and synthesized into the
[spec](../skill-sources/spec.md), which is the single source of truth here. Do not
re-litigate a decision; if implementation exposes one as wrong, mark the *planning*
ticket `undermined_by` and raise it, rather than quietly deviating.

**Per-session reading order:** the [spec](../skill-sources/spec.md) in full, then
this map, then your ticket, then only the planning ticket your ticket names if you
need the argument behind a decision rather than the decision itself. Vocabulary
comes from [`CONTEXT.md`](../../../CONTEXT.md) — and note that several of its
entries are edited *by tickets on this map*, so read it as of HEAD, not as of the
planning map. ADRs in [`docs/adr/`](../../../docs/adr/) bind; this effort writes
0017 and amends 0009, both in ticket 09.

**This map's tickets are the spec's fourteen, merged to ten.** Four folds were
taken deliberately: git sources into the registry (02), the map-discovery narrowing
into the conventions ticket (03), the role bindings into the seed ticket (05), and
the documentation pass into the settings ticket (10). Every forced edge the spec
named survives the folding. Where a ticket carries a warning the spec attached to a
now-merged half, it says so in the ticket.

**Per-ticket checks, before every commit:** `go vet ./...` and `go test ./...`; in
`web/`, the `check` and `build` scripts plus `vitest`. UI tickets (08, 10) are
additionally bound by [`docs/design-system.md`](../../../docs/design-system.md) per
CLAUDE.md — the split button and the settings rows compose existing primitives, so
no `shadcn-svelte add` step and no new token should be needed; if you reach for
one, the palette is missing a role and that is worth flagging.

**Develop every first-run path against a throwaway config root.**
`XDG_CONFIG_HOME=$(mktemp -d)` — `server.ConfigRoot` already honours it. Your real
config root gives you exactly one migration, and testing ticket 06 by hand will
spend it.

**Release freeze: once ticket 06 lands, no release may be cut until ticket 09
lands.** Migration fires on the absence of `sources.toml`, exactly once per
machine, and it is silent. An operator who upgrades inside that window burns their
one migration on a build where sources do not yet drive role resolution. Anything
up to and including 05 is releasable.

**Self-hosting is the binding constraint.** Every intermediate commit must leave a
chartr that builds, derives ticket status, and spawns — this map is worked from the
cockpit it is rebuilding. The escape hatch is weaker than the `simplify` map's: an
empty shell plus the last good payload read off disk. So land 06, 07 and 09 as
small, independently-green commits, and never leave a red tree overnight.

**Verification that is not a test:** read both payloads in the cockpit's preview,
and spawn a real grilling session on this repo's own next map. Ticket 09 adds one
grep — the dead symbol list returns nothing.

**Running the app:** `make build webview && ./build/shell/$(ls -t build/shell | head -1)`.
There is no map linter command and this effort does not add one — the cockpit's
existing diagnostics stay advisory. Review with `/code-review` before committing a
large diff; `/simplify` is worth a pass on 07, 08 and 10.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A
ticket that exposes a genuinely new question sends it back to the planning map — it
does not open fog here. -->

## Out of scope

Inherited from the [spec](../skill-sources/spec.md#out-of-scope); these never
graduate into tickets on this map.

- **Reaching agents in terminals chartr did not launch** — no managed block in a
  harness's global instruction file, no mirroring into a native skills directory,
  no shim on `PATH`. chartr guarantees what it launches.
- **`domain-modeling`, ADR tooling and `ideate`** — culled, nothing replaces them.
- **Redesigning the wayfinder method** — chartr restates the format, never changes it.
- **A plugin or extension API for skills** — the source list is the extension point.
- **A trust confirm before a git clone, and a changed-skills summary after a
  refresh** — both declined in favour of the fewest steps.
- **A migration report, a live warning on an inert workspace-skills directory, and
  an in-app removal action for either in-repo artifact** — every first-run write
  this effort produces is quiet, and chartr does not delete files in a repo it does
  not own.
- **Whether a space can pin a source**, and **whether a sourced skill's supporting
  files are separately addressable** — both left unresolved on the planning map.
- **A "reset to shipped" action** and **cleaning up the renamed-aside built-in
  directory** — named as revisit triggers, not scoped here.
