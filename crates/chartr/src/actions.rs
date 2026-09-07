//! Semantic actions and Zed-compatible default key bindings.
//!
//! Keeping actions separate from handlers gives chartr one command surface
//! for keymaps, buttons, menus, and the command palette.

use ::settings::{DEFAULT_KEYMAP_PATH, KeymapFile};
use gpui::{App, KeyBinding, Unbind};

use crate::keymap::{KeymapAction, KeymapStore};

// Match at terminal depth as well, so upstream shell bindings cannot swallow
// chartr shortcuts. Use the same predicate when rebinding or clearing them.
const WORKSPACE_CONTEXT: &str = "chartr || (chartr > Terminal)";

pub mod pane {
    gpui::actions!(
        chartr_pane,
        [CloseActiveItem, CloseAllItems, JoinIntoNext, MoveLeft, MoveRight, MoveUp, MoveDown]
    );
}

pub mod workspace {
    gpui::actions!(
        chartr_workspace,
        [
            NewTerminal,
            NewTerminalPane,
            NewSurface,
            NewSurfacePane,
            Ungroup,
            SidebarMode,
            TabbedMode,
            CycleViewMode,
            NewSpace,
            CloseSpace,
            ZoomIn,
            ZoomOut,
            TerminalZoomIn,
            TerminalZoomOut,
            NewFreeTerminal,
            NewFreeSurface,
            ActivatePaneLeft,
            ActivatePaneRight,
            ActivatePaneUp,
            ActivatePaneDown
        ]
    );
}

pub mod command_palette {
    gpui::actions!(chartr_command_palette, [Toggle]);
}

pub mod settings {
    gpui::actions!(chartr_settings, [Open]);
}

pub mod terminal_search {
    gpui::actions!(chartr_terminal_search, [Toggle, Next, Previous, Close]);
}

