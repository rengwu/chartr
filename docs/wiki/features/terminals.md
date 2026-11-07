---
title: Terminal interaction and search
section: Features
updated: 2026-11-07
basis: Source-reviewed migration baseline
---
# Terminal interaction and search

The [accepted task-name policy](../decisions/index.md#dec-rex-003-automatic-task-names-with-configurable-manual-name-lifetime)
is implemented in the working tree. The [implementation run](../verification/runs/2026-10-30-terminal-task-names.md)
records its actual verification and desktop-driver limits.

Feature ID: `FEAT-terminals`

## Intent and basis

This record uses the inspected current implementation under
[the accepted migration decision](../decisions/index.md). It describes the
working-tree baseline, not a newly executed product result. See
[baseline identity and limitations](../verification/setup.md#baseline-and-build-identity).

Architectural basis: [ADR 0001](../decisions/adr/0001-a-private-herdr.md), [ADR 0002](../decisions/adr/0002-the-zed-layer.md), [ADR 0004](../decisions/adr/0004-the-zed-terminal-stack.md). Historical portions of ADRs remain dated rationale.

Implementation owners: [terminal_names.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/terminal_names.rs), [app/rename.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/app/rename.rs), [crates/chartr/src/session.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/session.rs), [crates/chartr/src/session/launch.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/session/launch.rs), [crates/chartr/src/app/terminal_search.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/app/terminal_search.rs), [vendor/zed-terminal-view/chartr-PATCH.md](https://github.com/rengwu/chartr/blob/main/vendor/zed-terminal-view/chartr-PATCH.md).

Legacy story references: 23, 37, 38, 81, 82, 93. See the [complete crosswalk](story-crosswalk.md).

## Entry points and prerequisites

Use + to create a terminal. Search with Cmd+F on macOS or Ctrl+Shift+F on Linux; use terminal menus and platform clipboard shortcuts.

Use [shared setup](../verification/setup.md) and a disposable profile/project for
stateful recipes. Related areas are linked from the [feature index](index.md).

## Expected outcomes

- The pinned Zed model/view owns emulation, selection, clipboard, IME, mouse reporting and scrollback. Its local PTY runs the private Herdr attach client.
- Terminal labels resolve manual name → available agent conversation title → current process/shell fallback. Provider icons and activity remain separate. A terminal is non-cloneable.
- Literal search navigates matches without leaking search keystrokes into the shell. Font changes reflow the terminal grid live.
- Long/multiline agent launch commands use private staging rather than entering the complete command into interactive shell history.
- Attach failure is visible; retry targets the same confirmed session identity rather than silently creating a different one.
- When a known foreground job returns control to its shell, the backend clears
  mouse reporting that the job left enabled. Live TUIs retain mouse input across
  reattachment. The [November 7 run](../verification/runs/2026-11-07-mouse-recovery.md)
  records the observer's bounds and the killed-TUI regression.

## Terminal task names

Right-click a standalone sidebar row, outer terminal tab, or pane-local terminal
tab for **Rename Terminal**. The palette offers **Terminal: Rename active terminal**;
Hotkeys can bind it, with no default chord. Groups and plugins keep their own naming.

Saving trims the name; Unicode, numeric and duplicate names are valid. Control
characters and line breaks are rejected. An empty save or **Use automatic name**
removes the override. Cancel/Escape preserves it. The dialog captures the exact
space, item and persistent terminal identity, so replacement/closure cannot rename
a successor. Full name, provider/process and cwd appear in tooltip/accessibility
detail; visible labels truncate.

The [Terminal setting](settings.md) defaults to **Keep until renamed or reset**.
**Reset for a new conversation** requires a known normalized provider/native ID to
change. First identity, temporary missing identity and policy toggles preserve the
name. Keep mode still tracks identity, avoiding a retroactive reset after changing
the setting. Renames and known identity persist in the existing ordered SQLite
snapshot writer, keyed by terminal ID; pending attachment and remapped pane IDs do
not change the key. Confirmed retirement removes the associated metadata.

Automatic titles use existing backend metadata and are not persisted or retained
after agent exit. A trailing ` | project-folder` is removed when it matches the
terminal’s working-directory name; manual names and other pipe-separated task
text are preserved (October 30 screenshot follow-up). Startup/blank/control-character
titles fall back. A known
conversation/provider change suppresses an identical carried-over title until a
different title appears; identical legitimate titles can therefore be hidden.
Missing identity and frontend restart limit freshness guarantees. This feature
adds no transcript scanning or generated titles. Notification title formatting
remains `Space · Agent`.

### Naming recipe

**Native recipe not yet executed.** Use the isolated profile in shared setup.

1. Rename from the sidebar, outer tab, pane-local tab and command palette. Try a
   numeric and Unicode name, a long name, cancellation and empty-save reset. Expect
   the same resolved label and full detail across title surfaces.
2. Split, group, move and ungroup the terminal. Detach/reopen, restart the frontend
   and hand off the backend. Expect the same terminal ID and name, and usable input.
3. Close/replace the target while its rename dialog is open. Saving must affect no
   successor. Repeat beside an embedded plugin to check modal placement and focus.
4. Select each lifetime policy and reopen Settings. With a supported agent's
   reported identities, check A→unknown→B and switching policy without clearing.
   Confirm no automatic name remains after agent exit.

## Verification recipe

**Unvalidated recipe — not executed during this migration.** Complete shared
setup first; use actual UI entry points and observe underlying state where relevant.

1. Print several screens of styled Unicode in a disposable shell; select/copy/paste and search literal punctuation. Dismiss search and verify terminal focus.
2. Run less or another installed alternate-screen TUI. Exercise wheel/trackpad, keys, mouse and resize; expect input to reach the TUI without duplicate delivery.
3. Check multiline input, word movement, IME, file drag/paste, hyperlink actions and BEL indication using the acceptance matrix.
4. Change terminal font size, detach/reopen and repeat in standalone and split panes. Capture visible results and session identity, not screenshots alone.

Preserve the selected scenario's logs/screenshots/file observations and exact
build identity before [cleanup](../verification/setup.md#cleanup). Record a fresh
run if this recipe is exercised; source inspection alone is not a pass.

## Agent attention — October 15 simplification

[DEC-ATTENTION-005](../decisions/index.md#dec-attention-005-reuse-existing-detection-with-best-effort-alerts)
selects best-effort Herdr detection. The working tree now reads existing agent
states about once a second. Working-to-inactive or entry into blocked marks
attention; ordinary shells, unknown states and initial/reconnected snapshots do
not produce alerts. Idle and done are equivalent here. Detection can misclassify
cancellation or unfamiliar screens and miss short or repeated blocked requests.

An accent dot means an agent may need attention. Exact terminal focus in an active
foreground window clears it and rearms desktop delivery. At most one desktop
notification is requested until that visit. Unread survives resumed work, app
relaunch against a surviving backend, and live handoff without catch-up banners.
Terminal/group dots share the same state; showing a split or opening a group
cannot acknowledge its unfocused children. Spaces have no aggregate dot.

The strict October 14 provider/outcome implementation is superseded. Agent
coverage follows existing upstream detection, including Codex. At the time of
the October 15 simplification, the backend patch only preserved terminal identity
through handoff; the November 7 mouse recovery extends that patch. The prototype's unreleased
version-1 attention storage is reset; version 2 restores its own unread state.

### Bounded attention recipe

Use a fresh explicitly isolated packaged app and an installed supported agent.
Start with Codex and observe its actual detected states.
Trigger a harmless input request while another terminal is selected. Expect a dot
and an OS notification; focus the exact terminal and expect the dot to clear.
Repeat with an adjacent split and enclosing group, then with the app inactive,
minimized and without windows. Relaunch with unread attention: the dot should
return without another banner. Close or replace the origin before clicking an
old notice: no unrelated terminal should open.

The recheck below identifies which portions were executed. Unexecuted portions
of this recipe remain unvalidated.

## Coverage and limitations

The [October 14 implementation recheck](../verification/runs/2026-10-14-agent-attention.md)
remains historical evidence for the superseded strict design. The [October 15 recheck](../verification/runs/2026-10-15-lean-attention.md)
records passing automated and real Codex completion/approval/cancellation checks,
plus dots/focus, persistence, handoff and minimized/closed-window monitoring.
Direct banner inspection/clicks, native split/group focus, Hide and graceful Quit
remain unverified. Broader provider and Linux coverage is not established.

Clipboard, IME, device scrolling, alternate-screen interaction and platform-specific key delivery remain unvalidated. Test helpers cannot certify the native paths they bypass.

The [release matrix](../verification/acceptance.md) retains broader cross-feature,
platform, theme, pointer and accessibility coverage. Choose a bounded subset for
ordinary changes; release requirements remain separate from this advisory workflow.

## Verification history

[0.3.3 preparation](../verification/runs/2026-11-07-release-0.3.3.md) records
bounded automated checks, isolated real-terminal checks and candidate package
verification under the user's request to preserve the desktop. Native coverage
limits remain explicit.

[November 7 mouse recovery](../verification/runs/2026-11-07-mouse-recovery.md)
records the live incident, manual shell recovery, and automatic recovery checks.

[October 30 task names](../verification/runs/2026-10-30-terminal-task-names.md) records
the implementation checks and blocked native interaction coverage.

The [0.3.1 candidate follow-up](../verification/runs/2026-10-16-release-candidate.md)
at `e915365a` passes the macOS, Ubuntu and Arch/AUR CI matrix and all three
package builds. Downloaded-package inspection and selected native attention
checks are recorded separately; full desktop release acceptance remains incomplete.

The [October 16 release-readiness run](../verification/runs/2026-10-16-release-readiness.md)
at `c11b2dc5` passes all three real-session tests with the exact patched sidecar,
including direct attach, crash recovery and shell-versus-TUI mouse handling.
The broader native terminal input/accessibility matrix is not rerun.

The [October 15 lean recheck](../verification/runs/2026-10-15-lean-attention.md)
records the current implementation and its exact coverage. Its
[setup attempts](../verification/runs/2026-10-15-lean-attention-attempts.md) preserve
the failed launch and observation limitations.

The [October 14 implementation recheck](../verification/runs/2026-10-14-agent-attention.md)
links exact build identities, automated checks and selected native observations.
The [failed attempts](../verification/runs/2026-10-14-agent-attention-attempts.md)
remain separate historical evidence.


The [plugin feasibility run](../verification/runs/2026-10-13-agent-attention-plugin.md)
confirms that executable Herdr event hooks work, but do not resolve the missing
attention facts. No terminal UI or native notification was exercised.

The [attention contract investigation](../verification/runs/2026-10-13-agent-attention-contract.md)
exercises the pinned backend's detector against controlled PTY fixtures. It
records false completion candidates from cancellation/fallback and the event
contract changes needed; native notification delivery remains untested.

No current product run was performed for this migration. Earlier
[research and audits](../archive/research/index.md) remain historical evidence for
their recorded snapshots; they do not certify this baseline.
