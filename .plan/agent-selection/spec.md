# agent selection — pick the agent, not the binding (spec)

_Synthesised from a design conversation, not a planning map. It graduates to one
implementation map. It describes chartr **after** the cut; several CONTEXT.md
terms narrow here (see Implementation Decisions → Vocabulary), and two ADRs
change standing._

## Problem Statement

The operator cannot see what a session is about to run, and the machinery that
decides it is far larger than the decision.

- **There is an invisible default nobody registered.** With no agents and no
  config at all, spawning still launches `claude`. It comes from a hardcoded
  `builtins` table that is the bottom of a three-layer merge. Nothing in the
  agent library mentions it — the library says "No agents registered" while one
  is running. ADR 0009's own amendment states *"there is no shipped agent — an
  empty library is the starting state"*, which the code contradicts.
- **The spawn moment shows nothing.** The action bar offers `Start Implement`.
  No agent name, no adapter, no flags. The operator clicks and something starts.
  The payload preview — the one pre-spawn transparency surface — shows the role
  and the composed context, and is likewise silent about what will execute it.
- **The ideate on-ramp silently borrows the `grill` binding.** Ideate is not a
  role, appears nowhere in the settings surface, and its choice of agent is
  documented only in a Go comment.
- **The configuration is out of proportion to the question.** Answering "which
  CLI runs this?" costs three layers, four fields, per-field provenance, a
  committed workspace file, a per-space user table, field-level TOML line
  surgery that preserves comments, and warnings for conflicts between the layers
  — roughly 870 lines across two files, plus a settings surface built to render
  the provenance.
- **The bindings predate the thing that replaced them.** The agent library was
  added later, as *"an addition, never a migration"*. It is now the unit that
  actually carries meaning — an agent is a complete launch spec — while role
  bindings survive underneath it as a parallel, field-merged path that can
  disagree with it and must be reconciled with warnings.

## Solution

Delete the binding layer entirely and make the agent an explicit choice at the
moment of spawning, in five moves that ship as one sequenced effort:

1. **The agent library becomes the only execution config.** Role bindings, the
   committed workspace config file, the three-layer merge, and per-field
   provenance are deleted outright. An agent is registered once, globally, and
   never committed.

2. **No registered agents means no spawning.** Every spawn path — including
   ideate — refuses when the library is empty, and says so where the operator
   hit the wall, with a route to registering one. The invisible default is gone
   and nothing takes its place.

3. **The agent is chosen at spawn, freely, every time.** No role is bound to an
   agent. The first spawn in a space opens a picker; every spawn after it reuses
   what that space last used. That memory is **state, not config** — silent,
   uneditable, and overridable in one click.

4. **Every spawn surface names the agent it will run.** The action bar, the
   "Next up" drawer, and the payload preview all show it before anything starts.

5. **Ideate stops being a special case.** It is a session with no ticket. It
   picks an agent exactly as everything else does.

## User Stories

**Registering agents**

1. As an operator with a fresh install, I want the cockpit to tell me no agent is
   registered rather than quietly launching one, so that nothing runs on my
   machine that I did not choose.
2. As an operator with a fresh install, I want to be pointed at agent
   registration from the place I was blocked, so that I do not have to guess that
   the answer lives in settings.
3. As an operator registering my first agent, I want the adapter field to suggest
   CLIs that are actually on my PATH, so that I do not have to remember exact
   binary names.
4. As an operator whose PATH has no recognised CLI, I want a sensible example in
   the field anyway, so that I can see the shape of what is wanted.
5. As an operator, I want those suggestions to be hints and never a menu, so that
   I can register a harness chartr has never heard of.
6. As an operator, I want the suggestion to survive while I am typing, so that I
   can read it and type at the same time.
7. As an operator, I want to register several agents that differ only in flags,
   so that "the sandboxed one" and "the fast one" are distinct named things.
8. As an operator, I want to see the exact command each registered agent will
   run, so that I never have to reason about how my flags are assembled.
9. As an operator, I want an agent whose binary is missing to be badged in the
   library, so that I know it before I try to use it.
10. As an operator, I want to delete an agent without it silently changing what
    any space runs, so that a library edit is never an ambient behaviour change.

**Choosing an agent at spawn**

11. As an operator spawning in a space for the first time, I want to pick which
    agent runs it, so that the choice is mine and is made knowingly.
12. As an operator, I want that choice remembered for this space, so that routine
    spawning stays one click.
13. As an operator, I want the remembered agent to be per space, so that the
    permissive agent I use in a scratch repo never leaks into a real one.
