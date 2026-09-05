//! Versioned SQLite persistence for application-owned cockpit state.
//!
//! User-editable settings, keymaps, and themes remain files. This database
//! stores only the state Chartr owns: spaces, pane trees, restorable item
//! identities, chrome state, and window geometry.

use std::{
    collections::HashSet,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context as _, Result};
use rusqlite::{Connection, OptionalExtension as _, params};
use serde::{Deserialize, Serialize};

use crate::{mode::Mode, workspace::WorkspaceTabs};

pub const STATE_FILE: &str = "state.sqlite";
const SCHEMA_VERSION: i64 = 1;
const IMPLICIT_ROOT_CLEANUP: &str = "migration.implicit-root-space";

const fn default_show_space_picker() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub chrome: Mode,
    #[serde(default = "default_show_space_picker")]
    pub show_space_picker: bool,
    pub sidebar_width: f32,
    pub active_space: Option<String>,
    pub bounds: Option<WindowBounds>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            chrome: Mode::Sidebar,
            show_space_picker: false,
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

/// Serializes database access off-thread. Revisions also protect the final
/// synchronous shutdown flush from older work still queued on the executor.
pub struct StateWriter {
    state: Arc<Mutex<WriterState>>,
    revision: u64,
}

struct WriterState {
    store: StateStore,
    saved: Snapshot,
    revision: u64,
}

pub struct SaveRequest {
    state: Arc<Mutex<WriterState>>,
    snapshot: Snapshot,
    revision: u64,
}

impl StateWriter {
    pub fn new(store: StateStore, saved: Snapshot) -> Self {
        Self { state: Arc::new(Mutex::new(WriterState { store, saved, revision: 0 })), revision: 0 }
    }

    pub fn request(&mut self, snapshot: Snapshot) -> SaveRequest {
        self.revision += 1;
        SaveRequest { state: self.state.clone(), snapshot, revision: self.revision }
    }
}

impl SaveRequest {
    pub fn save(self) -> Result<()> {
        let mut state =
            self.state.lock().map_err(|_| anyhow::anyhow!("state writer lock poisoned"))?;
        if self.revision <= state.revision {
            return Ok(());
        }
        state.revision = self.revision;
        if self.snapshot != state.saved {
            state.store.save(&self.snapshot)?;
            // Failed saves must remain retryable; publish only after commit.
            state.saved = self.snapshot;
        }
        Ok(())
    }
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
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json
             WHERE app_state.value_json != excluded.value_json",
            [serde_json::to_string(&snapshot.window)?],
        )?;
        let keys: HashSet<_> = snapshot.spaces.iter().map(|space| space.key.as_str()).collect();
        let stored_keys = transaction
            .prepare("SELECT space_key FROM spaces")?
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for key in stored_keys {
            if !keys.contains(key.as_str()) {
                transaction.execute("DELETE FROM spaces WHERE space_key = ?1", [key])?;
            }
        }
        {
            let mut insert = transaction.prepare(
                "INSERT INTO spaces (space_key, ordinal, value_json) VALUES (?1, ?2, ?3)
                 ON CONFLICT(space_key) DO UPDATE SET ordinal = excluded.ordinal, value_json = excluded.value_json
                 WHERE spaces.ordinal != excluded.ordinal OR spaces.value_json != excluded.value_json",
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
        let version = store
            .connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn a_new_window_uses_the_shipped_chrome_defaults() {
        let window = WindowState::default();
        assert_eq!(window.chrome, Mode::Sidebar);
        assert!(!window.show_space_picker);
    }

    #[test]
    fn complete_snapshots_round_trip_in_space_order() {
        let mut store = StateStore::memory().unwrap();
        let snapshot = Snapshot {
            window: WindowState {
                chrome: Mode::Tabs,
                show_space_picker: false,
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
    fn saves_touch_only_changed_rows() {
        let mut store = StateStore::memory().unwrap();
        let mut snapshot =
            Snapshot { spaces: vec![space("one"), space("two")], ..Snapshot::default() };
        store.save(&snapshot).unwrap();
        let mut changes = store.connection.total_changes();
        store.save(&snapshot).unwrap();
        assert_eq!(store.connection.total_changes(), changes);
        snapshot.window.sidebar_width += 10.;
        store.save(&snapshot).unwrap();
        assert_eq!(
            store.connection.total_changes() - changes,
            1,
            "geometry updates only the window row"
        );
        changes = store.connection.total_changes();
        snapshot.spaces[0].name = "renamed".into();
        store.save(&snapshot).unwrap();
        assert_eq!(
            store.connection.total_changes() - changes,
            1,
            "renaming updates only that space"
        );
        changes = store.connection.total_changes();
        snapshot.spaces.swap(0, 1);
        store.save(&snapshot).unwrap();
        assert_eq!(
            store.connection.total_changes() - changes,
            2,
            "reordering updates the ordinals"
        );
        assert_eq!(store.load().unwrap(), snapshot);
        snapshot.spaces.pop();
        store.save(&snapshot).unwrap();
        assert_eq!(store.load().unwrap(), snapshot, "removed spaces do not reappear");
    }

    #[test]
    fn delayed_background_requests_cannot_overwrite_a_final_flush() {
        let store = StateStore::memory().unwrap();
        let mut writer = StateWriter::new(store, Snapshot::default());
        let mut snapshot = Snapshot { spaces: vec![space("one")], ..Snapshot::default() };
        let queued = writer.request(snapshot.clone());
        snapshot.window.sidebar_width = 444.;
        writer.request(snapshot.clone()).save().unwrap();
        std::thread::spawn(move || queued.save()).join().unwrap().unwrap();
        assert_eq!(writer.state.lock().unwrap().store.load().unwrap(), snapshot);
        let changes = writer.state.lock().unwrap().store.connection.total_changes();
        writer.request(snapshot).save().unwrap();
        assert_eq!(writer.state.lock().unwrap().store.connection.total_changes(), changes);
    }

    #[test]
    fn failed_transactions_roll_back_and_the_same_snapshot_can_be_retried() {
        let mut store = StateStore::memory().unwrap();
        let initial = Snapshot { spaces: vec![space("one")], ..Snapshot::default() };
        store.save(&initial).unwrap();
        store
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_space BEFORE UPDATE ON spaces
            BEGIN SELECT RAISE(ABORT, 'test write failure'); END;",
            )
            .unwrap();
        let mut writer = StateWriter::new(store, initial.clone());
        let mut changed = initial.clone();
        changed.window.sidebar_width = 555.;
        changed.spaces[0].name = "new name".into();
        assert!(writer.request(changed.clone()).save().is_err());
        {
            let state = writer.state.lock().unwrap();
            assert_eq!(state.store.load().unwrap(), initial, "window changes roll back too");
            assert_eq!(state.saved, initial, "failed saves do not poison the comparison cache");
            state.store.connection.execute_batch("DROP TRIGGER reject_space").unwrap();
        }
        writer.request(changed.clone()).save().unwrap();
        assert_eq!(writer.state.lock().unwrap().store.load().unwrap(), changed);
    }

    #[test]
    fn older_window_state_keeps_the_space_picker_visible() {
        let restored: WindowState = serde_json::from_str(
            r#"{"chrome":"sidebar","sidebar_scope":"all_spaces","sidebar_width":280.0,"active_space":null,"bounds":null}"#,
        )
        .unwrap();

        assert!(restored.show_space_picker);
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
