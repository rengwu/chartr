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
The orientation injected into a session at spawn — the map body, the ticket, its blockers' answers, and the sources block (the enabled sources in resolution order, each with its local path and the skill names found there). Assembled fresh each time and never accumulated. It is the data half of a payload: it renders below the `# Context` rule, after every instruction.
_Avoid_: memory, prompt context, preamble

**Source**:
A place skills come from — a local folder or a pinned git checkout — that the operator registers by name in an ordered list under their config root. Position in the list *is* resolution order: a bare skill name takes the first hit among the enabled sources, and a `Source/skill` reference addresses one source exactly and never falls through. The synthetic `chartr-skills` row sits last and cannot be removed or reordered.
_Avoid_: layer, repo, provider, registry entry

**Binding**:
The line saying which skill a role spawns with — one `role = "Source/skill"` row in the `[roles]` table of the user config, always source-qualified and never a bare name. Seeded once with four rows on a first startup and never auto-refilled: a row the operator deleted makes that role refuse until it is rebound. A binding that resolves to nothing refuses the spawn outright, before any claim commit.
_Avoid_: mapping, assignment, role config

**Seed**:
chartr's own skills, vendored into the binary as a checked-in copy of a pinned commit and materialized under the config root as the `chartr-skills` source, so a first run can spawn offline. The directory is in one of two states, read off the filesystem: no `.git` means chartr's bytes, reconciled against the compiled seed at every startup; a `.git` means the operator pinned it with a fetch and chartr never writes there again. Deleting the directory is the reset. Refresh with `make vendor-skills`; the shipped copy lives in `internal/sources/assets/chartr-skills/`.
_Avoid_: bundled skills, defaults, shipped library

**Skill library**:
The chartr-owned, hackable skills — the common core, one per role, the ideate on-ramp, the tracker convention, and the four method skills (`wayfinder`, `domain-modeling`, `to-spec`, `to-tickets`) — vendored from the wayfinder skills as standard `SKILL.md` directories and resolved through space → user → built-in layers at spawn by whole-skill shadowing. The method skills ship in the library but are never auto-composed into a session payload. Plain markdown on disk, editable by the operator and reusable in any agent CLI that reads the standard. The shipped copy lives in `internal/prompt/assets/skills/`; re-fitting upstream updates follows `docs/skill-sync.md`.
_Avoid_: prompt library, prompts, templates, system prompts

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
The operator's registered agents — the only execution config there is. Global and local: one set in the operator's own uncommitted file, shared by every space. An empty library is the starting state and refuses every spawn, ideate included, until one is registered.
_Avoid_: agent registry, profiles, presets

**Committed skills**:
The versioned skill overlays a space carries in `.chartr/skills/` — shared, portable, and winning over user skills for *content*. It is the only chartr config a space commits: there is no committed *execution* config, so nothing about how an agent runs is ever versioned into a repository.
_Avoid_: workspace config, project config, repo config

**Conventions**:
chartr's file-format contract: one generated `conventions.md` under the config root stating the fixed `.plan/maps/` layout, the permanent ticket numbering, `map.md`'s five sections, the recognized frontmatter, the structural headings, the derived-status table and the frontier. **Format only** — it never says how to interview, decide, decompose or behave; that is a skill's job. chartr owns the bytes: they are rewritten at startup and reconciled again before every composition, so it is the one thing in the config root an operator cannot shadow, disable or reorder away, and an edit lasts until the next compose. Discovery enforces the one rule it can — the fixed root — and the lint stays advisory.
_Avoid_: tracker convention, tracker adapter, ruleset, format skill, template

**Preferences**:
The operator's own standing instructions, in `preferences.md` beside the conventions. chartr creates it empty once and never writes it again; its bytes are appended to every payload verbatim, unranked and unmerged. A contradictory preference can make an agent write a map chartr cannot read — accepted, because the file is the operator's control surface, not chartr's. An unreadable one fails composition rather than being silently dropped.
_Avoid_: user prompt, custom instructions, overrides, settings

**User config**:
The operator's local, uncommitted chartr config under the state root. It carries the agent library and is keyed to this machine, never a space's repository.
_Avoid_: local settings, overrides

**Settings surface**:
The global settings route: the agent library and the paths of the files behind it, each openable in the operator's editor. Read-value-plus-open-file, never a second config store — there is nothing left to explain about layers.
_Avoid_: settings screen, preferences, config panel, options

### Ticket lifecycle

**Implementing**:
The state of an implementation ticket while a session holds it.

**Resolved**:
A ticket whose `## Answer` is written — the session said so. Nothing blesses it; a dependent unblocks the moment it lands.
_Avoid_: done, complete, merged, closed, blessed, approved