14. As an operator, I want to override the remembered agent for a single spawn
    without changing anything durable, so that a one-off stays a one-off.
15. As an operator, I want an override to become the new remembered agent for
    that space, so that switching does not need a second confirming action.
16. As an operator, I want the picker to show every registered agent, so that the
    full set of choices is one click away.
17. As an operator, I want an agent whose binary is missing to be unselectable in
    the picker, with the reason visible on the row, so that I learn it before
    committing to a choice.
18. As an operator, I want a spawn to still be refused if the binary vanished
    after the picker rendered, so that a stale list can never launch nothing.
19. As an operator whose remembered agent has since been deleted, I want the
    picker to open again rather than fall back to something, so that the gap is
    surfaced and not filled.
20. As an operator, I want the remembered agent to survive restarting the
    cockpit, so that it is a property of the space and not of the tab.

**Seeing what will run**

21. As an operator, I want the spawn button to name the agent it will use, so
    that the transparency is at the moment of action rather than in a settings
    screen.
22. As an operator using the "Next up" drawer, I want each row to name the agent
    it would spawn with, so that the fast path is quick without being opaque.
23. As an operator in the "Next up" drawer with nothing remembered for this
    space, I want the row to take me somewhere I can choose, rather than spawn,
    so that no one-click path ever bypasses the first choice.
24. As an operator, I want the payload preview to name the agent alongside the
    composed context, so that the pre-spawn window answers *what runs this*, not
    only *what does it read*.
25. As an operator, I want the live session tab to keep showing which agent it
    is, so that a running session is identifiable after the fact.

**Ideate**

26. As an operator, I want ideate to require a registered agent like everything
    else, so that there is no path that launches something unchosen.
27. As an operator, I want to choose ideate's agent, so that the on-ramp is not
    silently tied to a role I did not associate with it.
28. As an operator, I want ideate to reuse the space's remembered agent, so that
    it behaves like every other spawn.
29. As an operator, I want ideate's behaviour described in the interface, so that
    I understand it opens a ticketless agent tab with a starter prompt and
    nothing is claimed or committed.

**Audit and recovery**

30. As an operator reading a claim commit, I want the registered agent's name
    recorded, so that I can see which of my agents was chosen.
31. As an operator reading a claim commit on another machine, I want the adapter
    and args recorded too, so that the trailer means something where my local
    names do not exist.
32. As an operator respawning a dead session, I want the same agent the dead
    session used, so that "start over cleanly" changes the payload and not the
    execution.
33. As an operator, I want a refused spawn to leave the space exactly as it was,
    so that a blocked launch never half-claims a ticket.

**The simplification itself**

34. As an operator, I want role bindings gone entirely, so that there is one
    place that decides execution instead of two that can disagree.
35. As an operator, I want no per-space configuration file, so that a space is a
    git repository and not also a config surface.
36. As an operator, I want a repository containing an old `.chartr/config.toml`
    to resolve cleanly and ignore it, so that a stale file is inert rather than
    an error.
37. As an operator, I want the settings surface to be the agent library and the
    file paths behind it, so that there is nothing left to explain about layers.
38. As an operator, I want warnings that remain to be about live problems — an
    agent with no adapter, an unreadable prompt delivery — so that the warning
    channel means something.
39. As a teammate cloning the repository, I want nothing about execution to be
    committed, so that a `git pull` can never hand me a permission-skipping
    agent.
40. As an operator, I want the model to remain a flag I type, so that chartr
    continues to know nothing about what any CLI's flags mean.

## Implementation Decisions

### What is deleted

- **The `builtins` binding table.** No shipped agent. An empty library is the
  starting state, as ADR 0009 already says it should be.
- **Role bindings, entirely.** The `Binding` and `Resolved` types, the
  `[spaces."<path>".roles.<role>]` table, and the `adapter` / `args` / `prompt` /
  `agent` fields on a role. A role no longer resolves to anything executable.
- **The layer machinery.** `Layer`, `LayerBuiltin` / `LayerWorkspace` /
  `LayerUser`, the `*From` provenance fields, and the field-by-field merge.
- **The committed workspace config file.** `.chartr/config.toml` is read in
  exactly three places, all for role bindings; the skill library resolves from a
  *directory*, not that file. The constant, the reads, and the concept go. A file
  left over in a space is not read, not warned about, and not deleted.
- **The user-layer binding writer.** The comment-preserving TOML key surgery
  exists to edit one field of one role table in one space. With no role tables,
  it has nothing to edit.
