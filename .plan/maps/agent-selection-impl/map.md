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

- **01 — spawn takes an explicit agent**: the spawn request gains an optional `agent`, resolved against the operator's library off the `Resolution` that already carries it — unregistered is `400`, registered-but-off-PATH is `409` with the library's own message, both on the same doorstep and in the same order as the binding case, before the claim and before any write. Both paths converge on one `launchSpec` (`{Name, Adapter, Args, Prompt}` + provenance) *before* `launchSession`, so the launch mechanics no longer know bindings exist and ticket 05 is a subtraction. The claim trailer now carries the registered **name** as `Agent:` (omitted, never blank, when none was chosen) beside a new `Adapter:` for the binary, since a local name means nothing on a teammate's machine; the `*-From:` provenance is written only when a binding decided. The space remembers a successful spawn's agent as `LastAgent` on the existing `registry.Entry` — state with exactly the right lifecycle, no new store — surfaced on the pushed model as-is, with an unresolvable name deliberately left for the frontend to read as nothing remembered. [ticket](tickets/01-spawn-takes-an-explicit-agent.md)
- **02 — the action bar picks the agent**: the detail pane's spawn buttons become split controls. `web/src/lib/agentchoice.ts` is the testable pure function that decides `ready`/`unchosen`/`empty` from the registered library and the space's `lastAgent`; there is no automatic first choice, and a stale or missing-binary remembered name is treated as unchosen. The primary action spawns with the remembered agent and names it on the button; with nothing remembered it opens the picker instead. The secondary caret opens the picker for a one-off override — every registered agent is listed, absent-binary rows are disabled with their reason visible, and a selection spawns with that agent and becomes the space's new remembered choice. The frontend now sends an explicit `agent` on every spawn (empty string in the still-deferred empty-library case). A new `dropdown-menu` primitive was vendored through shadcn-svelte, its lucide icons swapped for Phosphor, and `@lucide/svelte` removed. [ticket](tickets/02-the-action-bar-picks-the-agent.md)
- **04 — an agent is required**: `agent` is required on every launch path — `handleSpawn`, `handleIdeate`, `handleRespawn` and `handleResume` settle through `agentSpec` alone, and `launchSpecFor`/`bindingFor`/`specOf` are deleted, so nothing reaches a role binding to launch (ticket 05 is now a pure deletion of the config layer). A spawn that names nothing is refused: with agents registered, `400` "an agent is required — pick one"; with **nothing** registered, a *distinct* `409` `emptyLibraryMessage` that names the fix — while a named-but-gone agent (respawn) still gets the specific "which agent is gone" `400`, kept deliberately. The frontend renders `agentchoice.ts`'s `empty` on the action bar and both ideate controls as "Register an agent", routed to the settings user scope via a new `onregister` threaded App → SpacePane/MapCard → DetailPane (never a silently-dead button). `config.DetectAgents` probes a curated `knownAgentCLIs` list via `LookPath`, carried on the snapshot as `model.Detected` and rendered as helper text beneath the adapter input (generic example when none), asserting only PATH existence — ADR 0002 upheld, ADR 0009's empty-library-is-the-start now true in fact. **Deviation, raised not quiet:** respawn gives the specific gone-agent message rather than the empty-library one, since it inherits its name. [ticket](tickets/04-an-agent-is-required.md)
- **05 — delete the binding layer**: role bindings, the committed `.chartr/config.toml` execution layer, and the config surface built to render them are gone — execution is the agent library and nothing else. `internal/config` is reduced to roles + `LookPath` + the agent library (`Resolution` trimmed to `{Agents, Warnings}`, the retired `Model` key now just an ignored unknown key); `s.resolve`/`deriveSpace`/`spaceLayers` stop reading any committed config; `handleSetBinding` and the claim's `*-From:` trailers are deleted. `Settings.svelte` collapses to the agent library + file-path open-hatches (skills stay on the wire but their provenance rows leave the surface, per the operator's call). ADR 0014 is superseded outright and ADR 0009's execution half with it (content half and its now-stronger safety property stand); `CONTEXT.md` promotes **Agent** to first-class and drops the binding vocabulary. **Deviation, raised not quiet:** `userbinding.go` is deleted as instructed, but the generic TOML line surgery it held is shared by the kept agent-library writer, so it was relocated to `tomlsurgery.go` rather than destroyed. [ticket](tickets/05-delete-the-binding-layer.md)
- **03 — every path carries the agent**: ticket 01's `launchSpecFor` grew `agentSpec` — the whole named-agent path (unregistered `400`, off-PATH `409` with the library's message, no name `400`) in one function every surface calls, so the refusals cannot drift apart. Ideate takes an explicit `agent` and the `grill` binding indirection is gone, with what ideate opens now said in its picker rather than in a Go comment. `terminal.Session` gained `AgentName` beside the adapter it already held, and respawn relaunches *that* rather than re-resolving a role — refusing, never substituting, when it has been deregistered or has fallen off PATH. The "Next up" drawer names the agent on each row and, with nothing remembered, selects its ticket instead of spawning, so no one-click path skips the first choice. The payload preview answers *what runs this* using the library's own `command`, built by the seam that builds the real argv. Ticket 02's inline split control was lifted into one `AgentSplitButton.svelte` used by the action bar and both ideate controls, making "exactly one picker" true of the codebase. **Deviation, raised not quiet:** resume was a fifth binding caller and takes the same treatment — ticket 04 should close it alongside spawn, ideate and respawn. [ticket](tickets/03-every-path-carries-the-agent.md)

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
