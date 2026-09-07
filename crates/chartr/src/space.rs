//! One space: a folder, its backend workspace, and its open items.
//!
//! A space is independently stateful in the same way a Zed `Workspace` held by
//! `MultiWorkspace` is: it owns its sessions, outer workspace tabs, and active
//! item, while the parent owns the ordered spaces and decides which one the
//! window presents.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use chartr_herdr::{PaneId, WorkspaceId, control::Client};
use gpui::{Context, EventEmitter};

use crate::{
    chrome::{Action, Entry},
    item::{Item, PluginItem, SessionItem},
    persistence::{PersistedItem, PersistedSpace, SpaceKind as PersistedSpaceKind},
    session::Session,
    spaces,
    workspace::{ItemId, SplitDirection, Workspace, WorkspaceTabId, WorkspaceTabs},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    AdHoc,
    Registered,
}

/// Window-owned integrations are created in response to these events. A
/// `Space` can finish attaching a session without access to a GPUI window;
/// emitting the stable item id keeps that asynchronous model boundary clean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceEvent {
    TerminalReady(ItemId),
}

pub struct Space {
    name: String,
    path: PathBuf,
    kind: Kind,
    client: Client,
    workspace: Option<WorkspaceId>,
    layout: WorkspaceTabs,
    items: HashMap<ItemId, Item>,
    sessions: HashMap<PaneId, ItemId>,
    starting: bool,
    closing: HashSet<ItemId>,
    /// Pane ids successfully closed during this daemon lifetime. A backend
    /// snapshot is assembled through several requests and can therefore arrive
    /// after a close while still containing the pane it observed beforehand.
    /// Never let such a snapshot resurrect the local item.
    retired_sessions: HashSet<PaneId>,
    /// Ended local attachments absent from consecutive backend snapshots.
    /// Requiring confirmation avoids treating a snapshot concurrent with
    /// session creation as authoritative evidence of deletion.
    missing_sessions: HashMap<PaneId, u8>,
    reattaching: HashSet<ItemId>,
    restoring_sessions: HashMap<String, ItemId>,
    restoring_plugins: Vec<PersistedItem>,
    drag_target: Option<(WorkspaceTabId, crate::workspace::PaneId, Option<SplitDirection>)>,
    problem: Option<String>,
}

impl Space {
    pub fn new(
        name: String,
        path: PathBuf,
        kind: Kind,
        client: Client,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            name,
            path,
            kind,
            client,
            workspace: None,
            layout: WorkspaceTabs::new(),
            items: HashMap::new(),
            sessions: HashMap::new(),
            starting: false,
            closing: HashSet::new(),
            retired_sessions: HashSet::new(),
            missing_sessions: HashMap::new(),
            reattaching: HashSet::new(),
            restoring_sessions: HashMap::new(),
            restoring_plugins: Vec::new(),
            drag_target: None,
            problem: None,
        }
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

    pub fn workspace_tabs(&self) -> &WorkspaceTabs {
        &self.layout
    }

    pub fn active_tab_id(&self) -> Option<WorkspaceTabId> {
        self.layout.active_tab_id()
    }

    pub fn active_layout(&self) -> Option<&Workspace> {
        self.layout.active_workspace()
    }

