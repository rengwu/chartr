# Space prompt presets

## Problem

Operators repeatedly give agents the same short behavioral instructions: keep
answers brief, use a repository's commit convention, stay on the main branch, or
choose the cheapest maintainable implementation. These are not skills. They are
small, reusable prompt fragments that should be owned once by Chartr and work
across every harness Chartr launches.

The existing `preferences.md` is global and always present in ticket payloads. It
cannot express a reusable library or a different selection per space. Provider
skill registries would duplicate the same text across harnesses and make Chartr's
agent-agnostic boundary the operator's synchronization problem.

## Solution

Chartr will own one ordered global prompt catalog in `prompts.toml`. Each preset
has a stable lower-kebab-case id, a human name, and non-empty prompt text. The
Prompts pane creates, edits, and deletes catalog entries; creation order is display
order. There is no separate folder format, frontmatter, grouping, or reorder UI.

Each registered space stores the ids selected for `At launch` in the existing
per-machine space registry. The selection is a set presented and composed in
catalog order. Editing a preset changes what future launches receive everywhere
it is selected. Deleting one removes it from every space's launch selection.
Scratch is unchanged in this first version.

For a ticket session, the selected presets are separate operator-origin prompt
parts in the existing payload, after global preferences and before context. They
therefore appear in payload preview and are covered by the existing payload hash.
For a free session, a non-empty selection creates a small run payload containing
the selected presets and delivers the existing read-this-file opener through the
chosen adapter. A free session with no selected presets remains a bare launch.
Changing `At launch` affects only later launches and never attempts to change or
retract instructions in an existing session.

The Prompts pane always targets the currently active tab; choosing another target
means selecting that tab through Chartr's existing session navigation. Live
delivery is offered whenever a live agent holds the tab's foreground: a session,
a free launch, and an agent the operator started themselves in an ordinary shell
alike, since what a delivery requires is an agent listening rather than a shell
that would run the preset as a command. A pending preset is aimed at that agent,
not at the tab: if the identified agent goes away or is replaced before delivery,
the pending item is dropped exactly as it is when the tab exits. An idle target receives the preset immediately. A
working, running, or blocked target stores one visible, cancellable pending preset
and submits it automatically on the next observed idle transition. A second
activation is refused while one is pending. The queue is runtime state and dies
with the tab or Chartr process.

Submission reuses the existing PTY prompt-typing behavior: prompt text first,
then a carriage return in a separate write. Writes for one submission are
serialized against ordinary terminal input so another input chunk cannot land
between its text and return. Chartr never injects into a blocked or working screen.

Idle is inferred from terminal evidence, not acknowledged by the provider. An
existing draft in the agent composer may therefore collide with a live delivery.
The UI describes queued delivery as “Queued for next idle” and makes no guarantee
beyond that best-effort contract. This limitation is accepted rather than hidden
behind provider-specific machinery.

## User-visible behavior

1. The operator can create, edit, and delete a short named preset from the
   Prompts pane.
2. The same catalog appears in every registered space.
3. Each preset has an `At launch` control whose value is remembered independently
   for each registered space.
4. Ticket payload preview shows the presets currently selected for the space in
   the same order a spawn will receive them.
5. A ticket or free agent launched afterward receives the selected presets in its
   opening payload; a free launch with no selection behaves exactly as it does
   today.
6. The pane sends a preset immediately to an idle active agent, whether Chartr
   launched it or the operator started it themselves.
7. The pane queues one preset for a busy or blocked active agent, shows which one
   is pending, and lets the operator cancel it before delivery.
8. The queued preset is submitted once when the agent next reads idle. It is never
   typed into a permission prompt or while work is visibly in progress.
9. A shell sitting at its own prompt, a dead tab, Scratch, or a missing active
   tab offers no Send or Queue action and explains the narrow target.
10. Disabling `At launch` does not claim to make an existing agent forget the
    preset; applying it to that agent requires an explicit Send or Queue action.

## Persistence and failure behavior

- `prompts.toml` is global, local to the operator, and atomically rewritten by
  catalog mutations. Prompt ids are unique and stable across name or text edits.
- Per-space launch ids live in `spaces.toml` beside the space's existing local
  state. A whole-list update writes the selection atomically.
- A missing catalog is an empty catalog. Malformed catalog input is surfaced as a
  warning and yields no executable presets; a mutation must not silently discard
  unreadable operator content.
- A missing selected id is ignored and surfaced, not substituted with another
  preset. Ordinary deletion removes known references as part of the same action.
- A pending live delivery snapshots the selected preset when it is queued. A
  later edit or deletion does not rewrite an action the operator already took.
- If the target exits before delivery — or the agent it was queued for leaves the
  tab's foreground, or another agent takes its place — the pending item disappears
  with it. There is no persistence, retry, or delivery notification subsystem.

## Testing boundary

- Config and registry tests cover catalog round trips, validation, malformed
  input, per-space isolation, deletion cleanup, and persistence across reload.
- Payload tests prove selected ordering and placement, ticket preview/spawn
  equality, free-session delivery, and the unchanged bare free launch when no
  preset is selected.
- Terminal-manager and server tests prove immediate idle submission, busy and
  blocked queueing, cancellation, one-pending refusal, next-idle delivery,
  submission write ordering, target refusal, and cleanup on exit.
- Frontend tests cover active-target eligibility and the small state vocabulary;
  the repository web checks verify the rendered integration.

