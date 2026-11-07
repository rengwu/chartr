---
title: Harp ideas for Chartr
section: Discussions
updated: 2026-10-21
status: Proposals; no adoption decision
---
# Harp ideas for Chartr

The October 21 request was to inspect Harp and identify useful ideas for Chartr.
This is source analysis, not implementation authorization or executed product
verification. No adoption choice has been accepted.

## Basis

Reviewed Harp main at
[`c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4`](https://github.com/dominikmartn/harp/tree/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4),
dated October 20, and a clean Chartr tree at
[`204e9a7985ac2944c1478b21602e6ca541226ae3`](https://github.com/rengwu/chartr/tree/204e9a7985ac2944c1478b21602e6ca541226ae3).
Harp's README, sidebar image, sidebar implementation, session state machine,
background hook, tests and limit-file contract were inspected. Neither app was
built or launched for this comparison; test source was read, not run.

Harp is a Warp fork centered on managing many agent sessions. Chartr already has
spaces, pane groups, activity icons, persistent backend sessions and unread
attention. Its greatest transferable value is making many sessions easier to
identify and triage. Replacing Chartr's terminal or backend is unnecessary for
these improvements.

Relevant existing contracts: [workspace views](../features/workspace-views.md),
[panes and attention](../features/panes.md),
[session persistence](../features/session-lifecycle.md),
[background status](../features/status-bar.md), and the accepted
[best-effort detection decision](../decisions/index.md#dec-attention-005-reuse-existing-detection-with-best-effort-alerts).

## Recommended order

These are relative scope estimates, not implementation commitments.

| Priority | Proposal | Value and scope |
| --- | --- | --- |
| 1 | Stable task names for individual terminals | Distinguish several Codex or Claude sessions immediately. Small-to-medium scope: title precedence, rename action and persisted override. |
| 1 | Space attention counts and navigation | Find waiting work without opening every group. Medium scope: aggregate terminal facts, show a compact count, navigate to the exact terminal. |
| 2 | Manual set-aside state | Keep long-lived sessions available while reducing distraction. Medium scope, with acknowledgment and notification policy to resolve. |
| 2 | Optional usage-limit plugin | See remaining provider capacity before starting work. Small first version if a producer already supplies a local file; data collection is separate work. |
| Supporting | Opt-in status traces and replay fixtures | Diagnose incorrect indicators without building another detector. Small-to-medium scope; extend existing attention tests. |

## Stable task names

Harp's [row-title logic](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/workspace/view/vertical_tabs.rs#L9395)
prefers an explicit title and otherwise uses available conversation/terminal
information. Its README exposes a quick rename action. Chartr's
[session title](https://github.com/rengwu/chartr/blob/204e9a7985ac2944c1478b21602e6ca541226ae3/crates/chartr-herdr/src/control.rs#L147)
currently prefers the detected running process over its saved label. The native
rename flow targets spaces and groups; a conversation title is already present
in the control model but does not drive this row title.

Proposed precedence: explicit terminal name, supported conversation title, then
the existing fallback. Keep the provider icon so a row such as “Fix OAuth retry”
still identifies its agent. Persist an explicit override with terminal identity,
never overwrite it with later automatic titles, and offer a reset to automatic.
Start with manual naming if automatic titles prove unstable. This does not
require a model call or terminal-text extraction.

## Attention summaries and age

Harp's [sidebar implementation](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/workspace/view/vertical_tabs.rs#L9137)
ranks split-pane states as needs-input, working, then finished; section headings
show the number needing input. Rows show elapsed time since a status change.
Its finished presentation includes success, error and cancellation, so that icon
is not proof of success.

Chartr's [entry projection](https://github.com/rengwu/chartr/blob/204e9a7985ac2944c1478b21602e6ca541226ae3/crates/chartr/src/space.rs#L526)
already aggregates unread attention across group members, but deliberately has
no aggregate group activity state. Space headings currently show the name and
controls without an attention count.

Start with a space-level count of unread terminals, including collapsed and
nonselected descendants, deduplicated by terminal identity. Label it “unread” or
“to review”: Chartr's unread state also includes completed or cancelled work,
whereas Harp's “needs you” count describes blocked activity. A separate blocked
count can follow if useful. Keep current activity and unread acknowledgment as
independent facts.

A proposed Chartr extension is “Next unread agent” in the command palette, using
stable workspace order and the existing exact-terminal navigation path. This is
our recommendation, not a verified Harp feature. Preserve manual row ordering;
do not move rows as states change. If age is added, describe the observed state
age rather than total task duration, use minute-level refreshes, and avoid
inventing timestamps when restoring an unknown history.

## Set sessions aside

Harp has a manual inactive flag, shown as a moon with normal status/unread
presentation suppressed. A new prompt wakes it. It also supports manual unread
marks. These are user intent layered over activity, not process suspension. See
the [tab flags](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/tab.rs#L400)
and [prompt handler](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/workspace/view.rs#L3932).

For Chartr, propose a persisted, reversible “Set aside” action that leaves the
terminal running and accessible. Store it separately from detected activity and
unread state. Before implementation, decide whether it mutes notifications and
whether blocked work can override it. Default recommendation: retain unread
history and explicit resume; do not assume the current Herdr contract supplies a
reliable new-prompt event. Manual “remind me” state could follow, but must not
manufacture a desktop notification or masquerade as a new agent event.

## Usage limits as an optional plugin

Harp's [limit-file contract](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/docs/limit-rings.md)
separates collection from presentation: another tool writes local JSON, and Harp
displays provider percentages and reset times. Old or failed readings become
unknown; the app does not fetch provider usage itself. This boundary is more
valuable to Chartr than the particular double-ring graphic.

Use a native plugin and the existing
[`BackgroundStatus`](https://github.com/rengwu/chartr/blob/204e9a7985ac2944c1478b21602e6ca541226ae3/crates/chartr-plugin/src/lib.rs#L264)
contract for a first version: a compact text label and tooltip with windows,
resets and freshness. The contract currently provides label/detail/state, not
arbitrary ring graphics. Read the file off the UI thread and cache the result;
the host queries status once a second. Keep provider names in the contribution,
consistent with the existing status-bar copy boundary.

Missing or expired data must not look like zero usage. A usable feature also
needs an identified producer: file display alone does not obtain live account
limits. Rich rings and additional plugin UI contracts can wait for demonstrated
need.

## Status diagnosis and background work

Harp's [opt-in row trace](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/workspace/view/vertical_tabs.rs#L9634)
records changes in the displayed state, and its
[replay tests](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/terminal/cli_agent_sessions/mod_tests.rs#L1627)
exercise event ordering, stale reports, cancellation and background completion.
Chartr already has pure attention tests; extend those with sanitized real
snapshot sequences and an optional trace of terminal ID, backend activity,
unread transition, focus and notification attempt. Avoid prompt text by default.
This would make detector versus presentation failures easier to separate.

The deeper lesson is that a foreground turn ending does not imply every child
task stopped. Harp's optional Claude hook reports running task kinds and guards
against old reports. At the inspected revision, the
[state machine](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/app/src/terminal/cli_agent_sessions/mod.rs#L202)
keeps non-shell background work active, while shell-only work permits a finished
state with a separate background dot. Its hook README/comments describe the
shell case more broadly; the implementation and corresponding test distinguish
it. This is an inspected source behavior, not a tested provider guarantee.

Defer new Chartr provider hooks. Child-task counts and kinds are absent from its
current session contract, and owning these adapters would revisit
DEC-ATTENTION-005's maintenance tradeoff. If real workflows justify it, investigate
an optional Herdr capability first. Ordinary foreground-process detection cannot
stand in for background subagent counts.

## Boundaries to preserve

- **Unread retention:** Harp suppresses automatic unread presentation while
  working or waiting. Chartr intentionally retains its dot through resumed work
  until exact foreground-terminal focus. Keep Chartr's accepted behavior.
- **Session ownership:** Harp's [tmux integration](https://github.com/dominikmartn/harp/blob/c0bdc5d308f9c0d9e8b258fe8341d01cb74f40b4/README.md)
  shares names, groups, ordering and flags with other clients and can adopt
  externally created sessions. Chartr already gets local survival from private
  Herdr. Phone/SSH handoff is a separate product capability, not a prerequisite
  for persistence; defer it until requested and preserve the
  [private-namespace boundary](../decisions/adr/0001-a-private-herdr.md).
- **Architecture and provenance:** Reimplement selected UX ideas within Chartr's
  current modules. Harp is a full Warp fork retaining upstream AI/cloud features;
  importing that architecture would add unrelated scope. Its app code declares
  AGPL-3.0 and selected UI crates MIT, while Chartr declares GPL-3.0-or-later.
  This review does not assess direct source-copying compatibility.

## Next decision

Recommended first slice: individual task names plus space unread counts and
exact-terminal navigation. It improves many-session use without changing the
detector. Set-aside semantics and a usage-data producer remain separate choices.
No feature-map expectations or accepted decisions change until a proposal is
selected. Any implementation should verify rename restoration, collapsed-group
counts, exact split focus and existing unread retention on an isolated profile.
