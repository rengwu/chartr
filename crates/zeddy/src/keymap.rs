//! User-global Chartr keybindings.
//!
//! The editable file is sparse, like Zed's keymap: omitted actions keep their
//! platform default. Settings records one chord at a time, rejects conflicts
//! in the shared `Chartr` context, and writes atomically. Bindings are loaded at
//! launch; Settings says so rather than pretending GPUI can remove one binding
//! without rebuilding the application keymap.

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
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    ToggleZoom,
    CommandPalette,
    OpenSettings,
}

impl KeymapAction {
    pub const ALL: [Self; 9] = [
        Self::CloseItem,
        Self::NewTerminal,
        Self::FocusLeft,
        Self::FocusRight,
        Self::FocusUp,
        Self::FocusDown,
        Self::ToggleZoom,
        Self::CommandPalette,
        Self::OpenSettings,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::CloseItem => "pane.close_active_item",
            Self::NewTerminal => "workspace.new_terminal",
            Self::FocusLeft => "workspace.activate_pane_left",
            Self::FocusRight => "workspace.activate_pane_right",
            Self::FocusUp => "workspace.activate_pane_up",
            Self::FocusDown => "workspace.activate_pane_down",
            Self::ToggleZoom => "workspace.toggle_zoom",
            Self::CommandPalette => "command_palette.toggle",
            Self::OpenSettings => "settings.open",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::CloseItem => "Close active item",
            Self::NewTerminal => "New terminal",
            Self::FocusLeft => "Focus pane left",
            Self::FocusRight => "Focus pane right",
            Self::FocusUp => "Focus pane up",
            Self::FocusDown => "Focus pane down",
            Self::ToggleZoom => "Toggle pane zoom",
            Self::CommandPalette => "Command palette",
            Self::OpenSettings => "Open Settings",
        }
    }

    pub fn default_key(self) -> &'static str {
        #[cfg(target_os = "macos")]
        return match self {
            Self::CloseItem => "cmd-w",
            Self::NewTerminal => "ctrl-~",
            Self::FocusLeft => "cmd-k cmd-left",
            Self::FocusRight => "cmd-k cmd-right",
            Self::FocusUp => "cmd-k cmd-up",
            Self::FocusDown => "cmd-k cmd-down",
            Self::ToggleZoom => "shift-escape",
            Self::CommandPalette => "cmd-shift-p",
            Self::OpenSettings => "cmd-,",
        };

        #[cfg(not(target_os = "macos"))]
        return match self {
            Self::CloseItem => "ctrl-w",
            Self::NewTerminal => "ctrl-~",
            Self::FocusLeft => "ctrl-k ctrl-left",
            Self::FocusRight => "ctrl-k ctrl-right",
            Self::FocusUp => "ctrl-k ctrl-up",
            Self::FocusDown => "ctrl-k ctrl-down",
            Self::ToggleZoom => "shift-escape",
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
                        "Chartr could not read {}, so default shortcuts are active: {error}",
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
                problem: Some(format!("Chartr could not read {}: {error}", file.display())),
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

    pub fn set(&mut self, action: KeymapAction, key: String) -> Result<(), Error> {
        validate_chord(&key)?;
        if let Some(conflict) = KeymapAction::ALL
            .into_iter()
            .find(|candidate| *candidate != action && self.key(*candidate) == key)
        {
            return Err(Error::Conflict { key, action: conflict });
        }
        if key == action.default_key() {
            self.content.bindings.remove(action.id());
        } else {
            self.content.bindings.insert(action.id().to_owned(), key);
        }
        self.save().map_err(Error::Write)
    }

    fn save(&self) -> io::Result<()> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        if let Some(problem) = &self.problem {
            return Err(io::Error::other(format!(
                "{problem}; Chartr will not overwrite a keymap it cannot read"
            )));
        }
        let parent = file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(
            toml::to_string_pretty(&self.content).map_err(io::Error::other)?.as_bytes(),
        )?;
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
    fn sparse_overrides_round_trip_and_defaults_remain() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(KEYMAP_FILE);
        let mut store = KeymapStore::load(&file);
        store.set(KeymapAction::CloseItem, "ctrl-alt-w".to_owned()).unwrap();
        let loaded = KeymapStore::load(file);
        assert_eq!(loaded.key(KeymapAction::CloseItem), "ctrl-alt-w");
        assert_eq!(loaded.key(KeymapAction::ToggleZoom), KeymapAction::ToggleZoom.default_key());
    }

    #[test]
    fn conflicts_in_the_chartr_context_are_refused() {
        let mut store = KeymapStore::bare();
        let key = store.key(KeymapAction::CloseItem).to_owned();
        let error = store.set(KeymapAction::NewTerminal, key).unwrap_err();
        assert!(matches!(error, Error::Conflict { action: KeymapAction::CloseItem, .. }));
    }
}
