//! User-global chartr keybindings.
//!
//! The editable file is sparse, like Zed's keymap: omitted actions keep their
//! platform default. Settings records one chord at a time, rejects conflicts
//! in the shared `chartr` context, writes atomically, and updates GPUI's live
//! keymap after each successful edit.

use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub const KEYMAP_FILE: &str = "keymap.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeymapAction {
    CloseItem,
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
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    CommandPalette,
    OpenSettings,
}

impl KeymapAction {
    pub const ALL: [Self; 23] = [
        Self::CloseItem,
        Self::NewTerminal,
        Self::NewTerminalPane,
        Self::NewSurface,
        Self::NewSurfacePane,
        Self::Ungroup,
        Self::SidebarMode,
        Self::TabbedMode,
        Self::CycleViewMode,
        Self::NewSpace,
        Self::CloseSpace,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::TerminalZoomIn,
        Self::TerminalZoomOut,
        Self::NewFreeTerminal,
        Self::NewFreeSurface,
        Self::FocusLeft,
        Self::FocusRight,
        Self::FocusUp,
        Self::FocusDown,
        Self::CommandPalette,
        Self::OpenSettings,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::CloseItem => "pane.close_active_item",
            Self::NewTerminal => "workspace.new_terminal",
            Self::NewTerminalPane => "workspace.new_terminal_pane",
            Self::NewSurface => "workspace.new_surface",
            Self::NewSurfacePane => "workspace.new_surface_pane",
            Self::Ungroup => "workspace.ungroup",
            Self::SidebarMode => "workspace.sidebar_mode",
            Self::TabbedMode => "workspace.tabbed_mode",
            Self::CycleViewMode => "workspace.cycle_view_mode",
            Self::NewSpace => "workspace.new_space",
            Self::CloseSpace => "workspace.close_space",
            Self::ZoomIn => "workspace.zoom_in",
            Self::ZoomOut => "workspace.zoom_out",
            Self::TerminalZoomIn => "workspace.terminal_zoom_in",
            Self::TerminalZoomOut => "workspace.terminal_zoom_out",
            Self::NewFreeTerminal => "workspace.new_free_terminal",
            Self::NewFreeSurface => "workspace.new_free_surface",
            Self::FocusLeft => "workspace.activate_pane_left",
            Self::FocusRight => "workspace.activate_pane_right",
            Self::FocusUp => "workspace.activate_pane_up",
            Self::FocusDown => "workspace.activate_pane_down",
            Self::CommandPalette => "command_palette.toggle",
            Self::OpenSettings => "settings.open",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::CloseItem => "Close active item",
            Self::NewTerminal => "New terminal",
            Self::NewTerminalPane => "New terminal pane",
            Self::NewSurface => "New surface tab",
            Self::NewSurfacePane => "New surface pane",
            Self::Ungroup => "Ungroup current group",
            Self::SidebarMode => "Switch to sidebar mode",
            Self::TabbedMode => "Switch to tabbed mode",
            Self::CycleViewMode => "Cycle view modes",
            Self::NewSpace => "Open new space",
            Self::CloseSpace => "Close current space",
            Self::ZoomIn => "Zoom in interface",
            Self::ZoomOut => "Zoom out interface",
            Self::TerminalZoomIn => "Zoom in terminal",
            Self::TerminalZoomOut => "Zoom out terminal",
            Self::NewFreeTerminal => "New free terminal session",
            Self::NewFreeSurface => "New free surface",
            Self::FocusLeft => "Focus pane left",
            Self::FocusRight => "Focus pane right",
            Self::FocusUp => "Focus pane up",
            Self::FocusDown => "Focus pane down",
            Self::CommandPalette => "Command palette",
            Self::OpenSettings => "Open Settings",
        }
    }

    pub fn default_key(self) -> &'static str {
        // GPUI's macOS/Linux backends fold Shift into punctuation (e.g. Shift+2
        // arrives as @). Bind the emitted character, including the shifted
        // =/+ key for terminal zoom, so it stays distinct from interface zoom.
        #[cfg(target_os = "macos")]
        return match self {
            Self::CloseItem => "cmd-w",
            Self::NewTerminal => "ctrl-~",
            Self::NewTerminalPane => "cmd-shift-t",
            Self::NewSurface => "cmd-n",
            Self::NewSurfacePane => "cmd-shift-n",
            Self::Ungroup => "cmd-shift-g",
            Self::SidebarMode => "cmd-@",
            Self::TabbedMode => "cmd-!",
            Self::CycleViewMode => "cmd-~",
            Self::NewSpace => "cmd-o",
            Self::CloseSpace => "cmd-shift-w",
            Self::ZoomIn => "cmd-=",
            Self::ZoomOut => "cmd--",
            Self::TerminalZoomIn => "cmd-+",
            Self::TerminalZoomOut => "cmd-_",
            Self::NewFreeTerminal => "",
            Self::NewFreeSurface => "",
            Self::FocusLeft => "cmd-k cmd-left",
            Self::FocusRight => "cmd-k cmd-right",
            Self::FocusUp => "cmd-k cmd-up",
            Self::FocusDown => "cmd-k cmd-down",
            Self::CommandPalette => "cmd-shift-p",
            Self::OpenSettings => "cmd-,",
        };

        #[cfg(not(target_os = "macos"))]
        return match self {
            Self::CloseItem => "ctrl-w",
            Self::NewTerminal => "ctrl-~",
            Self::NewTerminalPane => "ctrl-shift-t",
            Self::NewSurface => "ctrl-n",
            Self::NewSurfacePane => "ctrl-shift-n",
            Self::Ungroup => "ctrl-shift-g",
            Self::SidebarMode => "ctrl-@",
            Self::TabbedMode => "ctrl-!",
            // Ctrl+~ already opens a terminal on Linux.
            Self::CycleViewMode => "ctrl-alt-~",
            Self::NewSpace => "ctrl-o",
            Self::CloseSpace => "ctrl-shift-w",
            Self::ZoomIn => "ctrl-=",
            Self::ZoomOut => "ctrl--",
            Self::TerminalZoomIn => "ctrl-+",
            Self::TerminalZoomOut => "ctrl-_",
            Self::NewFreeTerminal => "",
            Self::NewFreeSurface => "",
            Self::FocusLeft => "ctrl-k ctrl-left",
            Self::FocusRight => "ctrl-k ctrl-right",
            Self::FocusUp => "ctrl-k ctrl-up",
            Self::FocusDown => "ctrl-k ctrl-down",
            Self::CommandPalette => "ctrl-shift-p",
            Self::OpenSettings => "ctrl-,",
        };
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Content {
    #[serde(default)]
    bindings: BTreeMap<String, String>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone)]
