//! Versioned SQLite persistence for application-owned cockpit state.
//!
//! User-editable settings, keymaps, and themes remain files. This database
//! stores only the state Chartr owns: spaces, pane trees, restorable item
//! identities, chrome state, and window geometry.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use rusqlite::{Connection, OptionalExtension as _, params};
use serde::{Deserialize, Serialize};

use crate::{mode::Mode, workspace::WorkspaceTabs};

pub const STATE_FILE: &str = "state.sqlite";
const SCHEMA_VERSION: i64 = 1;
const IMPLICIT_ROOT_CLEANUP: &str = "migration.implicit-root-space";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidebarScope {
    #[default]
    AllSpaces,
    ActiveSpace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub chrome: Mode,
    pub sidebar_scope: SidebarScope,
    pub sidebar_width: f32,
    pub active_space: Option<String>,
    pub bounds: Option<WindowBounds>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            chrome: Mode::Sidebar,
            sidebar_scope: SidebarScope::AllSpaces,
            sidebar_width: 280.,
            active_space: Some("ad-hoc".to_owned()),
            bounds: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceKind {
    AdHoc,
    Folder,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedSpace {
    pub key: String,
    pub name: String,
    pub path: Option<PathBuf>,
    pub kind: SpaceKind,
    pub layout: WorkspaceTabs,
    pub items: Vec<PersistedItem>,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedItem {
    Terminal {
        item_id: u64,
        backend_id: String,
    },
    Plugin {
        item_id: u64,
        plugin: String,
        pane: String,
        state: Option<String>,
        bound_session: Option<String>,
    },
}

impl PersistedItem {
    pub fn item_id(&self) -> u64 {
        match self {
            Self::Terminal { item_id, .. } | Self::Plugin { item_id, .. } => *item_id,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub window: WindowState,
    pub spaces: Vec<PersistedSpace>,
}

pub struct StateStore {
    connection: Connection,
}

impl StateStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating state directory {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("opening state database {}", path.display()))?;
        let mut store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    #[cfg(test)]
    fn memory() -> Result<Self> {
        let connection = Connection::open_in_memory()?;
        let mut store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let version: i64 =
            self.connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            anyhow::bail!(
                "state database schema {version} is newer than this Chartr supports ({SCHEMA_VERSION})"
            );
        }
        if version == 0 {
            let transaction = self.connection.transaction()?;
            transaction.execute_batch(
                "CREATE TABLE app_state (
                    key TEXT PRIMARY KEY NOT NULL,
                    value_json TEXT NOT NULL
                );
                CREATE TABLE spaces (
                    space_key TEXT PRIMARY KEY NOT NULL,
                    ordinal INTEGER NOT NULL,
                    value_json TEXT NOT NULL
                );",
            )?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            transaction.commit()?;
        }
        Ok(())
    }

    pub fn load(&self) -> Result<Snapshot> {
        let window = self
            .connection
            .query_row("SELECT value_json FROM app_state WHERE key = 'window'", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .context("decoding saved window state")?
            .unwrap_or_default();
        let mut statement =
            self.connection.prepare("SELECT value_json FROM spaces ORDER BY ordinal")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut spaces = Vec::new();
        for row in rows {
            spaces.push(serde_json::from_str(&row?).context("decoding a saved space")?);
        }
        Ok(Snapshot { window, spaces })
    }

    pub fn save(&mut self, snapshot: &Snapshot) -> Result<()> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO app_state (key, value_json) VALUES ('window', ?1)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            [serde_json::to_string(&snapshot.window)?],
        )?;
        transaction.execute("DELETE FROM spaces", [])?;
        {
            let mut insert = transaction.prepare(
                "INSERT INTO spaces (space_key, ordinal, value_json) VALUES (?1, ?2, ?3)",
            )?;
            for (ordinal, space) in snapshot.spaces.iter().enumerate() {
                insert.execute(params![
                    space.key,
                    ordinal as i64,
                    serde_json::to_string(space)?
                ])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    /// Whether this installation still needs the one-time cleanup for builds
    /// that mistook a desktop launcher's `/` working directory for a project.
    pub fn implicit_root_cleanup_pending(&self) -> Result<bool> {
        let completed = self
            .connection
            .query_row(
                "SELECT 1 FROM app_state WHERE key = ?1",
                [IMPLICIT_ROOT_CLEANUP],
                |_| Ok(()),
            )
            .optional()?;
        Ok(completed.is_none())
    }

    pub fn complete_implicit_root_cleanup(&mut self) -> Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO app_state (key, value_json) VALUES (?1, 'true')",
            [IMPLICIT_ROOT_CLEANUP],
        )?;
        Ok(())
    }

    #[cfg(test)]
    fn schema_version(&self) -> Result<i64> {
        Ok(self.connection.pragma_query_value(None, "user_version", |row| row.get(0))?)
    }
}

pub fn state_file() -> Result<PathBuf> {
    state_root_from(std::env::var_os("XDG_STATE_HOME"), std::env::home_dir())
        .map(|root| root.join(STATE_FILE))
}

fn state_root_from(xdg: Option<OsString>, home: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(xdg) = xdg.filter(|value| Path::new(value).is_absolute()) {
        return Ok(PathBuf::from(xdg).join("chartr-zeddy"));
    }
    home.filter(|path| !path.as_os_str().is_empty())
        .map(|home| home.join(".local/state/chartr-zeddy"))
        .context("no state directory is available")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(key: &str) -> PersistedSpace {
        let mut layout = WorkspaceTabs::new();
        let item = layout.alloc_item();
        layout.push_standalone(item).unwrap();
        let tab = layout.active_tab_id().unwrap();
        let root = layout.active_workspace().unwrap().active_pane();
        let right = layout
            .workspace_mut(tab)
            .unwrap()
            .split_pane(root, crate::workspace::SplitDirection::Right)
            .unwrap();
        layout.workspace_mut(tab).unwrap().move_item(item, right, None).unwrap();
        PersistedSpace {
            key: key.to_owned(),
            name: "Project".to_owned(),
            path: Some(PathBuf::from("/tmp/project")),
            kind: SpaceKind::Folder,
            layout,
            items: vec![PersistedItem::Terminal {
                item_id: item.get(),
                backend_id: "pane-1".to_owned(),
            }],
            expanded: true,
        }
    }

    #[test]
    fn a_new_database_runs_the_versioned_schema() {
        let store = StateStore::memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn complete_snapshots_round_trip_in_space_order() {
        let mut store = StateStore::memory().unwrap();
        let snapshot = Snapshot {
            window: WindowState {
                chrome: Mode::Tabs,
                sidebar_width: 312.,
                active_space: Some("two".to_owned()),
                ..WindowState::default()
            },
            spaces: vec![space("one"), space("two")],
        };
        store.save(&snapshot).unwrap();
        let restored = store.load().unwrap();
        assert_eq!(restored, snapshot);
        restored.spaces[0].layout.validate().unwrap();
    }

    #[test]
    fn saving_is_a_transaction_that_replaces_removed_spaces() {
        let mut store = StateStore::memory().unwrap();
        store
            .save(&Snapshot { spaces: vec![space("one"), space("two")], ..Snapshot::default() })
            .unwrap();
        store.save(&Snapshot { spaces: vec![space("two")], ..Snapshot::default() }).unwrap();
        assert_eq!(store.load().unwrap().spaces[0].key, "two");
        assert_eq!(store.load().unwrap().spaces.len(), 1);
    }

    #[test]
    fn one_time_migrations_have_an_explicit_completion_marker() {
        let mut store = StateStore::memory().unwrap();
        assert!(store.implicit_root_cleanup_pending().unwrap());
        store.complete_implicit_root_cleanup().unwrap();
        assert!(!store.implicit_root_cleanup_pending().unwrap());
    }

    #[test]
    fn state_paths_are_isolated_from_old_chartr() {
        assert_eq!(
            state_root_from(Some(OsString::from("/state")), None).unwrap(),
            PathBuf::from("/state/chartr-zeddy")
        );
        assert_eq!(
            state_root_from(None, Some(PathBuf::from("/home/op"))).unwrap(),
            PathBuf::from("/home/op/.local/state/chartr-zeddy")
        );
    }
}
