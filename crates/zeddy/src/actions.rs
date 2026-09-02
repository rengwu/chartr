//! Semantic actions and Zed-compatible default key bindings.
//!
//! Keeping actions separate from handlers gives Chartr one command surface
//! for keymaps, buttons, menus, and the command palette.

use ::settings::{DEFAULT_KEYMAP_PATH, KeymapFile};
use gpui::{App, KeyBinding};

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
    use gpui::TestAppContext;

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
}