pub struct KeymapStore {
    file: Option<PathBuf>,
    content: Content,
    problem: Option<String>,
}

impl gpui::Global for KeymapStore {}

impl KeymapStore {
    pub fn load(file: impl Into<PathBuf>) -> Self {
        let file = file.into();
        match fs::read_to_string(&file) {
            Ok(text) => match toml::from_str(&text) {
                Ok(content) => Self { file: Some(file), content, problem: None },
                Err(error) => Self {
                    file: Some(file.clone()),
                    content: Content::default(),
                    problem: Some(format!(
                        "chartr could not read {}, so default shortcuts are active: {error}",
                        file.display()
                    )),
                },
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Self { file: Some(file), content: Content::default(), problem: None }
            }
            Err(error) => Self {
                file: Some(file.clone()),
                content: Content::default(),
                problem: Some(format!("chartr could not read {}: {error}", file.display())),
            },
        }
    }

    pub fn bare() -> Self {
        Self { file: None, content: Content::default(), problem: None }
    }

    pub fn key(&self, action: KeymapAction) -> &str {
        self.content
            .bindings
            .get(action.id())
            .map(String::as_str)
            .unwrap_or_else(|| action.default_key())
    }

    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    pub fn shortcut_label(&self, action: KeymapAction) -> &str {
        let key = self.key(action);
        match key {
            "" => "Unbound",
            #[cfg(target_os = "macos")]
            "cmd-@" => "cmd-shift-2",
            #[cfg(target_os = "macos")]
            "cmd-!" => "cmd-shift-1",
            #[cfg(target_os = "macos")]
            "cmd-~" => "cmd-shift-`",
            #[cfg(target_os = "macos")]
            "cmd-+" => "cmd-shift-=",
            #[cfg(target_os = "macos")]
            "cmd-_" => "cmd-shift--",
            #[cfg(not(target_os = "macos"))]
            "ctrl-@" => "ctrl-shift-2",
            #[cfg(not(target_os = "macos"))]
            "ctrl-!" => "ctrl-shift-1",
            #[cfg(not(target_os = "macos"))]
            "ctrl-alt-~" => "ctrl-alt-shift-`",
            #[cfg(not(target_os = "macos"))]
            "ctrl-+" => "ctrl-shift-=",
            #[cfg(not(target_os = "macos"))]
            "ctrl-_" => "ctrl-shift--",
            _ => key,
        }
    }