pub fn init(keymap: &KeymapStore, cx: &mut App) {
    // Keep terminal behavior aligned with the exact pinned Zed revision. The
    // full default keymap also contains editor/workspace bindings chartr does
    // not own, so import only actions implemented by the terminal stack (plus
    // Select All, which TerminalView handles explicitly).
    cx.bind_keys(upstream_terminal_bindings(cx));

    cx.bind_keys(
        KeymapAction::ALL
            .into_iter()
            .filter(|action| !keymap.key(*action).is_empty())
            .map(|action| binding(action, keymap.key(action), WORKSPACE_CONTEXT)),
    );

    #[cfg(target_os = "macos")]
    cx.bind_keys([KeyBinding::new("cmd-f", terminal_search::Toggle, Some("Terminal"))]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-shift-f", terminal_search::Toggle, Some("Terminal"))]);

    cx.bind_keys([
        KeyBinding::new("enter", terminal_search::Next, Some("chartrTerminalSearch")),
        KeyBinding::new("shift-enter", terminal_search::Previous, Some("chartrTerminalSearch")),
        KeyBinding::new("escape", terminal_search::Close, Some("chartrTerminalSearch")),
    ]);

    // chartr uses Ctrl+K as a pane chord on non-macOS platforms. Override it
    // at Terminal context depth so shells still receive their conventional
    // kill-to-end-of-line command; the pane chord remains available elsewhere.
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-k", terminal_send_keystroke("ctrl-k"), Some("Terminal"))]);
}

/// Replace one user-editable binding in GPUI's live keymap.
///
/// GPUI bindings are append-only, with later entries taking precedence. A
/// targeted `Unbind` disables the prior action/chord without disturbing any
/// terminal, browser, or text-input bindings installed by other modules.
pub fn rebind(action: KeymapAction, previous_key: &str, new_key: &str, cx: &mut App) {
    let contexts: &[&str] = if action == KeymapAction::OpenSettings {
        &[WORKSPACE_CONTEXT, "chartrSettings"]
    } else {
        &[WORKSPACE_CONTEXT]
    };
    for context in contexts {
        if !previous_key.is_empty() {
            let previous = binding(action, previous_key, context);
            cx.bind_keys([KeyBinding::new(
                previous_key,
                Unbind(previous.action().name().into()),
                Some(context),
            )]);
        }
        if !new_key.is_empty() {
            cx.bind_keys([binding(action, new_key, context)]);
        }
    }
}

/// Keep startup and live rebinding on the same semantic action mapping.
fn binding(action: KeymapAction, key: &str, context: &str) -> KeyBinding {
    let context = Some(context);
    match action {
        KeymapAction::CloseItem => KeyBinding::new(key, pane::CloseActiveItem, context),
        KeymapAction::NewTerminal => KeyBinding::new(key, workspace::NewTerminal, context),
        KeymapAction::NewTerminalPane => KeyBinding::new(key, workspace::NewTerminalPane, context),
        KeymapAction::NewSurface => KeyBinding::new(key, workspace::NewSurface, context),
        KeymapAction::NewSurfacePane => KeyBinding::new(key, workspace::NewSurfacePane, context),
        KeymapAction::Ungroup => KeyBinding::new(key, workspace::Ungroup, context),
        KeymapAction::SidebarMode => KeyBinding::new(key, workspace::SidebarMode, context),
        KeymapAction::TabbedMode => KeyBinding::new(key, workspace::TabbedMode, context),
        KeymapAction::CycleViewMode => KeyBinding::new(key, workspace::CycleViewMode, context),
        KeymapAction::NewSpace => KeyBinding::new(key, workspace::NewSpace, context),
        KeymapAction::CloseSpace => KeyBinding::new(key, workspace::CloseSpace, context),
        KeymapAction::ZoomIn => KeyBinding::new(key, workspace::ZoomIn, context),
        KeymapAction::ZoomOut => KeyBinding::new(key, workspace::ZoomOut, context),
        KeymapAction::TerminalZoomIn => KeyBinding::new(key, workspace::TerminalZoomIn, context),
        KeymapAction::TerminalZoomOut => KeyBinding::new(key, workspace::TerminalZoomOut, context),
        KeymapAction::NewFreeTerminal => KeyBinding::new(key, workspace::NewFreeTerminal, context),
        KeymapAction::NewFreeSurface => KeyBinding::new(key, workspace::NewFreeSurface, context),
        KeymapAction::FocusLeft => KeyBinding::new(key, workspace::ActivatePaneLeft, context),
        KeymapAction::FocusRight => KeyBinding::new(key, workspace::ActivatePaneRight, context),
        KeymapAction::FocusUp => KeyBinding::new(key, workspace::ActivatePaneUp, context),
        KeymapAction::FocusDown => KeyBinding::new(key, workspace::ActivatePaneDown, context),
        KeymapAction::CommandPalette => KeyBinding::new(key, command_palette::Toggle, context),
        KeymapAction::OpenSettings => KeyBinding::new(key, settings::Open, context),
    }
}

fn upstream_terminal_bindings(cx: &App) -> Vec<KeyBinding> {
    KeymapFile::load_asset_allow_partial_failure(DEFAULT_KEYMAP_PATH, cx)
        .expect("the pinned Zed terminal keymap must remain loadable")
        .into_iter()
        .filter(|binding| {
            let action = binding.action().name();
            action.starts_with("terminal::") || action == "editor::SelectAll"
        })
        .collect()
}

/// Parameterized actions use the same serialized contract as Zed's keymap.
#[cfg(any(test, not(target_os = "macos")))]
fn terminal_send_keystroke(keystroke: &str) -> terminal_view::SendKeystroke {
    serde_json::from_value(serde_json::Value::String(keystroke.to_owned()))
        .expect("Zed's terminal::SendKeystroke action accepts a string")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{KeyContext, Keystroke, TestAppContext};

    #[gpui::test]
    fn defaults_dispatch_in_terminal_context_and_skip_unbound_actions(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let store = KeymapStore::bare();
            init(&store, cx);
            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let contexts =
                [KeyContext::parse("chartr").unwrap(), KeyContext::parse("Terminal").unwrap()];
            for action in KeymapAction::ALL {
                let key = store.key(action);
                if key.is_empty() {
                    let expected = binding(action, "ctrl-alt-z", "chartr");
                    assert_eq!(keymap.bindings_for_action(expected.action()).count(), 0);
                    continue;
                }
                let expected = binding(action, key, "chartr");
                let strokes = key
                    .split_whitespace()
                    .map(|key| Keystroke::parse(key).unwrap())
                    .collect::<Vec<_>>();
                let (matches, pending) = keymap.bindings_for_input(&strokes, &contexts);
                assert!(!pending, "{key}");
                assert!(
                    matches
                        .first()
                        .is_some_and(|matched| matched.action().partial_eq(expected.action())),
                    "{action:?}: {key}"
                );
            }
            #[cfg(not(target_os = "macos"))]
            {
                let (matches, pending) =
                    keymap.bindings_for_input(&[Keystroke::parse("ctrl-k").unwrap()], &contexts);
                assert!(!pending, "Ctrl+K must reach the shell without waiting for a pane chord");
                assert!(matches[0].action().partial_eq(&terminal_send_keystroke("ctrl-k")));
            }
        });
    }

    #[gpui::test]
    fn unbound_action_can_be_bound_cleared_and_rebound_live(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let action = KeymapAction::NewFreeTerminal;
            rebind(action, "", "ctrl-alt-z", cx);
            assert_eq!(
                cx.key_bindings().borrow().bindings_for_action(&workspace::NewFreeTerminal).count(),
                1
            );
            rebind(action, "ctrl-alt-z", "", cx);
            assert_eq!(
                cx.key_bindings().borrow().bindings_for_action(&workspace::NewFreeTerminal).count(),
                0
            );
            rebind(action, "", "ctrl-alt-z", cx);
            assert_eq!(
                cx.key_bindings().borrow().bindings_for_action(&workspace::NewFreeTerminal).count(),
                1
            );
        });
    }

    #[gpui::test]
    fn imports_the_pinned_zed_terminal_keymap(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let bindings = upstream_terminal_bindings(cx);
            let actions =
                bindings.iter().map(|binding| binding.action().name()).collect::<Vec<_>>();
            let has_binding = |key: &str, action: &str| {
                let key = gpui::Keystroke::parse(key).unwrap();
                bindings.iter().any(|binding| {
                    binding.action().name() == action
                        && binding.match_keystrokes(std::slice::from_ref(&key)) == Some(false)
                })
            };

            assert!(actions.contains(&"terminal::Copy"));
            assert!(actions.contains(&"terminal::Paste"));
            assert!(actions.contains(&"terminal::SendText"));
            assert!(has_binding("shift-pageup", "terminal::ScrollPageUp"));

            #[cfg(target_os = "macos")]
            {
                assert!(has_binding("alt-left", "terminal::SendText"));
                assert!(has_binding("alt-right", "terminal::SendText"));
                assert!(has_binding("cmd-v", "terminal::Paste"));
            }
            #[cfg(not(target_os = "macos"))]
            {
                assert!(has_binding("alt-b", "terminal::SendText"));
                assert!(has_binding("alt-f", "terminal::SendText"));
                assert!(has_binding("ctrl-shift-v", "terminal::Paste"));
            }
        });
    }

    #[gpui::test]
    fn live_rebind_disables_the_previous_shortcut(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let previous_key = KeymapAction::CloseItem.default_key();
            cx.bind_keys([binding(KeymapAction::CloseItem, previous_key, WORKSPACE_CONTEXT)]);

            rebind(KeymapAction::CloseItem, previous_key, "ctrl-alt-w", cx);

            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let active = keymap.bindings_for_action(&pane::CloseActiveItem).collect::<Vec<_>>();
            let previous = Keystroke::parse(previous_key).unwrap();
            let replacement = Keystroke::parse("ctrl-alt-w").unwrap();
            assert_eq!(active.len(), 1);
            assert_eq!(active[0].match_keystrokes(&[replacement]), Some(false));
            assert!(
                active
                    .iter()
                    .all(|binding| binding.match_keystrokes(std::slice::from_ref(&previous))
                        != Some(false))
            );
        });
    }

    #[gpui::test]
    fn rebinding_overrides_terminal_passthrough_and_clearing_restores_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let passthrough = terminal_send_keystroke("ctrl-w");
            cx.bind_keys([KeyBinding::new("ctrl-w", passthrough.clone(), Some("Terminal"))]);
            let action = KeymapAction::CloseItem;
            let stroke = [Keystroke::parse("ctrl-w").unwrap()];
            let contexts =
                [KeyContext::parse("chartr").unwrap(), KeyContext::parse("Terminal").unwrap()];

            rebind(action, "", "ctrl-w", cx);
            let keymap = cx.key_bindings();
            let (matches, pending) = keymap.borrow().bindings_for_input(&stroke, &contexts);
            assert!(!pending);
            assert!(matches[0].action().partial_eq(&pane::CloseActiveItem));

            rebind(action, "ctrl-w", "ctrl-alt-w", cx);
            let (matches, pending) = keymap.borrow().bindings_for_input(&stroke, &contexts);
            assert!(!pending);
            assert!(matches[0].action().partial_eq(&passthrough));

            rebind(action, "ctrl-alt-w", "", cx);
            assert_eq!(keymap.borrow().bindings_for_action(&pane::CloseActiveItem).count(), 0);
        });
    }

    #[gpui::test]
    fn open_settings_rebinds_in_both_application_contexts(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let previous_key = KeymapAction::OpenSettings.default_key();
            cx.bind_keys([
                binding(KeymapAction::OpenSettings, previous_key, WORKSPACE_CONTEXT),
                KeyBinding::new(previous_key, settings::Open, Some("chartrSettings")),
            ]);

            rebind(KeymapAction::OpenSettings, previous_key, "ctrl-alt-s", cx);

            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let replacement = Keystroke::parse("ctrl-alt-s").unwrap();
            for context in ["chartr", "chartrSettings"] {
                let contexts = [KeyContext::parse(context).unwrap()];
                let (matches, pending) =
                    keymap.bindings_for_input(std::slice::from_ref(&replacement), &contexts);
                assert!(!pending);
                assert!(matches.iter().any(|binding| binding.action().partial_eq(&settings::Open)));
            }
        });
    }
}
