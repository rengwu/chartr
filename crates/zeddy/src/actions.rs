//! Semantic actions and Zed-compatible default key bindings.
//!
//! Keeping actions separate from handlers gives Chartr one command surface
//! for keymaps, buttons, menus, and the command palette.

use gpui::{App, KeyBinding};

use crate::keymap::{KeymapAction, KeymapStore};

pub mod pane {
    gpui::actions!(
        pane,
        [
            CloseActiveItem,
            CloseAllItems,
            JoinIntoNext,
            SplitAndMoveLeft,
            SplitAndMoveRight,
            SplitAndMoveUp,
            SplitAndMoveDown,
            MoveLeft,
            MoveRight,
            MoveUp,
            MoveDown
        ]
    );
}

pub mod workspace {
    gpui::actions!(
        workspace,
        [
            NewTerminal,
            ActivatePaneLeft,
            ActivatePaneRight,
            ActivatePaneUp,
            ActivatePaneDown,
            ToggleZoom
        ]
    );
}

pub mod command_palette {
    gpui::actions!(command_palette, [Toggle]);
}

pub mod settings {
    gpui::actions!(settings, [Open]);
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
        KeyBinding::new(keymap.key(KeymapAction::ToggleZoom), workspace::ToggleZoom, context),
        KeyBinding::new(keymap.key(KeymapAction::CommandPalette), command_palette::Toggle, context),
        KeyBinding::new(keymap.key(KeymapAction::OpenSettings), settings::Open, context),
    ]);
}