    pub fn set(&mut self, action: KeymapAction, key: String) -> Result<(), Error> {
        if !key.is_empty() {
            validate_chord(&key)?;
        }
        if let Some(conflict) = KeymapAction::ALL.into_iter().find(|candidate| {
            !key.is_empty() && *candidate != action && self.key(*candidate) == key
        }) {
            return Err(Error::Conflict { key, action: conflict });
        }
        let mut candidate = self.content.clone();
        if key == action.default_key() {
            candidate.bindings.remove(action.id());
        } else {
            candidate.bindings.insert(action.id().to_owned(), key);
        }
        self.save(&candidate).map_err(Error::Write)?;
        self.content = candidate;
        Ok(())
    }

    fn save(&self, content: &Content) -> io::Result<()> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        if let Some(problem) = &self.problem {
            return Err(io::Error::other(format!(
                "{problem}; chartr will not overwrite a keymap it cannot read"
            )));
        }
        let parent = file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(toml::to_string_pretty(content).map_err(io::Error::other)?.as_bytes())?;
        staged.flush()?;
        staged.as_file().sync_all()?;
        staged.persist(file).map_err(|error| error.error)?;
        Ok(())
    }
}

fn validate_chord(chord: &str) -> Result<(), Error> {
    if chord.trim().is_empty() {
        return Err(Error::Invalid(chord.to_owned()));
    }
    for stroke in chord.split_whitespace() {
        gpui::Keystroke::parse(stroke).map_err(|_| Error::Invalid(chord.to_owned()))?;
    }
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Invalid(String),
    Conflict { key: String, action: KeymapAction },
    Write(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(key) => write!(formatter, "{key:?} is not a valid shortcut"),
            Self::Conflict { key, action } => {
                write!(formatter, "{key} is already assigned to {}", action.title())
            }
            Self::Write(error) => write!(formatter, "saving the keymap: {error}"),
        }
    }
}

impl std::error::Error for Error {}

