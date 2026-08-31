//! The window-level workspace.
//!
//! This follows Zed's `MultiWorkspace` ownership boundary: the window owns an
//! ordered collection of independently stateful space entities, keeps one
//! active, observes each child, and renders only the active one. A space owns
//! its sessions and selection; switching spaces therefore never moves or
//! recreates a session.

use std::{
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{
    Anchor, AnyView, DragMoveEvent, Entity, EntityId, FocusHandle, Focusable, PathPromptOptions,
    Role,
};
use ui::{
    Banner, ButtonSize, ContextMenu, DropdownMenu, DropdownStyle, IconButtonShape, IconPosition,
    ListItem, ListItemSpacing, PopoverMenu, Severity, Tab, TabBar, TabPosition, Tooltip,
    prelude::*,
};
use zeddy_herdr::{Namespace, Sidecar, WorkspaceId, control::Client};
use zeddy_plugin::{InstanceContext, manifest::Multiplicity};
use zeddy_plugin_host::{Catalog, FileBroker, PaneSource, Paths, SettingsSource};

use crate::{
    actions,
    chrome::{self, Action, DraggedItem, Entry, SpaceEntries, dragged_item_preview},
    fonts::Fonts,
    item::PluginItem,
    keymap::{KeymapAction, KeymapStore},
    keys,
    mode::Mode,
    palette,
    persistence::{
        SidebarScope, Snapshot, SpaceKind as PersistedSpaceKind, StateStore, WindowState,
    },
    settings::{
        AppearanceContent, CHARTR_DARK, CHARTR_LIGHT, GeneralContent, PluginSettingsContent,
        ResolvedSettings, SettingsPage, SettingsStore, TerminalContent, ThemeMode,
    },
    space::{Kind as SpaceKind, Space, name_for},
    spaces::{self, Registry},
    terminal::{Appearance, TerminalElement},
    workspace::{
        Axis as PaneAxisDirection, Member, PaneId as LayoutPaneId, SplitDirection, Workspace,
        WorkspaceTabId,
    },
};

const BACKEND_TIMEOUT: Duration = Duration::from_secs(10);
const BACKEND_SUPERVISION: Duration = Duration::from_secs(2);
const BACKEND_STEADY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Backend {
    Starting,
    Ready,
    Recovering(String),
    Failed(String),
}

#[derive(Clone)]
struct DraggedPaneDivider {
    axis_path: Vec<usize>,
    divider: usize,
    axis: PaneAxisDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaletteCommand {
    NewTerminal,
    CloseItem,
    CloseAllItems,
    SplitLeft,
    SplitRight,
    SplitUp,
    SplitDown,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    JoinPane,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    ToggleZoom,
    OpenSettings,
}

impl PaletteCommand {
    const ALL: [(Self, &'static str, &'static str); 18] = [
        (Self::NewTerminal, "Workspace: New Terminal", "Ctrl+~"),
        (Self::CloseItem, "Pane: Close Active Item", "Cmd/Ctrl+W"),
        (Self::CloseAllItems, "Pane: Close All Items", ""),
        (Self::SplitLeft, "Pane: Split and Move Left", ""),
        (Self::SplitRight, "Pane: Split and Move Right", ""),
        (Self::SplitUp, "Pane: Split and Move Up", ""),
        (Self::SplitDown, "Pane: Split and Move Down", ""),
        (Self::MoveLeft, "Pane: Move Active Item Left", ""),
        (Self::MoveRight, "Pane: Move Active Item Right", ""),
        (Self::MoveUp, "Pane: Move Active Item Up", ""),
        (Self::MoveDown, "Pane: Move Active Item Down", ""),
        (Self::JoinPane, "Pane: Join Into Next Pane", ""),
        (Self::FocusLeft, "Pane: Focus Left", "Cmd/Ctrl+K ←"),
        (Self::FocusRight, "Pane: Focus Right", "Cmd/Ctrl+K →"),
        (Self::FocusUp, "Pane: Focus Up", "Cmd/Ctrl+K ↑"),
        (Self::FocusDown, "Pane: Focus Down", "Cmd/Ctrl+K ↓"),
        (Self::ToggleZoom, "Pane: Toggle Zoom", "Shift+Esc"),
        (Self::OpenSettings, "Chartr: Open Settings", "Cmd/Ctrl+,"),
    ];
}

impl Render for DraggedPaneDivider {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// The root view, analogous to Zed's `MultiWorkspace`.
pub struct Zeddy {
    client: Option<Client>,
    backend: Backend,
    backend_ready_since: Option<Instant>,
    backend_restart_spent: bool,
    supervision_started: bool,
    registry: Option<Registry>,
    spaces: Vec<Entity<Space>>,
    active: Option<Entity<Space>>,
    mode: Mode,
    catalog: Catalog,
    plugins_restored: bool,
    plugin_settings: Option<(String, AnyView)>,
    settings: SettingsStore,
    keymap: KeymapStore,
    settings_open: bool,
    settings_page: SettingsPage,
    recording_keymap: Option<KeymapAction>,
    keymap_restart_required: bool,
    command_palette_open: bool,
    command_palette_query: String,
    command_palette_selected: usize,
    rename_space: Option<EntityId>,
    rename_query: String,
    sidebar_scope: SidebarScope,
    sidebar_width: f32,
    window_bounds: Option<crate::persistence::WindowBounds>,
    state: Option<StateStore>,
    last_persisted: Option<String>,
    focus: FocusHandle,
    problem: Option<String>,
}

impl Zeddy {
    pub fn new(
        cwd: PathBuf,
        settings: SettingsStore,
        keymap: KeymapStore,
        cx: &mut Context<Self>,
    ) -> Self {
        let (state, saved, state_problem) =
            match crate::persistence::state_file().and_then(StateStore::open) {
                Ok(store) => match store.load() {
                    Ok(saved) => (Some(store), saved, None),
                    Err(error) => (Some(store), Snapshot::default(), Some(error.to_string())),
                },
                Err(error) => (None, Snapshot::default(), Some(error.to_string())),
            };
        let saved_json = serde_json::to_string(&saved).ok();
        let client =
            Sidecar::beside_current_exe().map(|sidecar| Client::new(sidecar, Namespace::private()));
        let client = match client {
            Ok(client) => client,
            Err(error) => {
                return Self {
                    client: None,
                    backend: Backend::Failed(error.to_string()),
                    backend_ready_since: None,
                    backend_restart_spent: false,
                    supervision_started: false,
                    registry: None,
                    spaces: Vec::new(),
                    active: None,
                    mode: Mode::default(),
                    catalog: Catalog::default(),
                    plugins_restored: true,
                    plugin_settings: None,
                    settings,
                    keymap,
                    settings_open: false,
                    settings_page: SettingsPage::default(),
                    recording_keymap: None,
                    keymap_restart_required: false,
                    command_palette_open: false,
                    command_palette_query: String::new(),
                    command_palette_selected: 0,
                    rename_space: None,
                    rename_query: String::new(),
                    sidebar_scope: saved.window.sidebar_scope,
                    sidebar_width: saved.window.sidebar_width,
                    window_bounds: saved.window.bounds,
                    state,
                    last_persisted: saved_json,
                    focus: cx.focus_handle(),
                    problem: Some(state_problem.unwrap_or_else(|| error.to_string())),
                };
            }
        };

        let (registry, registry_problem) = load_registry(&cwd);
        let mut descriptors = Vec::new();
        let home = settings
            .resolved()
            .ad_hoc_directory
            .clone()
            .or_else(std::env::home_dir)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| cwd.clone());
        descriptors.push(("Ad-hoc sessions".to_owned(), home.clone(), SpaceKind::AdHoc));
        if let Some(registry) = registry.as_ref() {
            descriptors.extend(
                registry
                    .spaces()
                    .iter()
                    // The synthetic ad-hoc space already owns the home
                    // workspace. herdr has one workspace per directory, so a
                    // second row for the same path could not own independent
                    // sessions and would be a false distinction.
                    .filter(|space| !spaces::same_path(space.path(), &home))
                    .map(|space| {
                        (space.name().to_owned(), space.path().to_path_buf(), SpaceKind::Registered)
                    }),
            );
        }
        for saved_space in &saved.spaces {
            if saved_space.kind != PersistedSpaceKind::Folder {
                continue;
            }
            let Some(path) = saved_space.path.clone() else {
                continue;
            };
            if descriptors.iter().all(|(_, existing, _)| !spaces::same_path(existing, &path)) {
                descriptors.push((saved_space.name.clone(), path, SpaceKind::Registered));
            }
        }
        if descriptors.is_empty() {
            descriptors.push((
                name_for(SpaceKind::Registered, &cwd),
                cwd.clone(),
                SpaceKind::Registered,
            ));
        }

        let spaces: Vec<_> = descriptors
            .into_iter()
            .map(|(name, path, kind)| {
                let space = cx.new(|cx| Space::new(name, path, kind, client.clone(), cx));
                cx.observe(&space, |_, _, cx| cx.notify()).detach();
                space
            })
            .collect();
        for space in &spaces {
            let key = space.read(cx).persisted().key;
            if let Some(saved_space) = saved.spaces.iter().find(|saved| saved.key == key) {
                space.update(cx, |space, _| space.restore_saved(saved_space));
            }
        }
        let active = saved
            .window
            .active_space
            .as_ref()
            .and_then(|key| {
                spaces.iter().find(|space| space.read(cx).persisted().key == *key).cloned()
            })
            .or_else(|| {
                spaces
                    .iter()
                    .find(|space| {
                        let space = space.read(cx);
                        space.kind() == SpaceKind::Registered
                            && spaces::same_path(space.path(), &cwd)
                    })
                    .cloned()
            })
            .or_else(|| spaces.first().cloned());

        let catalog = zeddy_plugin_host::load_all_where(
            &plugin_paths(),
            |id| settings.resolved().plugin(id).enabled,
            cx,
        );
        let mut this = Self {
            client: Some(client),
            backend: Backend::Starting,
            backend_ready_since: None,
            backend_restart_spent: false,
            supervision_started: false,
            registry,
            spaces,
            active,
            mode: saved.window.chrome,
            catalog,
            plugins_restored: false,
            plugin_settings: None,
            settings,
            keymap,
            settings_open: false,
            settings_page: SettingsPage::default(),
            recording_keymap: None,
            keymap_restart_required: false,
            command_palette_open: false,
            command_palette_query: String::new(),
            command_palette_selected: 0,
            rename_space: None,
            rename_query: String::new(),
            sidebar_scope: saved.window.sidebar_scope,
            sidebar_width: saved.window.sidebar_width,
            window_bounds: saved.window.bounds,
            state,
            last_persisted: saved_json,
            focus: cx.focus_handle(),
            problem: state_problem.or(registry_problem),
        };
        this.connect(cx);
        this
    }

    /// Apply the explicit exit policy while the window and its entities are
    /// still reachable. The default does nothing; `Space::drop` then sends a
    /// clean release to every attachment so Herdr can be adopted next launch.
    pub fn apply_exit_policy(&mut self, cx: &mut Context<Self>) {
        if !self.settings.resolved().terminate_sessions_on_exit {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        for space in &self.spaces {
            let ids = space.read(cx).all_item_ids();
            let targets = space.read(cx).close_targets(&ids);
            let closed: Vec<_> = targets
                .into_iter()
                .filter_map(|(item, backend)| match backend {
                    None => Some(item),
                    Some(backend) if client.close_session(&backend).is_ok() => Some(item),
                    Some(_) => None,
                })
                .collect();
            space.update(cx, |space, _| space.finish_bulk_close(&closed));
        }
    }

    /// Bring the private backend up and take one snapshot of every running
    /// session. Like Zed's project I/O, the blocking transport stays on the
    /// background executor and only owned answers return to GPUI.
    fn connect(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    client.connect(BACKEND_TIMEOUT)?;
                    client.sessions(None)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => {
                    this.backend_became_ready();
                    this.distribute(infos, cx);
                    this.start_supervision(cx);
                    cx.notify();
                }
                Err(error) => {
                    let problem = error.to_string();
                    this.backend = Backend::Failed(problem.clone());
                    this.backend_ready_since = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn backend_became_ready(&mut self) {
        self.backend = Backend::Ready;
        self.backend_ready_since = Some(Instant::now());
    }

    /// Supervise the private daemon on the same discipline as Chartr-rs: the
    /// socket is checked every two seconds, the first failure in an episode is
    /// given one clean replacement, and a replacement that cannot hold for a
    /// minute is exposed as a crash loop rather than restarted again.
    fn start_supervision(&mut self, cx: &mut Context<Self>) {
        if self.supervision_started {
            return;
        }
        self.supervision_started = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            loop {
                executor.timer(BACKEND_SUPERVISION).await;
                let check = this.update(cx, |this, _| {
                    matches!(this.backend, Backend::Ready).then(|| this.client.clone()).flatten()
                });
                let Ok(Some(client)) = check else {
                    continue;
                };
                let probe = client.clone();
                let snapshot = executor.spawn(async move { probe.sessions(None) }).await;
                let answers = if snapshot.is_ok() {
                    true
                } else {
                    let probe = client.clone();
                    executor.spawn(async move { probe.answers() }).await
                };
                if answers {
                    let _ = this.update(cx, |this, cx| {
                        if this.backend_restart_spent
                            && this
                                .backend_ready_since
                                .is_some_and(|since| since.elapsed() >= BACKEND_STEADY)
                        {
                            this.backend_restart_spent = false;
                        }
                        match snapshot {
                            Ok(infos) => this.distribute(infos, cx),
                            Err(error) => this.problem = Some(error.to_string()),
                        }
                        cx.notify();
                    });
                    continue;
                }
                let _ = this.update(cx, |this, cx| this.backend_died(client, cx));
            }
        })
        .detach();
    }

    fn backend_died(&mut self, client: Client, cx: &mut Context<Self>) {
        if !matches!(self.backend, Backend::Ready) {
            return;
        }
        if self.backend_ready_since.is_some_and(|since| since.elapsed() >= BACKEND_STEADY) {
            self.backend_restart_spent = false;
        }
        self.drop_dead_terminals(cx);
        self.backend_ready_since = None;

        if self.backend_restart_spent {
            let problem = "The terminal backend failed again before it held for 60 seconds. Chartr stopped automatic recovery to expose the crash loop.".to_owned();
            self.backend = Backend::Failed(problem);
            let executor = cx.background_executor().clone();
            executor.spawn(async move { client.clear_saved_shape() }).detach();
            cx.notify();
            return;
        }

        self.backend_restart_spent = true;
        self.backend = Backend::Recovering(
            "The terminal backend stopped answering. Starting one clean replacement…".to_owned(),
        );
        cx.notify();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    client.restart()?;
                    client.reconnect(BACKEND_TIMEOUT)?;
                    client.sessions(None)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => {
                    this.backend_became_ready();
                    this.distribute(infos, cx);
                    cx.notify();
                }
                Err(error) => {
                    this.backend = Backend::Failed(format!(
                        "Chartr could not recover the terminal backend: {error}"
                    ));
                    this.backend_ready_since = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn drop_dead_terminals(&mut self, cx: &mut Context<Self>) {
        for space in &self.spaces {
            space.update(cx, |space, _| space.drop_dead_sessions());
        }
    }

    fn retry_backend(&mut self, clean_restart: bool, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        if clean_restart {
            self.drop_dead_terminals(cx);
        }
        self.backend = if clean_restart {
            Backend::Recovering("Restarting the terminal backend…".to_owned())
        } else {
            Backend::Starting
        };
        self.backend_ready_since = None;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    if clean_restart {
                        client.restart()?;
                        client.reconnect(BACKEND_TIMEOUT)?;
                    } else {
                        client.connect(BACKEND_TIMEOUT)?;
                    }
                    client.sessions(None)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => {
                    this.backend_restart_spent = false;
                    this.backend_became_ready();
                    this.distribute(infos, cx);
                    this.start_supervision(cx);
                    cx.notify();
                }
                Err(error) => {
                    this.backend = Backend::Failed(error.to_string());
                    this.backend_ready_since = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn request_backend_restart(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let terminal_count: usize = self
            .spaces
            .iter()
            .map(|space| {
                let ids = space.read(cx).all_item_ids();
                space
                    .read(cx)
                    .close_targets(&ids)
                    .iter()
                    .filter(|(_, backend)| backend.is_some())
                    .count()
            })
            .sum();
        if terminal_count <= 1 {
            self.retry_backend(true, cx);
            return;
        }
        let prompt = window.prompt(
            gpui::PromptLevel::Critical,
            "Restart the terminal backend?",
            Some(&format!(
                "Restarting is destructive: all {terminal_count} underlying sessions will be terminated."
            )),
            &["Restart and Terminate", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if prompt.await == Ok(0) {
                let _ = this.update(cx, |this, cx| this.retry_backend(true, cx));
            }
        })
        .detach();
    }

    /// Partition a single backend snapshot by workspace path, then let each
    /// space attach its own sessions.
    fn distribute(&mut self, infos: Vec<zeddy_herdr::control::Session>, cx: &mut Context<Self>) {
        let paths_by_workspace: HashMap<WorkspaceId, PathBuf> = infos
            .iter()
            .filter_map(|info| info.cwd.clone().map(|cwd| (info.workspace.clone(), cwd)))
            .collect();
        let mut by_space: HashMap<EntityId, Vec<zeddy_herdr::control::Session>> = HashMap::new();

        for info in infos {
            let path =
                info.cwd.clone().or_else(|| paths_by_workspace.get(&info.workspace).cloned());
            let Some(path) = path else {
                continue;
            };
            let target = self.spaces.iter().find(|space| {
                let space = space.read(cx);
                spaces::same_path(space.path(), &path)
            });
            if let Some(target) = target {
                by_space.entry(target.entity_id()).or_default().push(info);
            }
        }

        for space in &self.spaces {
            if let Some(infos) = by_space.remove(&space.entity_id()) {
                space.update(cx, |space, cx| space.adopt(infos, cx));
            }
        }
    }

    fn active_space(&self) -> Option<&Entity<Space>> {
        self.active.as_ref()
    }

    fn snapshot(&self, cx: &App) -> Snapshot {
        Snapshot {
            window: WindowState {
                chrome: self.mode,
                sidebar_scope: self.sidebar_scope,
                sidebar_width: self.sidebar_width,
                active_space: self.active.as_ref().map(|space| space.read(cx).persisted().key),
                bounds: self.window_bounds,
                ..WindowState::default()
            },
            spaces: self.spaces.iter().map(|space| space.read(cx).persisted()).collect(),
        }
    }

    fn persist_if_changed(&mut self, cx: &mut Context<Self>) {
        let snapshot = self.snapshot(cx);
        let Ok(encoded) = serde_json::to_string(&snapshot) else {
            return;
        };
        if self.last_persisted.as_deref() == Some(encoded.as_str()) {
            return;
        }
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match state.save(&snapshot) {
            Ok(()) => self.last_persisted = Some(encoded),
            Err(error) => self.problem = Some(error.to_string()),
        }
    }

    fn activate(&mut self, space: Entity<Space>, window: &mut Window, cx: &mut Context<Self>) {
        if self.active.as_ref() == Some(&space) {
            window.focus(&self.focus, cx);
            return;
        }
        self.active = Some(space.clone());
        space.update(cx, |space, _| space.fit_items());
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn pick_a_folder(&mut self, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Add".into()),
        });
        cx.spawn(async move |this, cx| {
            let outcome = chosen.await;
            let _ = this.update(cx, |this, cx| match outcome {
                Ok(Ok(Some(paths))) => {
                    for path in paths {
                        this.register(path, cx);
                    }
                }
                Ok(Ok(None)) => {}
                Ok(Err(error)) => {
                    this.problem = Some(format!("choosing a folder: {error}"));
                    cx.notify();
                }
                Err(_) => {
                    this.problem = Some("the folder picker closed unexpectedly".into());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn register(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let Some(registry) = self.registry.as_mut() else {
            self.problem = Some("the space registry is unavailable".into());
            cx.notify();
            return;
        };
        let path = match registry.register(&path) {
            Ok(path) => path,
            Err(error) => {
                self.problem = Some(error.to_string());
                cx.notify();
                return;
            }
        };
        if let Some(existing) = self
            .spaces
            .iter()
            .find(|space| spaces::same_path(space.read(cx).path(), &path))
            .cloned()
        {
            self.active = Some(existing);
            self.problem = None;
            cx.notify();
            return;
        }

        let Some(client) = self.client.clone() else {
            return;
        };
        let name = name_for(SpaceKind::Registered, &path);
        let space = cx.new(|cx| Space::new(name, path, SpaceKind::Registered, client, cx));
        cx.observe(&space, |_, _, cx| cx.notify()).detach();
        self.spaces.push(space.clone());
        self.active = Some(space);
        self.problem = None;
        self.refresh(cx);
        cx.notify();
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.backend, Backend::Ready) {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let request = client.clone();
            let (result, answers) = executor
                .spawn(async move {
                    let result = request.sessions(None);
                    let answers = request.answers();
                    (result, answers)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => this.distribute(infos, cx),
                Err(_) if !answers => this.backend_died(client, cx),
                Err(error) => this.problem = Some(error.to_string()),
            });
        })
        .detach();
    }

    fn entries(&self, cx: &App) -> Vec<Entry> {
        self.active_space()
            .map(|space| space.read(cx).entries(space.entity_id()))
            .unwrap_or_default()
    }

    fn sidebar_spaces(&self, cx: &App) -> Vec<SpaceEntries> {
        let spaces: Vec<_> = match self.sidebar_scope {
            SidebarScope::AllSpaces => self.spaces.iter().collect(),
            SidebarScope::ActiveSpace => self.active.iter().collect(),
        };
        spaces
            .into_iter()
            .map(|space| {
                let read = space.read(cx);
                SpaceEntries {
                    id: space.entity_id(),
                    name: read.name().to_owned(),
                    active: self.active.as_ref() == Some(space),
                    removable: read.kind() == SpaceKind::Registered,
                    available: read.available(),
                    entries: read.entries(space.entity_id()),
                }
            })
            .collect()
    }

    fn act(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        match action {
            Action::ToggleMode => self.mode = self.mode.toggled(),
            Action::ToggleSidebarScope => {
                self.sidebar_scope = match self.sidebar_scope {
                    SidebarScope::AllSpaces => SidebarScope::ActiveSpace,
                    SidebarScope::ActiveSpace => SidebarScope::AllSpaces,
                }
            }
            Action::New => {
                if matches!(self.backend, Backend::Ready)
                    && let Some(space) = self.active.clone()
                {
                    space.update(cx, |space, cx| space.start_session(cx));
                }
            }
            Action::NewInSpace { space } => {
                if matches!(self.backend, Backend::Ready)
                    && let Some(target) =
                        self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                {
                    self.activate(target.clone(), window, cx);
                    target.update(cx, |space, cx| space.start_session(cx));
                }
            }
            Action::CloseGroup { space, tab } => {
                let Some(space) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                else {
                    return;
                };
                let ids = space.read(cx).tab_item_ids(tab);
                self.request_bulk_close(space, ids, false, "group", window, cx);
            }
            Action::MoveWorkspaceTab { space, tab, target_index } => {
                if let Some(space) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                {
                    space.update(cx, |space, _| space.move_workspace_tab(tab, target_index));
                }
            }
            Action::CloseSpace { space } => self.request_close_space(space, window, cx),
            Action::RenameSpace { space } => {
                if let Some(target) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space)
                {
                    self.rename_space = Some(space);
                    self.rename_query = target.read(cx).name().to_owned();
                    window.focus(&self.focus, cx);
                }
            }
            Action::LocateSpace { space } => self.locate_space(space, cx),
            action @ (Action::Select { .. } | Action::Close { .. }) => {
                let target = match &action {
                    Action::Select { space, .. } | Action::Close { space, .. } => space
                        .and_then(|id| self.spaces.iter().find(|space| space.entity_id() == id))
                        .cloned()
                        .or_else(|| self.active.clone()),
                    _ => None,
                };
                if let Some(space) = target {
                    if matches!(action, Action::Select { .. }) {
                        self.activate(space.clone(), window, cx);
                    }
                    space.update(cx, |space, cx| space.act(action, cx));
                }
            }
        }
        cx.notify();
    }

    fn close_active_item(&mut self, cx: &mut Context<Self>) {
        if self.command_palette_open {
            self.command_palette_open = false;
            self.command_palette_query.clear();
            cx.notify();
            return;
        }
        if self.settings_open {
            self.settings_open = false;
            self.plugin_settings = None;
            cx.notify();
            return;
        }
        let Some(space) = self.active.clone() else {
            return;
        };
        let (active, empty_pane) = space.read_with(cx, |space, _| {
            let active = space.active();
            let empty_pane = if active.is_none() {
                space.active_tab_id().and_then(|tab| {
                    let layout = space.active_layout()?;
                    let pane = layout.active_pane();
                    layout.pane(pane)?.items().is_empty().then_some((tab, pane))
                })
            } else {
                None
            };
            (active, empty_pane)
        });
        if let Some(active) = active {
            space
                .update(cx, |space, cx| space.act(Action::Close { space: None, item: active }, cx));
        } else if let Some((tab, pane)) = empty_pane {
            space.update(cx, |space, _| space.remove_empty_pane(tab, pane));
            cx.notify();
        }
    }

    fn request_close_active_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(space) = self.active.clone() else {
            return;
        };
        let (Some(tab), Some(pane)) = space.read_with(cx, |space, _| {
            (space.active_tab_id(), space.active_layout().map(Workspace::active_pane))
        }) else {
            return;
        };
        let ids = space.read(cx).pane_item_ids(tab, pane);
        if ids.is_empty() {
            space.update(cx, |space, _| space.remove_empty_pane(tab, pane));
            cx.notify();
            return;
        }
        self.request_bulk_close(space, ids, false, "pane", window, cx);
    }

    fn request_close_space(
        &mut self,
        space_id: EntityId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(space) = self.spaces.iter().find(|space| space.entity_id() == space_id).cloned()
        else {
            return;
        };
        if space.read(cx).kind() == SpaceKind::AdHoc {
            return;
        }
        let ids = space.read(cx).all_item_ids();
        self.request_bulk_close(space, ids, true, "space", window, cx);
    }

    fn request_bulk_close(
        &mut self,
        space: Entity<Space>,
        ids: Vec<crate::workspace::ItemId>,
        remove_space: bool,
        noun: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let terminal_count = space
            .read(cx)
            .close_targets(&ids)
            .iter()
            .filter(|(_, backend)| backend.is_some())
            .count();
        if terminal_count <= 1 {
            self.start_bulk_close(space, ids, remove_space, cx);
            return;
        }

        let message = format!("Close this {noun} and terminate {terminal_count} sessions?");
        let detail =
            "Closing is destructive: every underlying shell or agent process is terminated.";
        let prompt = window.prompt(
            gpui::PromptLevel::Critical,
            &message,
            Some(detail),
            &["Close and Terminate", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if prompt.await == Ok(0) {
                let _ =
                    this.update(cx, |this, cx| this.start_bulk_close(space, ids, remove_space, cx));
            }
        })
        .detach();
    }

    fn start_bulk_close(
        &mut self,
        space: Entity<Space>,
        ids: Vec<crate::workspace::ItemId>,
        remove_space: bool,
        cx: &mut Context<Self>,
    ) {
        let targets = space.read(cx).close_targets(&ids);
        let client = self.client.clone();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let results = executor
                .spawn(async move {
                    targets
                        .into_iter()
                        .map(|(item, backend)| {
                            let result = match (&client, backend) {
                                (_, None) => Ok(()),
                                (Some(client), Some(backend)) => client.close_session(&backend),
                                (None, Some(_)) => Err(zeddy_herdr::Error::Protocol(
                                    "the terminal backend is unavailable".to_owned(),
                                )),
                            };
                            (item, result)
                        })
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                let successful: Vec<_> = results
                    .iter()
                    .filter_map(|(item, result)| result.is_ok().then_some(*item))
                    .collect();
                let failures: Vec<_> = results
                    .iter()
                    .filter_map(|(_, result)| result.as_ref().err().map(ToString::to_string))
                    .collect();
                space.update(cx, |space, _| space.finish_bulk_close(&successful));
                if !failures.is_empty() {
                    this.problem = Some(failures.join("\n"));
                } else if remove_space {
                    this.remove_space_after_close(&space, cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn remove_space_after_close(&mut self, space: &Entity<Space>, cx: &mut Context<Self>) {
        let path = space.read(cx).path().clone();
        if let Some(registry) = self.registry.as_mut()
            && let Err(error) = registry.remove(&path)
        {
            self.problem = Some(error.to_string());
            return;
        }
        let Some(index) = self.spaces.iter().position(|candidate| candidate == space) else {
            return;
        };
        let was_active = self.active.as_ref() == Some(space);
        self.spaces.remove(index);
        if was_active {
            self.active = self.spaces.get(index.min(self.spaces.len().saturating_sub(1))).cloned();
        }
    }

    fn commit_space_rename(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.rename_space.take() else {
            return;
        };
        let name = self.rename_query.trim().to_owned();
        self.rename_query.clear();
        let Some(space) = self.spaces.iter().find(|space| space.entity_id() == id).cloned() else {
            return;
        };
        let path = space.read(cx).path().clone();
        let result = self
            .registry
            .as_mut()
            .map(|registry| registry.rename(&path, name.clone()))
            .unwrap_or(Ok(()));
        match result {
            Ok(()) => {
                space.update(cx, |space, _| space.set_name(name));
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn locate_space(&mut self, id: EntityId, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Locate Space".into()),
        });
        cx.spawn(async move |this, cx| {
            let outcome = chosen.await;
            let _ = this.update(cx, |this, cx| match outcome {
                Ok(Ok(Some(paths))) if !paths.is_empty() => {
                    let Some(space) =
                        this.spaces.iter().find(|space| space.entity_id() == id).cloned()
                    else {
                        return;
                    };
                    let old = space.read(cx).path().clone();
                    let new = paths[0].clone();
                    let result = this
                        .registry
                        .as_mut()
                        .map(|registry| registry.relocate(&old, &new))
                        .unwrap_or(Ok(new));
                    match result {
                        Ok(path) => {
                            space.update(cx, |space, _| space.set_path(path));
                            this.problem = None;
                            this.refresh(cx);
                        }
                        Err(error) => this.problem = Some(error.to_string()),
                    }
                    cx.notify();
                }
                Ok(Ok(_)) | Err(_) => {}
                Ok(Err(error)) => {
                    this.problem = Some(error.to_string());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.command_palette_open = false;
        self.settings_open = true;
        self.settings_page = SettingsPage::default();
        self.plugin_settings = None;
        cx.notify();
    }

    fn cycle_settings_page(&mut self, backwards: bool, cx: &mut Context<Self>) {
        let current =
            SettingsPage::ALL.iter().position(|page| *page == self.settings_page).unwrap_or(0);
        let next = if backwards {
            current.checked_sub(1).unwrap_or(SettingsPage::ALL.len() - 1)
        } else {
            (current + 1) % SettingsPage::ALL.len()
        };
        self.settings_page = SettingsPage::ALL[next];
        self.plugin_settings = None;
        cx.notify();
    }

    fn set_theme_preference(
        &mut self,
        mode: ThemeMode,
        fixed_theme: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let result = self.settings.update(|content| {
            let appearance = content.appearance.get_or_insert_with(AppearanceContent::default);
            appearance.theme_mode = Some(mode);
            if let Some(theme) = fixed_theme {
                appearance.fixed_theme = Some(theme.to_owned());
            }
        });
        match result {
            Ok(settings) => {
                crate::settings::apply_theme(settings, cx);
                theme::set_theme_settings_provider(Box::new(Fonts::from_settings(settings)), cx);
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn set_terminate_on_exit(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let result = self.settings.update(|content| {
            content
                .general
                .get_or_insert_with(GeneralContent::default)
                .terminate_sessions_on_exit = Some(enabled);
        });
        match result {
            Ok(_) => self.problem = None,
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn close_plugin_instances(&mut self, plugin: &str, cx: &mut Context<Self>) {
        for space in &self.spaces {
            let ids = space.read(cx).plugin_item_ids(plugin);
            space.update(cx, |space, _| space.finish_bulk_close(&ids));
        }
    }

    fn set_plugin_enabled(&mut self, plugin: String, enabled: bool, cx: &mut Context<Self>) {
        if enabled {
            match self.catalog.enable(&plugin_paths(), &plugin, cx) {
                Ok(()) => {}
                Err(error) => {
                    self.problem = Some(error.to_string());
                    cx.notify();
                    return;
                }
            }
        }
        let result = self.settings.update(|content| {
            content
                .plugins
                .entry(plugin.clone())
                .or_insert_with(PluginSettingsContent::default)
                .enabled = Some(enabled);
        });
        match result {
            Ok(_) => {
                if !enabled {
                    self.close_plugin_instances(&plugin, cx);
                    if self.plugin_settings.as_ref().is_some_and(|(id, _)| id == &plugin) {
                        self.plugin_settings = None;
                    }
                    self.catalog.disable(&plugin);
                }
                self.problem = None;
            }
            Err(error) => {
                if enabled {
                    self.catalog.disable(&plugin);
                }
                self.problem = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn set_plugin_unsafe(&mut self, plugin: String, enabled: bool, cx: &mut Context<Self>) {
        let result = self.settings.update(|content| {
            content
                .plugins
                .entry(plugin.clone())
                .or_insert_with(PluginSettingsContent::default)
                .unsafe_filesystem = Some(enabled);
        });
        match result {
            Ok(_) => {
                // Brokers are instance-owned. Destroying the view is the
                // revocation boundary; reopening constructs one with the new grant.
                self.close_plugin_instances(&plugin, cx);
                if self.plugin_settings.as_ref().is_some_and(|(id, _)| id == &plugin) {
                    self.plugin_settings = None;
                }
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn open_plugin_settings(
        &mut self,
        plugin: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(loaded) = self.catalog.get(&plugin) else {
            return;
        };
        let permissions = loaded.permissions().clone();
        let unsafe_filesystem = self.settings.resolved().plugin(&plugin).unsafe_filesystem;
        let source = self.catalog.get_mut(&plugin).and_then(|loaded| loaded.settings(window, cx));
        let view = match source {
            Some(SettingsSource::Native(view)) => view,
            Some(SettingsSource::Web(entry)) => crate::web_plugin::view(
                entry,
                FileBroker::new(
                    None,
                    plugin_paths().data.join(&plugin),
                    permissions.project_files,
                    unsafe_filesystem,
                ),
                permissions,
                None,
                None,
                window,
                cx,
            ),
            None => return,
        };
        self.plugin_settings = Some((plugin, view));
        cx.notify();
    }

    fn set_ui_font(&mut self, family: String, cx: &mut Context<Self>) {
        let result = self.settings.update(|content| {
            content.appearance.get_or_insert_with(AppearanceContent::default).ui_font_family =
                Some(family);
        });
        match result {
            Ok(settings) => {
                theme::set_theme_settings_provider(Box::new(Fonts::from_settings(settings)), cx);
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn adjust_ui_font_size(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = self.settings.resolved().ui_font_size;
        let result = self.settings.update(|content| {
            content.appearance.get_or_insert_with(AppearanceContent::default).ui_font_size =
                Some((current + delta).clamp(8., 32.));
        });
        match result {
            Ok(settings) => {
                theme::set_theme_settings_provider(Box::new(Fonts::from_settings(settings)), cx);
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn set_terminal_font(&mut self, family: String, cx: &mut Context<Self>) {
        let result = self.settings.update(|content| {
            content.terminal.get_or_insert_with(TerminalContent::default).font_family =
                Some(family);
        });
        match result {
            Ok(_) => self.problem = None,
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn adjust_terminal_font_size(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = self.settings.resolved().terminal_font_size;
        let result = self.settings.update(|content| {
            content.terminal.get_or_insert_with(TerminalContent::default).font_size =
                Some((current + delta).clamp(8., 72.));
        });
        match result {
            Ok(_) => self.problem = None,
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn pick_ad_hoc_directory(&mut self, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Use for Ad-hoc sessions".into()),
        });
        cx.spawn(async move |this, cx| {
            let outcome = chosen.await;
            let _ = this.update(cx, |this, cx| match outcome {
                Ok(Ok(Some(paths))) if !paths.is_empty() => {
                    let path = paths[0].clone();
                    match this.settings.update(|content| {
                        content
                            .terminal
                            .get_or_insert_with(TerminalContent::default)
                            .ad_hoc_directory = Some(path.clone());
                    }) {
                        Ok(_) => {
                            if let Some(space) = this
                                .spaces
                                .iter()
                                .find(|space| space.read(cx).kind() == SpaceKind::AdHoc)
                            {
                                space.update(cx, |space, _| space.set_path(path));
                            }
                            this.problem = None;
                        }
                        Err(error) => this.problem = Some(error.to_string()),
                    }
                    cx.notify();
                }
                Ok(Ok(_)) | Err(_) => {}
                Ok(Err(error)) => {
                    this.problem = Some(error.to_string());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn split_and_move(&mut self, direction: SplitDirection, cx: &mut Context<Self>) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.split_and_move(direction));
            cx.notify();
        }
    }

    fn split_and_move_in(
        &mut self,
        tab: WorkspaceTabId,
        pane: LayoutPaneId,
        direction: SplitDirection,
        cx: &mut Context<Self>,
    ) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.split_and_move_in(tab, pane, direction));
            cx.notify();
        }
    }

    fn move_active_to_pane(&mut self, direction: SplitDirection, cx: &mut Context<Self>) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.move_active_to_pane(direction));
            cx.notify();
        }
    }

    fn activate_pane_in_direction(
        &mut self,
        direction: SplitDirection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.activate_pane_in_direction(direction));
            window.focus(&self.focus, cx);
            cx.notify();
        }
    }

    fn join_active_into_next(&mut self, cx: &mut Context<Self>) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.join_active_into_next());
            cx.notify();
        }
    }

    fn toggle_zoom(&mut self, cx: &mut Context<Self>) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.toggle_zoom());
            cx.notify();
        }
    }

    fn toggle_zoom_in(&mut self, tab: WorkspaceTabId, pane: LayoutPaneId, cx: &mut Context<Self>) {
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.toggle_zoom_in(tab, pane));
            cx.notify();
        }
    }

    fn toggle_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = !self.command_palette_open;
        self.command_palette_query.clear();
        self.command_palette_selected = 0;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn filtered_palette_commands(&self) -> Vec<(PaletteCommand, &'static str, &'static str)> {
        let query = self.command_palette_query.trim().to_lowercase();
        PaletteCommand::ALL
            .into_iter()
            .filter(|(_, label, _)| query.is_empty() || label.to_lowercase().contains(&query))
            .collect()
    }

    fn invoke_palette_command(
        &mut self,
        command: PaletteCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_palette_open = false;
        self.command_palette_query.clear();
        let action: Box<dyn gpui::Action> = match command {
            PaletteCommand::NewTerminal => Box::new(actions::workspace::NewTerminal),
            PaletteCommand::CloseItem => Box::new(actions::pane::CloseActiveItem),
            PaletteCommand::CloseAllItems => Box::new(actions::pane::CloseAllItems),
            PaletteCommand::SplitLeft => Box::new(actions::pane::SplitAndMoveLeft),
            PaletteCommand::SplitRight => Box::new(actions::pane::SplitAndMoveRight),
            PaletteCommand::SplitUp => Box::new(actions::pane::SplitAndMoveUp),
            PaletteCommand::SplitDown => Box::new(actions::pane::SplitAndMoveDown),
            PaletteCommand::MoveLeft => Box::new(actions::pane::MoveLeft),
            PaletteCommand::MoveRight => Box::new(actions::pane::MoveRight),
            PaletteCommand::MoveUp => Box::new(actions::pane::MoveUp),
            PaletteCommand::MoveDown => Box::new(actions::pane::MoveDown),
            PaletteCommand::JoinPane => Box::new(actions::pane::JoinIntoNext),
            PaletteCommand::FocusLeft => Box::new(actions::workspace::ActivatePaneLeft),
            PaletteCommand::FocusRight => Box::new(actions::workspace::ActivatePaneRight),
            PaletteCommand::FocusUp => Box::new(actions::workspace::ActivatePaneUp),
            PaletteCommand::FocusDown => Box::new(actions::workspace::ActivatePaneDown),
            PaletteCommand::ToggleZoom => Box::new(actions::workspace::ToggleZoom),
            PaletteCommand::OpenSettings => Box::new(actions::settings::Open),
        };
        window.dispatch_action(action, cx);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_key(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.rename_space.is_some() {
            cx.stop_propagation();
            match event.keystroke.key.as_str() {
                "escape" => {
                    self.rename_space = None;
                    self.rename_query.clear();
                }
                "enter" => {
                    self.commit_space_rename(cx);
                    return;
                }
                "backspace" => {
                    self.rename_query.pop();
                }
                _ if !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt =>
                {
                    if let Some(text) = event.keystroke.key_char.as_deref() {
                        self.rename_query.push_str(text);
                    }
                }
                _ => {}
            }
            cx.notify();
            return;
        }
        if let Some(action) = self.recording_keymap {
            cx.stop_propagation();
            if event.keystroke.key == "escape" {
                self.recording_keymap = None;
                cx.notify();
                return;
            }
            if matches!(
                event.keystroke.key.as_str(),
                "shift" | "control" | "alt" | "cmd" | "super" | "fn"
            ) {
                return;
            }
            let key = event.keystroke.unparse();
            match self.keymap.set(action, key) {
                Ok(()) => {
                    self.recording_keymap = None;
                    self.keymap_restart_required = true;
                    self.problem = None;
                }
                Err(error) => self.problem = Some(error.to_string()),
            }
            cx.notify();
            return;
        }
        if self.command_palette_open {
            cx.stop_propagation();
            let key = event.keystroke.key.as_str();
            match key {
                "escape" => {
                    self.command_palette_open = false;
                    self.command_palette_query.clear();
                }
                "backspace" => {
                    self.command_palette_query.pop();
                    self.command_palette_selected = 0;
                }
                "up" => {
                    let count = self.filtered_palette_commands().len();
                    if count > 0 {
                        self.command_palette_selected =
                            self.command_palette_selected.checked_sub(1).unwrap_or(count - 1);
                    }
                }
                "down" => {
                    let count = self.filtered_palette_commands().len();
                    if count > 0 {
                        self.command_palette_selected = (self.command_palette_selected + 1) % count;
                    }
                }
                "enter" => {
                    if let Some((command, _, _)) =
                        self.filtered_palette_commands().get(self.command_palette_selected).copied()
                    {
                        self.invoke_palette_command(command, window, cx);
                        return;
                    }
                }
                _ if !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt =>
                {
                    if let Some(text) = event.keystroke.key_char.as_deref() {
                        self.command_palette_query.push_str(text);
                        self.command_palette_selected = 0;
                    }
                }
                _ => {}
            }
            cx.notify();
            return;
        }
        if event.keystroke.key == "escape" && cx.stop_active_drag(window) {
            if let Some(space) = self.active.clone() {
                space.update(cx, |space, _| {
                    space.clear_drag_target();
                });
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.settings_open {
            if event.keystroke.modifiers.control && event.keystroke.key == "tab" {
                cx.stop_propagation();
                self.cycle_settings_page(event.keystroke.modifiers.shift, cx);
            }
            return;
        }
        let Some(bytes) = keys::bytes_for(&event.keystroke) else {
            return;
        };
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, cx| space.send_active(&bytes, cx));
        }
    }

    fn space_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let current = self
            .active
            .as_ref()
            .map(|space| space.read(cx).name().to_owned())
            .unwrap_or_else(|| "No space".to_owned());
        let active_id = self.active.as_ref().map(Entity::entity_id);
        let weak = cx.weak_entity();
        let spaces: Vec<_> = self
            .spaces
            .iter()
            .map(|space| {
                let read = space.read(cx);
                (space.clone(), read.name().to_owned(), read.kind())
            })
            .collect();

        let menu = ContextMenu::build(window, cx, move |menu, _, _| {
            let add = weak.clone();
            let mut menu = menu.entry("New space…", None, move |_, cx| {
                let _ = add.update(cx, |this, cx| this.pick_a_folder(cx));
            });

            for (space, name, _kind) in
                spaces.iter().filter(|(_, _, kind)| *kind == SpaceKind::AdHoc)
            {
                let target = space.clone();
                let select = weak.clone();
                menu = menu.toggleable_entry(
                    name.clone(),
                    active_id == Some(space.entity_id()),
                    IconPosition::Start,
                    None,
                    move |window, cx| {
                        let _ =
                            select.update(cx, |this, cx| this.activate(target.clone(), window, cx));
                    },
                );
            }

            let registered: Vec<_> =
                spaces.iter().filter(|(_, _, kind)| *kind == SpaceKind::Registered).collect();
            if !registered.is_empty() {
                menu = menu.separator();
            }
            for (space, name, _) in registered {
                let target = space.clone();
                let select = weak.clone();
                menu = menu.toggleable_entry(
                    name.clone(),
                    active_id == Some(space.entity_id()),
                    IconPosition::Start,
                    None,
                    move |window, cx| {
                        let _ =
                            select.update(cx, |this, cx| this.activate(target.clone(), window, cx));
                    },
                );
            }
            menu
        });

        DropdownMenu::new("space-switcher", current, menu)
            .style(DropdownStyle::Ghost)
            .full_width(true)
            .attach(Anchor::BottomLeft)
            .aria_label("Current space")
            .into_any_element()
    }

    fn new_item_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let weak = cx.weak_entity();
        let panes: Vec<_> = self.catalog.panes().into_iter().cloned().collect();
        PopoverMenu::new("new-item-menu")
            .trigger_with_tooltip(
                IconButton::new("new-item", IconName::Plus).icon_size(IconSize::Small),
                Tooltip::text("New…"),
            )
            .anchor(Anchor::TopRight)
            .menu(move |window, cx| {
                let weak = weak.clone();
                let panes = panes.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    let start = weak.clone();
                    let mut menu = menu.entry("New session", None, move |window, cx| {
                        let _ = start.update(cx, |this, cx| this.act(Action::New, window, cx));
                    });
                    if !panes.is_empty() {
                        menu = menu.separator();
                    }
                    for pane in &panes {
                        let open = weak.clone();
                        let key = pane.key.clone();
                        menu = menu.entry(pane.title.clone(), None, move |window, cx| {
                            let _ = open
                                .update(cx, |this, cx| this.open_plugin(key.clone(), window, cx));
                        });
                    }
                    menu
                }))
            })
            .into_any_element()
    }

    fn web_plugin_focus_handler(
        space: Entity<Space>,
        cx: &Context<Self>,
    ) -> crate::web_plugin::FocusHandler {
        let weak = cx.weak_entity();
        Rc::new(move |view, cx| {
            let space = space.clone();
            let _ = weak.update(cx, |this, cx| {
                if space.update(cx, |space, _| space.activate_plugin_view(view)) {
                    this.active = Some(space);
                    cx.notify();
                }
            });
        })
    }

    fn open_plugin(
        &mut self,
        key: zeddy_plugin::PaneKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title =
            self.catalog.panes().iter().find(|pane| pane.key == key).map(|pane| pane.title.clone());
        let Some(title) = title else {
            self.problem = Some("That plugin contribution is no longer available.".to_owned());
            cx.notify();
            return;
        };
        let (capabilities, permissions) = match self.catalog.get(&key.plugin) {
            Some(plugin) => (plugin.capabilities().clone(), plugin.permissions().clone()),
            None => {
                self.problem = Some("That plugin is no longer loaded.".to_owned());
                cx.notify();
                return;
            }
        };
        let Some(space) = self.active.clone() else {
            return;
        };
        let bound_session = if capabilities.session_binding {
            let Some(session) = space.read(cx).active_session_id() else {
                self.problem =
                    Some("Select a terminal before opening this session-bound plugin.".to_owned());
                cx.notify();
                return;
            };
            Some(session)
        } else {
            None
        };
        if capabilities.multiplicity == Multiplicity::PerSpace
            && space.update(cx, |space, _| space.activate_plugin(&key))
        {
            self.problem = None;
            cx.notify();
            return;
        }
        let project =
            (space.read(cx).kind() == SpaceKind::Registered).then(|| space.read(cx).path().clone());
        let instance = InstanceContext {
            space: space.read(cx).key(),
            project_dir: project.clone(),
            bound_session: bound_session.as_ref().map(|session| session.0.clone()),
        };
        let session_access =
            bound_session.as_ref().and_then(|session| space.read(cx).session_access(session));
        let on_focus = Some(Self::web_plugin_focus_handler(space.clone(), cx));
        let unsafe_filesystem = self.settings.resolved().plugin(&key.plugin).unsafe_filesystem;
        let Some(plugin) = self.catalog.get_mut(&key.plugin) else {
            self.problem = Some("That plugin is no longer loaded.".to_owned());
            cx.notify();
            return;
        };
        let view = match plugin.pane(&key) {
            Some(PaneSource::Native(plugin)) => plugin.view(&key, &instance, window, cx),
            Some(PaneSource::Web(entry)) => {
                let broker = FileBroker::new(
                    project,
                    plugin_paths().data.join(&key.plugin),
                    permissions.project_files,
                    unsafe_filesystem,
                );
                crate::web_plugin::view(
                    entry.to_path_buf(),
                    broker,
                    permissions.clone(),
                    session_access,
                    on_focus,
                    window,
                    cx,
                )
            }
            None => {
                self.problem = Some("That pane is no longer contributed.".to_owned());
                cx.notify();
                return;
            }
        };
        space.update(cx, |space, cx| {
            space.open_plugin(
                PluginItem {
                    contribution: key,
                    title,
                    view,
                    bound_session,
                    can_clone: capabilities.cloneable,
                    restorable: capabilities.restorable,
                },
                cx,
            );
        });
        self.problem = None;
        cx.notify();
    }

    fn restore_plugins_once(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.plugins_restored {
            return;
        }
        if matches!(self.backend, Backend::Starting | Backend::Recovering(_)) {
            return;
        }
        self.plugins_restored = true;
        let mut failures = Vec::new();
        for space in self.spaces.clone() {
            let records = space.update(cx, |space, _| space.take_restoring_plugins());
            for record in &records {
                let crate::persistence::PersistedItem::Plugin {
                    plugin, pane, bound_session, ..
                } = record
                else {
                    continue;
                };
                let key = zeddy_plugin::PaneKey { plugin: plugin.clone(), key: pane.clone() };
                let descriptor = self.catalog.get(plugin).and_then(|loaded| {
                    loaded.panes.iter().find(|candidate| candidate.key == key).map(|candidate| {
                        (
                            candidate.title.clone(),
                            loaded.capabilities().clone(),
                            loaded.permissions().clone(),
                        )
                    })
                });
                let Some((title, capabilities, permissions)) = descriptor else {
                    failures.push(format!("{plugin}:{pane} is unavailable"));
                    continue;
                };
                if !capabilities.restorable {
                    failures.push(format!("{plugin}:{pane} does not support restoration"));
                    continue;
                }
                let project = (space.read(cx).kind() == SpaceKind::Registered)
                    .then(|| space.read(cx).path().clone());
                let unsafe_filesystem = self.settings.resolved().plugin(plugin).unsafe_filesystem;
                let bound = bound_session.clone().map(zeddy_herdr::PaneId);
                let session_access =
                    bound.as_ref().and_then(|session| space.read(cx).session_access(session));
                let on_focus = Some(Self::web_plugin_focus_handler(space.clone(), cx));
                if bound.is_some() && session_access.is_none() {
                    failures.push(format!("{plugin}:{pane} lost its bound session"));
                    continue;
                }
                let instance = InstanceContext {
                    space: space.read(cx).key(),
                    project_dir: project.clone(),
                    bound_session: bound_session.clone(),
                };
                let Some(loaded) = self.catalog.get_mut(plugin) else {
                    failures.push(format!("{plugin}:{pane} is disabled"));
                    continue;
                };
                let view = match loaded.pane(&key) {
                    Some(PaneSource::Native(plugin)) => plugin.view(&key, &instance, window, cx),
                    Some(PaneSource::Web(entry)) => {
                        let broker = FileBroker::new(
                            project,
                            plugin_paths().data.join(plugin),
                            permissions.project_files,
                            unsafe_filesystem,
                        );
                        crate::web_plugin::view(
                            entry.to_path_buf(),
                            broker,
                            permissions.clone(),
                            session_access,
                            on_focus,
                            window,
                            cx,
                        )
                    }
                    None => {
                        failures.push(format!("{plugin}:{pane} is no longer contributed"));
                        continue;
                    }
                };
                let item = PluginItem {
                    contribution: key,
                    title,
                    view,
                    bound_session: bound,
                    can_clone: capabilities.cloneable,
                    restorable: true,
                };
                if !space.update(cx, |space, cx| space.restore_plugin(record, item, cx)) {
                    failures.push(format!("{plugin}:{pane} had no saved layout item"));
                }
            }
            space.update(cx, |space, _| space.remove_plugin_placeholders(&records));
        }
        if !failures.is_empty() {
            self.problem =
                Some(format!("Some plugin items could not be restored: {}.", failures.join(", ")));
        }
    }

    fn clone_plugin_drop(
        &mut self,
        space: Entity<Space>,
        source_item: crate::workspace::ItemId,
        target_tab: WorkspaceTabId,
        target: LayoutPaneId,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some((key, title, bound_session)) = space.read(cx).cloneable_plugin(source_item) else {
            return false;
        };
        let Some(loaded) = self.catalog.get(&key.plugin) else {
            return false;
        };
        let capabilities = loaded.capabilities().clone();
        let permissions = loaded.permissions().clone();
        let project =
            (space.read(cx).kind() == SpaceKind::Registered).then(|| space.read(cx).path().clone());
        let instance = InstanceContext {
            space: space.read(cx).key(),
            project_dir: project.clone(),
            bound_session: bound_session.as_ref().map(|session| session.0.clone()),
        };
        let session_access =
            bound_session.as_ref().and_then(|session| space.read(cx).session_access(session));
        let on_focus = Some(Self::web_plugin_focus_handler(space.clone(), cx));
        let unsafe_filesystem = self.settings.resolved().plugin(&key.plugin).unsafe_filesystem;
        let Some(destination) =
            space.update(cx, |space, _| space.prepare_drop_destination(target_tab, target))
        else {
            return true;
        };
        let Some(loaded) = self.catalog.get_mut(&key.plugin) else {
            return false;
        };
        let view = match loaded.pane(&key) {
            Some(PaneSource::Native(plugin)) => plugin.view(&key, &instance, window, cx),
            Some(PaneSource::Web(entry)) => crate::web_plugin::view(
                entry.to_path_buf(),
                FileBroker::new(
                    project,
                    plugin_paths().data.join(&key.plugin),
                    permissions.project_files,
                    unsafe_filesystem,
                ),
                permissions,
                session_access,
                on_focus,
                window,
                cx,
            ),
            None => return false,
        };
        space.update(cx, |space, cx| {
            space.open_plugin_in(
                PluginItem {
                    contribution: key,
                    title,
                    view,
                    bound_session,
                    can_clone: capabilities.cloneable,
                    restorable: capabilities.restorable,
                },
                target_tab,
                destination,
                index,
                cx,
            );
        });
        true
    }

    /// Zed has one pane drop path shared by tab targets and the pane body.
    /// Body drops may consume the current edge split direction; tab-bar drops
    /// explicitly clear it and only reorder or move into the target pane.
    fn handle_item_drop(
        &mut self,
        dragged: &DraggedItem,
        target_tab: WorkspaceTabId,
        target: LayoutPaneId,
        index: usize,
        allow_split: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(space) = self.active.clone() else {
            return;
        };
        if space.read(cx).key() != dragged.space {
            space.update(cx, |space, _| {
                space.clear_drag_target();
            });
            cx.notify();
            return;
        }
        if !allow_split {
            space.update(cx, |space, _| space.set_drag_target(target_tab, target, None));
        }
        let clone = cfg!(target_os = "macos") && window.modifiers().alt
            || cfg!(not(target_os = "macos")) && window.modifiers().control;
        if clone
            && self.clone_plugin_drop(
                space.clone(),
                dragged.item,
                target_tab,
                target,
                Some(index),
                window,
                cx,
            )
        {
            cx.notify();
            return;
        }
        space.update(cx, |space, _| {
            space.drop_item(
                dragged.item,
                dragged.tab,
                dragged.pane,
                target_tab,
                target,
                Some(index),
            );
        });
        cx.notify();
    }

    fn workspace_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(space) = self.active.clone() else {
            return message("No space. Add a folder to begin.", cx).into_any_element();
        };
        let (problem, active) = space.update(cx, |space, _| {
            space.fit_items();
            (space.problem().map(str::to_owned), space.active())
        });
        let on_action =
            cx.listener(|this, action: &Action, window, cx| this.act(action.clone(), window, cx));
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));
        let space = space.read(cx);
        if active.is_some_and(|active| space.item(active).is_none()) {
            return message("That item is gone.", cx).into_any_element();
        }
        let weak = cx.weak_entity();
        let workspace = if let Some(tab) = space.workspace_tabs().active_tab() {
            let layout = &tab.layout;
            let show_pane_headers = tab.is_grouped();
            if let Some(maximized) = layout.center.maximized {
                self.render_pane(
                    &space,
                    tab.id,
                    layout,
                    maximized,
                    show_pane_headers,
                    &emit,
                    &weak,
                    window,
                    cx,
                )
            } else {
                self.render_member(
                    &space,
                    tab.id,
                    layout,
                    &layout.center.root,
                    show_pane_headers,
                    &emit,
                    &weak,
                    &[],
                    window,
                    cx,
                )
            }
        } else {
            message("No tabs. Create a new item to begin.", cx).into_any_element()
        };
        let notices = self.workspace_notices(problem, cx);
        v_flex()
            .size_full()
            .min_h_0()
            .children(notices)
            .child(div().flex_1().min_h_0().child(workspace))
            .into_any_element()
    }

    fn workspace_notices(
        &mut self,
        space_problem: Option<String>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut notices = Vec::new();
        match self.backend.clone() {
            Backend::Ready => {}
            Backend::Starting => notices.push(
                Banner::new()
                    .child(Label::new("Starting the terminal backend…").size(LabelSize::Small))
                    .into_any_element(),
            ),
            Backend::Recovering(detail) => notices.push(
                Banner::new()
                    .severity(Severity::Warning)
                    .child(Label::new(detail).size(LabelSize::Small))
                    .into_any_element(),
            ),
            Backend::Failed(detail) => {
                let retry = cx.listener(|this, _, _, cx| this.retry_backend(false, cx));
                let restart =
                    cx.listener(|this, _, window, cx| this.request_backend_restart(window, cx));
                notices.push(
                    Banner::new()
                        .severity(Severity::Error)
                        .wrap_content(true)
                        .child(Label::new(detail).size(LabelSize::Small))
                        .action_slot(
                            h_flex()
                                .gap_1()
                                .child(Button::new("retry-backend", "Retry").on_click(retry))
                                .child(
                                    Button::new("restart-backend", "Restart Backend")
                                        .on_click(restart),
                                ),
                        )
                        .into_any_element(),
                );
            }
        }
        if let Some(problem) = self.problem.clone() {
            notices.push(
                Banner::new()
                    .severity(Severity::Warning)
                    .child(Label::new(problem).size(LabelSize::Small))
                    .into_any_element(),
            );
        }
        if let Some(problem) = space_problem {
            notices.push(
                Banner::new()
                    .severity(Severity::Warning)
                    .child(Label::new(problem).size(LabelSize::Small))
                    .into_any_element(),
            );
        }
        notices
    }

    fn render_member(
        &self,
        space: &Space,
        tab_id: WorkspaceTabId,
        layout: &Workspace,
        member: &Member,
        show_pane_headers: bool,
        on: &chrome::Emit,
        weak: &gpui::WeakEntity<Self>,
        axis_path: &[usize],
        window: &mut Window,
        cx: &App,
    ) -> AnyElement {
        match member {
            Member::Pane { pane } => self.render_pane(
                space,
                tab_id,
                layout,
                *pane,
                show_pane_headers,
                on,
                weak,
                window,
                cx,
            ),
            Member::Axis(axis) => {
                let member_count = axis.members.len();
                let children: Vec<_> = axis
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, member)| {
                        let flex = axis.flexes.get(index).copied().unwrap_or(1.).max(0.01);
                        let mut child_path = axis_path.to_vec();
                        child_path.push(index);
                        let divider = DraggedPaneDivider {
                            axis_path: axis_path.to_vec(),
                            divider: index,
                            axis: axis.axis,
                        };
                        div()
                            .relative()
                            .flex_grow(flex)
                            .flex_basis(relative(0.))
                            .min_w_0()
                            .min_h_0()
                            .child(self.render_member(
                                space,
                                tab_id,
                                layout,
                                member,
                                show_pane_headers,
                                on,
                                weak,
                                &child_path,
                                window,
                                cx,
                            ))
                            .when(index + 1 < member_count, |child| {
                                child.child(pane_resize_handle(divider, axis.axis))
                            })
                    })
                    .collect();
                let resize = weak.clone();
                let current_path = axis_path.to_vec();
                let axis_direction = axis.axis;
                match axis.axis {
                    PaneAxisDirection::Horizontal => h_flex()
                        .id(format!("pane-axis-h-{current_path:?}"))
                        .size_full()
                        .items_stretch()
                        .min_w_0()
                        .min_h_0()
                        .gap_px()
                        .bg(cx.theme().colors().border)
                        .on_drag_move::<DraggedPaneDivider>(move |event, _, cx| {
                            let dragged = event.drag(cx).clone();
                            if dragged.axis_path != current_path || dragged.axis != axis_direction {
                                return;
                            }
                            let fraction = (event.event.position.x - event.bounds.left())
                                / event.bounds.size.width;
                            let _ = resize.update(cx, |this, cx| {
                                if let Some(space) = this.active.clone() {
                                    space.update(cx, |space, _| {
                                        space.resize_divider(
                                            tab_id,
                                            &dragged.axis_path,
                                            dragged.divider,
                                            fraction,
                                        )
                                    });
                                }
                                cx.notify();
                            });
                        })
                        .children(children)
                        .into_any_element(),
                    PaneAxisDirection::Vertical => {
                        let resize = weak.clone();
                        let current_path = axis_path.to_vec();
                        v_flex()
                            .id(format!("pane-axis-v-{current_path:?}"))
                            .size_full()
                            .min_w_0()
                            .min_h_0()
                            .gap_px()
                            .bg(cx.theme().colors().border)
                            .on_drag_move::<DraggedPaneDivider>(move |event, _, cx| {
                                let dragged = event.drag(cx).clone();
                                if dragged.axis_path != current_path
                                    || dragged.axis != PaneAxisDirection::Vertical
                                {
                                    return;
                                }
                                let fraction = (event.event.position.y - event.bounds.top())
                                    / event.bounds.size.height;
                                let _ = resize.update(cx, |this, cx| {
                                    if let Some(space) = this.active.clone() {
                                        space.update(cx, |space, _| {
                                            space.resize_divider(
                                                tab_id,
                                                &dragged.axis_path,
                                                dragged.divider,
                                                fraction,
                                            )
                                        });
                                    }
                                    cx.notify();
                                });
                            })
                            .children(children)
                            .into_any_element()
                    }
                }
            }
        }
    }

    fn render_pane(
        &self,
        space: &Space,
        tab_id: WorkspaceTabId,
        layout: &Workspace,
        pane_id: LayoutPaneId,
        show_header: bool,
        on: &chrome::Emit,
        weak: &gpui::WeakEntity<Self>,
        window: &mut Window,
        cx: &App,
    ) -> AnyElement {
        let Some(pane) = layout.pane(pane_id) else {
            return message("Pane layout is unavailable.", cx).into_any_element();
        };
        let active_pane = layout.active_pane() == pane_id;
        let header = show_header.then(|| {
            if pane.active().is_some() {
                self.pane_header(space, tab_id, layout, pane_id, on, weak, cx)
            } else {
                self.empty_pane_header(tab_id, pane_id, weak)
            }
        });
        let content = pane
            .active()
            .and_then(|id| space.item(id).map(|item| (id, item)))
            .map(|(id, item)| match item {
                crate::item::Item::Session(item) => {
                    let terminal = terminal(
                        item,
                        active_pane && self.focus.is_focused(window),
                        self.settings.resolved(),
                        cx,
                    )
                    .into_any_element();
                    let ended = item.session.ended();
                    let retrying = space.reattaching(id);
                    let retry = weak.clone();
                    v_flex()
                        .relative()
                        .size_full()
                        .child(terminal)
                        .when_some(ended, |view, ended| {
                            let detail = match &ended {
                                crate::session::Ended::Closed => {
                                    "Session ended. Close this tab when you are done reviewing it."
                                        .to_owned()
                                }
                                crate::session::Ended::Failed(error) => {
                                    format!("Terminal connection failed: {error}")
                                }
                            };
                            view.child(
                                div().absolute().left_2().right_2().bottom_2().child(
                                    Banner::new()
                                        .severity(Severity::Error)
                                        .child(Label::new(detail).size(LabelSize::Small))
                                        .when(
                                            matches!(ended, crate::session::Ended::Failed(_)),
                                            |banner| {
                                                banner.action_slot(
                                                    Button::new(
                                                        format!("reattach-session-{}", id.get()),
                                                        if retrying {
                                                            "Reattaching…"
                                                        } else {
                                                            "Reattach"
                                                        },
                                                    )
                                                    .disabled(retrying)
                                                    .on_click(move |_, _, cx| {
                                                        let _ = retry.update(cx, |this, cx| {
                                                            if let Some(space) = this.active.clone()
                                                            {
                                                                space.update(cx, |space, cx| {
                                                                    space.reattach(id, cx)
                                                                });
                                                            }
                                                        });
                                                    }),
                                                )
                                            },
                                        ),
                                ),
                            )
                        })
                        .into_any_element()
                }
                crate::item::Item::Plugin(item) => item.view.clone().into_any_element(),
            })
            .unwrap_or_else(|| {
                empty_pane_message("Drop a tab here or create a new item.", cx).into_any_element()
            });

        let drag_move = weak.clone();
        let drop_item = weak.clone();
        let focus_pane = weak.clone();
        let drop_group = format!("workspace-tab-{}-pane-drop-{}", tab_id.get(), pane_id.get());
        let drop_space = space.key();
        let drag_space = drop_space.clone();
        let pane_drop_index = pane
            .active()
            .and_then(|active| pane.items().iter().position(|item| *item == active))
            .unwrap_or(pane.items().len());
        let drop_direction = space
            .drag_target()
            .filter(|(tab, pane, _)| *tab == tab_id && *pane == pane_id)
            .and_then(|(_, _, direction)| direction);
        v_flex()
            .id(format!("workspace-tab-{}-pane-{}", tab_id.get(), pane_id.get()))
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(cx.theme().colors().editor_background)
            .when(pane.active().is_none() && active_pane, |pane| {
                pane.role(Role::Group).aria_label("Empty pane").tab_group().tab_index(0)
            })
            .capture_any_mouse_down(move |_, window, cx| {
                let _ = focus_pane.update(cx, |this, cx| {
                    if let Some(space) = this.active.clone() {
                        space.update(cx, |space, _| space.activate_pane(tab_id, pane_id));
                    }
                    window.focus(&this.focus, cx);
                    cx.notify();
                });
            })
            .children(header)
            .child(
                div()
                    .flex_1()
                    .relative()
                    .min_h_0()
                    .min_w_0()
                    .group(drop_group.clone())
                    .on_drag_move::<DraggedItem>(move |event, _, cx| {
                        let Some(direction) = pane_drop_direction_for_drag(event) else {
                            // GPUI dispatches drag-move callbacks during capture even when the
                            // pointer is outside this element. Zed keeps split intent on each
                            // Pane entity; Chartr's shared space state must therefore ignore
                            // callbacks from every pane except the one under the pointer.
                            return;
                        };
                        let accepted = event.drag(cx).space == drag_space;
                        let _ = drag_move.update(cx, |this, cx| {
                            let changed = this.active.clone().is_some_and(|space| {
                                space.update(cx, |space, _| {
                                    if accepted {
                                        space.set_drag_target(tab_id, pane_id, direction)
                                    } else {
                                        space.clear_drag_target()
                                    }
                                })
                            });
                            if changed {
                                cx.notify();
                            }
                        });
                    })
                    .child(content)
                    .child(drop_target(drop_direction, drop_group, drop_space, cx).on_drop(
                        move |dragged: &DraggedItem, window, cx| {
                            let dragged = dragged.clone();
                            let _ = drop_item.update(cx, |this, cx| {
                                this.handle_item_drop(
                                    &dragged,
                                    tab_id,
                                    pane_id,
                                    pane_drop_index,
                                    true,
                                    window,
                                    cx,
                                );
                            });
                        },
                    )),
            )
            .into_any_element()
    }

    fn empty_pane_header(
        &self,
        tab_id: WorkspaceTabId,
        pane_id: LayoutPaneId,
        weak: &gpui::WeakEntity<Self>,
    ) -> AnyElement {
        let close = weak.clone();
        TabBar::new(format!("workspace-tab-{}-pane-{}-empty", tab_id.get(), pane_id.get()))
            .end_child(
                IconButton::new(
                    format!("close-empty-pane-{}-{}", tab_id.get(), pane_id.get()),
                    IconName::Close,
                )
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .icon_size(IconSize::XSmall)
                .aria_label("Close Empty Pane")
                .tooltip(Tooltip::text("Close Empty Pane"))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    let _ = close.update(cx, |this, cx| {
                        if let Some(space) = this.active.clone() {
                            space.update(cx, |space, _| space.remove_empty_pane(tab_id, pane_id));
                        }
                        cx.notify();
                    });
                }),
            )
            .into_any_element()
    }

    fn pane_header(
        &self,
        space: &Space,
        tab_id: WorkspaceTabId,
        layout: &Workspace,
        pane_id: LayoutPaneId,
        on: &chrome::Emit,
        weak: &gpui::WeakEntity<Self>,
        cx: &App,
    ) -> AnyElement {
        let Some(pane) = layout.pane(pane_id) else {
            return div().into_any_element();
        };

        let active_index =
            pane.active().and_then(|active| pane.items().iter().position(|item| *item == active));
        let space_key = space.key();
        let tabs = pane.items().iter().enumerate().filter_map(|(index, id)| {
            let item = space.item(*id)?;
            let selected = pane.active() == Some(*id);
            let status = item.status();
            let process_running = item.process_running();
            let ended = item.ended();
            let position = if index == 0 {
                TabPosition::First
            } else if index + 1 == pane.items().len() {
                TabPosition::Last
            } else {
                TabPosition::Middle(index.cmp(&active_index.unwrap_or(index)))
            };
            let select = *id;
            let close = *id;
            let select_item = on.clone();
            let close_item = on.clone();
            let drop_item = weak.clone();
            let drop_space = space_key.clone();
            let dragged = DraggedItem {
                space: space_key.clone(),
                tab: tab_id,
                pane: pane_id,
                index,
                item: *id,
                title: item.title(),
                selected,
                top_level: false,
            };
            Some(
                Tab::new(format!("pane-{}-item-{}", pane_id.get(), id.get()))
                    .role(Role::Tab)
                    .aria_label(item.title())
                    .aria_selected(selected)
                    .position(position)
                    .toggle_state(selected)
                    .on_click(move |_, window, cx| {
                        select_item(Action::Select { space: None, item: select }, window, cx)
                    })
                    .on_drag(dragged, |dragged, offset, _, cx| {
                        dragged_item_preview(dragged, offset, cx)
                    })
                    .can_drop(move |value, _, _| {
                        value
                            .downcast_ref::<DraggedItem>()
                            .is_some_and(|dragged| dragged.space == drop_space)
                    })
                    .drag_over::<DraggedItem>(move |tab, dragged, _, cx| {
                        let mut tab = tab
                            .bg(cx.theme().colors().drop_target_background)
                            .border_color(cx.theme().colors().drop_target_border)
                            .border_0();
                        if index < dragged.index {
                            tab = tab.border_l_2();
                        } else if index > dragged.index {
                            tab = tab.border_r_2();
                        }
                        tab
                    })
                    .on_drop(move |dragged: &DraggedItem, window, cx| {
                        let dragged = dragged.clone();
                        let _ = drop_item.update(cx, |this, cx| {
                            this.handle_item_drop(
                                &dragged, tab_id, pane_id, index, false, window, cx,
                            );
                        });
                    })
                    .start_slot(chrome::status_indicator(
                        status,
                        process_running,
                        ended,
                        &space_key,
                        *id,
                        cx,
                    ))
                    .end_slot(
                        IconButton::new(
                            format!("close-pane-{}-item-{}", pane_id.get(), id.get()),
                            IconName::Close,
                        )
                        .shape(IconButtonShape::Square)
                        .size(ButtonSize::None)
                        .icon_size(IconSize::XSmall)
                        .tooltip(Tooltip::text("Close"))
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            close_item(Action::Close { space: None, item: close }, window, cx)
                        }),
                    )
                    .child(Label::new(item.title()).size(LabelSize::Small).truncate())
                    .into_any_element(),
            )
        });
        let append_drop = weak.clone();
        let append_index = pane.items().len();
        let append_space = space_key.clone();
        let tab_bar_drop_target = div()
            .id(format!("pane-{}-tab-bar-drop-target", pane_id.get()))
            .min_w_6()
            .h(Tab::container_height(cx))
            .flex_grow_1()
            .child("")
            .can_drop(move |value, _, _| {
                value
                    .downcast_ref::<DraggedItem>()
                    .is_some_and(|dragged| dragged.space == append_space)
            })
            .drag_over::<DraggedItem>(|bar, _, _, cx| {
                bar.bg(cx.theme().colors().drop_target_background)
            })
            .on_drop(move |dragged: &DraggedItem, window, cx| {
                let dragged = dragged.clone();
                let _ = append_drop.update(cx, |this, cx| {
                    this.handle_item_drop(
                        &dragged,
                        tab_id,
                        pane_id,
                        append_index,
                        false,
                        window,
                        cx,
                    );
                });
            });
        TabBar::new(format!("workspace-tab-{}-pane-{}-tabs", tab_id.get(), pane_id.get()))
            .children(tabs)
            .child(tab_bar_drop_target)
            .end_child(pane_controls(weak, tab_id, pane_id))
            .into_any_element()
    }

    fn command_palette(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.command_palette_open {
            return None;
        }
        let commands = self.filtered_palette_commands();
        if self.command_palette_selected >= commands.len() {
            self.command_palette_selected = 0;
        }
        let selected = self.command_palette_selected;
        let weak = cx.weak_entity();
        let rows: Vec<_> = commands
            .into_iter()
            .enumerate()
            .map(|(index, (command, label, shortcut))| {
                let choose = weak.clone();
                ListItem::new(("command-palette-item", index))
                    .spacing(ListItemSpacing::Dense)
                    .toggle_state(index == selected)
                    .aria_role(gpui::Role::ListBoxOption)
                    .aria_label(label)
                    .when(!shortcut.is_empty(), |item| item.aria_keyshortcuts(shortcut))
                    .when(index == selected, ListItem::aria_active_descendant)
                    .on_click(move |_, window, cx| {
                        let _ = choose.update(cx, |this, cx| {
                            this.invoke_palette_command(command, window, cx)
                        });
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .child(Label::new(label).size(LabelSize::Small))
                            .when(!shortcut.is_empty(), |row| {
                                row.child(
                                    Label::new(shortcut)
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                            }),
                    )
            })
            .collect();
        let dismiss = cx.listener(|this, _, _, cx| {
            this.command_palette_open = false;
            this.command_palette_query.clear();
            cx.notify();
        });
        let query = if self.command_palette_query.is_empty() {
            "Type a command…".to_owned()
        } else {
            self.command_palette_query.clone()
        };
        let query_color =
            if self.command_palette_query.is_empty() { Color::Muted } else { Color::Default };

        Some(
            div()
                .id("command-palette-scrim")
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .bg(gpui::black().opacity(0.35))
                .on_mouse_down(gpui::MouseButton::Left, dismiss)
                .child(
                    v_flex()
                        .id("command-palette")
                        .absolute()
                        .top(px(48.))
                        .left(relative(0.5))
                        .ml(px(-320.))
                        .w(px(640.))
                        .max_h(px(480.))
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().colors().border)
                        .bg(cx.theme().colors().elevated_surface_background)
                        .shadow_lg()
                        .overflow_hidden()
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(
                            h_flex()
                                .h(px(42.))
                                .px_3()
                                .gap_2()
                                .border_b_1()
                                .border_color(cx.theme().colors().border)
                                .child(
                                    Icon::new(IconName::MagnifyingGlass)
                                        .size(IconSize::Small)
                                        .color(Color::Muted),
                                )
                                .child(Label::new(query).size(LabelSize::Small).color(query_color)),
                        )
                        .child(
                            v_flex()
                                .id("command-palette-results")
                                .role(gpui::Role::ListBox)
                                .aria_label("Commands")
                                .p_1()
                                .overflow_y_scroll()
                                .when(rows.is_empty(), |list| {
                                    list.child(div().p_3().child(
                                        Label::new("No matching commands").color(Color::Muted),
                                    ))
                                })
                                .children(rows),
                        ),
                )
                .into_any_element(),
        )
    }

    fn rename_space_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        self.rename_space?;
        let cancel_scrim = cx.listener(|this, _, _, cx| {
            this.rename_space = None;
            this.rename_query.clear();
            cx.notify();
        });
        let cancel_button = cx.listener(|this, _, _, cx| {
            this.rename_space = None;
            this.rename_query.clear();
            cx.notify();
        });
        let save = cx.listener(|this, _, _, cx| this.commit_space_rename(cx));
        Some(
            div()
                .id("rename-space-scrim")
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .bg(gpui::black().opacity(0.35))
                .on_mouse_down(gpui::MouseButton::Left, cancel_scrim)
                .child(
                    v_flex()
                        .id("rename-space-dialog")
                        .absolute()
                        .top(px(96.))
                        .left(relative(0.5))
                        .ml(px(-220.))
                        .w(px(440.))
                        .p_4()
                        .gap_3()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().colors().border)
                        .bg(cx.theme().colors().elevated_surface_background)
                        .shadow_lg()
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(Label::new("Rename Space").size(LabelSize::Large))
                        .child(
                            h_flex()
                                .h(px(36.))
                                .px_2()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().colors().border_focused)
                                .bg(cx.theme().colors().editor_background)
                                .child(
                                    Label::new(if self.rename_query.is_empty() {
                                        "Type a space name…".to_owned()
                                    } else {
                                        self.rename_query.clone()
                                    })
                                    .size(LabelSize::Small)
                                    .color(
                                        if self.rename_query.is_empty() {
                                            Color::Muted
                                        } else {
                                            Color::Default
                                        },
                                    ),
                                ),
                        )
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_1()
                                .child(
                                    Button::new("cancel-space-rename", "Cancel")
                                        .on_click(cancel_button),
                                )
                                .child(Button::new("save-space-rename", "Rename").on_click(save)),
                        ),
                )
                .into_any_element(),
        )
    }

    fn settings_workspace(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let close = cx.listener(|this, _, _, cx| {
            this.settings_open = false;
            this.plugin_settings = None;
            cx.notify();
        });
        let selected = self.settings_page;
        let navigation: Vec<_> = SettingsPage::ALL
            .into_iter()
            .map(|page| {
                div()
                    .id(format!("settings-page-{}", page.slug()))
                    .role(Role::Tab)
                    .aria_label(page.title())
                    .aria_selected(page == selected)
                    .mx_1()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(page == selected, |row| {
                        row.bg(cx.theme().colors().element_selected)
                            .text_color(cx.theme().colors().text)
                    })
                    .when(page != selected, |row| {
                        row.text_color(cx.theme().colors().text_muted)
                            .hover(|row| row.bg(cx.theme().colors().element_hover))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.settings_page = page;
                        if page != SettingsPage::Plugins {
                            this.plugin_settings = None;
                        }
                        cx.notify();
                    }))
                    .child(Label::new(page.title()).size(LabelSize::Small))
            })
            .collect();
        let content = self.settings_content(cx);

        v_flex()
            .id("settings-workspace")
            .size_full()
            .min_h_0()
            .bg(cx.theme().colors().background)
            .child(
                h_flex()
                    .h(Tab::container_height(cx))
                    .px_3()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().colors().border)
                    .child(Label::new("Settings").size(LabelSize::Small))
                    .child(
                        IconButton::new("close-settings", IconName::Close)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text("Close Settings"))
                            .on_click(close),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(176.))
                            .h_full()
                            .py_2()
                            .border_r_1()
                            .border_color(cx.theme().colors().border)
                            .bg(cx.theme().colors().surface_background)
                            .child(div().px_3().py_1().child(
                                Label::new("Options").size(LabelSize::XSmall).color(Color::Muted),
                            ))
                            .children(navigation),
                    )
                    .child(content),
            )
            .into_any_element()
    }

    fn settings_content(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let page = self.settings_page;
        let plugin_override = (page == SettingsPage::Plugins)
            .then(|| self.plugin_settings.as_ref())
            .flatten()
            .map(|(plugin, view)| {
                let back = cx.listener(|this, _, _, cx| {
                    this.plugin_settings = None;
                    cx.notify();
                });
                v_flex()
                    .gap_3()
                    .child(Button::new("plugin-settings-back", "Back to plugins").on_click(back))
                    .child(Label::new(plugin.clone()).size(LabelSize::XSmall).color(Color::Muted))
                    .child(div().min_h(px(320.)).child(view.clone()))
                    .into_any_element()
            });
        let body = if let Some(plugin_override) = plugin_override {
            plugin_override
        } else {
            match page {
                SettingsPage::General => {
                    let terminate = self.settings.resolved().terminate_sessions_on_exit;
                    let toggle = cx
                        .listener(move |this, _, _, cx| this.set_terminate_on_exit(!terminate, cx));
                    v_flex()
                    .gap_4()
                    .child(Label::new("Chartr").size(LabelSize::Large))
                    .child(
                        Label::new(format!(
                            "Version {} · configuration namespace chartr-zeddy",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .gap_4()
                            .child(
                                v_flex()
                                    .child(
                                        Label::new("Terminate sessions on exit")
                                            .size(LabelSize::Small),
                                    )
                                    .child(
                                        Label::new(
                                            "Normal app exit detaches and leaves sessions running.",
                                        )
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                    ),
                            )
                            .child(
                                Button::new(
                                    "terminate-sessions-on-exit",
                                    if terminate { "On" } else { "Off" },
                                )
                                .toggle_state(terminate)
                                .on_click(toggle),
                            ),
                    )
                    .into_any_element()
                }
                SettingsPage::Appearance => {
                    let selected = self.settings.resolved().fixed_theme.clone();
                    let mode = self.settings.resolved().theme_mode;
                    let dark = cx.listener(|this, _, _, cx| {
                        this.set_theme_preference(ThemeMode::Fixed, Some(CHARTR_DARK), cx)
                    });
                    let light = cx.listener(|this, _, _, cx| {
                        this.set_theme_preference(ThemeMode::Fixed, Some(CHARTR_LIGHT), cx)
                    });
                    let system = cx.listener(|this, _, _, cx| {
                        this.set_theme_preference(ThemeMode::System, None, cx)
                    });
                    let font = cx.weak_entity();
                    let smaller = cx.listener(|this, _, _, cx| this.adjust_ui_font_size(-1., cx));
                    let larger = cx.listener(|this, _, _, cx| this.adjust_ui_font_size(1., cx));
                    v_flex()
                        .gap_3()
                        .child(setting_label("Theme"))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("theme-chartr-dark", CHARTR_DARK)
                                        .toggle_state(
                                            mode == ThemeMode::Fixed && selected == CHARTR_DARK,
                                        )
                                        .on_click(dark),
                                )
                                .child(
                                    Button::new("theme-chartr-light", CHARTR_LIGHT)
                                        .toggle_state(
                                            mode == ThemeMode::Fixed && selected == CHARTR_LIGHT,
                                        )
                                        .on_click(light),
                                )
                                .child(
                                    Button::new("theme-system", "System")
                                        .toggle_state(mode == ThemeMode::System)
                                        .on_click(system),
                                ),
                        )
                        .child(setting_label("Interface font"))
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    PopoverMenu::new("ui-font-menu")
                                        .trigger(
                                            Button::new(
                                                "ui-font-family",
                                                self.settings.resolved().ui_font_family.clone(),
                                            )
                                            .end_icon(Icon::new(IconName::ChevronDown)),
                                        )
                                        .anchor(Anchor::BottomLeft)
                                        .menu(move |window, cx| {
                                            let font = font.clone();
                                            Some(ContextMenu::build(
                                                window,
                                                cx,
                                                move |menu, _, _| {
                                                    ["IBM Plex Sans", ".ZedSans", "System UI"]
                                                        .into_iter()
                                                        .fold(menu, |menu, family| {
                                                            let set = font.clone();
                                                            menu.entry(
                                                                family,
                                                                None,
                                                                move |_, cx| {
                                                                    let _ = set.update(
                                                                        cx,
                                                                        |this, cx| {
                                                                            this.set_ui_font(
                                                                                family.to_owned(),
                                                                                cx,
                                                                            )
                                                                        },
                                                                    );
                                                                },
                                                            )
                                                        })
                                                },
                                            ))
                                        }),
                                )
                                .child(
                                    IconButton::new("ui-font-smaller", IconName::Dash)
                                        .tooltip(Tooltip::text("Decrease interface font size"))
                                        .on_click(smaller),
                                )
                                .child(
                                    Label::new(format!(
                                        "{} px",
                                        self.settings.resolved().ui_font_size
                                    ))
                                    .size(LabelSize::Small),
                                )
                                .child(
                                    IconButton::new("ui-font-larger", IconName::Plus)
                                        .tooltip(Tooltip::text("Increase interface font size"))
                                        .on_click(larger),
                                ),
                        )
                        .into_any_element()
                }
                SettingsPage::Terminal => {
                    let font = cx.weak_entity();
                    let smaller =
                        cx.listener(|this, _, _, cx| this.adjust_terminal_font_size(-1., cx));
                    let larger =
                        cx.listener(|this, _, _, cx| this.adjust_terminal_font_size(1., cx));
                    let choose_directory =
                        cx.listener(|this, _, _, cx| this.pick_ad_hoc_directory(cx));
                    let retry = cx.listener(|this, _, _, cx| this.retry_backend(false, cx));
                    let restart =
                        cx.listener(|this, _, window, cx| this.request_backend_restart(window, cx));
                    let backend = match &self.backend {
                        Backend::Ready => "Connected".to_owned(),
                        Backend::Starting => "Starting".to_owned(),
                        Backend::Recovering(detail) | Backend::Failed(detail) => detail.clone(),
                    };
                    v_flex()
                        .gap_3()
                        .child(setting_label("Terminal font"))
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    PopoverMenu::new("terminal-font-menu")
                                        .trigger(
                                            Button::new(
                                                "terminal-font-family",
                                                self.settings
                                                    .resolved()
                                                    .terminal_font_family
                                                    .clone(),
                                            )
                                            .end_icon(Icon::new(IconName::ChevronDown)),
                                        )
                                        .anchor(Anchor::BottomLeft)
                                        .menu(move |window, cx| {
                                            let font = font.clone();
                                            Some(ContextMenu::build(
                                                window,
                                                cx,
                                                move |menu, _, _| {
                                                    ["IBM Plex Mono", "Lilex", ".ZedMono"]
                                                        .into_iter()
                                                        .fold(menu, |menu, family| {
                                                            let set = font.clone();
                                                            menu.entry(
                                                                family,
                                                                None,
                                                                move |_, cx| {
                                                                    let _ = set.update(
                                                                        cx,
                                                                        |this, cx| {
                                                                            this.set_terminal_font(
                                                                                family.to_owned(),
                                                                                cx,
                                                                            )
                                                                        },
                                                                    );
                                                                },
                                                            )
                                                        })
                                                },
                                            ))
                                        }),
                                )
                                .child(
                                    IconButton::new("terminal-font-smaller", IconName::Dash)
                                        .tooltip(Tooltip::text("Decrease terminal font size"))
                                        .on_click(smaller),
                                )
                                .child(
                                    Label::new(format!(
                                        "{} px",
                                        self.settings.resolved().terminal_font_size
                                    ))
                                    .size(LabelSize::Small),
                                )
                                .child(
                                    IconButton::new("terminal-font-larger", IconName::Plus)
                                        .tooltip(Tooltip::text("Increase terminal font size"))
                                        .on_click(larger),
                                ),
                        )
                        .child(setting_label("Ad-hoc directory"))
                        .child(
                            Button::new(
                                "choose-ad-hoc-directory",
                                self.settings.resolved().ad_hoc_directory.as_ref().map_or_else(
                                    || "Home directory".to_owned(),
                                    |path| path.display().to_string(),
                                ),
                            )
                            .on_click(choose_directory),
                        )
                        .child(setting_value("Backend", backend))
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    Button::new("settings-retry-backend", "Retry").on_click(retry),
                                )
                                .child(
                                    Button::new("settings-restart-backend", "Restart Backend")
                                        .on_click(restart),
                                ),
                        )
                        .into_any_element()
                }
                SettingsPage::Hotkeys => {
                    let recording = self.recording_keymap;
                    let rows: Vec<_> = KeymapAction::ALL
                        .into_iter()
                        .map(|action| {
                            let capture = cx.listener(move |this, _, _, cx| {
                                this.recording_keymap = Some(action);
                                this.problem = None;
                                cx.notify();
                            });
                            h_flex()
                                .justify_between()
                                .gap_4()
                                .child(Label::new(action.title()).size(LabelSize::Small))
                                .child(
                                    Button::new(
                                        format!("record-hotkey-{}", action.id()),
                                        if recording == Some(action) {
                                            "Press shortcut…".to_owned()
                                        } else {
                                            self.keymap.key(action).to_owned()
                                        },
                                    )
                                    .toggle_state(recording == Some(action))
                                    .on_click(capture),
                                )
                        })
                        .collect();
                    v_flex()
                    .gap_2()
                    .when_some(self.keymap.problem().map(str::to_owned), |view, problem| {
                        view.child(
                            Banner::new()
                                .severity(Severity::Error)
                                .child(Label::new(problem).size(LabelSize::Small)),
                        )
                    })
                    .when(self.keymap_restart_required, |view| {
                        view.child(Banner::new().child(
                            Label::new(
                                "Shortcut changes are saved. Restart Chartr to rebuild the application keymap.",
                            )
                            .size(LabelSize::Small),
                        ))
                    })
                    .child(
                        Label::new(
                            "Click a shortcut, then press one key chord. Conflicts in the Chartr context are rejected.",
                        )
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                    )
                    .children(rows)
                    .into_any_element()
                }
                SettingsPage::Plugins => {
                    let mut descriptors: Vec<_> = self
                        .catalog
                        .loaded
                        .values()
                        .map(|loaded| (loaded.manifest.clone(), true, loaded.has_settings))
                        .chain(
                            self.catalog
                                .disabled
                                .values()
                                .map(|disabled| (disabled.manifest.clone(), false, false)),
                        )
                        .collect();
                    descriptors.sort_by(|(left, _, _), (right, _, _)| left.name.cmp(&right.name));
                    let rows: Vec<_> = descriptors
                        .into_iter()
                        .map(|(manifest, enabled, has_settings)| {
                            let id = manifest.id.clone();
                            let control_id = id.clone();
                            let configured = self.settings.resolved().plugin(&id);
                            let toggle = cx.listener(move |this, _, _, cx| {
                                this.set_plugin_enabled(id.clone(), !enabled, cx)
                            });
                            let trust = match manifest.kind {
                                zeddy_plugin::manifest::Kind::Native => {
                                    "Native — fully trusted code".to_owned()
                                }
                                zeddy_plugin::manifest::Kind::Web => {
                                    let project = match manifest.permissions.project_files {
                                        zeddy_plugin::manifest::ProjectAccess::None => {
                                            "no project files"
                                        }
                                        zeddy_plugin::manifest::ProjectAccess::Read => {
                                            "read project files"
                                        }
                                        zeddy_plugin::manifest::ProjectAccess::ReadWrite => {
                                            "read/write project files"
                                        }
                                    };
                                    let mut grants = vec![project.to_owned()];
                                    if !manifest.permissions.network.is_empty() {
                                        grants.push(format!(
                                            "network: {}",
                                            manifest.permissions.network.join(", ")
                                        ));
                                    }
                                    if manifest.permissions.process {
                                        grants.push("process actions".to_owned());
                                    }
                                    if manifest.permissions.session {
                                        grants.push("bound-session actions".to_owned());
                                    }
                                    format!("Web — {}", grants.join(" · "))
                                }
                            };
                            let unsafe_control =
                                (manifest.kind == zeddy_plugin::manifest::Kind::Web).then(|| {
                                    let id = manifest.id.clone();
                                    let change = cx.listener(move |this, _, _, cx| {
                                        this.set_plugin_unsafe(
                                            id.clone(),
                                            !configured.unsafe_filesystem,
                                            cx,
                                        )
                                    });
                                    Button::new(
                                        format!("plugin-unsafe-{}", manifest.id),
                                        if configured.unsafe_filesystem {
                                            "Unsafe filesystem granted"
                                        } else {
                                            "Grant unsafe filesystem"
                                        },
                                    )
                                    .toggle_state(configured.unsafe_filesystem)
                                    .on_click(change)
                                });
                            let configure = has_settings.then(|| {
                                let id = manifest.id.clone();
                                Button::new(format!("plugin-settings-{}", manifest.id), "Configure")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_plugin_settings(id.clone(), window, cx)
                                    }))
                            });
                            v_flex()
                                .gap_2()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().colors().border)
                                .rounded_md()
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .child(
                                            v_flex()
                                                .child(
                                                    Label::new(manifest.name)
                                                        .size(LabelSize::Small),
                                                )
                                                .child(
                                                    Label::new(manifest.id)
                                                        .size(LabelSize::XSmall)
                                                        .color(Color::Muted),
                                                ),
                                        )
                                        .child(
                                            Button::new(
                                                format!("plugin-enabled-{control_id}"),
                                                if enabled { "Enabled" } else { "Disabled" },
                                            )
                                            .toggle_state(enabled)
                                            .on_click(toggle),
                                        ),
                                )
                                .child(
                                    Label::new(trust).size(LabelSize::XSmall).color(Color::Muted),
                                )
                                .when_some(configure, |row, control| row.child(control))
                                .when_some(unsafe_control, |row, control| row.child(control))
                        })
                        .collect();
                    let rejected: Vec<_> = self
                        .catalog
                        .rejected
                        .iter()
                        .map(|rejected| {
                            Banner::new().severity(Severity::Error).child(
                                Label::new(format!("{}: {}", rejected.dir.display(), rejected.why))
                                    .size(LabelSize::XSmall),
                            )
                        })
                        .collect();
                    v_flex()
                        .gap_2()
                        .when(rows.is_empty() && rejected.is_empty(), |view| {
                            view.child(Label::new("No plugins installed.").color(Color::Muted))
                        })
                        .children(rows)
                        .children(rejected)
                        .into_any_element()
                }
            }
        };
        v_flex()
            .id(format!("settings-content-{}", page.slug()))
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .items_center()
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(680.))
                    .p_6()
                    .gap_5()
                    .child(Label::new(page.title()).size(LabelSize::Large))
                    .when_some(self.settings.unreadable().map(str::to_owned), |view, error| {
                        view.child(Label::new(error).size(LabelSize::Small).color(Color::Error))
                    })
                    .child(body),
            )
            .into_any_element()
    }
}

