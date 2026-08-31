//! One space: a folder, its backend workspace, and its open items.
//!
//! A space is independently stateful in the same way a Zed `Workspace` held by
//! `MultiWorkspace` is: it owns its sessions and active item, while the parent
//! owns the ordered collection and decides which space the window presents.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use futures::{StreamExt as _, channel::mpsc};
use gpui::{Context, Task};
use zeddy_herdr::{PaneId, WorkspaceId, control::Client};

use crate::{
    chrome::{Action, Entry, PaneEntries},
    item::{Item, PluginItem, SessionItem},
    persistence::{PersistedItem, PersistedSpace, SpaceKind as PersistedSpaceKind},
    session::Session,
    spaces,
    workspace::{ItemId, SplitDirection, Workspace},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    AdHoc,
    Registered,
}

pub struct Space {
    name: String,
    path: PathBuf,
    kind: Kind,
    client: Client,
    workspace: Option<WorkspaceId>,
    layout: Workspace,
    items: HashMap<ItemId, Item>,
    sessions: HashMap<PaneId, ItemId>,
    starting: bool,
    closing: HashSet<ItemId>,
    reattaching: HashSet<ItemId>,
    restoring_sessions: HashMap<String, ItemId>,
    restoring_plugins: Vec<PersistedItem>,
    drag_target: Option<(crate::workspace::PaneId, Option<SplitDirection>)>,
    problem: Option<String>,
    wakeup_tx: mpsc::UnboundedSender<()>,
    _wakeups: Task<()>,
}

impl Space {
    pub fn new(
        name: String,
        path: PathBuf,
        kind: Kind,
        client: Client,
        cx: &mut Context<Self>,
    ) -> Self {
        let (wakeup_tx, wakeup_rx) = mpsc::unbounded();
        Self {
            name,
            path,
            kind,
            client,
            workspace: None,
            layout: Workspace::new(),
            items: HashMap::new(),
            sessions: HashMap::new(),
            starting: false,
            closing: HashSet::new(),
            reattaching: HashSet::new(),
            restoring_sessions: HashMap::new(),
            restoring_plugins: Vec::new(),
            drag_target: None,
            problem: None,
            wakeup_tx,
            _wakeups: Self::watch(wakeup_rx, cx),
        }
    }

    fn watch(mut wakeups: mpsc::UnboundedReceiver<()>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            while wakeups.next().await.is_some() {
                while wakeups.try_recv().is_ok() {}
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    return;
                }
            }
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
        self.workspace = None;
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn available(&self) -> bool {
        self.kind == Kind::AdHoc || self.path.is_dir()
    }

    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    pub fn layout(&self) -> &Workspace {
        &self.layout
    }

