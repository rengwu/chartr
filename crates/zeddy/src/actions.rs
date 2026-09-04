//! Semantic actions and Zed-compatible default key bindings.
//!
//! Keeping actions separate from handlers gives Chartr one command surface
//! for keymaps, buttons, menus, and the command palette.

use ::settings::{DEFAULT_KEYMAP_PATH, KeymapFile};
use gpui::{App, KeyBinding, Unbind};

use crate::keymap::{KeymapAction, KeymapStore};

pub mod pane {
    gpui::actions!(
        chartr_pane,
        [CloseActiveItem, CloseAllItems, JoinIntoNext, MoveLeft, MoveRight, MoveUp, MoveDown]
    );
}

pub mod workspace {
    gpui::actions!(
        chartr_workspace,
        [NewTerminal, ActivatePaneLeft, ActivatePaneRight, ActivatePaneUp, ActivatePaneDown]
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
    let context = Some("Chartr");
    cx.bind_keys([
        KeyBinding::new(keymap.key(KeymapAction::CloseItem), pane::CloseActiveItem, context),
        KeyBinding::new(keymap.key(KeymapAction::NewTerminal), workspace::NewTerminal, context),
        KeyBinding::new(keymap.key(KeymapAction::FocusLeft), workspace::ActivatePaneLeft, context),
        KeyBinding::new(
            keymap.key(KeymapAction::FocusRight),
            workspace::ActivatePaneRight,
            context,
        ),
        KeyBinding::new(keymap.key(KeymapAction::FocusUp), workspace::ActivatePaneUp, context),
        KeyBinding::new(keymap.key(KeymapAction::FocusDown), workspace::ActivatePaneDown, context),
        KeyBinding::new(keymap.key(KeymapAction::CommandPalette), command_palette::Toggle, context),
        KeyBinding::new(keymap.key(KeymapAction::OpenSettings), settings::Open, context),
    ]);

    // Keep terminal behavior aligned with the exact pinned Zed revision. The
    // full default keymap also contains editor/workspace bindings Chartr does
    // not own, so import only actions implemented by the terminal stack (plus
    // Select All, which TerminalView handles explicitly).
    cx.bind_keys(upstream_terminal_bindings(cx));

    #[cfg(target_os = "macos")]
    cx.bind_keys([KeyBinding::new("cmd-f", terminal_search::Toggle, Some("Terminal"))]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-shift-f", terminal_search::Toggle, Some("Terminal"))]);

    cx.bind_keys([
        KeyBinding::new("enter", terminal_search::Next, Some("ChartrTerminalSearch")),
        KeyBinding::new("shift-enter", terminal_search::Previous, Some("ChartrTerminalSearch")),
        KeyBinding::new("escape", terminal_search::Close, Some("ChartrTerminalSearch")),
    ]);

    // Chartr uses Ctrl+K as a pane chord on non-macOS platforms. Override it
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
    let context = Some("Chartr");
    match action {
        KeymapAction::CloseItem => {
            replace_binding(previous_key, new_key, pane::CloseActiveItem, context, cx)
        }
        KeymapAction::NewTerminal => {
            replace_binding(previous_key, new_key, workspace::NewTerminal, context, cx)
        }
        KeymapAction::FocusLeft => {
            replace_binding(previous_key, new_key, workspace::ActivatePaneLeft, context, cx)
        }
        KeymapAction::FocusRight => {
            replace_binding(previous_key, new_key, workspace::ActivatePaneRight, context, cx)
        }
        KeymapAction::FocusUp => {
            replace_binding(previous_key, new_key, workspace::ActivatePaneUp, context, cx)
        }
        KeymapAction::FocusDown => {
            replace_binding(previous_key, new_key, workspace::ActivatePaneDown, context, cx)
        }
        KeymapAction::CommandPalette => {
            replace_binding(previous_key, new_key, command_palette::Toggle, context, cx)
        }
        KeymapAction::OpenSettings => {
            replace_binding(previous_key, new_key, settings::Open, context, cx);
            replace_binding(previous_key, new_key, settings::Open, Some("ChartrSettings"), cx);
        }
    }
}

fn replace_binding(
    previous_key: &str,
    new_key: &str,
    action: impl gpui::Action,
    context: Option<&str>,
    cx: &mut App,
) {
    let action_name = action.name();
    cx.bind_keys([
        KeyBinding::new(previous_key, Unbind(action_name.into()), context),
        KeyBinding::new(new_key, action, context),
    ]);
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
#[cfg(not(target_os = "macos"))]
fn terminal_send_keystroke(keystroke: &str) -> terminal_view::SendKeystroke {
    serde_json::from_value(serde_json::Value::String(keystroke.to_owned()))
        .expect("Zed's terminal::SendKeystroke action accepts a string")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{KeyContext, Keystroke, TestAppContext};

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
            assert!(has_binding("alt-left", "terminal::SendText"));
            assert!(has_binding("alt-right", "terminal::SendText"));
            assert!(has_binding("shift-pageup", "terminal::ScrollPageUp"));

            #[cfg(target_os = "macos")]
            assert!(has_binding("cmd-v", "terminal::Paste"));
            #[cfg(not(target_os = "macos"))]
            assert!(has_binding("ctrl-shift-v", "terminal::Paste"));
        });
    }

    #[gpui::test]
    fn live_rebind_disables_the_previous_shortcut(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let previous_key = KeymapAction::CloseItem.default_key();
            cx.bind_keys([KeyBinding::new(previous_key, pane::CloseActiveItem, Some("Chartr"))]);

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
    fn open_settings_rebinds_in_both_application_contexts(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let previous_key = KeymapAction::OpenSettings.default_key();
            cx.bind_keys([
                KeyBinding::new(previous_key, settings::Open, Some("Chartr")),
                KeyBinding::new(previous_key, settings::Open, Some("ChartrSettings")),
            ]);

            rebind(KeymapAction::OpenSettings, previous_key, "ctrl-alt-s", cx);

            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let replacement = Keystroke::parse("ctrl-alt-s").unwrap();
            for context in ["Chartr", "ChartrSettings"] {
                let contexts = [KeyContext::parse(context).unwrap()];
                let (matches, pending) =
                    keymap.bindings_for_input(std::slice::from_ref(&replacement), &contexts);
                assert!(!pending);
                assert!(matches.iter().any(|binding| binding.action().partial_eq(&settings::Open)));
            }
        });
    }
}
