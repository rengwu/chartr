# chartr

A cockpit for driving wayfinder maps to completion: switch between project spaces, read a map as a star-map, and spawn agent sessions against its frontier.

## Language

### The map

**Space**:
A git repository the chartr drives, registered once and switched between. It owns exactly one working tree, which is what makes it the unit of serialisation.
_Avoid_: project, workspace, repo, folder

**Map**:
A wayfinder effort — a `map.md` and its tickets under `.plan/<slug>/`. Always either a planning map or an implementation map.
_Avoid_: plan, effort, graph, board

**Planning map**:
A map whose tickets resolve decisions. Worked live with a human.
_Avoid_: design map, wayfinder map, decision map

**Implementation map**:
A map whose tickets deliver code, with every decision already settled in a spec.
_Avoid_: impl map, task map, build map

**Kind**:
Whether a map is a planning map or an implementation map — the property that decides which roles its sessions may be spawned as. Both kinds share one lifecycle. A property of the *map*, not the ticket. Declared explicitly in committed chartr config, never inferred from the map's contents; an undeclared map is inert until a human classifies it.
_Avoid_: type, mode, flavour, class

**Ticket**:
One question or one unit of work in a map, sized to a single session. Its status is derived from its file, never stored in it.

**Frontier**:
The open, unblocked, unclaimed tickets of a map — the edge of the known. Wayfinder's own: a blocker counts as cleared the moment it is resolved.

### Sessions

**Session**:
A PTY running an agent CLI against exactly one ticket, wired by a pre-injected prompt.
_Avoid_: run, job, task, agent, terminal

**Role**:
What a session is spawned to do — grill, prototype, research, or implement. Resolves through config to a concrete agent command.
_Avoid_: mode, kind, job type

**Adapter**:
The per-agent shim that knows how to launch one agent CLI, inject its prompt and context, and observe it.
_Avoid_: driver, plugin, backend, integration

**Context bundle**:
The orientation injected into a session at spawn — the map body, the ticket, its blockers' answers, this glossary. Assembled fresh each time and never accumulated.
_Avoid_: memory, prompt context, preamble

**Skill library**:
The chartr-owned, hackable skills — one per role, plus the common core, the ideate on-ramp, and the tracker convention — vendored from the wayfinder skills as standard `SKILL.md` directories and resolved through space → user → built-in layers at spawn by whole-skill shadowing. Plain markdown on disk, editable by the operator and reusable in any agent CLI that reads the standard.
_Avoid_: prompt library, prompts, templates, system prompts

**Cockpit**:
The chartr's interface — the star-map, the ticket pane, and the multiplexed terminals, nested under a space.
_Avoid_: dashboard, IDE, console, GUI

### The frontend

**Chrome**:
The Svelte-rendered UI around the islands — sidebar, tabs, queue, brief, panes — reacting to the pushed model.
_Avoid_: shell, layout, wrapper

**Island**:
An imperative surface the chrome hosts but never reaches inside: an xterm.js terminal, or the star-map's canvas renderer behind its narrow seam (mount, receive model, emit selection).
_Avoid_: component, widget, embed

**Control socket**:
The one JSON websocket per browser carrying the derived model downstream — server-authoritative, whole-snapshot on every change, resent on reconnect.
_Avoid_: state socket, event bus, sync channel

**Terminal socket**:
The binary websocket per attached terminal — raw PTY bytes down, keystrokes up, buffered scrollback replayed on attach.
_Avoid_: pty stream, data channel

### Configuration

**Role binding**:
What a role resolves to — an `{adapter, model, args?}` triple. Structured so the chartr can reason about it (compare models, probe the binary); the `args` hatch reaches flags the adapter doesn't model, forfeiting that introspection. Resolved by merging workspace and user config; the *effective* binding is what actually runs.
_Avoid_: mapping, agent config, role config

**Workspace config**:
The committed, shared chartr config in a space's repo — map kinds (ADR 0007) and role bindings — versioned and portable. Wins over user config for *content* (skills); yields to it for *execution* (bindings).
_Avoid_: project config, repo config, settings

**User config**:
The operator's local, uncommitted chartr config under `~/.config/chartr/`, keyed by space. Overrides workspace bindings for execution choices.
_Avoid_: local settings, preferences, overrides

**Effective config surface**:
The global settings route showing every value the three layers resolve, with the layer it came from and the file that layer lives in. Edits exactly one thing — a role binding, into the user layer; everything else is read-value-plus-open-file. Never a second config store.
_Avoid_: settings screen, preferences, config panel, options

### Ticket lifecycle

**Implementing**:
The state of an implementation ticket while a session holds it.

**Resolved**:
A ticket whose `## Answer` is written — the session said so. Nothing blesses it; a dependent unblocks the moment it lands.
_Avoid_: done, complete, merged, closed, blessed, approved
