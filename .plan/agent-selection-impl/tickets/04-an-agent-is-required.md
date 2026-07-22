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