- **The migration path.** There are no users. Old keys simply do not decode:
  no warnings, no fallbacks, no compatibility shims. `model` on an agent is
  likewise no longer decoded to be warned about — it is just an unknown key.

### What survives

- **The agent library**, unchanged in shape and scope: `[agents.<name>]` =
  `{adapter, args?, prompt?}`, global, in the operator's local config, never
  committed. Adapter is the only required field. Flags stay an opaque list.
- **Agent-library validation** — a name the library can hold, an adapter to
  launch, a prompt delivery the adapter seam can read. These describe live
  config, not dead config, and keep their warnings.
- **The four roles**, in their remaining jobs: choosing which skill composes the
  payload, deriving the default role from a ticket's type, and the AFK/HITL split
  that governs the quiet hint. A role no longer implies an agent.
- **The adapter seam**, which continues to model exactly one thing about a CLI:
  prompt delivery.

### Choosing and remembering

- **Selection happens at spawn and is never stored as config.** The spawn request
  carries an agent name alongside the role. The server resolves that name against
  the library, refuses an unknown or PATH-absent one, and launches it.
- **The remembered agent lives on the space registry entry**, beside the
  `last_active` timestamp it already carries. This is the decisive point: the
  registry is chartr-written per-space *state* in the data root, already exactly
  the right shape and lifecycle. No new store, no new file, no new seam.
- **One remembered agent per space**, not per role. A role switch does not change
  which agent runs.
- **There is no automatic first choice.** The rule does not vary with library
  size: nothing remembered means the picker opens, whether one agent is
  registered or ten. A rule that changes behaviour when a second agent is
  registered would be a surprise with no visible cause.
- **A remembered name that no longer resolves is treated as nothing remembered.**
  The picker opens; no substitute is chosen.
- **A successful spawn writes the agent it used.** An override is therefore
  self-persisting, and needs no separate confirming action.

### Spawn surfaces

- **The action bar becomes a split control**: the primary action spawns with the
  space's remembered agent and names it; the secondary opens the agent list. With
  nothing remembered, the primary action opens the list rather than spawning.
- **The "Next up" drawer** names the remembered agent on each row and spawns on
  click. With nothing remembered, a row selects its ticket and surfaces the
  deliberate spawn control instead of spawning — so there is exactly one picker
  implementation and no one-click path that skips the first choice.
- **The payload preview** gains the resolved agent and its command line beside
  the composed context.
- **Which agent a space will spawn with** is a pure function of the library and
  the remembered name, lifted into a testable frontend module rather than living
  inside a component. It answers three cases: an agent, nothing remembered, and
  an empty library.

### Onboarding

- **PATH detection is advisory and additive.** The server probes a small curated
  list of known agent CLIs and reports which are present. The list is used to
  *suggest*, never to constrain: any binary name can be registered, and chartr
  claims no knowledge of what any of them do or what flags they take. This keeps
  ADR 0002 intact — the fact asserted is "this binary exists on your PATH",
  which is not agent-specific knowledge.
- **Suggestions render as helper text beneath the adapter input, not as a
  placeholder.** A placeholder disappears on the first keystroke, which is
  exactly when the list is most useful. With no detections, the field shows a
  single generic example instead.
- **The empty-library state is surfaced where the operator is blocked** — the
  spawn control and the ideate control — with a route to registration, not only
  in the settings surface.

### Ideate

- Ideate keeps its own entry point and its own starter prompt; it remains the
  one opinionated nudge toward charting, and remains a tab with no session
  attached — nothing claimed, nothing committed, no death halt.
- It stops borrowing the `grill` binding. It takes an agent the same way a
  session does: the space's remembered agent, or the picker.
- Its behaviour is described in the interface rather than only in source
  comments.

### Audit trail

- The claim commit records **both** the registered agent's name and the adapter
  plus args that ran. The name captures what the operator chose; the adapter and
  args are what the trailer means to anyone reading it on another machine, where
  local agent names do not exist.
- The provenance trailers (`AdapterFrom`, `ArgsFrom`) are removed — they named
  config layers that no longer exist.

### Respawn

Respawn reuses the agent recorded on the dead session rather than re-resolving
or re-picking. "Start over cleanly" composes a fresh payload and writes a fresh
claim; it does not change what executes.

### ADR consequences

- **ADR 0009** (config layers) is superseded in the part that matters: there is
  no longer a committed execution layer, and therefore no layering question to
  answer for bindings. Its content-versus-execution rule survives only on the
  content side (skills), which is untouched. Its safety property strengthens —
  with no committed execution config at all, nothing about how an agent runs can
  arrive by `git pull`.
