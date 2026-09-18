//! Coalesced workspace snapshots and ordered off-thread database writes.

use super::*;

const SAVE_INTERVAL: Duration = Duration::from_millis(250);
const RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// Only a complete restore may hand a writable store to startup cleanup and
/// the state writer. A fallback snapshot must never replace unreadable state.
pub(super) fn restore_state(
    path: anyhow::Result<PathBuf>,
) -> (Option<StateStore>, Snapshot, Option<String>) {
    match path.and_then(StateStore::open).and_then(|store| {
        let snapshot = store.load()?;
        Ok((store, snapshot))
    }) {
        Ok((store, snapshot)) => (Some(store), snapshot, None),
        Err(error) => (None, Snapshot::default(), Some(format!("{error:#}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{PersistedItem, PersistedSpace, SpaceKind, StateWriter};
    use crate::workspace::WorkspaceTabs;

    fn saved_workspace() -> Snapshot {
        let mut layout = WorkspaceTabs::new();
        let item = layout.alloc_item();
        layout.push_standalone(item).unwrap();
        Snapshot {
            spaces: vec![PersistedSpace {
                key: "healthy".into(),
                name: "Healthy project".into(),
                path: Some(PathBuf::from("/tmp/project")),
                kind: SpaceKind::Folder,
                layout,
                items: vec![PersistedItem::Plugin {
                    item_id: item.get(),
                    plugin: "com.chartr.agent".into(),
                    pane: "launcher".into(),
                    state: Some("saved plugin state".into()),
                    bound_session: None,
                }],
                expanded: true,
            }],
            ..Snapshot::default()
        }
    }

    fn assert_failed_restore_preserves_database(damage: &str) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.sqlite");
        StateStore::open(&path).unwrap().save(&saved_workspace()).unwrap();
        rusqlite::Connection::open(&path).unwrap().execute_batch(damage).unwrap();
        let before = std::fs::read(&path).unwrap();

        let (mut store, fallback, problem) = restore_state(Ok(path.clone()));
        assert!(problem.is_some());
        assert_eq!(fallback, Snapshot::default());
        // Exercise the guarded startup and writer paths with a fallback that
        // would delete the healthy space if a failed restore exposed the store.
        if let Some(store) = store.as_mut() {
            store.save(&fallback).unwrap();
            store.complete_implicit_root_cleanup().unwrap();
        }
        let mut writer = store.map(|store| StateWriter::new(store, fallback.clone()));
        if let Some(writer) = writer.as_mut() {
            let mut changed = fallback;
            changed.window.sidebar_width += 10.;
            let autosave = writer.request(changed.clone());
            writer.request(changed).save().unwrap(); // shutdown flush
            autosave.save().unwrap();
        }
        assert!(writer.is_none(), "failed restoration must disable every state write");
        assert_eq!(std::fs::read(&path).unwrap(), before, "retain even the damaged records");
    }

    #[test]
    fn malformed_space_preserves_healthy_layouts_and_plugin_state() {
        assert_failed_restore_preserves_database(
            "INSERT INTO spaces (space_key, ordinal, value_json) VALUES ('damaged', 1, '{')",
        );
    }

    #[test]
    fn malformed_window_state_disables_saving_too() {
        assert_failed_restore_preserves_database(
            "UPDATE app_state SET value_json = '{' WHERE key = 'window'",
        );
    }

    #[test]
    fn new_and_healthy_workspaces_remain_writable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.sqlite");
        let (store, saved, problem) = restore_state(Ok(path.clone()));
        assert!(problem.is_none());
        assert_eq!(saved, Snapshot::default());
        let snapshot = saved_workspace();
        StateWriter::new(store.unwrap(), saved).request(snapshot.clone()).save().unwrap();

        let (store, saved, problem) = restore_state(Ok(path.clone()));
        assert!(problem.is_none());
        assert_eq!(saved, snapshot);
        let mut changed = snapshot;
        changed.window.sidebar_width += 10.;
        StateWriter::new(store.unwrap(), saved).request(changed.clone()).save().unwrap();
        assert_eq!(StateStore::open(&path).unwrap().load().unwrap(), changed);
    }
}

impl WorkspaceWindow {
    pub(super) fn capture_window_bounds(&mut self, window: &Window) {
        let bounds = match window.window_bounds() {
            gpui::WindowBounds::Windowed(bounds)
            | gpui::WindowBounds::Maximized(bounds)
            | gpui::WindowBounds::Fullscreen(bounds) => bounds,
        };
        self.window_bounds = Some(crate::persistence::WindowBounds {
            x: bounds.origin.x / px(1.),
            y: bounds.origin.y / px(1.),
            width: bounds.size.width / px(1.),
            height: bounds.size.height / px(1.),
        });
    }

    pub(super) fn schedule_persistence(&mut self, cx: &mut Context<Self>) {
        if self.state.is_none() {
            return;
        }
        self.persistence_dirty = true;
        if self.persistence_task.is_some() {
            return;
        }
        let executor = cx.background_executor().clone();
        self.persistence_task = Some(cx.spawn(async move |this, cx| {
            let mut delay = SAVE_INTERVAL;
            loop {
                executor.timer(delay).await;
                let Ok(Some(request)) = this.update(cx, |this, cx| {
                    if !this.persistence_dirty {
                        this.persistence_task = None;
                        return None;
                    }
                    this.persistence_dirty = false;
                    let snapshot = this.snapshot(cx);
                    this.state.as_mut().map(|state| state.request(snapshot))
                }) else {
                    break;
                };
                // Only one request is in flight. Changes made while it saves
                // set the dirty bit; the next turn captures the latest state.
                let result = executor.spawn(async move { request.save() }).await;
                delay = if let Err(error) = result {
                    let _ = this.update(cx, |this, cx| {
                        this.persistence_dirty = true;
                        let problem = error.to_string();
                        if this.problem.as_ref() != Some(&problem) {
                            this.problem = Some(problem);
                            cx.notify();
                        }
                    });
                    RETRY_INTERVAL
                } else {
                    SAVE_INTERVAL
                };
            }
        }));
    }

    /// Shutdown is the only synchronous save path after initialization. GPUI's
    /// quit-future budget is only 200 ms, so flush before returning from close/
    /// quit callbacks instead of risking loss of the final coalesced changes.
    pub(crate) fn flush_state(&mut self, cx: &App) {
        self.persistence_task = None;
        self.persistence_dirty = false;
        if self.state.is_none() {
            return;
        }
        let snapshot = self.snapshot(cx);
        if let Some(state) = self.state.as_mut()
            && let Err(error) = state.request(snapshot).save()
        {
            eprintln!("Could not save workspace state on close: {error}");
            self.problem = Some(error.to_string());
        }
    }
}
