# agent selection — implementation

## Destination

The [spec](../agent-selection/spec.md) implemented end to end: no hardcoded
default agent, no role bindings, no committed execution config. The operator
registers agents in one global library, picks one at the moment of spawning, and
every spawn surface names the agent it will run. A space quietly remembers what
it last used; an empty library refuses every spawn — ideate included — and points
at registration. Done looks like `internal/config` reduced to the agent library
and a PATH probe, `.chartr/config.toml` gone from the codebase, and the settings
route showing agents and file paths and nothing else.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were settled in the
[spec](../agent-selection/spec.md), which is the single source of truth here. Do
not re-litigate a decision; if implementation exposes one as wrong, raise it
rather than quietly deviating. This effort has no planning map: it was charted
straight from a design conversation into the spec.

**Per-session reading order:** the spec, then this map, then your ticket. The
spec carries the settled seams and symbols each ticket names — prefer them to
brittle line-level paths, which go stale. Vocabulary comes from `CONTEXT.md` at
the repo root; ticket 05 updates two of its entries.

**The ADRs under `docs/adr/` are binding, and this effort supersedes two.**
ADR 0009's execution half (role bindings, the committed layer, field-level
merge) and ADR 0014 (the effective config surface) both fall in ticket 05 —
their content halves and the skills layering are untouched. **ADR 0002
(agent-agnostic adapters) is upheld and load-bearing:** the PATH probe in ticket
04 asserts only that a binary exists, never what it does or what flags it takes,
and no ticket may introduce a curated per-CLI flag UI. A ticket that touches an
ADR says so in its answer.

**Sequencing is expand-then-contract, and the chain is the point.** The order is
`01 → 02 → 03 → 04 → 05`. Explicit agent selection is added *beside* the binding
path (01), the frontend is taught to always send one (02, 03), the parameter then
becomes required so nothing can reach the binding path (04), and only then is the
binding layer deleted (05). Every intermediate ticket is independently green and
independently demoable; no ticket leaves a half-migrated resolver behind.

**Self-hosting is the binding constraint, and it bites here.** This repo drives
itself through chartr — the sessions working this map are spawned by the code
these tickets change. Two specific hazards:

- **After ticket 04, spawning requires a registered agent.** The operator must
  register one in the cockpit before working ticket 05, or the map cannot be
  worked from the cockpit at all. Register before rebuilding, not after.
- **The running binary is rebuilt between tickets**, so each ticket's session is
  spawned by the *previous* ticket's code. Anything that breaks spawn breaks the
  next session's ability to start. The escape hatch is a terminal and `git`:
  the vanilla-wayfinder path (write `## Answer`, commit) finishes any ticket
  without the cockpit.

**Before commit:** run the CLAUDE.md gates — `go vet ./...` and `go test ./...`
(the embed test compiles against `web/dist/`), the frontend `check` / `build`
scripts and `vitest`, and confirm **no amber in the built CSS**. Review the diff
and drive the real behaviour where a "Done when" is only real at runtime — the
picker, the empty state and the ideate path are all runtime facts. No map linter
is wired in this repo.

**Frontend work obeys `docs/design-system.md` (ADR 0012).** Tokens for every
colour, a vendored primitive for every component, Phosphor icons, no chroma in
the chrome. Ticket 02 adds one primitive; it follows the *Adding a primitive*
procedure rather than hand-rolling a menu.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the spec — it does not open fog here. -->

## Out of scope

<!-- Inherited from the spec's Out of Scope; these never graduate into tickets on this map. -->

- **Per-role agent memory** — one remembered agent per space, deliberately; running a role on a different agent means picking it.
- **Editing or viewing the remembered agent as a setting** — it is state, not config; the moment it is editable it is a binding again.
- **Collapsing the four role buttons in the action bar** — the bar gets slightly more crowded and stays that way; deferred to separate work by the operator.
- **Curated per-CLI flag UI** — flags remain an opaque list the operator types; the command preview is the honest substitute (ADR 0002).
- **A shared or committed agent library** — it stays local and global; never-committed is a safety guarantee.
- **Migrating existing configuration** — there are no users, so old keys simply stop decoding: no warnings, no shims, no fallbacks.
- **Changing how payloads are composed** — the context bundle, skill library and role prompts are untouched.
