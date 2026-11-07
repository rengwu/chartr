---
title: Terminal task names
section: Discussions
updated: 2026-10-30
status: Implemented; automated checks passed; desktop verification pending
---
# Terminal task names

The user selected **stable task names for individual terminals**, accepted the
naming policy in [DEC-REX-003](../decisions/index.md#dec-rex-003-automatic-task-names-with-configurable-manual-name-lifetime),
then explicitly requested implementation using work-until-grill on October 30.
The working tree now implements the policy. Current behavior lives in the
[terminal feature record](../features/terminals.md#terminal-task-names); executed
checks and limits belong to the [implementation run](../verification/runs/2026-10-30-terminal-task-names.md).

The remaining sections preserve the design/feasibility record written before
implementation. Their planned checks are not claims of executed verification.

## Agreed behavior and planned experience

Three terminals currently labeled `Codex` could instead show `Fix OAuth retry`,
`Review migration` and `Add billing tests`. Each retains its live provider icon
and activity indicator. A task name describes the terminal's purpose; it does
not become the process identity or an attention signal.

Names resolve as **manual name → available conversation title → current process/
shell fallback**. The provider icon appears beside the task name. Manual-name
lifetime is configurable: keep it until explicit rename/reset, or reset it when
a new agent conversation is identified. Task names in desktop notifications are
deferred.

The planned editing flow supports any terminal, including ordinary shells.
Right-click its standalone sidebar row, outer terminal tab or
pane-local tab and choose **Rename Terminal**. The command palette also exposes
**Terminal: Rename active terminal**. Reuse the existing rename dialog and keymap
system, with an assignable action and no new default key that could interfere
with terminal programs.

Saving updates every place that already shows that individual terminal's title.
**Use automatic name**, or saving an empty field, removes the override. Cancel
and Escape preserve the previous name. The same terminal keeps its manual name
through process changes, rearrangement, detach/reopen and attachment recovery
under the default Keep policy. The optional reset policy responds to identified
conversation changes as described below. A new terminal starts without a manual
name. Duplicate names are allowed. Full names,
provider/process and working directory remain available in tooltip/accessibility
detail; long visible labels truncate using existing chrome behavior.

Groups retain their own separate names. This feature operates on individual
terminals inside those groups and does not need new sidebar nesting or an
activity/search feature.

The dialog entry points, empty-field reset, duplicate-name handling and tooltip
details are implementation recommendations. The accepted design does not require
another preference round before this plan can be delivered.

## Feasibility findings

Source reviewed at
[`369298062bfd5ba00ad0c273d580ce408797b19c`](https://github.com/rengwu/chartr/tree/369298062bfd5ba00ad0c273d580ce408797b19c).
The working tree has documentation changes. No product build, live Chartr run or
new behavior test was executed for this exploration.

| Finding | Implication |
| --- | --- |
| [`control::Session::title`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr-herdr/src/control.rs#L148) prioritizes the running agent/process over its fallback label. | Merely renaming a backend label would still hide the task name while an agent runs. Add an explicit host-owned display override. |
| [`Item::title`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/item.rs#L66) feeds standalone workspace entries and pane tabs. | One display policy can cover existing terminal surfaces. Provider icons already use separate agent/process metadata. |
| [`rename.rs`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/app/rename.rs) supports spaces and groups, including a native modal above embedded views. | Extend this interaction to an exact terminal target; do not build another editor. |
| [`PersistedItem::Terminal`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/persistence.rs#L77) saves only workspace item ID and backend pane ID. | Durable names need additional host metadata. An existing label field is not already available in Chartr's saved terminal record. |
| [`Space::restore_saved`, `persisted`, `adopt`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/space.rs#L297) use backend pane IDs and have asynchronous attachment placeholders. | Preserve names while attachments are pending, and do not key durable names only by the current pane number or UI position. |
| The [Herdr patch](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/vendor/herdr/terminal-identity.patch) preserves terminal IDs across handoff, including pane-ID aliases. | Persistent terminal identity is the right key for names; pane identity can be remapped. No additional backend protocol is evidently needed for a host-owned name. |
| [`Session::update_info` and attachment replacement](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/session.rs#L159) replace backend information. | Keep manual metadata outside those replaceable snapshots so polling and reattachment cannot erase it. |

The feasibility judgment for the selected version is **medium scope**: existing
UI can be reused, while persistence, automatic titles and the reset setting need
coordinated changes. The main integration risks are stable identity through
handoff/restore and the limits of conversation metadata.

## Automatic naming policy

The initial recommendation was manual naming with the existing process/shell
fallback. Question 5 explicitly selected available conversation titles too,
superseding that narrower recommendation.

Chartr already exposes a
[`conversation_title`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr-herdr/src/control.rs#L82),
derived from an explicit backend pane title or terminal OSC title with some
provider-specific cleanup. It is not uniformly a supported-log conversation
name, is not guaranteed for every agent, and can reflect changing or stale
terminal metadata. Existing history ingestion does not make it a stable tab name.

Planned rules:

- A manual override wins until explicitly removed or the selected reset policy
  observes a qualifying conversation change.
- For a currently detected agent, use a meaningful available conversation title
  from existing backend metadata. Reuse its provider-title cleanup and apply
  single-line display validation. Blank/startup/control-character titles fall
  back to the current agent/process label.
- On agent exit, use the ordinary process/shell fallback unless a manual override
  exists. A previous agent's automatic title is not a permanent terminal name.
- Avoid adding a second persistent cache of automatic title text. Drop transient
  title association when the known provider/native conversation changes. If a
  changed conversation is still reporting exactly the previous title, prefer
  the process label until the title changes; metadata cannot prove that an
  identical title is fresh. This can conservatively hide a legitimate repeated
  title, and cannot detect every stale title when identity is absent.
- Automatic names remain best effort because existing metadata has incomplete
  coverage and no uniform title-freshness guarantee. No title-generation model,
  new provider adapter or transcript scanner is needed for the selected scope.

These fallback rules implement the chosen available-title behavior without
claiming stronger freshness than the current backend exposes.

## Manual-name lifetime setting

Add a global setting under **Settings → Terminal → Manual task names**:

- **Keep until renamed or reset** — default, following the selected question 6
  answer. A name describes a reusable terminal work slot.
- **Reset when a new conversation is identified** — the additional behavior
  requested in the answer's note.

Suggested helper: “Automatic reset requires the agent to report its conversation
identity.” Reuse the existing Terminal settings model/page and normal settings
file persistence. This is a user preference, separate from the SQLite record
that stores each terminal's manual name.

Use the existing provider/native-session normalization in
[`NativeSession::from_identity`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr-conversations/src/lib.rs#L28)
and the provider-matching check already used by
[`observe_conversations`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/app/conversations.rs#L76).
Track a nonblank normalized provider/conversation key for each terminal; do not
use title text, activity status, cwd, PID or backend pane ID as conversation identity.

| Observation | Planned behavior |
| --- | --- |
| Keep policy | Preserve the override. Update known identity bookkeeping for later changes. |
| Reset policy; known conversation A becomes known B | Remove the override and resolve the new automatic title/fallback. |
| First usable identity after none was known | Establish the baseline; preserve the override. |
| Identity temporarily missing or backend read fails | Preserve the override and last known identity; absence is not a new conversation. |
| Same normalized identity with a different reporting path/source or remapped pane | Preserve the override. |
| User changes the policy | Apply the policy to subsequent observations; changing the setting itself does not immediately clear names. |
| Relaunch, reattach or handoff | Preserve the name and known identity. Only an actual known-to-known conversation change can qualify for reset. |

Keep relevant last-known identity with the terminal's persisted naming metadata
so reconnecting to the same conversation does not look like a new one. Unknown
agents still support manual naming and automatic label fallback; their names
are not silently reset based on guesses. Missed/unreported conversation changes
remain a documented limit of the optional policy.

## Proposed implementation sequence

1. **Define and test the naming model.** Implement the precedence and observation
   rules above as host-owned state separate from backend agent/process/status
   information. Reuse normalized native identity; keep automatic-title freshness
   tracking bounded and local to this feature.
2. **Make names durable.** Store host-owned overrides in the existing SQLite
   persistence system, associated with persistent terminal ID in the private
   backend namespace. Recommended shape: a versioned `terminal_names` value in
   `app_state`, carried by `Snapshot` and the existing ordered state writer, with
   manual name and last-known normalized conversation identity per terminal.
   Absence in older databases means an empty map. Keep the map independent of
   current pane numbering, so an attached terminal with a remapped pane ID can
   resolve its name. Never match a name by cwd, provider or a reused tab position.
   Remove records only after confirmed terminal retirement, not a transient
   missing snapshot or a local attach failure.
3. **Connect the editing flow.** Add exact-item rename/reset actions to terminal
   context menus and the active-terminal command. Reuse native modal hosting,
   current input/IME behavior and focus restoration. Validate the target still
   exists when Save runs; a closed/replaced terminal must not rename its successor.
4. **Add the lifetime setting.** Extend `TerminalContent`, `ResolvedSettings`,
   the Terminal settings page and its update path. Persist it through the normal
   settings store; apply changes without restarting or clearing names immediately.
5. **Integrate display.** Use the same resolved label in standalone chrome and
   inner pane tabs; keep group naming independent. Preserve activity indicators
   and provider/process metadata. A manual name accepts Unicode and numeric names
   such as `123`; it must not inherit the backend rule that rejects numeric tab
   labels. Trim whitespace and reject control characters at the edit boundary.
6. **Validate persistence and interaction.** Exercise the scenarios below and
   update the existing terminal/pane feature records during implementation.

An app-owned name map avoids copying a name only into the current `Session` or
`restoring_sessions` item-ID map, which would not cover backend snapshot
replacement and remapped pane identities.
There is no source evidence requiring a new Herdr patch for manual naming.

## Deferred scope

Current notification titles are built as `Space · Agent` in
[`attention/content.rs`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/attention/content.rs#L22),
separately from the terminal item label. Question 8 explicitly defers task names
in notifications, so the implementation must preserve that title contract even
when terminal labels change. The earlier native notification-test deferral remains
in force. Search, activity summaries, recipes, agent control APIs and naming
plugin surfaces are also outside this first feature.

## Verification plan — not executed

- Test manual/conversation/process fallback, explicit reset,
  whitespace/control-character handling, Unicode and numeric names. Ordinary
  backend metadata changes must leave an override intact under Keep mode.
- Exercise both lifetime policies: known A→B, A→unknown→B, first identity,
  same identity reported differently, mismatched provider, identity-free agents,
  policy toggles, resume and restart. Confirm that reset is scoped to the same
  persistent terminal and only qualifying identity changes clear a manual name.
- Check title arrival, startup placeholders, agent exit, carried-over old titles
  after a known conversation change, and missing identity/title fallbacks.
- Round-trip saved names; load older records; preserve names through pending
  asynchronous restore. Exercise ordered writes and failed-save retry behavior.
- Reconcile the same terminal ID with a changed backend pane ID; reject reuse
  for a genuinely different terminal. Check names across handoff and reattachment.
- In an isolated Chartr profile, rename from both standalone and inner pane tabs,
  then move, split, group, ungroup, detach/reopen and restart the frontend. Confirm
  the same live terminal retains its name and receives normal input.
- Cancel/escape, reset to automatic, close the target while the dialog is open,
  and use an embedded plugin surface adjacent to it. Confirm correct target and
  focus handling without input leaking into the terminal.
- Verify settings default, save/reload and live policy updates. Confirm that
  existing notification-title formatting remains unchanged. Use the repository's
  normal required checks when implementation begins; documentation build/check
  only validates this plan's structure.

All runtime checks must use the [isolated verification setup](../verification/setup.md).
The user's installed sessions are not test fixtures.
