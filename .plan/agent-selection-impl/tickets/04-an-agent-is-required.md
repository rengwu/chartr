---
type: task
blocked_by: [03]
---

# An agent is required; an empty library refuses every spawn

## Question

Close the door. Every path now supplies an agent explicitly (tickets 01–03), so
the parameter becomes **required** and the binding path becomes unreachable — the
contract step that makes ticket 05 a pure deletion. In the same slice, make the
state this creates survivable: a new operator with nothing registered must be
told, where they were blocked, how to get running.

**`agent` becomes required on every spawn path.** `handleSpawn`,
`handleIdeate` and `handleRespawn` refuse a request without one. With no agents
registered at all the refusal is specific and different from "unknown agent": it
says the library is empty and that registering one is the fix. There is no
default, no fallback, and no shipped agent — an empty library is the starting
state, exactly as ADR 0009 already claims it is.

**The empty state is surfaced where the operator hit the wall**, not only in the
settings route. The action bar and the ideate control render the **empty** state
from `agentchoice.ts` with a route to registration. Do not disable the control
silently: a dead button that explains nothing is the same opacity this effort
exists to remove.

**Registration suggests adapters that are actually on this machine.** Add a
server-side probe over a small curated list of known agent CLIs, reported to the
frontend and rendered by `web/src/lib/AgentLibrary.svelte` as **helper text
beneath the adapter input** — not as a placeholder, which vanishes on the first
keystroke, exactly when the list is most useful. With no detections the field
falls back to one generic example.

The curated list is the one place this effort brushes ADR 0002, so it stays on
the correct side of the line: the probe asserts only **that a binary exists on
PATH**. It is a hint and never a menu — any binary name can be registered, chartr
claims no knowledge of what any of them do, and **no per-CLI flag UI may be added
here**. `config.LookPath` is the existing probe; reuse it.

Done when: `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green with no amber in the built CSS; process-boundary tests show a
spawn, an ideate and a respawn each refused without an `agent`, and refused with
a distinct empty-library message when nothing is registered, in every case
**leaving the space untouched — no claim, no payload, no tab**; a test with stub
binaries on PATH shows the probe reporting exactly those it detects and reporting
nothing when none are present; and in the running cockpit a data root with no
registered agents shows the empty state on both the spawn control and the ideate
control, each routing to registration, where the adapter field names the CLIs
found on PATH and still accepts a binary that is not among them.

**Register an agent in the cockpit before rebuilding past this ticket** — after
it lands, this repo cannot spawn its own sessions without one (see the map's
self-hosting note).

## Answer

The door is closed: an explicit agent is required on every launch path, the
binding path is unreachable, and the state that creates is survivable. Four
parts, matching the question.

- **`agent` is required everywhere; `agentSpec` is the only resolver.** Ticket
  03's `agentSpec` was already the shared named-agent path, so ticket 04 finished
  the job by deleting the fallback: `launchSpecFor`, `bindingFor` and `specOf`
  are gone from `internal/server/spawn.go`, and `handleSpawn`, `handleIdeate`,
  `handleRespawn` and `handleResume` all settle through `agentSpec(res, name)`
  directly. Nothing reaches a role binding to launch, so ticket 05 is a pure
  deletion of the config layer rather than a rewrite. `launchSpec`'s
  `AdapterFrom`/`ArgsFrom` are now always empty (the explicit path consults no
  layers), so those provenance trailers are omitted and travel out with the
  binding layer in 05.

- **The empty library is a distinct refusal that names the fix.** `agentSpec`
  splits "no name given" in two: with agents registered it is a picker never
  opened (`400`, "an agent is required — pick one from your library"); with
  nothing registered at all it is the fresh operator's wall (`409`,
  `emptyLibraryMessage` — "no agents are registered — register one in settings
  before you can spawn"). A named-but-gone agent falls through to "no agent named
  X is registered" instead — even when it was the last one — because the name is
  what the operator needs to re-register, which keeps ticket 03's
  `TestHaltRespawnRefusesWhenTheAgentIsGone` honest. Every refusal lands before
  the claim, so a blocked launch leaves the space exactly as it was — no claim,
  no payload, no tab (asserted in the tests, and confirmed against the running
  binary).
  **Deviation, raised not quiet:** the question lists respawn among the surfaces
  that give the empty-library message, but respawn always *inherits* a name from
  the dead session, so it gives the more specific "which agent is gone" message;
  the empty-library message is a spawn/ideate concern, where the operator supplies
  (or fails to supply) the name directly. The spec does not fix the wording, so
  this is a reading, not a contradiction — but it is worth knowing.

- **The empty state is surfaced where the operator hit the wall, not just in
  settings.** `AgentSplitButton.svelte`'s `empty` case no longer falls through to
  the server: it labels itself "Register an agent", is never silently disabled,
  and routes via a new `onregister` callback threaded App → SpacePane / MapCard →
  DetailPane and both ideate controls, landing on the settings **user** scope
  where the library lives. The action bar's four role buttons and both ideate
  controls all render it; the "Next up" drawer already sends non-ready rows to the
  deliberate control (ticket 03), so it needs no empty-specific path.

- **Registration suggests adapters actually on PATH — a hint, never a menu.**
  `config.DetectAgents(onPath)` probes a small curated `knownAgentCLIs` list with
  the existing `LookPath`, in curated order, returning only those present. It
  rides the snapshot as `model.Detected` and renders in `AgentLibrary.svelte` as
  **helper text beneath the adapter input** — surviving the first keystroke,
  unlike a placeholder — with one generic example when none are found. This is the
  one place the effort brushes ADR 0002, and it stays on the correct side: the
  only fact asserted about any name is "this binary is on your PATH", any binary
  can still be registered whether or not it appears, and no per-CLI flag UI is
  built on the list.

**Testing.** `mustSpawn` now registers a default `claude` agent (reproducing the
old built-in default of `claude --model sonnet`) and spawns with it, so the many
role/lifecycle tests are unchanged; the binding-drives-spawn tests became explicit
`SpawnWithAgent`; the binding-fallback and binding-to-absent tests are deleted;
and new process-boundary tests cover spawn and ideate refused without an agent and
against an empty library (each leaving the space untouched), plus a hermetic
`DetectAgents` unit test with an injected probe. All gates are green (`go vet`,
`go test ./...` including the embed test, frontend `check` / `build` / `vitest`,
no amber), and the three runtime facts — the empty state on the action bar and
both ideate controls, the route to registration, and the PATH-hint adapter field
naming `claude, opencode` — were driven in the running cockpit.

**ADRs.** ADR 0002 is upheld (the probe asserts only existence). ADR 0009's "an
empty library is the starting state" is now true in fact, not just in claim: with
no `builtins` reached at spawn, nothing runs that the operator did not register.
Ticket 05 next deletes the binding layer the launch path no longer touches.

