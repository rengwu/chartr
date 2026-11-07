---
title: Settings, themes and keybindings
section: Features
updated: 2026-11-07
basis: Source-reviewed migration baseline
---
# Settings, themes and keybindings

Feature ID: `FEAT-settings`

## Intent and basis

This record uses the inspected current implementation under
[the accepted migration decision](../decisions/index.md). It describes the
working-tree baseline, not a newly executed product result. See
[baseline identity and limitations](../verification/setup.md#baseline-and-build-identity).

Architectural basis: [ADR 0002](../decisions/adr/0002-the-zed-layer.md), [ADR 0005](../decisions/adr/0005-spaces-follow-zed-multi-workspace.md). Historical portions of ADRs remain dated rationale.

Implementation owners: [crates/chartr/src/settings.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/settings.rs), [crates/chartr/src/settings_window.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/settings_window.rs), [crates/chartr/src/settings_window/pages.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/settings_window/pages.rs), [crates/chartr/src/keymap.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/keymap.rs), [crates/chartr/src/components.rs](https://github.com/rengwu/chartr/blob/main/crates/chartr/src/components.rs).

Legacy story references: 5, 42, 44, 53, 66, 67, 68, 69, 70, 71, 72, 74, 75, 76, 77, 78, 80, 92. See the [complete crosswalk](story-crosswalk.md).

## Entry points and prerequisites

Open the single Settings window with the gear, command palette or Cmd/Ctrl+,. Navigate General, Appearance, Terminal, Hotkeys and Plugins.

Use [shared setup](../verification/setup.md) and a disposable profile/project for
stateful recipes. Related areas are linked from the [feature index](index.md).

## Expected outcomes

- Settings are user-global, persisted atomically and applied live where supported. Reopening focuses the existing window; closing Settings does not close an underlying terminal.
- General → View mode is the only control for switching Tabs/Spaces (accepted 2026-10-18). The title-header picker and its hide/show setting are removed. Existing `show_view_mode_picker` values are preserved as unknown settings and have no effect.
- Inactive pane tabs use a subtle background across all 15 built-in themes: 25% of the former inactive/card tint blended over the active pane surface. This reduces their visual emphasis while retaining theme hue and a slight distinction from the active tab (accepted 2026-10-19).
- VSCode Dark Modern uses neutral `#464646` structural borders, raised from `#2b2b2b` so pane dividers and tab outlines remain visible against its dark surfaces (accepted 2026-10-19).
- Defaults are chartr Dark, Geist for the interface and IBM Plex Mono for terminals; theme mode/pairs, fonts, UI/terminal scale and reduced motion are configurable.
- Terminal → Manual task names offers Keep until renamed or reset (default), or Reset for a new conversation. It persists as `terminal.manual_name_lifetime` (`keep` / `conversation`) and applies to subsequent observations without immediately clearing names. See [terminal naming behavior](terminals.md#terminal-task-names).
- Rename active terminal is assignable in Hotkeys and unbound by default.
- Hotkeys stores sparse overrides and rejects conflicts in the shared context. Unset actions retain platform defaults.
- Plugins contribute settings lazily. Skill sources and Saved Prompts are settings-only; restart-bound changes disclose their requirement.
- Middle-click tab closing and its dependent sidebar option follow their settings rather than acting unconditionally.

## Verification recipe

**Unvalidated recipe — not executed during this migration.** Complete shared
setup first; use actual UI entry points and observe underlying state where relevant.

1. Open Settings twice from different entry points and close with Cmd/Ctrl+W. Expect one window and no terminal termination.
2. Change theme, fonts, scale and Reduce Motion; observe workspace updates and relaunch persistence. Check light/dark contrast and narrow-window controls.
3. Rebind a harmless action, try a conflicting chord, then restore it. Expect persisted accepted edits and visible rejection of the conflict.
4. Exercise the parent middle-click option with/without its sidebar child, then disable it. Expect only enabled targets to close.
5. Visit each plugin settings page and use keyboard focus/page cycling; verify modal focus remains within the active dialog.

Preserve the selected scenario's logs/screenshots/file observations and exact
build identity before [cleanup](../verification/setup.md#cleanup). Record a fresh
run if this recipe is exercised; source inspection alone is not a pass.

## Coverage and limitations

Native focus, accessibility roles, font rendering, theme refresh and malformed-file recovery are not executed. Dirty keymap additions are included in the current source baseline.

The [release matrix](../verification/acceptance.md) retains broader cross-feature,
platform, theme, pointer and accessibility coverage. Choose a bounded subset for
ordinary changes; release requirements remain separate from this advisory workflow.

## Verification history

[0.3.3 preparation](../verification/runs/2026-11-07-release-0.3.3-recheck.md) records
bounded automated checks, isolated real-terminal checks and candidate package
verification under the user's request to preserve the desktop. Native coverage
limits remain explicit.

[October 30 task names](../verification/runs/2026-10-30-terminal-task-names.md) covers
the lifetime preference and rename command alongside naming behavior.

The [October 19 0.3.2 preparation](../verification/runs/2026-10-19-release-0.3.2.md)
records passing local tests, the macOS/Ubuntu/Arch CI matrix and all shipping
package builds for candidate `4afea26f`. Native notifications are explicitly
deferred; the complete native acceptance matrix remains unverified.

No current product run was performed for this migration. Earlier
[research and audits](../archive/research/index.md) remain historical evidence for
their recorded snapshots; they do not certify this baseline.
