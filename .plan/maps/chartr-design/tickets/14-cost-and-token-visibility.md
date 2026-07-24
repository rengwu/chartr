---
type: grilling
blocked_by: [01]
---

# Cost and token visibility

## Question

Graduated from the **Cost and token visibility** fog patch once [the agent adapter
contract](./01-the-agent-adapter-contract.md) established what the CLIs actually report:
**token counts are universal** in every agent's JSON event stream; **native dollar cost** exists only
on claude (in-stream `total_cost_usd`) and opencode (`stats`, out of band), while codex and pi report
tokens only; **native budget caps** exist only on claude (`--max-budget-usd`, `--max-turns`).

So the raw material is settled — the open decision is what the cockpit *does* with it. Runaway cost
is a named risk of automating this at all (map Notes), and long-running sessions across several
spaces burn real money.

Decide:

1. **What the cockpit surfaces, and at what grain** — tokens always; dollars derived from tokens × a
   per-model price table chartr maintains (uniform across agents, rather than trusting each
   agent's native figure). Per session? Per map? Per space? A running total?
2. **Whether it caps or merely reports.** A chartr-enforced cap (watch stream tokens, call `stop()`
   at a threshold) works for every agent and doesn't depend on claude's native `--max-budget-usd`.
   Is a cap in scope, and is it a hard stop or a warn-and-continue? Who sets the threshold — per
   space, per role, global?
3. **Where the price table lives and how it stays current** — config the operator edits, since model
   pricing drifts and chartr is agent-agnostic.

Keep it a *cockpit, not an autopilot* (map Notes): the human should see spend accumulating and be
able to intervene; anything that must always hold (a hard ceiling, if there is one) belongs in
deterministic code, not an agent.

## Answer

**Cost and token visibility is declined, not designed — like [charting](./07-charting-a-new-map-from-a-space.md), this ticket resolves by *not building*.** The test that settles it: a cost readout earns its place only if seeing the number changes what the operator does, and today it cannot. It can't **stop** (a chartr cap left the design with [ticket 02](./02-knowing-a-session-finished-hung-or-died.md)), can't **warn** (no threshold earns its keep against a human who is already watching), and can't **price** (dollars need a per-model table chartr must keep current — pure carrying cost for a number nobody acts on). A readout that triggers no action is telemetry for its own sake.

**And it is redundant, not merely inert.** The **per-session** figure is already live in the agent's own TUI — every session runs the real interactive client in a PTY ([ticket 02](./02-knowing-a-session-finished-hung-or-died.md)) and the operator is looking straight at its token line; re-surfacing it, worse and after-the-fact, adds nothing. The **global** cross-session total — the one figure no single TUI owns, and the last thing worth being tempted by — is a *lagging abstraction of signals the cockpit already shows*: a space burning tokens is a space with a live session, which already reads as [ticket 11](./11-the-space-registry-and-switching.md)'s liveness dot and [ticket 09](./09-rendering-chartr-states-on-the-star-map.md)'s orbiting moon. Operators intervene on a session *behaving* wrong — spinning, quiet past threshold, looping — never on an integer they can't price. **Cost control is the human watching the cockpit** (map Notes, ratified by [ticket 02](./02-knowing-a-session-finished-hung-or-died.md)): the real mechanism is *attention*, and a number is a weaker second channel for a job the attention design already carries.

**The "build the plumbing now as substrate for later" temptation is refused as YAGNI.** Pre-building token-observation code for a deferred feature buys nothing: [ticket 01](./01-the-agent-adapter-contract.md) already *surveyed* the raw material, so the design re-enters from that survey cheaply, without parsing code sitting warm in chartr meanwhile. This ticket is a leaf with no edges — nothing depends on it (ticket 13 hangs on [12](./12-frontend-framework-and-build.md), not here) — and it touches none of the three things this design exists to make correct: orchestration that is reliable, reversible, and gated. Cost is orthogonal to all of it.

**The trigger is named so the deferral is a decision, not a shrug: [autopilot](./05-mapping-roles-to-agents.md).** It is the one state where "the human watching is the cost control" collapses — no watcher, so runaway spend needs a *machine* ceiling in deterministic code, exactly where the map says a must-always-hold belongs. Until autopilot exists (opt-in, local-only, non-default), cost visibility solves a problem the human's presence already solves. Explicit operator demand for money-legible spend is the softer secondary trigger. Either reopens this ticket; neither exists yet.

**Banked for re-entry, so reopening is a design pass and not a rediscovery.** Raw material ([ticket 01](./01-the-agent-adapter-contract.md)): token counts universal; native dollars only on claude (`total_cost_usd`) and opencode (`stats`); native caps only on claude (`--max-budget-usd`, `--max-turns`). Enforcement stance ([ticket 02](./02-knowing-a-session-finished-hung-or-died.md)): telemetry is best-effort, out-of-band, after-the-fact from per-adapter session files — no live stream, so no live chartr cap. And the one mechanism this grilling surfaced, worth not losing: **were a readout ever built, cost attributes by `(cwd, PTY-window)`, not the agent's session id** — chartr owns the PTY and its window, per-space serialisation ([ADR 0003](../../../docs/adr/0003-serialise-per-space-no-worktrees.md)) makes that window map to exactly one ticket, and summing every session file in it is what makes the figure robust to mid-session death, `/new` resets, respawns, and rejected-then-reimplemented attempts; dollars, if they return, derive uniformly from tokens × a hackable on-disk price table rather than trusting heterogeneous native figures. That is the design-in-waiting — not the build.