impl Focusable for Zeddy {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Zeddy {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
        self.restore_plugins_once(window, cx);
        self.persist_if_changed(cx);
        let entries = self.entries(cx);
        let sidebar_spaces = self.sidebar_spaces(cx);
        let chrome_entries: &[Entry] = &entries;
        let switcher = self.space_switcher(window, cx);
        let new_item = self.new_item_menu(cx);
        let (background, text, workspace_background) = {
            let colors = cx.theme().colors();
            (colors.background, colors.text, colors.editor_background)
        };

        let on_action =
            cx.listener(|this, action: &Action, window, cx| this.act(action.clone(), window, cx));
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));

        let workspace =
            v_flex().flex_1().h_full().overflow_hidden().bg(workspace_background).child(
                if self.settings_open {
                    self.settings_workspace(cx)
                } else {
                    self.workspace_pane(window, cx)
                },
            );

        let body = if self.settings_open {
            h_flex()
                .size_full()
                .child(chrome::sidebar::render(
                    &sidebar_spaces,
                    switcher,
                    new_item,
                    emit.clone(),
                    self.sidebar_width,
                    cx,
                ))
                .child(workspace)
                .into_any_element()
        } else {
            match self.mode {
                Mode::Sidebar => h_flex()
                    .size_full()
                    .child(chrome::sidebar::render(
                        &sidebar_spaces,
                        switcher,
                        new_item,
                        emit.clone(),
                        self.sidebar_width,
                        cx,
                    ))
                    .child(workspace)
                    .into_any_element(),
                Mode::Tabs => v_flex()
                    .size_full()
                    .child(chrome::tabs::render(chrome_entries, switcher, new_item, emit, cx))
                    .child(workspace)
                    .into_any_element(),
            }
        };

        let command_palette = self.command_palette(cx);
        let rename_space = self.rename_space_overlay(cx);

        div()
            .relative()
            .track_focus(&self.focus)
            .key_context(if self.rename_space.is_some() {
                "RenameSpace"
            } else if self.command_palette_open {
                "CommandPalette"
            } else if self.settings_open {
                "Chartr Settings"
            } else {
                "Chartr"
            })
            .size_full()
            .bg(background)
            .text_color(text)
            .on_drag_move::<chrome::DraggedSidebar>(cx.listener(
                |this, event: &DragMoveEvent<chrome::DraggedSidebar>, _, cx| {
                    this.sidebar_width = (event.event.position.x / px(1.))
                        .clamp(chrome::sidebar::MIN_WIDTH, chrome::sidebar::MAX_WIDTH);
                    cx.notify();
                },
            ))
            .on_action(cx.listener(|this, _: &actions::pane::CloseActiveItem, _, cx| {
                this.close_active_item(cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::CloseAllItems, window, cx| {
                this.request_close_active_pane(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::SplitAndMoveLeft, _, cx| {
                this.split_and_move(SplitDirection::Left, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::SplitAndMoveRight, _, cx| {
                this.split_and_move(SplitDirection::Right, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::SplitAndMoveUp, _, cx| {
                this.split_and_move(SplitDirection::Up, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::SplitAndMoveDown, _, cx| {
                this.split_and_move(SplitDirection::Down, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveLeft, _, cx| {
                this.move_active_to_pane(SplitDirection::Left, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveRight, _, cx| {
                this.move_active_to_pane(SplitDirection::Right, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveUp, _, cx| {
                this.move_active_to_pane(SplitDirection::Up, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveDown, _, cx| {
                this.move_active_to_pane(SplitDirection::Down, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::JoinIntoNext, _, cx| {
                this.join_active_into_next(cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::ActivatePaneLeft, window, cx| {
                this.activate_pane_in_direction(SplitDirection::Left, window, cx)
            }))
            .on_action(cx.listener(
                |this, _: &actions::workspace::ActivatePaneRight, window, cx| {
                    this.activate_pane_in_direction(SplitDirection::Right, window, cx)
                },
            ))
            .on_action(cx.listener(|this, _: &actions::workspace::ActivatePaneUp, window, cx| {
                this.activate_pane_in_direction(SplitDirection::Up, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::ActivatePaneDown, window, cx| {
                this.activate_pane_in_direction(SplitDirection::Down, window, cx)
            }))
            .on_action(
                cx.listener(|this, _: &actions::workspace::ToggleZoom, _, cx| this.toggle_zoom(cx)),
            )
            .on_action(cx.listener(|this, _: &actions::workspace::NewTerminal, window, cx| {
                this.act(Action::New, window, cx)
            }))
            .on_action(
                cx.listener(|this, _: &actions::settings::Open, _, cx| this.open_settings(cx)),
            )
            .on_action(cx.listener(|this, _: &actions::command_palette::Toggle, window, cx| {
                this.toggle_command_palette(window, cx)
            }))
            .on_key_down(cx.listener(|this, event, window, cx| this.on_key(event, window, cx)))
            .child(body)
            .children(command_palette)
            .children(rename_space)
    }
}

fn setting_label(label: &'static str) -> AnyElement {
    Label::new(label).size(LabelSize::Small).color(Color::Muted).into_any_element()
}

fn pane_drop_direction_for_drag(
    event: &DragMoveEvent<DraggedItem>,
) -> Option<Option<SplitDirection>> {
    let bounds = event.bounds;
    let x = event.event.position.x - bounds.left();
    let y = event.event.position.y - bounds.top();
    pane_drop_direction_for_position(
        bounds.size.width.into(),
        bounds.size.height.into(),
        x.into(),
        y.into(),
    )
}

/// `None` means this pane is not under the pointer and must not overwrite a
/// sibling's drag state. `Some(None)` is the pane's center drop target.
fn pane_drop_direction_for_position(
    width: f32,
    height: f32,
    x: f32,
    y: f32,
) -> Option<Option<SplitDirection>> {
    if x < 0. || x > width || y < 0. || y > height {
        return None;
    }
    Some(split_direction_for_position(width, height, x, y))
}

/// Zed's pane-body hit test. The edge band is 20% of the pane's shorter side;
/// corners resolve to the nearest edge in Up, Right, Down, Left tie order.
fn split_direction_for_position(width: f32, height: f32, x: f32, y: f32) -> Option<SplitDirection> {
    let size = width.min(height) * 0.2;
    if x >= size && x <= width - size && y >= size && y <= height - size {
        return None;
    }
    [
        (SplitDirection::Up, y),
        (SplitDirection::Right, width - x),
        (SplitDirection::Down, height - y),
        (SplitDirection::Left, x),
    ]
    .into_iter()
    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    .map(|(direction, _)| direction)
}

fn drop_target(direction: Option<SplitDirection>, group: String, space: String, cx: &App) -> Div {
    div()
        .invisible()
        .absolute()
        .bg(cx.theme().colors().drop_target_background)
        .can_drop(move |value, _, _| {
            value.downcast_ref::<DraggedItem>().is_some_and(|dragged| dragged.space == space)
        })
        .group_drag_over::<DraggedItem>(group, |style| style.visible())
        .map(|target| match direction {
            None => target.top_0().right_0().bottom_0().left_0(),
            Some(SplitDirection::Up) => target.top_0().left_0().right_0().h(relative(0.5)),
            Some(SplitDirection::Down) => target.bottom_0().left_0().right_0().h(relative(0.5)),
            Some(SplitDirection::Left) => target.top_0().left_0().bottom_0().w(relative(0.5)),
            Some(SplitDirection::Right) => target.top_0().right_0().bottom_0().w(relative(0.5)),
        })
}

fn pane_resize_handle(dragged: DraggedPaneDivider, axis: PaneAxisDirection) -> impl IntoElement {
    div()
        .id(format!("pane-divider-{:?}-{}", dragged.axis_path, dragged.divider))
        .absolute()
        .when(axis == PaneAxisDirection::Horizontal, |handle| {
            handle.right(px(-3.)).top_0().h_full().w(px(6.)).cursor_col_resize()
        })
        .when(axis == PaneAxisDirection::Vertical, |handle| {
            handle.bottom(px(-3.)).left_0().w_full().h(px(6.)).cursor_row_resize()
        })
        .on_drag(dragged, |dragged, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| dragged.clone())
        })
        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .occlude()
}

fn pane_controls(
    weak: &gpui::WeakEntity<Zeddy>,
    tab_id: WorkspaceTabId,
    pane_id: LayoutPaneId,
) -> AnyElement {
    let focus = weak.clone();
    let split = weak.clone();
    let zoom = weak.clone();

    h_flex()
        .id(format!("workspace-tab-{}-pane-{}-controls", tab_id.get(), pane_id.get()))
        .gap_0p5()
        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
            let _ = focus.update(cx, |this, cx| {
                if let Some(space) = this.active.clone() {
                    space.update(cx, |space, _| space.activate_pane(tab_id, pane_id));
                }
                cx.notify();
            });
        })
        .child(
            PopoverMenu::new(format!(
                "workspace-tab-{}-pane-{}-split-menu",
                tab_id.get(),
                pane_id.get()
            ))
            .trigger_with_tooltip(
                IconButton::new(
                    format!("workspace-tab-{}-pane-{}-split", tab_id.get(), pane_id.get()),
                    IconName::Split,
                )
                .icon_size(IconSize::XSmall),
                Tooltip::text("Split Pane"),
            )
            .anchor(Anchor::TopRight)
            .menu(move |window, cx| {
                let split = split.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    let left = split.clone();
                    let right = split.clone();
                    let up = split.clone();
                    let down = split.clone();
                    menu.entry("Split Left", None, move |_, cx| {
                        let _ = left.update(cx, |this, cx| {
                            this.split_and_move_in(tab_id, pane_id, SplitDirection::Left, cx)
                        });
                    })
                    .entry("Split Right", None, move |_, cx| {
                        let _ = right.update(cx, |this, cx| {
                            this.split_and_move_in(tab_id, pane_id, SplitDirection::Right, cx)
                        });
                    })
                    .entry("Split Up", None, move |_, cx| {
                        let _ = up.update(cx, |this, cx| {
                            this.split_and_move_in(tab_id, pane_id, SplitDirection::Up, cx)
                        });
                    })
                    .entry("Split Down", None, move |_, cx| {
                        let _ = down.update(cx, |this, cx| {
                            this.split_and_move_in(tab_id, pane_id, SplitDirection::Down, cx)
                        });
                    })
                }))
            }),
        )
        .child(
            IconButton::new(
                format!("workspace-tab-{}-pane-{}-zoom", tab_id.get(), pane_id.get()),
                IconName::Maximize,
            )
            .icon_size(IconSize::XSmall)
            .tooltip(Tooltip::text("Toggle Pane Zoom"))
            .on_click(move |_, _, cx| {
                let _ = zoom.update(cx, |this, cx| this.toggle_zoom_in(tab_id, pane_id, cx));
            }),
        )
        .into_any_element()
}

fn setting_value(label: &'static str, value: String) -> AnyElement {
    v_flex()
        .gap_1()
        .child(setting_label(label))
        .child(Label::new(value).size(LabelSize::Small))
        .into_any_element()
}

fn load_registry(cwd: &std::path::Path) -> (Option<Registry>, Option<String>) {
    let file = match spaces::spaces_file() {
        Ok(file) => file,
        Err(error) => return (None, Some(error.to_string())),
    };
    let mut registry = match Registry::load(file) {
        Ok(registry) => registry,
        Err(error) => return (None, Some(error.to_string())),
    };
    // Launching zeddy in a folder is the command-line equivalent of Zed's
    // `zed <path>`: the opened project joins the persisted recent/space list.
    let is_ad_hoc_home = std::env::home_dir().is_some_and(|home| spaces::same_path(&home, cwd));
    if !is_ad_hoc_home
        && !registry.spaces().iter().any(|space| spaces::same_path(space.path(), cwd))
        && let Err(error) = registry.register(cwd)
    {
        return (Some(registry), Some(error.to_string()));
    }
    (Some(registry), None)
}

fn terminal(
    item: &crate::item::SessionItem,
    focused: bool,
    settings: &ResolvedSettings,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let screen = item.session.screen();
    let colors = screen
        .rows
        .iter()
        .map(|row| row.iter().map(|cell| palette::cell_colors(cell, theme)).collect())
        .collect();

    let (font, font_size, line_height) = Fonts::from_settings(settings).terminal();
    let appearance = Appearance {
        font,
        font_size,
        line_height,
        background: theme.colors().terminal_background,
        cursor: theme.colors().terminal_foreground,
    };

    v_flex().size_full().p_2().child(TerminalElement::new(
        screen,
        colors,
        appearance,
        focused,
        item.fit.clone(),
    ))
}

fn message(text: &str, cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .child(Label::new(text.to_owned()).color(Color::Muted))
        .bg(cx.theme().colors().editor_background)
}

fn empty_pane_message(text: &str, cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .p_2()
        .items_center()
        .justify_center()
        .child(Label::new(text.to_owned()).size(LabelSize::Small).color(Color::Muted))
        .bg(cx.theme().colors().editor_background)
}

fn plugin_paths() -> Paths {
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        });
    Paths::under(root.join("chartr-zeddy"))
}

#[cfg(test)]
mod pane_drop_tests {
    use super::{SplitDirection, pane_drop_direction_for_position, split_direction_for_position};

    #[test]
    fn panes_outside_the_pointer_do_not_overwrite_the_hovered_panes_drop_target() {
        assert_eq!(pane_drop_direction_for_position(100., 100., -0.1, 50.), None);
        assert_eq!(pane_drop_direction_for_position(100., 100., 100.1, 50.), None);
        assert_eq!(pane_drop_direction_for_position(100., 100., 50., -0.1), None);
        assert_eq!(pane_drop_direction_for_position(100., 100., 50., 100.1), None);
        assert_eq!(pane_drop_direction_for_position(100., 100., 50., 50.), Some(None));
        assert_eq!(
            pane_drop_direction_for_position(100., 100., 5., 50.),
            Some(Some(SplitDirection::Left))
        );
    }

    #[test]
    fn zed_drop_zone_has_a_center_and_four_edge_bands() {
        assert_eq!(split_direction_for_position(100., 100., 50., 50.), None);
        assert_eq!(split_direction_for_position(100., 100., 19.9, 50.), Some(SplitDirection::Left));
        assert_eq!(
            split_direction_for_position(100., 100., 80.1, 50.),
            Some(SplitDirection::Right)
        );
        assert_eq!(split_direction_for_position(100., 100., 50., 19.9), Some(SplitDirection::Up));
        assert_eq!(split_direction_for_position(100., 100., 50., 80.1), Some(SplitDirection::Down));
    }

    #[test]
    fn zed_drop_zone_uses_the_shorter_side_and_excludes_the_boundary() {
        assert_eq!(split_direction_for_position(400., 100., 20., 50.), None);
        assert_eq!(split_direction_for_position(400., 100., 19.9, 50.), Some(SplitDirection::Left));
        assert_eq!(split_direction_for_position(400., 100., 200., 20.), None);
        assert_eq!(split_direction_for_position(400., 100., 200., 19.9), Some(SplitDirection::Up));
    }

    #[test]
    fn zed_drop_zone_resolves_corners_to_the_nearest_edge() {
        assert_eq!(split_direction_for_position(100., 100., 5., 5.), Some(SplitDirection::Up));
        assert_eq!(split_direction_for_position(100., 100., 96., 8.), Some(SplitDirection::Right));
        assert_eq!(split_direction_for_position(100., 100., 92., 97.), Some(SplitDirection::Down));
        assert_eq!(split_direction_for_position(100., 100., 3., 90.), Some(SplitDirection::Left));
    }
}