    pub fn active(&self) -> Option<ItemId> {
        self.layout.pane(self.layout.active_pane()).and_then(|pane| pane.active())
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    pub fn reattaching(&self, id: ItemId) -> bool {
        self.reattaching.contains(&id)
    }

    pub fn session_access(&self, backend: &PaneId) -> Option<crate::session::SessionAccess> {
        let item = self.sessions.get(backend)?;
        self.items.get(item)?.as_session().map(|item| item.session.access())
    }

    pub fn active_session_id(&self) -> Option<PaneId> {
        self.active()
            .and_then(|item| self.items.get(&item))
            .and_then(Item::as_session)
            .map(|item| item.session.id().clone())
    }

    pub fn reattach(&mut self, id: ItemId, cx: &mut Context<Self>) {
        if !self.reattaching.insert(id) {
            return;
        }
        let Some(session) = self.items.get(&id).and_then(Item::as_session) else {
            self.reattaching.remove(&id);
            return;
        };
        let backend_id = session.session.id().clone();
        let size = session.session.size();
        let client = self.client.clone();
        let wakeups = self.wakeup_tx.clone();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    let info = client
                        .sessions(None)?
                        .into_iter()
                        .find(|info| info.id == backend_id)
                        .ok_or_else(|| {
                            zeddy_herdr::Error::Protocol(format!(
                                "Herdr no longer reports session {}",
                                backend_id.0
                            ))
                        })?;
                    Session::attach(&client, info, size, wakeups)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.reattaching.remove(&id);
                match result {
                    Ok(session) => {
                        if let Some(item) = this.items.get_mut(&id).and_then(Item::as_session_mut) {
                            item.session = session;
                            this.problem = None;
                        }
                    }
                    Err(error) => this.problem = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn key(&self) -> String {
        match self.kind {
            Kind::AdHoc => "ad-hoc".to_owned(),
            Kind::Registered => format!("folder:{}", self.path.to_string_lossy()),
        }
    }

    pub fn restore_saved(&mut self, saved: &PersistedSpace) {
        self.layout = saved.layout.clone();
        self.restoring_sessions.clear();
        self.restoring_plugins.clear();
        let mut retained = HashSet::new();
        for item in &saved.items {
            let id = self.layout.item_ids().find(|candidate| candidate.get() == item.item_id());
            if let Some(id) = id {
                retained.insert(id);
                match item {
                    PersistedItem::Terminal { backend_id, .. } => {
                        self.restoring_sessions.insert(backend_id.clone(), id);
                    }
                    PersistedItem::Plugin { .. } => self.restoring_plugins.push(item.clone()),
                }
            }
        }
        let invalid: Vec<_> =
            self.layout.item_ids().filter(|item| !retained.contains(item)).collect();
        for item in invalid {
            let _ = self.layout.remove_item(item);
        }
    }

    pub fn persisted(&self) -> PersistedSpace {
        let key = self.key();
        let mut items: Vec<_> = self
            .items
            .iter()
            .filter_map(|(id, item)| match item {
                Item::Session(item) => Some(PersistedItem::Terminal {
                    item_id: id.get(),
                    backend_id: item.session.id().0.clone(),
                }),
                Item::Plugin(item) if item.restorable => Some(PersistedItem::Plugin {
                    item_id: id.get(),
                    plugin: item.contribution.plugin.clone(),
                    pane: item.contribution.key.clone(),
                    state: None,
                    bound_session: item.bound_session.as_ref().map(|id| id.0.clone()),
                }),
                Item::Plugin(_) => None,
            })
            .collect();
        items.extend(self.restoring_sessions.iter().map(|(backend_id, item)| {
            PersistedItem::Terminal { item_id: item.get(), backend_id: backend_id.clone() }
        }));
        items.extend(self.restoring_plugins.iter().cloned());
        PersistedSpace {
            key,
            name: self.name.clone(),
            path: (self.kind == Kind::Registered).then(|| self.path.clone()),
            kind: match self.kind {
                Kind::AdHoc => PersistedSpaceKind::AdHoc,
                Kind::Registered => PersistedSpaceKind::Folder,
            },
            layout: self.layout.clone(),
            items,
            expanded: true,
        }
    }

    pub fn activate_pane_in_direction(&mut self, direction: SplitDirection) {
        self.layout.activate_pane_in_direction(direction);
    }

    pub fn activate_pane(&mut self, pane: crate::workspace::PaneId) {
        if let Err(error) = self.layout.activate_pane(pane) {
            self.problem = Some(error.to_string());
        }
    }

    pub fn drag_target(&self) -> Option<(crate::workspace::PaneId, Option<SplitDirection>)> {
        self.drag_target
    }

    pub fn set_drag_target(
        &mut self,
        pane: crate::workspace::PaneId,
        direction: Option<SplitDirection>,
    ) {
        self.drag_target = Some((pane, direction));
    }

    pub fn resize_divider(&mut self, axis_path: &[usize], divider: usize, fraction: f32) {
        if let Err(error) = self.layout.center.resize_divider(axis_path, divider, fraction) {
            self.problem = Some(error.to_string());
        }
    }

    pub fn drop_item(
        &mut self,
        item: ItemId,
        source: crate::workspace::PaneId,
        target: crate::workspace::PaneId,
        index: Option<usize>,
    ) {
        if self.layout.pane_for_item(item) != Some(source) {
            self.drag_target = None;
            return;
        }
        let direction = self
            .drag_target
            .filter(|(pane, _)| *pane == target)
            .and_then(|(_, direction)| direction);
        self.drag_target = None;
        let destination = match direction {
            Some(direction) => match self.layout.split_pane(target, direction) {
                Ok(pane) => pane,
                Err(error) => {
                    self.problem = Some(error.to_string());
                    return;
                }
            },
            None => target,
        };
        if let Err(error) = self.layout.move_item(item, destination, index) {
            self.problem = Some(error.to_string());
        }
    }

    pub fn prepare_drop_destination(
        &mut self,
        target: crate::workspace::PaneId,
    ) -> Option<crate::workspace::PaneId> {
        let direction = self
            .drag_target
            .take()
            .filter(|(pane, _)| *pane == target)
            .and_then(|(_, direction)| direction);
        match direction {
            Some(direction) => match self.layout.split_pane(target, direction) {
                Ok(pane) => Some(pane),
                Err(error) => {
                    self.problem = Some(error.to_string());
                    None
                }
            },
            None => Some(target),
        }
    }

    /// Zed's split-and-move action creates the neighboring pane and moves the
    /// active item into it. Items remain unique; terminals are never cloned.
    pub fn split_and_move(&mut self, direction: SplitDirection) {
        let source = self.layout.active_pane();
        let active = self.layout.pane(source).and_then(|pane| pane.active());
        match self.layout.split_pane(source, direction) {
            Ok(destination) => {
                if let Some(active) = active
                    && let Err(error) = self.layout.move_item(active, destination, None)
                {
                    self.problem = Some(error.to_string());
                }
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
    }

    pub fn move_active_to_pane(&mut self, direction: SplitDirection) {
        let source = self.layout.active_pane();
        let active = self.layout.pane(source).and_then(|pane| pane.active());
        let destination = self.layout.pane_in_direction(direction);
        if let (Some(active), Some(destination)) = (active, destination)
            && let Err(error) = self.layout.move_item(active, destination, None)
        {
            self.problem = Some(error.to_string());
        }
    }

    pub fn join_active_into_next(&mut self) {
        let source = self.layout.active_pane();
        let destination =
            [SplitDirection::Right, SplitDirection::Down, SplitDirection::Left, SplitDirection::Up]
                .into_iter()
                .find_map(|direction| self.layout.pane_in_direction(direction));
        if let Some(destination) = destination
            && let Err(error) = self.layout.join_pane(source, destination)
        {
            self.problem = Some(error.to_string());
        }
    }

    pub fn toggle_zoom(&mut self) {
        let active = self.layout.active_pane();
        if let Err(error) = self.layout.center.toggle_maximized(active) {
            self.problem = Some(error.to_string());
        }
    }

    pub fn entries(&self, space: gpui::EntityId) -> Vec<Entry> {
        self.layout
            .panes()
            .flat_map(|pane| {
                pane.items().iter().filter_map(move |id| {
                    let item = self.items.get(id)?;
                    Some(Entry {
                        space,
                        space_key: self.key(),
                        key: *id,
                        pane: pane.id,
                        title: item.title(),
                        agent: item.agent(),
                        ended: item.ended(),
                        selected: pane.active() == Some(*id)
                            && self.layout.active_pane() == pane.id,
                        closable: true,
                    })
                })
            })
            .collect()
    }

    pub fn pane_entries(&self, space: gpui::EntityId) -> Vec<PaneEntries> {
        self.layout
            .panes()
            .map(|pane| PaneEntries {
                id: pane.id,
                entries: pane
                    .items()
                    .iter()
                    .filter_map(|id| {
                        let item = self.items.get(id)?;
                        Some(Entry {
                            space,
                            space_key: self.key(),
                            key: *id,
                            pane: pane.id,
                            title: item.title(),
                            agent: item.agent(),
                            ended: item.ended(),
                            selected: pane.active() == Some(*id)
                                && self.layout.active_pane() == pane.id,
                            closable: true,
                        })
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn pane_item_ids(&self, pane: crate::workspace::PaneId) -> Vec<ItemId> {
        self.layout.pane(pane).map(|pane| pane.items().to_vec()).unwrap_or_default()
    }

    pub fn all_item_ids(&self) -> Vec<ItemId> {
        self.layout.item_ids().collect()
    }

    pub fn plugin_item_ids(&self, plugin: &str) -> Vec<ItemId> {
        self.items
            .iter()
            .filter_map(|(id, item)| {
                item.as_plugin().filter(|item| item.contribution.plugin == plugin).map(|_| *id)
            })
            .collect()
    }

    pub fn close_targets(&self, ids: &[ItemId]) -> Vec<(ItemId, Option<PaneId>)> {
        ids.iter()
            .filter_map(|id| {
                self.items
                    .get(id)
                    .map(|item| (*id, item.as_session().map(|item| item.session.id().clone())))
            })
            .collect()
    }

    pub fn finish_bulk_close(&mut self, ids: &[ItemId]) {
        for id in ids {
            self.remove_item(*id);
        }
    }

    pub fn act(&mut self, action: Action, cx: &mut Context<Self>) {
        match action {
            Action::Select { item, .. } => {
                if let Err(error) = self.layout.activate_item(item) {
                    self.problem = Some(error.to_string());
                }
                self.fit_items();
            }
            Action::Close { item, .. } => self.close_item(item, cx),
            Action::MoveToPane { item, source, target, .. } => {
                self.drop_item(item, source, target, None)
            }
            Action::New
            | Action::NewInSpace { .. }
            | Action::ClosePane { .. }
            | Action::CloseSpace { .. }
            | Action::RenameSpace { .. }
            | Action::LocateSpace { .. }
            | Action::ToggleMode
            | Action::ToggleSidebarScope => {}
        }
        cx.notify();
    }

    pub fn open_plugin(&mut self, plugin: PluginItem, cx: &mut Context<Self>) -> ItemId {
        let id = self.layout.alloc_item();
        self.open_plugin_at(id, plugin, cx);
        id
    }

    pub fn open_plugin_in(
        &mut self,
        plugin: PluginItem,
        pane: crate::workspace::PaneId,
        index: Option<usize>,
        cx: &mut Context<Self>,
    ) -> ItemId {
        let id = self.layout.alloc_item();
        self.items.insert(id, Item::Plugin(plugin));
        if let Err(error) = self.layout.add_item(id, Some(pane), index) {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
        }
        cx.notify();
        id
    }

    pub fn cloneable_plugin(
        &self,
        item: ItemId,
    ) -> Option<(zeddy_plugin::PaneKey, String, Option<PaneId>)> {
        let plugin = self.items.get(&item)?.as_plugin()?;
        plugin.can_clone.then(|| {
            (plugin.contribution.clone(), plugin.title.clone(), plugin.bound_session.clone())
        })
    }

    pub fn open_plugin_at(&mut self, id: ItemId, plugin: PluginItem, cx: &mut Context<Self>) {
        self.items.insert(id, Item::Plugin(plugin));
        if self.layout.pane_for_item(id).is_none()
            && let Err(error) = self.layout.add_item(id, None, None)
        {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
        }
        cx.notify();
    }

    pub fn plugin_item(&self, contribution: &zeddy_plugin::PaneKey) -> Option<ItemId> {
        self.items.iter().find_map(|(id, item)| {
            item.as_plugin().filter(|item| &item.contribution == contribution).map(|_| *id)
        })
    }

    pub fn activate_plugin(&mut self, contribution: &zeddy_plugin::PaneKey) -> bool {
        let Some(item) = self.plugin_item(contribution) else {
            return false;
        };
        self.layout.activate_item(item).is_ok()
    }

    pub fn take_restoring_plugins(&mut self) -> Vec<PersistedItem> {
        std::mem::take(&mut self.restoring_plugins)
    }

    pub fn restore_plugin(
        &mut self,
        record: &PersistedItem,
        plugin: PluginItem,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(item) =
            self.layout.item_ids().find(|candidate| candidate.get() == record.item_id())
        else {
            return false;
        };
        self.open_plugin_at(item, plugin, cx);
        true
    }

    pub fn remove_plugin_placeholders(&mut self, records: &[PersistedItem]) {
        for record in records {
            let Some(item) =
                self.layout.item_ids().find(|candidate| candidate.get() == record.item_id())
            else {
                continue;
            };
            if !self.items.contains_key(&item) {
                let _ = self.layout.remove_item(item);
            }
        }
    }

    pub fn send_active(&mut self, bytes: &[u8], cx: &mut Context<Self>) {
        let Some(id) = self.active() else {
            return;
        };
        if let Some(session) = self.items.get_mut(&id).and_then(Item::as_session_mut)
            && let Err(error) = session.session.send(bytes)
        {
            self.problem = Some(error.to_string());
            cx.notify();
        }
    }

    pub fn fit_items(&mut self) {
        for item in self.items.values_mut() {
            let Some(item) = item.as_session_mut() else {
                continue;
            };
            let Some(size) = item.fit.get() else {
                continue;
            };
            if let Err(error) = item.session.resize(size) {
                self.problem = Some(error.to_string());
            }
        }
    }

    /// Attach sessions discovered by the parent's one backend snapshot.
    /// Process spawning and stream setup stay off the frame thread.
    pub fn adopt(&mut self, infos: Vec<zeddy_herdr::control::Session>, cx: &mut Context<Self>) {
        let infos: Vec<_> =
            infos.into_iter().filter(|info| !self.sessions.contains_key(&info.id)).collect();
        if infos.is_empty() {
            return;
        }

        let client = self.client.clone();
        let wakeups = self.wakeup_tx.clone();
        let size = zeddy_vt::Size::default();
        let executor = cx.background_executor().clone();
        let restored_ids: HashMap<_, _> = infos
            .iter()
            .filter_map(|info| {
                self.restoring_sessions.remove(&info.id.0).map(|item| (info.id.clone(), item))
            })
            .collect();
        let stale: Vec<_> = self.restoring_sessions.drain().map(|(_, item)| item).collect();
        for item in stale {
            let _ = self.layout.remove_item(item);
        }
        cx.spawn(async move |this, cx| {
            let attached = executor
                .spawn(async move {
                    infos
                        .into_iter()
                        .map(|info| {
                            let restored = restored_ids.get(&info.id).copied();
                            (restored, Session::attach(&client, info, size, wakeups.clone()))
                        })
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                for (restored, result) in attached {
                    match result {
                        Ok(session) => this.insert_session_with_id(session, restored),
                        Err(error) => {
                            if let Some(item) = restored {
                                let _ = this.layout.remove_item(item);
                            }
                            this.problem = Some(error.to_string());
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn start_session(&mut self, cx: &mut Context<Self>) {
        if self.starting {
            return;
        }
        if !self.path.is_dir() {
            self.problem = Some(format!(
                "{} is unavailable. Locate the space folder before opening a session.",
                self.path.display()
            ));
            cx.notify();
            return;
        }
        self.starting = true;
        self.problem = None;
        cx.notify();

        let client = self.client.clone();
        let workspace = self.workspace.clone();
        let path = self.path.clone();
        let label = self.name.clone();
        let wakeups = self.wakeup_tx.clone();
        let size = zeddy_vt::Size::default();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    let info = match workspace {
                        Some(workspace) => client.start_session(&workspace, None),
                        None => client.create_workspace(&path, Some(&label)),
                    }?;
                    Session::attach(&client, info, size, wakeups)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.starting = false;
                match result {
                    Ok(session) => {
                        this.insert_session(session);
                        this.problem = None;
                    }
                    Err(error) => this.problem = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn insert_session(&mut self, session: Session) {
        self.insert_session_with_id(session, None);
    }

    fn insert_session_with_id(&mut self, session: Session, restored: Option<ItemId>) {
        self.workspace = Some(session.info.workspace.clone());
        let backend_id = session.id().clone();
        if self.sessions.contains_key(&backend_id) {
            return;
        }
        let id = restored.unwrap_or_else(|| self.layout.alloc_item());
        self.items.insert(id, Item::Session(SessionItem::new(session)));
        if self.layout.pane_for_item(id).is_none()
            && let Err(error) = self.layout.add_item(id, None, None)
        {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
            return;
        }
        self.sessions.insert(backend_id, id);
    }

    fn close_item(&mut self, id: ItemId, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(&id) else {
            return;
        };
        let Some(backend_id) = item.as_session().map(|item| item.session.id().clone()) else {
            self.remove_item(id);
            cx.notify();
            return;
        };
        if !self.closing.insert(id.clone()) {
            return;
        }
        let client = self.client.clone();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let asked = backend_id.clone();
            let result = executor.spawn(async move { client.close_session(&asked) }).await;
            let _ = this.update(cx, |this, cx| {
                this.closing.remove(&id);
                match result {
                    Ok(()) => this.remove_item(id),
                    Err(error) => this.problem = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn remove_item(&mut self, id: ItemId) {
        let Some(mut item) = self.items.remove(&id) else {
            return;
        };
        if let Some(session) = item.as_session_mut() {
            let backend_id = session.session.id().clone();
            self.sessions.remove(&backend_id);
            session.session.release();
            let dependents: Vec<_> = self
                .items
                .iter()
                .filter_map(|(id, item)| {
                    item.as_plugin().and_then(|plugin| {
                        (plugin.bound_session.as_ref() == Some(&backend_id)).then_some(*id)
                    })
                })
                .collect();
            for dependent in dependents {
                self.remove_item(dependent);
            }
        }
        if let Err(error) = self.layout.remove_item(id) {
            self.problem = Some(error.to_string());
        }
        if self.sessions.is_empty() {
            self.workspace = None;
        }
    }

    /// Remove terminal items after the daemon that owned their PTYs died.
    /// Plugin items and the space itself survive; no shell is recreated under
    /// a dead tab's identity.
    pub fn drop_dead_sessions(&mut self) {
        let terminal_items: Vec<_> = self
            .items
            .iter()
            .filter_map(|(id, item)| item.as_session().map(|_| *id))
            .chain(self.restoring_sessions.values().copied())
            .collect();
        self.restoring_sessions.clear();
        for id in terminal_items {
            self.remove_item(id);
        }
        self.workspace = None;
        self.starting = false;
        self.closing.clear();
    }
}

impl Drop for Space {
    fn drop(&mut self) {
        for item in self.items.values_mut() {
            if let Some(item) = item.as_session_mut() {
                item.session.release();
            }
        }
    }
}

pub fn name_for(kind: Kind, path: &std::path::Path) -> String {
    match kind {
        Kind::AdHoc => "Ad-hoc sessions".to_owned(),
        Kind::Registered => spaces::display_name(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_synthetic_space_has_the_product_name_from_the_sketch() {
        assert_eq!(name_for(Kind::AdHoc, std::path::Path::new("/home/op")), "Ad-hoc sessions");
    }
}