    pub fn active(&self) -> Option<ItemId> {
        self.layout.active_item()
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    pub fn install_terminal_view(
        &mut self,
        id: ItemId,
        view: gpui::Entity<terminal_view::TerminalView>,
    ) {
        if let Some(item) = self.items.get_mut(&id).and_then(Item::as_session_mut) {
            item.install_terminal_view(view);
        }
    }

    pub fn set_terminal_bell(&mut self, id: ItemId, bell: bool) -> bool {
        self.items
            .get_mut(&id)
            .and_then(Item::as_session_mut)
            .is_some_and(|item| item.set_bell(bell))
    }

    pub fn set_plugin_title(&mut self, id: ItemId, title: String) -> bool {
        let Some(item) = self.items.get_mut(&id).and_then(Item::as_plugin_mut) else {
            return false;
        };
        if item.title == title {
            return false;
        }
        item.title = title;
        true
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

    pub fn activate_session(&mut self, backend: &PaneId, cx: &mut Context<Self>) -> bool {
        let Some(item) = self.sessions.get(backend).copied() else {
            return false;
        };
        let activated = self.layout.activate_item(item).is_ok();
        if activated {
            cx.notify();
        }
        activated
    }

    fn session_from_builder(
        &mut self,
        info: chartr_herdr::control::Session,
        builder: terminal::TerminalBuilder,
        cx: &mut Context<Self>,
    ) -> Session {
        let backend_id = info.id.clone();
        let session = Session::from_builder(info, builder, cx);
        let terminal = session.terminal();
        let terminal_entity = terminal.entity_id();
        cx.subscribe(&terminal, move |this, _, event, cx| {
            if !matches!(event, terminal::Event::CloseTerminal) {
                return;
            }
            let Some(item_id) = this.sessions.get(&backend_id).copied() else {
                return;
            };
            let Some(item) = this.items.get_mut(&item_id).and_then(Item::as_session_mut) else {
                return;
            };
            // A reattach uses `--takeover`, which closes the superseded local
            // client. Do not let that old client's exit mark the replacement.
            if item.session.terminal().entity_id() == terminal_entity {
                item.session.mark_ended();
                cx.notify();
            }
        })
        .detach();
        session
    }

    pub fn reattach(&mut self, id: ItemId, cx: &mut Context<Self>) {
        if !self.reattaching.insert(id) {
            return;
        }
        let Some(session) = self.items.get(&id).and_then(Item::as_session) else {
            self.reattaching.remove(&id);
            return;
        };
        let info = session.session.info.clone();
        let client = self.client.clone();
        let attach = Session::attach_builder(&client, &info, cx.entity_id().as_u64(), cx);
        cx.spawn(async move |this, cx| {
            let result = attach.await;
            let _ = this.update(cx, |this, cx| {
                this.reattaching.remove(&id);
                match result {
                    Ok(builder) => {
                        let session = this.session_from_builder(info, builder, cx);
                        if let Some(item) = this.items.get_mut(&id).and_then(Item::as_session_mut) {
                            item.session.replace_attachment(session);
                            item.clear_terminal_view();
                            this.problem = None;
                            cx.emit(SpaceEvent::TerminalReady(id));
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
        let _ = self.layout.prune_empty();
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
                Item::Plugin(_) | Item::PluginLauncher { .. } => None,
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

    pub fn activate_pane_in_direction(&mut self, direction: SplitDirection) -> bool {
        self.layout
            .active_workspace_mut()
            .and_then(|layout| layout.activate_pane_in_direction(direction))
            .is_some()
    }

    pub fn activate_pane(&mut self, tab: WorkspaceTabId, pane: crate::workspace::PaneId) {
        let result = self
            .layout
            .activate_tab(tab)
            .and_then(|()| self.layout.workspace_mut(tab).expect("known tab").activate_pane(pane));
        if let Err(error) = result {
            self.problem = Some(error.to_string());
        }
    }

    pub fn drag_target(
        &self,
    ) -> Option<(WorkspaceTabId, crate::workspace::PaneId, Option<SplitDirection>)> {
        self.drag_target
    }

    pub fn set_drag_target(
        &mut self,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        direction: Option<SplitDirection>,
    ) -> bool {
        let target = Some((tab, pane, direction));
        if self.drag_target == target {
            return false;
        }
        self.drag_target = target;
        true
    }

    pub fn clear_drag_target(&mut self) -> bool {
        self.drag_target.take().is_some()
    }

    pub fn resize_divider(
        &mut self,
        tab: WorkspaceTabId,
        axis_path: &[usize],
        divider: usize,
        fraction: f32,
    ) {
        let result = self
            .layout
            .workspace_mut(tab)
            .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(tab))
            .and_then(|layout| layout.center.resize_divider(axis_path, divider, fraction));
        if let Err(error) = result {
            self.problem = Some(error.to_string());
        }
    }

    pub fn drop_item(
        &mut self,
        item: ItemId,
        source_tab: WorkspaceTabId,
        source_pane: crate::workspace::PaneId,
        target_tab: WorkspaceTabId,
        target_pane: crate::workspace::PaneId,
        index: Option<usize>,
    ) {
        if self.layout.location(item) != Some((source_tab, source_pane)) {
            self.drag_target = None;
            return;
        }
        let direction = self
            .drag_target
            .filter(|(tab, pane, _)| *tab == target_tab && *pane == target_pane)
            .and_then(|(_, _, direction)| direction);
        self.drag_target = None;
        let destination = match direction {
            Some(direction) => match self
                .layout
                .workspace_mut(target_tab)
                .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(target_tab))
                .and_then(|layout| layout.split_pane(target_pane, direction))
            {
                Ok(pane) => pane,
                Err(error) => {
                    self.problem = Some(error.to_string());
                    return;
                }
            },
            None => target_pane,
        };
        if let Err(error) =
            self.layout.move_item(item, source_tab, source_pane, target_tab, destination, index)
        {
            self.problem = Some(error.to_string());
        }
    }

    pub fn prepare_drop_destination(
        &mut self,
        tab: WorkspaceTabId,
        target: crate::workspace::PaneId,
    ) -> Option<crate::workspace::PaneId> {
        let direction = self
            .drag_target
            .take()
            .filter(|(candidate, pane, _)| *candidate == tab && *pane == target)
            .and_then(|(_, _, direction)| direction);
        match direction {
            Some(direction) => match self
                .layout
                .workspace_mut(tab)
                .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(tab))
                .and_then(|layout| layout.split_pane(target, direction))
            {
                Ok(pane) => Some(pane),
                Err(error) => {
                    self.problem = Some(error.to_string());
                    None
                }
            },
            None => Some(target),
        }
    }

    pub fn remove_empty_pane(&mut self, tab: WorkspaceTabId, pane: crate::workspace::PaneId) {
        let result = self
            .layout
            .workspace_mut(tab)
            .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(tab))
            .and_then(|layout| layout.remove_empty_pane(pane));
        if let Err(error) = result {
            self.problem = Some(error.to_string());
        }
    }

    pub fn move_active_to_pane(&mut self, direction: SplitDirection) {
        let Some(layout) = self.layout.active_workspace_mut() else {
            return;
        };
        let source = layout.active_pane();
        let active = layout.pane(source).and_then(|pane| pane.active());
        let destination = layout.pane_in_direction(direction);
        if let (Some(active), Some(destination)) = (active, destination)
            && let Err(error) = layout.move_item(active, destination, None)
        {
            self.problem = Some(error.to_string());
        }
    }

    pub fn join_active_into_next(&mut self) {
        let Some(layout) = self.layout.active_workspace_mut() else {
            return;
        };
        let source = layout.active_pane();
        let destination =
            [SplitDirection::Right, SplitDirection::Down, SplitDirection::Left, SplitDirection::Up]
                .into_iter()
                .find_map(|direction| layout.pane_in_direction(direction));
        if let Some(destination) = destination
            && let Err(error) = layout.join_pane(source, destination)
        {
            self.problem = Some(error.to_string());
        }
    }

    pub fn entries(&self, space: gpui::EntityId) -> Vec<Entry> {
        self.layout
            .tabs()
            .iter()
            .filter_map(|tab| {
                let id = tab.representative_item()?;
                let pane = tab.layout.pane_for_item(id)?;
                let index =
                    tab.layout.pane(pane)?.items().iter().position(|candidate| *candidate == id)?;
                let item = self.items.get(&id)?;
                let grouped = tab.is_grouped();
                Some(Entry {
                    space,
                    space_key: self.key(),
                    key: id,
                    tab: tab.id,
                    pane,
                    index,
                    title: if grouped {
                        tab.name()
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("{} tabs", tab.layout.item_count()))
                    } else {
                        item.title()
                    },
                    icon_path: (!grouped).then(|| item.icon_path()).flatten(),
                    status: (!grouped).then(|| item.status()).flatten(),
                    process_running: !grouped && item.process_running(),
                    ended: !grouped && item.ended(),
                    bell: !grouped && item.as_session().is_some_and(SessionItem::bell),
                    selected: self.layout.active_tab_id() == Some(tab.id),
                    closable: true,
                    grouped,
                })
            })
            .collect()
    }

    pub fn pane_item_ids(
        &self,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
    ) -> Vec<ItemId> {
        self.layout
            .workspace(tab)
            .and_then(|layout| layout.pane(pane))
            .map(|pane| pane.items().to_vec())
            .unwrap_or_default()
    }

    pub fn tab_item_ids(&self, tab: WorkspaceTabId) -> Vec<ItemId> {
        self.layout.workspace(tab).map(|layout| layout.item_ids().collect()).unwrap_or_default()
    }

    pub fn move_workspace_tab(&mut self, tab: WorkspaceTabId, target_index: usize) {
        if let Err(error) = self.layout.move_tab(tab, target_index) {
            self.problem = Some(error.to_string());
        }
    }

    pub fn ungroup_pane(&mut self, tab: WorkspaceTabId) {
        if let Err(error) = self.layout.ungroup_tab(tab) {
            self.problem = Some(error.to_string());
        }
    }

    pub fn group_name(&self, tab: WorkspaceTabId) -> Option<&str> {
        self.layout.tab(tab).filter(|tab| tab.is_grouped()).and_then(|tab| tab.name())
    }

    pub fn rename_group(&mut self, tab: WorkspaceTabId, name: Option<String>) {
        if let Err(error) = self.layout.rename_tab(tab, name) {
            self.problem = Some(error.to_string());
        }
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
            self.retire_item(*id);
        }
    }

    pub fn act(&mut self, action: Action, cx: &mut Context<Self>) {
        match action {
            Action::Select { item, .. } => {
                if let Err(error) = self.layout.activate_item(item) {
                    self.problem = Some(error.to_string());
                }
            }
            Action::Close { item, .. } => self.close_item(item, cx),
            Action::New
            | Action::NewSpace
            | Action::NewPluginPane
            | Action::NewPluginPaneInSpace { .. }
            | Action::ActivateSpace { .. }
            | Action::NewInSpace { .. }
            | Action::MoveWorkspaceTab { .. }
            | Action::CloseGroup { .. }
            | Action::UngroupPane { .. }
            | Action::RenameGroup { .. }
            | Action::CloseSpace { .. }
            | Action::RenameSpace { .. }
            | Action::OpenSpaceFolder { .. }
            | Action::LocateSpace { .. }
            | Action::SwitchToTabs
            | Action::SwitchToSidebar
            | Action::BeginSpaceDrag { .. }
            | Action::OpenSettings => {}
        }
        cx.notify();
    }

    pub fn open_plugin_launcher(&mut self, cx: &mut Context<Self>) -> ItemId {
        let id = self.layout.alloc_item();
        let bound_session = self.active_session_id();
        self.items.insert(id, Item::PluginLauncher { bound_session });
        if let Err(error) = self.layout.push_standalone(id) {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
        }
        cx.notify();
        id
    }

    pub fn open_plugin_launcher_in(
        &mut self,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        cx: &mut Context<Self>,
    ) -> ItemId {
        let bound_session = self
            .layout
            .workspace(tab)
            .and_then(|layout| layout.pane(pane))
            .and_then(|pane| pane.active())
            .and_then(|item| self.items.get(&item))
            .and_then(Item::as_session)
            .map(|item| item.session.id().clone());
        let id = self.layout.alloc_item();
        self.items.insert(id, Item::PluginLauncher { bound_session });
        let result = self
            .layout
            .workspace_mut(tab)
            .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(tab))
            .and_then(|layout| layout.add_item(id, Some(pane), None));
        if let Err(error) = result {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
        } else {
            let _ = self.layout.activate_tab(tab);
        }
        cx.notify();
        id
    }

    pub fn open_plugin_launcher_dropped(
        &mut self,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        direction: Option<SplitDirection>,
        cx: &mut Context<Self>,
    ) -> Option<ItemId> {
        self.layout.workspace(tab)?.pane(pane)?;
        // Capture the target terminal binding before moving the chooser into a split.
        let launcher = self.open_plugin_launcher_in(tab, pane, cx);
        self.set_drag_target(tab, pane, direction);
        self.drop_item(launcher, tab, pane, tab, pane, None);
        Some(launcher)
    }

    pub fn is_plugin_launcher(&self, id: ItemId) -> bool {
        self.items.get(&id).is_some_and(Item::is_plugin_launcher)
    }

    pub fn plugin_launcher_bound_session(&self, id: ItemId) -> Option<PaneId> {
        self.items.get(&id).and_then(Item::plugin_launcher_bound_session).cloned()
    }

    /// Turn the picker into a plugin without changing its workspace location.
    pub fn replace_plugin_launcher(
        &mut self,
        id: ItemId,
        plugin: PluginItem,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.is_plugin_launcher(id) || self.layout.location(id).is_none() {
            return false;
        }
        self.items.insert(id, Item::Plugin(plugin));
        cx.notify();
        true
    }

    pub fn reserve_plugin_item(&mut self) -> ItemId {
        self.layout.alloc_item()
    }

    pub fn open_plugin_in_at(
        &mut self,
        id: ItemId,
        plugin: PluginItem,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        index: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.items.insert(id, Item::Plugin(plugin));
        let result = self
            .layout
            .workspace_mut(tab)
            .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(tab))
            .and_then(|layout| layout.add_item(id, Some(pane), index));
        if let Err(error) = result {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
        } else {
            let _ = self.layout.activate_tab(tab);
        }
        cx.notify();
    }

    pub fn cloneable_plugin(
        &self,
        item: ItemId,
    ) -> Option<(chartr_plugin::PaneKey, String, Option<PaneId>)> {
        let plugin = self.items.get(&item)?.as_plugin()?;
        plugin.can_clone.then(|| {
            (plugin.contribution.clone(), plugin.title.clone(), plugin.bound_session.clone())
        })
    }

    pub fn open_plugin_at(&mut self, id: ItemId, plugin: PluginItem, cx: &mut Context<Self>) {
        self.items.insert(id, Item::Plugin(plugin));
        if self.layout.location(id).is_none()
            && let Err(error) = self.layout.push_standalone(id)
        {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
        }
        cx.notify();
    }

    pub fn plugin_item(&self, contribution: &chartr_plugin::PaneKey) -> Option<ItemId> {
        self.items.iter().find_map(|(id, item)| {
            item.as_plugin().filter(|item| &item.contribution == contribution).map(|_| *id)
        })
    }

    pub fn activate_plugin(&mut self, contribution: &chartr_plugin::PaneKey) -> bool {
        let Some(item) = self.plugin_item(contribution) else {
            return false;
        };
        self.layout.activate_item(item).is_ok()
    }

    pub fn activate_plugin_view(&mut self, view: gpui::EntityId) -> bool {
        let Some(item) = self.items.iter().find_map(|(item, candidate)| {
            candidate
                .as_plugin()
                .filter(|plugin| plugin.view.any_view().entity_id() == view)
                .map(|_| *item)
        }) else {
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

    /// Attach sessions discovered by the parent's one backend snapshot.
    /// Local PTY creation stays off the frame thread.
    pub fn adopt(&mut self, infos: Vec<chartr_herdr::control::Session>, cx: &mut Context<Self>) {
        let snapshot_ids: HashSet<_> = infos.iter().map(|info| info.id.clone()).collect();
        let local = self
            .sessions
            .iter()
            .map(|(backend, item)| (backend.clone(), self.items.get(item).is_some_and(Item::ended)))
            .collect::<Vec<_>>();
        for backend in confirmed_missing_sessions(local, &snapshot_ids, &mut self.missing_sessions)
        {
            if let Some(item) = self.sessions.get(&backend).copied() {
                self.retire_item(item);
            }
        }

        let mut discovered = Vec::new();
        for info in infos {
            if self.retired_sessions.contains(&info.id) {
                continue;
            }
            if let Some(item) = self.sessions.get(&info.id).copied() {
                if let Some(session) = self.items.get_mut(&item).and_then(Item::as_session_mut) {
                    session.session.update_info(info);
                }
            } else {
                discovered.push(info);
            }
        }
        let infos = discovered;
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
        if infos.is_empty() {
            return;
        }
        let client = self.client.clone();
        let window_id = cx.entity_id().as_u64();
        let pending = infos
            .into_iter()
            .map(|info| {
                let restored = restored_ids.get(&info.id).copied();
                let attach = Session::attach_builder(&client, &info, window_id, cx);
                (restored, info, attach)
            })
            .collect::<Vec<_>>();
        cx.spawn(async move |this, cx| {
            let mut attached = Vec::with_capacity(pending.len());
            for (restored, info, attach) in pending {
                attached.push((restored, info, attach.await));
            }
            let _ = this.update(cx, |this, cx| {
                for (restored, info, result) in attached {
                    if this.retired_sessions.contains(&info.id) {
                        continue;
                    }
                    match result {
                        Ok(builder) => {
                            let session = this.session_from_builder(info, builder, cx);
                            if let Some(id) = this.insert_session_with_id(session, restored) {
                                cx.emit(SpaceEvent::TerminalReady(id));
                            }
                        }
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
        self.start_session_at(None, Vec::new(), cx).detach();
    }

    /// Create an ordinary chartr-owned terminal and queue its first shell/TUI
    /// input before publishing the new tab. Native plugins use this path so
    /// launched tools remain part of the owning space's normal lifecycle.
    pub fn start_session_with_input(&mut self, input: Vec<u8>, cx: &mut Context<Self>) {
        self.start_session_at(None, input, cx).detach();
    }

    pub fn prepare_plugin_session(
        &mut self,
        cx: &mut Context<Self>,
    ) -> gpui::Task<Result<chartr_plugin::PreparedTerminal, String>> {
        self.start_session_at(None, Vec::new(), cx)
    }

    pub fn start_session_in(
        &mut self,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        cx: &mut Context<Self>,
    ) {
        self.start_session_at(Some((tab, pane, None)), Vec::new(), cx).detach();
    }

    /// Resolve a dropped terminal's split only after the backend has started it.
    pub fn start_session_dropped(
        &mut self,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        direction: Option<SplitDirection>,
        cx: &mut Context<Self>,
    ) {
        self.start_session_at(Some((tab, pane, direction)), Vec::new(), cx).detach();
    }

    fn start_session_at(
        &mut self,
        destination: Option<(WorkspaceTabId, crate::workspace::PaneId, Option<SplitDirection>)>,
        initial_input: Vec<u8>,
        cx: &mut Context<Self>,
    ) -> gpui::Task<Result<chartr_plugin::PreparedTerminal, String>> {
        if self.starting {
            return gpui::Task::ready(Err(
                "Another terminal is still starting in this space.".into()
            ));
        }
        if !self.path.is_dir() {
            self.problem = Some(format!(
                "{} is unavailable. Locate the space folder before opening a session.",
                self.path.display()
            ));
            cx.notify();
            return gpui::Task::ready(Err(self.problem.clone().unwrap()));
        }
        self.starting = true;
        self.problem = None;
        cx.notify();

        let client = self.client.clone();
        let workspace = self.workspace.clone();
        let path = self.path.clone();
        let label = self.name.clone();
        let create_client = client.clone();
        let executor = cx.background_executor().clone();
        let window_id = cx.entity_id().as_u64();
        cx.spawn(async move |this, cx| {
            let info = executor
                .spawn(async move {
                    let info = match workspace {
                        Some(workspace) => create_client.start_session(&workspace, None),
                        None => create_client.create_workspace(&path, Some(&label)),
                    }?;
                    chartr_herdr::Result::Ok(info)
                })
                .await;
            let info = match info {
                Ok(info) => info,
                Err(error) => {
                    let message = error.to_string();
                    let _ = this.update(cx, |this, cx| {
                        this.starting = false;
                        this.problem = Some(error.to_string());
                        cx.notify();
                    });
                    return Err(message);
                }
            };
            let Ok(attach) =
                this.update(cx, |_, cx| Session::attach_builder(&client, &info, window_id, cx))
            else {
                return Err("The owning space was closed.".into());
            };
            let result = attach.await;
            this.update(cx, |this, cx| {
                this.starting = false;
                let result = match result {
                    Ok(builder) => {
                        let session = this.session_from_builder(info, builder, cx);
                        let session_id = session.id().0.clone();
                        let input = session.access();
                        let inserted = if let Some((tab, pane, direction)) = destination {
                            this.insert_session_in(session, tab, pane, direction)
                        } else {
                            this.insert_session(session)
                        };
                        if let Some(id) = inserted {
                            if !initial_input.is_empty()
                                && let Err(error) = input.send(&initial_input)
                            {
                                this.problem = Some(error.to_string());
                            }
                            cx.emit(SpaceEvent::TerminalReady(id));
                            Ok(chartr_plugin::PreparedTerminal::new(session_id, move |bytes| {
                                input.send(bytes).map_err(|error| error.to_string())
                            }))
                        } else {
                            Err("The new terminal could not be inserted into its space.".into())
                        }
                    }
                    Err(error) => {
                        this.problem = Some(error.to_string());
                        Err(error.to_string())
                    }
                };
                cx.notify();
                result
            })
            .map_err(|_| "The owning space was closed.".to_owned())?
        })
    }

    fn insert_session(&mut self, session: Session) -> Option<ItemId> {
        self.insert_session_with_id(session, None)
    }

    fn insert_session_in(
        &mut self,
        session: Session,
        tab: WorkspaceTabId,
        pane: crate::workspace::PaneId,
        direction: Option<SplitDirection>,
    ) -> Option<ItemId> {
        self.workspace = Some(session.info.workspace.clone());
        let backend_id = session.id().clone();
        if self.sessions.contains_key(&backend_id) || self.retired_sessions.contains(&backend_id) {
            return None;
        }

        let pane = direction
            .and_then(|direction| {
                self.layout
                    .workspace_mut(tab)
                    .and_then(|layout| layout.split_pane(pane, direction).ok())
            })
            .unwrap_or(pane);
        let id = self.layout.alloc_item();
        self.items.insert(id, Item::Session(SessionItem::new(session)));
        let placed = self
            .layout
            .workspace_mut(tab)
            .ok_or(crate::workspace::ModelError::WorkspaceTabNotFound(tab))
            .and_then(|layout| layout.add_item(id, Some(pane), None));
        if placed.is_ok() {
            let _ = self.layout.activate_tab(tab);
        } else if let Err(error) = self.layout.push_standalone(id) {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
            return None;
        }
        self.sessions.insert(backend_id, id);
        Some(id)
    }

    fn insert_session_with_id(
        &mut self,
        session: Session,
        restored: Option<ItemId>,
    ) -> Option<ItemId> {
        self.workspace = Some(session.info.workspace.clone());
        let backend_id = session.id().clone();
        if self.sessions.contains_key(&backend_id) || self.retired_sessions.contains(&backend_id) {
            return None;
        }
        let id = restored.unwrap_or_else(|| self.layout.alloc_item());
        self.items.insert(id, Item::Session(SessionItem::new(session)));
        if self.layout.location(id).is_none()
            && let Err(error) = self.layout.push_standalone(id)
        {
            self.items.remove(&id);
            self.problem = Some(error.to_string());
            return None;
        }
        self.sessions.insert(backend_id, id);
        Some(id)
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
                    Ok(()) => {
                        this.retired_sessions.insert(backend_id);
                        this.remove_item(id);
                    }
                    Err(error) => this.problem = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn remove_item(&mut self, id: ItemId) {
        let Some(item) = self.items.remove(&id) else {
            return;
        };
        if let Some(session) = item.as_session() {
            let backend_id = session.session.id().clone();
            self.sessions.remove(&backend_id);
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

    fn retire_item(&mut self, id: ItemId) {
        if let Some(backend) =
            self.items.get(&id).and_then(Item::as_session).map(|item| item.session.id().clone())
        {
            self.missing_sessions.remove(&backend);
            self.retired_sessions.insert(backend);
        }
        self.remove_item(id);
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
        self.retired_sessions.clear();
        self.missing_sessions.clear();
    }
}

const MISSING_SNAPSHOT_CONFIRMATIONS: u8 = 2;

/// Return ended local sessions absent from enough consecutive full snapshots
/// to establish that Herdr no longer owns them.
fn confirmed_missing_sessions(
    local: Vec<(PaneId, bool)>,
    snapshot: &HashSet<PaneId>,
    misses: &mut HashMap<PaneId, u8>,
) -> Vec<PaneId> {
    let local_ids: HashSet<_> = local.iter().map(|(backend, _)| backend.clone()).collect();
    misses.retain(|backend, _| local_ids.contains(backend) && !snapshot.contains(backend));

    let mut confirmed = Vec::new();
    for (backend, ended) in local {
        if snapshot.contains(&backend) || !ended {
            misses.remove(&backend);
            continue;
        }
        let count = misses.entry(backend.clone()).or_default();
        *count = count.saturating_add(1);
        if *count >= MISSING_SNAPSHOT_CONFIRMATIONS {
            confirmed.push(backend);
        }
    }
    confirmed
}

impl EventEmitter<SpaceEvent> for Space {}

pub fn name_for(kind: Kind, path: &std::path::Path) -> String {
    match kind {
        Kind::AdHoc => "Free sessions".to_owned(),
        Kind::Registered => spaces::display_name(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::AppContext as _;
    use std::{cell::Cell, rc::Rc};

    fn backend_session(id: &str, path: &std::path::Path) -> chartr_herdr::control::Session {
        chartr_herdr::control::Session {
            id: PaneId(id.to_owned()),
            terminal: chartr_herdr::TerminalId(format!("terminal-{id}")),
            workspace: WorkspaceId("w1".to_owned()),
            label: "1".to_owned(),
            running: None,
            status: chartr_herdr::control::SessionStatus::Unknown,
            agent: None,
            cwd: Some(path.to_owned()),
        }
    }

    struct LauncherReplacementView;

    impl gpui::Render for LauncherReplacementView {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            gpui::div()
        }
    }

    #[gpui::test]
    fn a_snapshot_cannot_resurrect_a_retired_backend_pane(cx: &mut gpui::TestAppContext) {
        let temporary = tempfile::tempdir().unwrap();
        let sidecar = temporary.path().join("herdr");
        std::fs::write(&sidecar, []).unwrap();
        let client = Client::new(
            chartr_herdr::Sidecar::at(sidecar).unwrap(),
            chartr_herdr::Namespace::rooted(temporary.path().join("namespace")),
        );
        let space = cx.new(|cx| {
            Space::new(
                "example".to_owned(),
                temporary.path().to_owned(),
                Kind::Registered,
                client,
                cx,
            )
        });
        let retired = PaneId("w1:p1".to_owned());

        space.update(cx, |space, cx| {
            space.retired_sessions.insert(retired.clone());
            space.adopt(vec![backend_session(&retired.0, temporary.path())], cx);
        });

        cx.read(|cx| {
            let space = space.read(cx);
            assert!(space.sessions.is_empty());
            assert!(space.items.is_empty());
            assert!(space.workspace_tabs().tabs().is_empty());
        });
    }

    #[test]
    fn an_ended_session_needs_two_missing_snapshots_before_removal() {
        let missing = PaneId("w1:p1".to_owned());
        let present = PaneId("w1:p2".to_owned());
        let live = PaneId("w1:p3".to_owned());
        let snapshot = HashSet::from([present.clone()]);
        let local = vec![(missing.clone(), true), (present.clone(), true), (live.clone(), false)];
        let mut misses = HashMap::new();

        assert!(
            confirmed_missing_sessions(local.clone(), &snapshot, &mut misses).is_empty(),
            "one possibly concurrent snapshot is not conclusive"
        );
        assert_eq!(misses.get(&missing), Some(&1));
        assert!(!misses.contains_key(&present));
        assert!(!misses.contains_key(&live));
        assert_eq!(confirmed_missing_sessions(local, &snapshot, &mut misses), vec![missing]);
    }

    #[gpui::test]
    fn plugin_launcher_opens_in_the_requested_existing_pane(cx: &mut gpui::TestAppContext) {
        let temporary = tempfile::tempdir().unwrap();
        let sidecar = temporary.path().join("herdr");
        std::fs::write(&sidecar, []).unwrap();
        let client = Client::new(
            chartr_herdr::Sidecar::at(sidecar).unwrap(),
            chartr_herdr::Namespace::rooted(temporary.path().join("namespace")),
        );
        let space = cx.new(|cx| {
            Space::new(
                "Free sessions".to_owned(),
                temporary.path().to_owned(),
                Kind::AdHoc,
                client,
                cx,
            )
        });
        let first = space.update(cx, |space, cx| space.open_plugin_launcher(cx));
        let (tab, pane) = cx.read(|cx| space.read(cx).workspace_tabs().location(first).unwrap());

        let second = space.update(cx, |space, cx| space.open_plugin_launcher_in(tab, pane, cx));

        cx.read(|cx| {
            let space = space.read(cx);
            let target = space.workspace_tabs().workspace(tab).unwrap().pane(pane).unwrap();
            assert_eq!(space.workspace_tabs().location(second), Some((tab, pane)));
            assert_eq!(target.items(), &[first, second]);
            assert_eq!(target.active(), Some(second));
            assert!(space.item(second).unwrap().is_plugin_launcher());
            assert_eq!(
                space.item(second).unwrap().icon_path().as_deref(),
                Some(crate::assets::PLUGIN_LAUNCHER_ICON_PATH)
            );
        });
    }

    #[gpui::test]
    fn dropped_creation_places_the_picker_and_rejected_terminal_preserves_layout(
        cx: &mut gpui::TestAppContext,
    ) {
        let temporary = tempfile::tempdir().unwrap();
        let sidecar = temporary.path().join("herdr");
        std::fs::write(&sidecar, []).unwrap();
        let client = Client::new(
            chartr_herdr::Sidecar::at(sidecar).unwrap(),
            chartr_herdr::Namespace::rooted(temporary.path().join("namespace")),
        );
        let space = cx.new(|cx| {
            Space::new("Test".into(), temporary.path().to_owned(), Kind::AdHoc, client, cx)
        });
        space.update(cx, |space, cx| {
            let first = space.open_plugin_launcher(cx);
            let (tab, pane) = space.layout.location(first).unwrap();
            let center = space.open_plugin_launcher_dropped(tab, pane, None, cx).unwrap();
            assert_eq!(space.layout.location(center), Some((tab, pane)));
            assert_eq!(
                space.layout.workspace(tab).unwrap().pane(pane).unwrap().items(),
                &[first, center]
            );

            for direction in [
                SplitDirection::Left,
                SplitDirection::Right,
                SplitDirection::Up,
                SplitDirection::Down,
            ] {
                let launcher =
                    space.open_plugin_launcher_dropped(tab, pane, Some(direction), cx).unwrap();
                let (placed_tab, placed_pane) = space.layout.location(launcher).unwrap();
                assert_eq!(placed_tab, tab);
                assert_ne!(placed_pane, pane);
                let layout = space.layout.workspace(tab).unwrap();
                assert_eq!(layout.pane(placed_pane).unwrap().items(), &[launcher]);
                assert_eq!(layout.center.pane_in_direction(pane, direction), Some(placed_pane));
                assert_eq!(layout.pane(pane).unwrap().items(), &[first, center]);
                assert_eq!(space.drag_target(), None);
            }

            let before = space.layout.workspace(tab).unwrap().center.panes();
            let item_count = space.items.len();
            space.path = temporary.path().join("missing-folder");
            space.start_session_dropped(tab, pane, Some(SplitDirection::Right), cx);
            assert!(space.problem.as_deref().unwrap().contains("unavailable"));
            assert!(!space.starting);
            assert_eq!(space.layout.workspace(tab).unwrap().center.panes(), before);
            assert_eq!(space.items.len(), item_count);
        });
    }

    #[gpui::test]
    fn selecting_a_plugin_replaces_the_launcher_and_close_tears_it_down_immediately(
        cx: &mut gpui::TestAppContext,
    ) {
        let temporary = tempfile::tempdir().unwrap();
        let sidecar = temporary.path().join("herdr");
        std::fs::write(&sidecar, []).unwrap();
        let client = Client::new(
            chartr_herdr::Sidecar::at(sidecar).unwrap(),
            chartr_herdr::Namespace::rooted(temporary.path().join("namespace")),
        );
        let space = cx.new(|cx| {
            Space::new(
                "Free sessions".to_owned(),
                temporary.path().to_owned(),
                Kind::AdHoc,
                client,
                cx,
            )
        });
        let launcher = space.update(cx, |space, cx| space.open_plugin_launcher(cx));
        let before = cx.read(|cx| space.read(cx).workspace_tabs().location(launcher));
        let closed = Rc::new(Cell::new(false));
        let close_signal = closed.clone();
        let view = crate::item::PluginView::with_close(
            cx.new(|_| LauncherReplacementView).into(),
            move || close_signal.set(true),
        );

        let replaced = space.update(cx, |space, cx| {
            space.replace_plugin_launcher(
                launcher,
                PluginItem {
                    contribution: chartr_plugin::PaneKey::new("com.example.hello", "main"),
                    title: "Hello".to_owned(),
                    icon_path: "/plugins/hello/icons/WavingHand01Icon.svg".into(),
                    view,
                    bound_session: None,
                    can_clone: false,
                    restorable: true,
                },
                cx,
            )
        });

        let space_entity = space.entity_id();
        cx.read(|cx| {
            let space = space.read(cx);
            assert!(replaced);
            assert_eq!(space.workspace_tabs().location(launcher), before);
            assert_eq!(space.item(launcher).unwrap().title(), "Hello");
            assert_eq!(
                space.item(launcher).unwrap().icon_path().as_deref(),
                Some("/plugins/hello/icons/WavingHand01Icon.svg")
            );
            assert_eq!(
                space.entries(space_entity)[0].icon_path.as_deref(),
                Some("/plugins/hello/icons/WavingHand01Icon.svg")
            );
            assert!(!space.item(launcher).unwrap().is_plugin_launcher());
        });

        assert!(space.update(cx, |space, _| {
            space.set_plugin_title(launcher, "Example Page".to_owned())
        }));
        cx.read(|cx| {
            let space = space.read(cx);
            assert_eq!(space.item(launcher).unwrap().title(), "Example Page");
            assert_eq!(space.entries(space_entity)[0].title, "Example Page");
        });

        space.update(cx, |space, cx| space.act(Action::Close { space: None, item: launcher }, cx));
        assert!(closed.get());
        cx.read(|cx| assert!(space.read(cx).item(launcher).is_none()));
    }
}
