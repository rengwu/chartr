# chartr

A cockpit for driving wayfinder maps to completion: switch between project spaces, read a map as a star-map, and spawn agent sessions against its frontier.

## Language

### The map

**Space**:
A git repository chartr drives, registered once and switched between. It owns exactly one working tree, which is what makes it the unit of serialisation.
_Avoid_: project, workspace, repo, folder

**Map**:
A wayfinder effort — a `map.md` and its tickets under `.plan/maps/<slug>/`. Always either a planning map or an implementation map.
_Avoid_: plan, effort, graph, board

**Planning map**:
A map whose tickets resolve decisions. Worked live with a human.
_Avoid_: design map, wayfinder map, decision map

**Implementation map**:
A map whose tickets deliver code, with every decision already settled in a spec.
_Avoid_: impl map, task map, build map

**Ticket**:
One question or one unit of work in a map, sized to a single session. Its status is derived from its file, never stored in it.

**Frontier**:
The open, unblocked, unclaimed tickets of a map — the edge of the known. Wayfinder's own: a blocker counts as cleared the moment it is resolved.

### Sessions

**Session**:
A PTY running an agent CLI against exactly one ticket, wired by a pre-injected prompt.
_Avoid_: run, job, task, terminal

**Free session**:
An agent chartr launched into a space with no ticket and no role, started by picking a registered agent from the space card's `new shell` menu. It launches bare — nothing is injected and the operator types their first message themselves — except that a space with **prompt presets** selected at launch hands it those, and only those, through the ordinary read-this-file opener. It shares only the adapter's spawn primitive with a session: no map or ticket is looked up, no claim is written, no lifecycle derives for it, and it never counts toward the one-session-per-space gate. The tab is titled by the agent's registered name. The same control's body opens an **empty shell**, which is a plain shell with nothing injected at all.
_Avoid_: on-ramp, ideate, skill launch, ticketless session

**Role**:
What a session is spawned to do — grill, prototype, research, or implement. Follows from the ticket's own `type:` (`grilling`, `prototype`, `research`, `task`), which the spawn gate offers pre-selected while leaving all four to the operator. It is the key a **binding** names a skill under, it shapes the payload, and it is recorded on the claim; it does not itself select a skill, and it does not resolve to an agent.
_Avoid_: mode, kind, job type

**Agent**:
A registered, named, complete way to run a harness — an adapter, whatever flags it takes (its model among them), and how it receives its opening prompt. Chosen per spawn from the operator's library; it is the whole of what runs a session. Never committed, so a permission-skipping agent is something an operator grants themselves, not something a `git pull` can hand a teammate.
_Avoid_: binding, agent config, profile, preset

**Adapter**:
The per-agent shim that knows how to launch one agent CLI, inject its prompt and context, and observe it.
_Avoid_: driver, plugin, backend, integration

**Context bundle**:
The orientation injected into a session at spawn — the map body, the ticket, its blockers' answers, and the sources block (the enabled sources in resolution order, each with its path in the repo-local mirror and the skill names found there). Assembled fresh each time and never accumulated. It is the data half of a payload: it renders below the `# Context` rule, after every instruction.
_Avoid_: memory, prompt context, preamble

**Source**:
A place skills come from — a local folder or a pinned git checkout — that the operator registers by name in an ordered list under their config root. Position in the list *is* resolution order: a bare skill name takes the first hit among the enabled sources, and a `Source/skill` reference addresses one source exactly and never falls through. chartr ships none of its own (ADR 0018): every source is one the operator registered, and an empty list is the first-run state.
_Avoid_: layer, repo, provider, registry entry

**Binding**:
The line saying which skill a role spawns with — one `role = "Source/skill"` row in the `[roles]` table of the user config, always source-qualified and never a bare name. chartr seeds none: with no shipped skills there is nothing to bind to, so every role starts unbound and refuses to spawn until the operator binds it against a source they registered. A binding that resolves to nothing refuses the spawn outright, before any claim commit.
_Avoid_: mapping, assignment, role config

**Mirror**:
The repo-local copy of every enabled source's skills, written to `<space>/.chartr/skills/<source>/<skill>/` and reconciled in place before each session (regular files only, symlinks skipped), so an agent sandboxed to its own working tree can read the skills a payload names. Gitignored and per-machine, never committed — the same footing as `CHARTR.md`. The payload's sources block points at it with repo-relative paths.
_Avoid_: cache, workspace layer, checkout, index, materialized library

**Cockpit**:
chartr's interface — the star-map, the ticket pane, and the multiplexed terminals, nested under a space.
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

**Agent library**:
The operator's registered agents — the only execution config there is. Global and local: one set in the operator's own uncommitted file, shared by every space. An empty library is the starting state and refuses every spawn — a free session included — until one is registered.
_Avoid_: agent registry, profiles, presets

**Prompt preset**:
One short named behavioural instruction in the operator's own catalog — an id chartr derives once, a name, and the text an agent is told. Global and local, like the agent library: the same catalog in every space, with each registered space carrying only which of them it applies at launch, and each preset composing as its own operator prompt part after preferences. Editing one changes every space that selected it; deleting one removes it everywhere. It is not a skill and not a payload region: nothing looks it up by role, and nothing installs it into a harness. The **Prompts pane** is where it is written, selected, and — for the space's active Chartr-launched agent tab — sent or queued for that tab's next idle.
_Avoid_: system prompt, instruction set, profile, template, snippet

**Conventions**:
chartr's file-format contract: one generated `conventions.md` under the config root stating the fixed `.plan/maps/` layout, the permanent ticket numbering, `map.md`'s five sections, the recognized frontmatter, the structural headings, the derived-status table and the frontier. **Format only** — it never says how to interview, decide, decompose or behave; that is a skill's job. chartr owns the bytes: they are rewritten at startup and reconciled again before every composition, so it is the one thing in the config root an operator cannot shadow, disable or reorder away, and an edit lasts until the next compose. Discovery enforces the one rule it can — the fixed root — and the lint stays advisory.
_Avoid_: tracker convention, ruleset, format skill, template

**Preferences**:
The operator's own standing instructions, in `preferences.md` beside the conventions. chartr creates it empty once and never writes it again; its bytes are appended to every payload verbatim, unranked and unmerged. A contradictory preference can make an agent write a map chartr cannot read — accepted, because the file is the operator's control surface, not chartr's. An unreadable one fails composition rather than being silently dropped.
_Avoid_: user prompt, custom instructions, overrides, settings

**User config**:
The operator's local, uncommitted chartr config under the state root. It carries the agent library and the `[roles]` bindings, and is keyed to this machine, never a space's repository.
_Avoid_: local settings, overrides

**Settings surface**:
The global settings route: the agent library, the skill sources in resolution order, the four role bindings, and the paths of the files behind all of it, each openable in the operator's editor. Five things are edited inline — register a source, remove one, toggle it, reorder, refresh a git source — and everything else is read-value-plus-open-file, never a second config store. It is also where the free payload previews, and where a silently migrated source first becomes visible.
_Avoid_: settings screen, preferences, config panel, options

### Ticket lifecycle

**Implementing**:
The state of an implementation ticket while a session holds it.

**Resolved**:
A ticket whose `## Answer` is written — the session said so. Nothing blesses it; a dependent unblocks the moment it lands.
_Avoid_: done, complete, merged, closed, blessed, approved
