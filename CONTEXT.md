# wayfinder-harness

A cockpit for driving wayfinder maps to completion: switch between project spaces, read a map as a star-map, and spawn agent sessions against its frontier — with implementation work gated behind review.

## Language

### The map

**Space**:
A git repository the harness drives, registered once and switched between. It owns exactly one working tree, which is what makes it the unit of serialisation.
_Avoid_: project, workspace, repo, folder

**Map**:
A wayfinder effort — a `map.md` and its tickets under `.plan/<slug>/`. Always either a planning map or an implementation map.
_Avoid_: plan, effort, graph, board

**Planning map**:
A map whose tickets resolve decisions. Worked live with a human, and subject to no review apparatus.
_Avoid_: design map, wayfinder map, decision map

**Implementation map**:
A map whose tickets deliver code, with every decision already settled in a spec. Its tickets pass through review before they resolve.
_Avoid_: impl map, task map, build map

**Kind**:
Whether a map is a planning map or an implementation map — the property that decides which lifecycle its tickets follow. A property of the *map*, not the ticket: every ticket inherits the map's one lifecycle. Declared explicitly in committed harness config, never inferred from the map's contents; an undeclared map is inert until a human classifies it.
_Avoid_: type, mode, flavour, class

**Ticket**:
One question or one unit of work in a map, sized to a single session. Its status is derived from its file, never stored in it.

**Frontier**:
The open, unblocked, unclaimed tickets of a map — the edge of the known. The harness's frontier is stricter than wayfinder's: a blocker must be resolved *and* human-approved.

### Sessions

**Session**:
A PTY running an agent CLI against exactly one ticket, wired by a pre-injected prompt.
_Avoid_: run, job, task, agent, terminal

**Role**:
What a session is spawned to do — grill, prototype, research, implement, or review. Resolves through config to a concrete agent command.
_Avoid_: mode, kind, job type

**Adapter**:
The per-agent shim that knows how to launch one agent CLI, inject its prompt and context, and observe it.
_Avoid_: driver, plugin, backend, integration

**Context bundle**:
The orientation injected into a session at spawn — the map body, the ticket, its blockers' answers, this glossary. Assembled fresh each time and never accumulated.
_Avoid_: memory, prompt context, preamble

**Cockpit**:
The harness's interface — the star-map, the ticket pane, and the multiplexed terminals, nested under a space.
_Avoid_: dashboard, IDE, console, GUI

### Ticket lifecycle

**Implementing**:
The state of an implementation ticket while a session holds it.

**Proposed**:
An implementation ticket whose session has committed its work and written a `## Proposed Answer`, but which no gate has yet blessed. Not resolved.
_Avoid_: pending, staged, candidate, done, complete

**Agent review**:
An adversarial session, on a different model than the implementer, critiquing a proposed ticket against its Done-when and its spec.
_Avoid_: QA, lint, check, verification

**Human review**:
The hub where a human reads the diff and any agent-review verdict, then approves, takes it further, or abandons it.
_Avoid_: gate, approval, sign-off

**Resolved**:
A ticket whose `## Answer` is written. On disk, resolved always means blessed.
_Avoid_: done, complete, merged, closed

**Abandon**:
The human-review outcome that rejects a proposed ticket: the harness demotes its `## Proposed Answer` to `### Rejected` prose (keeping the record for the next attempt) and the ticket returns to the frontier. Undoing the rejected commits is the human's act, with revert offered as a lever.
_Avoid_: discard, reject, rollback

**Autopilot**:
The opt-in, non-default configuration in which both reviews are disabled and a ticket resolves with no human in the loop.
_Avoid_: auto, unattended, headless, YOLO