pub fn keymap_file() -> Result<PathBuf, crate::spaces::Error> {
    Ok(crate::spaces::config_root()?.join(KEYMAP_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_unique_chords_with_two_unbound_commands() {
        let mut keys = std::collections::HashSet::new();
        let mut ids = std::collections::HashSet::new();
        for action in KeymapAction::ALL {
            assert!(ids.insert(action.id()));
            let key = action.default_key();
            if key.is_empty() {
                assert!(matches!(
                    action,
                    KeymapAction::NewFreeTerminal | KeymapAction::NewFreeSurface
                ));
            } else {
                validate_chord(key).unwrap();
                assert!(keys.insert(key), "duplicate default: {key}");
            }
        }
        assert_eq!(keys.len(), KeymapAction::ALL.len() - 2);
    }

    #[test]
    fn unbound_commands_can_be_assigned_cleared_and_restored() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(KEYMAP_FILE);
        let mut store = KeymapStore::load(&file);
        for action in [KeymapAction::NewFreeTerminal, KeymapAction::NewFreeSurface] {
            assert_eq!(store.shortcut_label(action), "Unbound");
            store.set(action, "ctrl-alt-z".into()).unwrap();
            assert_eq!(KeymapStore::load(&file).key(action), "ctrl-alt-z");
            store.set(action, String::new()).unwrap();
            assert_eq!(KeymapStore::load(&file).key(action), "");
        }
        store.set(KeymapAction::NewSurface, String::new()).unwrap();
        assert_eq!(KeymapStore::load(file).key(KeymapAction::NewSurface), "");
    }

    #[test]
    fn sparse_overrides_round_trip_and_defaults_remain() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(KEYMAP_FILE);
        let mut store = KeymapStore::load(&file);
        store.set(KeymapAction::CloseItem, "ctrl-alt-w".to_owned()).unwrap();
        let loaded = KeymapStore::load(file);
        assert_eq!(loaded.key(KeymapAction::CloseItem), "ctrl-alt-w");
        assert_eq!(loaded.key(KeymapAction::FocusLeft), KeymapAction::FocusLeft.default_key());
    }

    #[test]
    fn conflicts_in_the_chartr_context_are_refused() {
        let mut store = KeymapStore::bare();
        let key = store.key(KeymapAction::CloseItem).to_owned();
        let error = store.set(KeymapAction::NewTerminal, key).unwrap_err();
        assert!(matches!(error, Error::Conflict { action: KeymapAction::CloseItem, .. }));
    }

    #[test]
    fn failed_writes_preserve_overrides_and_do_not_reserve_rejected_keys() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(KEYMAP_FILE);
        let mut store = KeymapStore::load(&file);
        store.set(KeymapAction::CloseItem, "ctrl-alt-w".into()).unwrap();
        let original = fs::read_to_string(&file).unwrap();
        // A directory at the destination forces atomic replacement to fail,
        // including when the tests run as a privileged user.
        let backup = temp.path().join("original.toml");
        fs::rename(&file, &backup).unwrap();
        fs::create_dir(&file).unwrap();
        for key in ["ctrl-alt-z", KeymapAction::CloseItem.default_key()] {
            assert!(matches!(store.set(KeymapAction::CloseItem, key.into()), Err(Error::Write(_))));
            assert_eq!(store.key(KeymapAction::CloseItem), "ctrl-alt-w");
        }
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        fs::remove_dir(&file).unwrap();
        fs::rename(backup, &file).unwrap();
        store.set(KeymapAction::NewTerminal, "ctrl-alt-z".into()).unwrap();
        let loaded = KeymapStore::load(&file);
        assert_eq!(loaded.key(KeymapAction::CloseItem), "ctrl-alt-w");
        assert_eq!(loaded.key(KeymapAction::NewTerminal), "ctrl-alt-z");
        store.set(KeymapAction::CloseItem, KeymapAction::CloseItem.default_key().into()).unwrap();
        assert!(
            !KeymapStore::load(file).content.bindings.contains_key(KeymapAction::CloseItem.id())
        );
    }

    #[test]
    fn unreadable_keymaps_keep_defaults_and_are_never_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(KEYMAP_FILE);
        fs::write(&file, "[invalid").unwrap();
        let mut store = KeymapStore::load(&file);
        assert!(store.set(KeymapAction::CloseItem, "ctrl-alt-w".into()).is_err());
        assert_eq!(store.key(KeymapAction::CloseItem), KeymapAction::CloseItem.default_key());
        assert_eq!(fs::read_to_string(file).unwrap(), "[invalid");
    }
}