- **ADR 0014** (the effective config surface) is superseded. It is built on
  per-field provenance across layers, which ceases to exist. What replaces it is
  smaller and needs no ADR: the agent library, plus the paths of the files
  behind it.
- **ADR 0002** (agent-agnostic adapters) is upheld and load-bearing. PATH
  detection is the one place this effort brushes against it, and stays on the
  correct side of the line.

### Vocabulary

`CONTEXT.md` changes in two entries:

- **Role** — drop *"Resolves through config to a concrete agent command."* A role
  selects a skill and a payload; it does not resolve to an agent.
- **Agent** — currently listed only as a term to *avoid* (under Session). It
  becomes a first-class entry: a registered, named, complete way to run a
  harness — adapter, flags, prompt delivery — chosen per spawn.

## Testing Decisions

**What makes a good test here.** Only externally observable behaviour: HTTP
responses, the pushed model, files on disk, git history, and the argv a stub
agent actually received. No test reaches into resolution internals — most of the
deleted code was tested that way, which is part of why it was able to grow.

**Seam 1 — the process boundary (`internal/chartrtest`).** The established seam,
and the one every server test already uses. Its rig doc is explicit: *"the one
seam is the process boundary."* It already supports stub agents on PATH, which is
what makes flag-verbatim assertions possible. Everything in this effort is
observable there:

- an empty library refuses every spawn, including ideate, with a specific message
- registering an agent unblocks spawning
- a spawn names its agent; the launched process receives that agent's flags
  verbatim and in order
- the remembered agent appears in the pushed model after a spawn, and survives a
  restart of the server against the same data root
- a spawn naming an unregistered or PATH-absent agent is refused, and refuses
  without writing a claim
- a remembered agent that has been deleted resolves to "nothing remembered"
- respawn launches the dead session's agent
- claim trailers carry the agent name, the adapter, and the args, and carry no
  provenance
- a space containing a leftover `.chartr/config.toml` resolves clean
- PATH detection reports stub binaries placed on PATH

Prior art to follow directly: `internal/server/spawn_test.go` for the claim and
argv assertions, `internal/server/agents_test.go` for library behaviour and
verbatim flag delivery, `internal/server/ideate_test.go` for the ticketless tab.

**Seam 2 — pure frontend modules (`web/src/lib/*.test.ts`).** The repository has
no Svelte component testing and this effort proposes none. The rule instead: any
selection logic worth asserting is lifted into a pure module and tested there.
Prior art: `args.ts` / `args.test.ts`, `attention.ts`, `model.ts`.

**Tests that are deleted, not rewritten.** `internal/config/binding_test.go` and
`internal/config/userbinding_test.go` test layer merging, per-field provenance,
and comment-preserving TOML surgery — behaviour that ceases to exist.
`internal/server/configsurface_test.go` shrinks to the agent library and its file
paths. `internal/config/agents_test.go` stays and grows.

## Out of Scope

- **Per-role agent memory** — one remembered agent per space, deliberately. The
  cost is that a role switch does not change agents; running one role on a
  different model means picking it.
- **Editing or viewing the remembered agent as a setting** — it is state. The
  moment it becomes editable it is config again, which is the thing being
  deleted.
- **Collapsing the four role buttons in the action bar** — the bar is crowded and
  will get slightly more so. Acknowledged and deferred to separate work.
- **Curated per-CLI flag UI** — flags remain an opaque list the operator types.
  The command preview is the honest substitute.
- **A shared or committed agent library** — the library stays local and global.
  Its never-committed property is a safety guarantee, not an oversight.
- **Migrating existing configuration** — there are no users.
- **Changing how payloads are composed** — the context bundle, the skill library,
  and the role prompts are untouched.

## Further Notes

**The accepted cost.** The deleted `builtins` table encoded a real preference:
`grill` ran opus, the rest sonnet, because a live dialogue and an unattended
grind want different things. Free selection means that preference becomes the
operator's to express — two registered agents and a choice — rather than a
default they never saw. This is knowingly traded for having no unchosen default
at all.

**The revisit trigger.** Per-space last-used memory is, structurally, a binding
with a better name. The distinction holds only while it stays invisible and
uneditable. The first request to *edit* the remembered agent, or to see it in the
settings surface, means it has become config — and at that point it should either
be given a proper home or refused on purpose. Not resolved silently.
