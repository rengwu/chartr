---
title: Session lifecycle, recovery and persistence
section: Features
updated: 2026-11-07
basis: Source-reviewed migration baseline
---
# Session lifecycle, recovery and persistence

Feature ID: `FEAT-session-lifecycle`

## Intent and basis

This record uses the inspected current implementation under
[the accepted migration decision](../decisions/index.md). It describes the
working-tree baseline, not a newly executed product result. See
[baseline identity and limitations](../verification/setup.md#baseline-and-build-identity).

Architectural basis: [ADR 0001](../decisions/adr/0001-a-private-herdr.md), [ADR 0004](../decisions/adr/0004-the-zed-terminal-stack.md), [ADR 0005](../decisions/adr/0005-spaces-follow-zed-multi-workspace.md). Historical portions of ADRs remain dated rationale.

Implementation owners: [crates/chartr/src/app/backend.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/app/backend.rs), [crates/chartr/src/app/persistence.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/app/persistence.rs), [crates/chartr/src/persistence.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/persistence.rs), [crates/chartr-herdr/src/namespace.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr-herdr/src/namespace.rs), [crates/chartr/tests/live_session.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/tests/live_session.rs), [crates/chartr/src/app.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/app.rs).

Legacy story references: 26, 35, 36, 37, 39, 40, 41, 42, 43, 51, 52, 81, 82, 83, 84, 85, 86, 87, 89, 90. See the [complete crosswalk](story-crosswalk.md).

## Entry points and prerequisites

Close an item/group/space, quit/reopen the app, or use the Problems menu and terminal recovery actions. Use only an isolated profile and disposable processes for failure checks.

Use [shared setup](../verification/setup.md) and a disposable profile/project for
stateful recipes. Related areas are linked from the [feature index](index.md).

## Expected outcomes

- On macOS, opening the still-running app after its last workspace window closes restores a workspace from saved state. Repeated opens activate an existing workspace rather than duplicating it. Quit policy and persistence apply to the current workspace after reopening. See the bounded October 12 native verification below.
- Normal app exit detaches sessions by default; the configured exit policy may terminate them. Closing a terminal terminates that session, while closing a plugin destroys its view.
- Reattachment preserves surviving shells and jobs; it cannot revive terminated
  TUIs. The [November 7 mouse recovery run](../verification/runs/2026-11-07-mouse-recovery.md)
  distinguishes surviving shells from killed agents and covers stale mouse modes
  after a job returns to its shell, including detached and handed-off runtimes.
- Session-bound plugin items cascade when their subject closes. Bulk operations confirm before terminating multiple sessions.
- Backend recovery permits one clean restart before a stable crash-loop state; Retry is explicit. Live backend identity overrides stale records while unrelated layout survives.
- Layout writes coalesce, serialize in the background and flush on shutdown; stale queued revisions cannot overwrite newer state. Missing plugins/sessions are summarized.
- Close All Sessions is included in v0.3.2. It gathers terminal items across spaces, prompts above one target, keeps spaces open and invokes normal close/cascade behavior.

## Verification recipe

**Unvalidated recipe — not executed during this migration.** Complete shared
setup first; use actual UI entry points and observe underlying state where relevant.

1. Start two disposable sessions in different fixture spaces and a bound plugin if available. Quit normally and reopen; expect identity/layout continuity.
2. Close one terminal and verify the other survives. Cancel and then confirm a multi-session group/space close; verify only selected descendants terminate.
3. For Close All Sessions, use the command palette; cancel first, then confirm in a disposable profile. Expect spaces and unrelated unbound tools to remain.
4. For backend failure scenarios, run the real-sidecar suite from shared setup or exercise only the isolated daemon. Check identity replacement, stale-session rejection and visible recovery states.
5. On macOS, close the last workspace window without quitting, then open the same app bundle. Verify the PID is unchanged, the workspace is visible, and disposable terminal identity/layout/bounds are restored. Repeat twice, open with a window already present, and minimize/open; expect one workspace. Quit the recreated window with default detach and configured terminate in separate disposable fixtures.
6. Relaunch after changing layout and verify persisted state and registry order. Never infer persistence solely from the pre-exit screenshot.

Preserve the selected scenario's logs/screenshots/file observations and exact
build identity before [cleanup](../verification/setup.md#cleanup). Record a fresh
run if this recipe is exercised; source inspection alone is not a pass.

## Coverage and limitations

No daemon was started or killed during migration. At that time, crash loops,
exit-policy variants, the then-uncommitted bulk-close action, stale writes and
restoration remained unverified. Later dated runs below record their own scope;
the migration itself establishes no behavioral pass.

The [release matrix](../verification/acceptance.md) retains broader cross-feature,
platform, theme, pointer and accessibility coverage. Choose a bounded subset for
ordinary changes; release requirements remain separate from this advisory workflow.

## Verification history

[0.3.3 preparation](../verification/runs/2026-11-07-release-0.3.3.md) records
bounded automated checks, isolated real-terminal checks and candidate package
verification under the user's request to preserve the desktop. Native coverage
limits remain explicit.

The [October 16 desktop follow-up](../verification/runs/2026-10-16-release-desktop.md)
verifies macOS Cmd+Q with default detach and configured termination after adding
application-wide Hide/Quit actions in `b807e4c3`. Relaunch restores terminal
identities and unread state. Actual hidden-state monitoring remains unverified.

The [0.3.1 candidate follow-up](../verification/runs/2026-10-16-release-candidate.md)
at `e915365a` passes the macOS, Ubuntu and Arch/AUR CI matrix and all three
package builds. Downloaded-package inspection and selected native attention
checks are recorded separately; full desktop release acceptance remains incomplete.

The [October 16 release-readiness run](../verification/runs/2026-10-16-release-readiness.md)
at `c11b2dc5` passes the three real-backend session tests, including isolated
daemon crash recovery, and the focused backend terminal-identity handoff test.
This does not rerun the full native reopen, Hide, quit-policy or layout matrix.

No current product run was performed for this migration. Earlier
[research and audits](../archive/research/index.md) remain historical evidence for
their recorded snapshots; they do not certify this baseline.

The [October 12 reopen implementation run](../verification/runs/2026-10-12-window-reopen.md)
records the scoped patch, automated regression passes, six same-process reopen
cycles, terminal continuity, minimize/restore and both quit policies. The [October 11 failure](../verification/runs/2026-10-11-metal-frame-delivery-recheck.md)
remains historical evidence.
