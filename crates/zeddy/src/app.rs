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
    Anchor, AnyView, DragMoveEvent, Entity, EntityId, FocusHandle, Focusable, MouseButton,
    PathPromptOptions, Role,
};
use ui::{
    Banner, ButtonLike, ButtonSize, IconButtonShape, IconPosition, ListItem, ListItemSpacing,
    PopoverMenu, Severity, Tab, TabBar, TabPosition, Tooltip, prelude::*,
};
use zeddy_herdr::{Namespace, Sidecar, WorkspaceId, control::Client};
use zeddy_plugin::{InstanceContext, manifest::Multiplicity};
use zeddy_plugin_host::{Catalog, FileBroker, PaneSource, Paths, SettingsSource};

use crate::{
    actions,
    chrome::{self, Action, DraggedItem, Entry, SpaceEntries, dragged_item_preview},
    components::ContextMenu,
    fonts::{Fonts, UI_LABEL_DEFAULT, UI_LABEL_LARGE, UI_LABEL_SMALL, UI_TEXT_DEFAULT},
    item::PluginItem,
    keys,
    mode::Mode,
    palette,
    persistence::{
        SidebarScope, Snapshot, SpaceKind as PersistedSpaceKind, StateStore, WindowState,
    },
    settings::{PluginSettingsContent, ResolvedSettings, SettingsStore},
    space::{Kind as SpaceKind, Space, name_for},
    spaces::{self, Registry},
    terminal::{Appearance, TerminalElement},
    text_input::{InputEvent, TextInput},
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
pub(crate) struct SettingsPluginDescriptor {
    pub manifest: zeddy_plugin::manifest::Manifest,
    pub enabled: bool,
    pub has_settings: bool,
}

#[derive(Clone)]
pub(crate) struct SettingsPluginRejection {
    pub dir: PathBuf,
    pub why: String,
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
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    JoinPane,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    OpenSettings,
}

impl PaletteCommand {
    const ALL: [(Self, &'static str, &'static str); 13] = [
        (Self::NewTerminal, "Workspace: New Terminal", "Ctrl+~"),
        (Self::CloseItem, "Pane: Close Active Item", "Cmd/Ctrl+W"),
        (Self::CloseAllItems, "Pane: Close All Items", ""),
        (Self::MoveLeft, "Pane: Move Active Item Left", ""),
        (Self::MoveRight, "Pane: Move Active Item Right", ""),
        (Self::MoveUp, "Pane: Move Active Item Up", ""),
        (Self::MoveDown, "Pane: Move Active Item Down", ""),
        (Self::JoinPane, "Pane: Join Into Next Pane", ""),
        (Self::FocusLeft, "Pane: Focus Left", "Cmd/Ctrl+K ←"),
        (Self::FocusRight, "Pane: Focus Right", "Cmd/Ctrl+K →"),
        (Self::FocusUp, "Pane: Focus Up", "Cmd/Ctrl+K ↑"),
        (Self::FocusDown, "Pane: Focus Down", "Cmd/Ctrl+K ↓"),
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
    space_sorter: chrome::sidebar::SpaceSorter,
    active: Option<Entity<Space>>,
    mode: Mode,
    catalog: Catalog,
    plugins_restored: bool,
    settings: SettingsStore,
    command_palette_open: bool,
    command_palette_input: Entity<TextInput>,
    command_palette_query: String,
    command_palette_selected: usize,
    rename_space: Option<EntityId>,
    rename_group: Option<(EntityId, WorkspaceTabId)>,
    rename_input: Entity<TextInput>,
    rename_query: String,
    sidebar_scope: SidebarScope,
    sidebar_width: f32,
    window_bounds: Option<crate::persistence::WindowBounds>,
    state: Option<StateStore>,
    last_persisted: Option<String>,
    title_bar: Entity<crate::title_bar::TitleBar>,
    focus: FocusHandle,
    problem: Option<String>,
}

impl Zeddy {
    pub fn new(cwd: PathBuf, cx: &mut Context<Self>) -> Self {
        let settings = cx.global::<SettingsStore>().clone();
        cx.observe_global::<SettingsStore>(|this, cx| {
            this.settings = cx.global::<SettingsStore>().clone();
            cx.set_reduce_motion(this.settings.resolved().reduce_motion);
            cx.notify();
        })
        .detach();
        let command_palette_input = cx.new(|cx| TextInput::new("Type a command…", cx));
        let rename_input = cx.new(|cx| TextInput::new("Type a name…", cx));
        let title_bar = cx.new(|_| crate::title_bar::TitleBar::new("workspace-title-bar"));
        cx.subscribe(&command_palette_input, |this, input, _: &InputEvent, cx| {
            this.command_palette_query = input.read(cx).text().to_owned();
            this.command_palette_selected = 0;
            cx.notify();
        })
        .detach();
        cx.subscribe(&rename_input, |this, input, _: &InputEvent, cx| {
            this.rename_query = input.read(cx).text().to_owned();
            cx.notify();
        })
        .detach();
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
                    space_sorter: chrome::sidebar::SpaceSorter::default(),
                    active: None,
                    mode: Mode::default(),
                    catalog: Catalog::default(),
                    plugins_restored: true,
                    settings,
                    command_palette_open: false,
                    command_palette_input,
                    command_palette_query: String::new(),
                    command_palette_selected: 0,
                    rename_space: None,
                    rename_group: None,
                    rename_input,
                    rename_query: String::new(),
                    sidebar_scope: saved.window.sidebar_scope,
                    sidebar_width: saved.window.sidebar_width,
                    window_bounds: saved.window.bounds,
                    state,
                    last_persisted: saved_json,
                    title_bar,
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
        descriptors.push(("Free sessions".to_owned(), home.clone(), SpaceKind::AdHoc));
        if let Some(registry) = registry.as_ref() {
            descriptors.extend(
                registry
                    .spaces()
                    .iter()
                    // The synthetic Free sessions space already owns the home
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
        let spaces = restore_space_order(spaces, &saved.spaces, cx);
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
            space_sorter: chrome::sidebar::SpaceSorter::default(),
            active,
            mode: saved.window.chrome,
            catalog,
            plugins_restored: false,
            settings,
            command_palette_open: false,
            command_palette_input,
            command_palette_query: String::new(),
            command_palette_selected: 0,
            rename_space: None,
            rename_group: None,
            rename_input,
            rename_query: String::new(),
            sidebar_scope: saved.window.sidebar_scope,
            sidebar_width: saved.window.sidebar_width,
            window_bounds: saved.window.bounds,
            state,
            last_persisted: saved_json,
            title_bar,
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
            Action::BeginSpaceDrag { at } => self.space_sorter.press(at),
            Action::ActivateSpace { space } => {
                if let Some(target) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                {
                    self.activate(target, window, cx);
                }
            }
            Action::SwitchToTabs => self.settings_set_mode(Mode::Tabs, cx),
            Action::SwitchToSidebar => self.settings_set_mode(Mode::Sidebar, cx),
            Action::ToggleActiveSpaceOnly => {
                let scope = match self.sidebar_scope {
                    SidebarScope::AllSpaces => SidebarScope::ActiveSpace,
                    SidebarScope::ActiveSpace => SidebarScope::AllSpaces,
                };
                self.settings_set_sidebar_scope(scope, cx);
            }
            Action::OpenSettings => self.open_settings(window, cx),
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
            Action::UngroupPane { space, tab } => {
                if let Some(space) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                {
                    space.update(cx, |space, _| space.ungroup_pane(tab));
                }
            }
            Action::RenameGroup { space, tab } => {
                if let Some(target) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                {
                    self.rename_space = None;
                    self.rename_group = Some((space, tab));
                    self.rename_query =
                        target.read(cx).group_name(tab).unwrap_or_default().to_owned();
                    self.rename_input.update(cx, |input, cx| {
                        input.set_text(self.rename_query.clone(), true, cx)
                    });
                    window.focus(&self.rename_input.focus_handle(cx), cx);
                }
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
                    self.rename_group = None;
                    self.rename_space = Some(space);
                    self.rename_query = target.read(cx).name().to_owned();
                    self.rename_input.update(cx, |input, cx| {
                        input.set_text(self.rename_query.clone(), true, cx)
                    });
                    window.focus(&self.rename_input.focus_handle(cx), cx);
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

    fn finish_space_drag(
        &mut self,
        pointer_y: gpui::Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let now = cx.background_executor().now();
        let Some((space, target)) =
            self.space_sorter.drop_at(pointer_y, window.rem_size(), now, cx.reduce_motion())
        else {
            return;
        };
        if self.commit_space_order(space, target, cx) {
            self.space_sorter.accept_drop(now, cx.reduce_motion());
        } else {
            // The registry is the durable folder-list authority. If it refuses
            // the arrangement, the temporary drawn order was never a model
            // change and disappears in one frame.
            self.space_sorter.cancel();
        }
        cx.notify();
    }

    fn commit_space_order(&mut self, space: EntityId, target: usize, cx: &App) -> bool {
        let Some(from) = self.spaces.iter().position(|candidate| candidate.entity_id() == space)
        else {
            return false;
        };
        let target = target.min(self.spaces.len().saturating_sub(1));
        if from == target {
            return true;
        }

        let mut candidate = self.spaces.clone();
        let moved = candidate.remove(from);
        candidate.insert(target, moved);

        let Some(registry) = self.registry.as_ref() else {
            self.problem = Some("the space registry is unavailable".into());
            return false;
        };
        // The synthetic Free space has no registry row unless the operator had
        // explicitly registered its directory. Recovered state-only folders
        // likewise stay out. The remaining projection names every registry row
        // exactly once and preserves its relative sidebar order.
        let registered: Vec<_> = candidate
            .iter()
            .filter_map(|space| {
                let path = space.read(cx).path();
                registry
                    .spaces()
                    .iter()
                    .any(|registered| spaces::same_path(registered.path(), path))
                    .then(|| path.clone())
            })
            .collect();
        match self.registry.as_mut().expect("checked above").reorder(&registered) {
            Ok(_) => {
                self.spaces = candidate;
                true
            }
            Err(error) => {
                self.problem = Some(error.to_string());
                false
            }
        }
    }

    fn close_active_item(&mut self, cx: &mut Context<Self>) {
        if self.command_palette_open {
            self.command_palette_open = false;
            self.command_palette_query.clear();
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

    fn commit_space_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.rename_space.take() else {
            return;
        };
        // Read from the input directly so Enter always commits the latest IME
        // transaction, even before the subscription's mirrored value flushes.
        let name = self.rename_input.read(cx).text().trim().to_owned();
        self.rename_query.clear();
        self.rename_input.update(cx, |input, cx| input.clear(cx));
        window.focus(&self.focus, cx);
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

    fn commit_group_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((space_id, tab)) = self.rename_group.take() else {
            return;
        };
        // Read from the input directly so Enter always commits the latest IME
        // transaction, even before the subscription's mirrored value flushes.
        let name = self.rename_input.read(cx).text().trim().to_owned();
        let name = (!name.is_empty()).then_some(name);
        self.rename_query.clear();
        self.rename_input.update(cx, |input, cx| input.clear(cx));
        window.focus(&self.focus, cx);
        if let Some(space) = self.spaces.iter().find(|space| space.entity_id() == space_id).cloned()
        {
            space.update(cx, |space, _| space.rename_group(tab, name));
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

    fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_input.update(cx, |input, cx| input.clear(cx));
        let Some(original_window) = window.window_handle().downcast::<Self>() else {
            return;
        };
        crate::settings_window::open(original_window, cx.weak_entity(), cx);
        cx.notify();
    }

    fn close_plugin_instances(&mut self, plugin: &str, cx: &mut Context<Self>) {
        for space in &self.spaces {
            let ids = space.read(cx).plugin_item_ids(plugin);
            space.update(cx, |space, _| space.finish_bulk_close(&ids));
        }
    }

    pub(crate) fn settings_backend_label(&self) -> String {
        match &self.backend {
            Backend::Ready => "Connected".to_owned(),
            Backend::Starting => "Starting".to_owned(),
            Backend::Recovering(detail) | Backend::Failed(detail) => detail.clone(),
        }
    }

    pub(crate) fn settings_sidebar_scope(&self) -> SidebarScope {
        self.sidebar_scope
    }

    pub(crate) fn settings_mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn settings_set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        self.mode = mode;
        cx.notify();
    }

    pub(crate) fn settings_set_sidebar_scope(
        &mut self,
        scope: SidebarScope,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_scope = scope;
        cx.notify();
    }

    pub(crate) fn settings_retry_backend(&mut self, cx: &mut Context<Self>) {
        self.retry_backend(false, cx);
    }

    pub(crate) fn settings_restart_backend(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.request_backend_restart(window, cx);
    }

    pub(crate) fn settings_plugins(
        &self,
    ) -> (Vec<SettingsPluginDescriptor>, Vec<SettingsPluginRejection>) {
        let mut descriptors: Vec<_> = self
            .catalog
            .loaded
            .values()
            .map(|loaded| SettingsPluginDescriptor {
                manifest: loaded.manifest.clone(),
                enabled: true,
                has_settings: loaded.has_settings,
            })
            .chain(self.catalog.disabled.values().map(|disabled| SettingsPluginDescriptor {
                manifest: disabled.manifest.clone(),
                enabled: false,
                has_settings: false,
            }))
            .collect();
        descriptors.sort_by(|left, right| left.manifest.name.cmp(&right.manifest.name));
        let rejected = self
            .catalog
            .rejected
            .iter()
            .map(|rejected| SettingsPluginRejection {
                dir: rejected.dir.clone(),
                why: rejected.why.clone(),
            })
            .collect();
        (descriptors, rejected)
    }

    pub(crate) fn settings_set_plugin_enabled(
        &mut self,
        plugin: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if enabled {
            self.catalog.enable(&plugin_paths(), &plugin, cx).map_err(|error| error.to_string())?;
        }
        let result = crate::settings::update_global(cx, |content| {
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
                    self.catalog.disable(&plugin);
                }
                self.problem = None;
                cx.notify();
                Ok(())
            }
            Err(error) => {
                if enabled {
                    self.catalog.disable(&plugin);
                }
                Err(error.to_string())
            }
        }
    }

    pub(crate) fn settings_set_plugin_unsafe(
        &mut self,
        plugin: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        crate::settings::update_global(cx, |content| {
            content
                .plugins
                .entry(plugin.clone())
                .or_insert_with(PluginSettingsContent::default)
                .unsafe_filesystem = Some(enabled);
        })
        .map_err(|error| error.to_string())?;
        // Brokers are instance-owned. Destroying every instance is the
        // revocation boundary; reopening constructs one with the new grant.
        self.close_plugin_instances(&plugin, cx);
        self.problem = None;
        cx.notify();
        Ok(())
    }

    pub(crate) fn settings_plugin_view(
        &mut self,
        plugin: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyView> {
        let loaded = self.catalog.get(plugin)?;
        let permissions = loaded.permissions().clone();
        let unsafe_filesystem =
            cx.global::<SettingsStore>().resolved().plugin(plugin).unsafe_filesystem;
        let source = self.catalog.get_mut(plugin)?.settings(window, cx)?;
        Some(match source {
            SettingsSource::Native(view) => view,
            SettingsSource::Web(entry) => crate::web_plugin::view(
                entry,
                FileBroker::new(
                    None,
                    plugin_paths().data.join(plugin),
                    permissions.project_files,
                    unsafe_filesystem,
                ),
                permissions,
                None,
                None,
                window,
                cx,
            ),
        })
    }

    pub(crate) fn settings_set_free_sessions_directory(
        &mut self,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        if let Some(space) =
            self.spaces.iter().find(|space| space.read(cx).kind() == SpaceKind::AdHoc)
        {
            space.update(cx, |space, _| space.set_path(path));
        }
        cx.notify();
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

    fn toggle_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = !self.command_palette_open;
        self.command_palette_query.clear();
        self.command_palette_input.update(cx, |input, cx| input.clear(cx));
        self.command_palette_selected = 0;
        if self.command_palette_open {
            window.focus(&self.command_palette_input.focus_handle(cx), cx);
        } else {
            window.focus(&self.focus, cx);
        }
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
        self.command_palette_input.update(cx, |input, cx| input.clear(cx));
        let action: Box<dyn gpui::Action> = match command {
            PaletteCommand::NewTerminal => Box::new(actions::workspace::NewTerminal),
            PaletteCommand::CloseItem => Box::new(actions::pane::CloseActiveItem),
            PaletteCommand::CloseAllItems => Box::new(actions::pane::CloseAllItems),
            PaletteCommand::MoveLeft => Box::new(actions::pane::MoveLeft),
            PaletteCommand::MoveRight => Box::new(actions::pane::MoveRight),
            PaletteCommand::MoveUp => Box::new(actions::pane::MoveUp),
            PaletteCommand::MoveDown => Box::new(actions::pane::MoveDown),
            PaletteCommand::JoinPane => Box::new(actions::pane::JoinIntoNext),
            PaletteCommand::FocusLeft => Box::new(actions::workspace::ActivatePaneLeft),
            PaletteCommand::FocusRight => Box::new(actions::workspace::ActivatePaneRight),
            PaletteCommand::FocusUp => Box::new(actions::workspace::ActivatePaneUp),
            PaletteCommand::FocusDown => Box::new(actions::workspace::ActivatePaneDown),
            PaletteCommand::OpenSettings => Box::new(actions::settings::Open),
        };
        window.dispatch_action(action, cx);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_key(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.rename_space.is_some() || self.rename_group.is_some() {
            match event.keystroke.key.as_str() {
                "escape" => {
                    cx.stop_propagation();
                    self.rename_space = None;
                    self.rename_group = None;
                    self.rename_query.clear();
                    self.rename_input.update(cx, |input, cx| input.clear(cx));
                    window.focus(&self.focus, cx);
                }
                "enter" => {
                    cx.stop_propagation();
                    if self.rename_group.is_some() {
                        self.commit_group_rename(window, cx);
                    } else {
                        self.commit_space_rename(window, cx);
                    }
                    return;
                }
                _ => return,
            }
            cx.notify();
            return;
        }
        if self.command_palette_open {
            let key = event.keystroke.key.as_str();
            match key {
                "escape" => {
                    cx.stop_propagation();
                    self.command_palette_open = false;
                    self.command_palette_query.clear();
                    self.command_palette_input.update(cx, |input, cx| input.clear(cx));
                    window.focus(&self.focus, cx);
                }
                "up" => {
                    cx.stop_propagation();
                    let count = self.filtered_palette_commands().len();
                    if count > 0 {
                        self.command_palette_selected =
                            self.command_palette_selected.checked_sub(1).unwrap_or(count - 1);
                    }
                }
                "down" => {
                    cx.stop_propagation();
                    let count = self.filtered_palette_commands().len();
                    if count > 0 {
                        self.command_palette_selected = (self.command_palette_selected + 1) % count;
                    }
                }
                "enter" => {
                    cx.stop_propagation();
                    if let Some((command, _, _)) =
                        self.filtered_palette_commands().get(self.command_palette_selected).copied()
                    {
                        self.invoke_palette_command(command, window, cx);
                        return;
                    }
                }
                _ => return,
            }
            cx.notify();
            return;
        }
        if event.keystroke.key == "escape" && cx.stop_active_drag(window) {
            self.space_sorter.cancel();
            if let Some(space) = self.active.clone() {
                space.update(cx, |space, _| {
                    space.clear_drag_target();
                });
            }
            cx.stop_propagation();
            cx.notify();
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
            let mut menu = menu.entry("New Space…", None, move |_, cx| {
                let _ = add.update(cx, |this, cx| this.pick_a_folder(cx));
            });

            let registered: Vec<_> =
                spaces.iter().filter(|(_, _, kind)| *kind == SpaceKind::Registered).collect();
            if !registered.is_empty() {
                menu = menu.separator().header("Project Spaces");
            }
            for (space, name, _) in registered {
                let target = space.clone();
                let select = weak.clone();
                menu = menu.toggleable_entry(
                    name.clone(),
                    active_id == Some(space.entity_id()),
                    IconPosition::End,
                    None,
                    move |window, cx| {
                        let _ =
                            select.update(cx, |this, cx| this.activate(target.clone(), window, cx));
                    },
                );
            }

            menu = menu.separator();
            for (space, name, _kind) in
                spaces.iter().filter(|(_, _, kind)| *kind == SpaceKind::AdHoc)
            {
                let target = space.clone();
                let select = weak.clone();
                menu = menu.toggleable_entry(
                    name.clone(),
                    active_id == Some(space.entity_id()),
                    IconPosition::End,
                    None,
                    move |window, cx| {
                        let _ =
                            select.update(cx, |this, cx| this.activate(target.clone(), window, cx));
                    },
                );
            }
            menu
        });

        let menu_for_open = menu.clone();
        let menu_for_render = menu.clone();
        PopoverMenu::new("space-switcher")
            .trigger(
                ButtonLike::new("space-switcher-trigger")
                    .aria_label("Current space")
                    .aria_value(current.clone())
                    .selected_style(ButtonStyle::Filled)
                    .child(div().min_w_0().max_w(px(148.)).child(Label::new(current).truncate()))
                    .child(
                        Icon::new(IconName::ChevronUpDown)
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    ),
            )
            .anchor(Anchor::TopLeft)
            .on_open(Rc::new(move |window, cx| {
                menu_for_open.update(cx, |menu, cx| {
                    menu.select_toggled_or_first(window, cx);
                });
            }))
            .menu(move |_, _| Some(menu_for_render.clone()))
            .into_any_element()
    }

    fn view_menu(&self, on: chrome::Emit) -> AnyElement {
        match self.mode {
            Mode::Sidebar => {
                let active_space_only = self.sidebar_scope == SidebarScope::ActiveSpace;
                PopoverMenu::new("chrome-menu")
                    .trigger_with_tooltip(
                        IconButton::new("chrome-menu-trigger", IconName::ChevronDown)
                            .icon_size(IconSize::Small),
                        Tooltip::text("View options"),
                    )
                    .anchor(Anchor::TopRight)
                    .menu(move |window, cx| {
                        let switch = on.clone();
                        let toggle_scope = on.clone();
                        let settings = on.clone();
                        Some(ContextMenu::build(window, cx, move |menu, _, _| {
                            menu.entry("Switch to Tabbed mode", None, move |window, cx| {
                                switch(Action::SwitchToTabs, window, cx)
                            })
                            .toggleable_entry(
                                "Show only active space",
                                active_space_only,
                                IconPosition::End,
                                None,
                                move |window, cx| {
                                    toggle_scope(Action::ToggleActiveSpaceOnly, window, cx)
                                },
                            )
                            .separator()
                            .entry(
                                "Settings",
                                None,
                                move |window, cx| settings(Action::OpenSettings, window, cx),
                            )
                        }))
                    })
                    .into_any_element()
            }
            Mode::Tabs => PopoverMenu::new("chrome-menu")
                .trigger_with_tooltip(
                    IconButton::new("chrome-menu-trigger", IconName::ChevronDown)
                        .icon_size(IconSize::Small),
                    Tooltip::text("View options"),
                )
                .anchor(Anchor::TopRight)
                .menu(move |window, cx| {
                    let switch = on.clone();
                    let settings = on.clone();
                    Some(ContextMenu::build(window, cx, move |menu, _, _| {
                        menu.entry("Switch to Sidebar mode", None, move |window, cx| {
                            switch(Action::SwitchToSidebar, window, cx)
                        })
                        .separator()
                        .entry("Settings", None, move |window, cx| {
                            settings(Action::OpenSettings, window, cx)
                        })
                    }))
                })
                .into_any_element(),
        }
    }

    fn workspace_title_bar(&self, controls: Option<(AnyElement, AnyElement)>) -> AnyElement {
        if !cfg!(target_os = "macos") {
            return self.title_bar.clone().into_any_element();
        }

        let mut overlays = Vec::with_capacity(2);
        if let Some((space_switcher, view_menu)) = controls {
            overlays.push(
                h_flex()
                    .absolute()
                    // Clear the native macOS traffic-light cluster.
                    .left(px(78.))
                    .top_0()
                    .h(px(crate::title_bar::HEIGHT))
                    .max_w(px(200.))
                    .child(space_switcher)
                    .into_any_element(),
            );
            overlays.push(
                h_flex()
                    .absolute()
                    .right(px(6.))
                    .top_0()
                    .h(px(crate::title_bar::HEIGHT))
                    .child(view_menu)
                    .into_any_element(),
            );
        }

        div()
            .id("workspace-title-bar-with-controls")
            .relative()
            .w_full()
            .h(px(crate::title_bar::HEIGHT))
            .flex_none()
            .child(self.title_bar.clone())
            .children(overlays)
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

    fn pane_new_item_button(
        &self,
        tab_id: WorkspaceTabId,
        pane_id: LayoutPaneId,
        weak: &gpui::WeakEntity<Self>,
    ) -> AnyElement {
        let button_id = format!("new-item-pane-{}-{}", tab_id.get(), pane_id.get());
        let start = weak.clone();
        IconButton::new(button_id, IconName::Plus)
            .icon_size(IconSize::XSmall)
            .tooltip(Tooltip::text("New session in this pane"))
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                let _ = start.update(cx, |this, cx| {
                    if matches!(this.backend, Backend::Ready)
                        && let Some(space) = this.active.clone()
                    {
                        space.update(cx, |space, cx| space.start_session_in(tab_id, pane_id, cx));
                    }
                });
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
        if dragged.grouped {
            space.update(cx, |space, _| {
                space.clear_drag_target();
            });
            cx.notify();
            return;
        }
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
                    .child(Label::new("Starting the terminal backend…").size(UI_LABEL_DEFAULT))
                    .into_any_element(),
            ),
            Backend::Recovering(detail) => notices.push(
                Banner::new()
                    .severity(Severity::Warning)
                    .child(Label::new(detail).size(UI_LABEL_DEFAULT))
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
                        .child(Label::new(detail).size(UI_LABEL_DEFAULT))
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
                    .child(Label::new(problem).size(UI_LABEL_DEFAULT))
                    .into_any_element(),
            );
        }
        if let Some(problem) = space_problem {
            notices.push(
                Banner::new()
                    .severity(Severity::Warning)
                    .child(Label::new(problem).size(UI_LABEL_DEFAULT))
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
                                        .child(Label::new(detail).size(UI_LABEL_DEFAULT))
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
                        let dragged = event.drag(cx);
                        let accepted = !dragged.grouped && dragged.space == drag_space;
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
                h_flex().gap_1().child(self.pane_new_item_button(tab_id, pane_id, weak)).child(
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
                                space.update(cx, |space, _| {
                                    space.remove_empty_pane(tab_id, pane_id)
                                });
                            }
                            cx.notify();
                        });
                    }),
                ),
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
                top_level: false,
                grouped: false,
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
                            .is_some_and(|dragged| !dragged.grouped && dragged.space == drop_space)
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
                        false,
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
                    .child(chrome::tab_label(item.title()))
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
                    .is_some_and(|dragged| !dragged.grouped && dragged.space == append_space)
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
            .end_child(self.pane_new_item_button(tab_id, pane_id, weak))
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
                            .child(Label::new(label).size(UI_LABEL_DEFAULT))
                            .when(!shortcut.is_empty(), |row| {
                                row.child(
                                    Label::new(shortcut).size(UI_LABEL_SMALL).color(Color::Muted),
                                )
                            }),
                    )
            })
            .collect();
        let dismiss = cx.listener(|this, _, window, cx| {
            this.command_palette_open = false;
            this.command_palette_query.clear();
            this.command_palette_input.update(cx, |input, cx| input.clear(cx));
            window.focus(&this.focus, cx);
            cx.notify();
        });

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
                                .child(self.command_palette_input.clone()),
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
        let cancel_scrim = cx.listener(|this, _, window, cx| {
            this.rename_space = None;
            this.rename_query.clear();
            this.rename_input.update(cx, |input, cx| input.clear(cx));
            window.focus(&this.focus, cx);
            cx.notify();
        });
        let cancel_button = cx.listener(|this, _, window, cx| {
            this.rename_space = None;
            this.rename_query.clear();
            this.rename_input.update(cx, |input, cx| input.clear(cx));
            window.focus(&this.focus, cx);
            cx.notify();
        });
        let save = cx.listener(|this, _, window, cx| this.commit_space_rename(window, cx));
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
                        .child(Label::new("Rename Space").size(UI_LABEL_LARGE))
                        .child(
                            h_flex()
                                .h(px(36.))
                                .px_2()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().colors().border_focused)
                                .bg(cx.theme().colors().editor_background)
                                .child(self.rename_input.clone()),
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

    fn rename_group_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        self.rename_group?;
        let cancel_scrim = cx.listener(|this, _, window, cx| {
            this.rename_group = None;
            this.rename_query.clear();
            this.rename_input.update(cx, |input, cx| input.clear(cx));
            window.focus(&this.focus, cx);
            cx.notify();
        });
        let cancel_button = cx.listener(|this, _, window, cx| {
            this.rename_group = None;
            this.rename_query.clear();
            this.rename_input.update(cx, |input, cx| input.clear(cx));
            window.focus(&this.focus, cx);
            cx.notify();
        });
        let save = cx.listener(|this, _, window, cx| this.commit_group_rename(window, cx));
        Some(
            div()
                .id("rename-group-scrim")
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .bg(gpui::black().opacity(0.35))
                .on_mouse_down(gpui::MouseButton::Left, cancel_scrim)
                .child(
                    v_flex()
                        .id("rename-group-dialog")
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
                        .child(Label::new("Rename Group").size(UI_LABEL_LARGE))
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .h(px(36.))
                                        .px_2()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(cx.theme().colors().border_focused)
                                        .bg(cx.theme().colors().editor_background)
                                        .child(self.rename_input.clone()),
                                )
                                .child(
                                    Label::new("Leave blank to use the tab count.")
                                        .size(UI_LABEL_SMALL)
                                        .color(Color::Muted),
                                ),
                        )
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_1()
                                .child(
                                    Button::new("cancel-group-rename", "Cancel")
                                        .on_click(cancel_button),
                                )
                                .child(Button::new("save-group-rename", "Rename").on_click(save)),
                        ),
                )
                .into_any_element(),
        )
    }
}

impl Focusable for Zeddy {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Zeddy {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ui_font = Fonts::setup_ui(window, cx);
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
        let now = cx.background_executor().now();
        if self.space_sorter.tick(now, window.rem_size(), cx.reduce_motion()) {
            window.request_animation_frame();
        }
        let mut sidebar_spaces = self.sidebar_spaces(cx);
        self.space_sorter.arrange(&mut sidebar_spaces, |space| space.id);
        let chrome_entries: &[Entry] = &entries;
        let new_item = self.new_item_menu(cx);
        let (background, text, workspace_background) = {
            let colors = cx.theme().colors();
            (colors.background, colors.text, colors.editor_background)
        };

        let on_action =
            cx.listener(|this, action: &Action, window, cx| this.act(action.clone(), window, cx));
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));
        let title_controls = cfg!(target_os = "macos")
            .then(|| (self.space_switcher(window, cx), self.view_menu(emit.clone())));
        let title_bar = self.workspace_title_bar(title_controls);

        let workspace = v_flex()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .bg(workspace_background)
            .child(self.workspace_pane(window, cx));

        let body = match self.mode {
            Mode::Sidebar => {
                let controls = (!cfg!(target_os = "macos"))
                    .then(|| (self.space_switcher(window, cx), self.view_menu(emit.clone())));
                h_flex()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .child(chrome::sidebar::render(
                        &sidebar_spaces,
                        controls,
                        emit.clone(),
                        &self.space_sorter,
                        self.sidebar_width,
                        cx,
                    ))
                    .child(workspace)
                    .into_any_element()
            }
            Mode::Tabs => {
                let controls = (!cfg!(target_os = "macos"))
                    .then(|| (self.space_switcher(window, cx), self.view_menu(emit.clone())));
                v_flex()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .child(chrome::tabs::render(chrome_entries, controls, new_item, emit, cx))
                    .child(workspace)
                    .into_any_element()
            }
        };

        let command_palette = self.command_palette(cx);
        let rename_space = self.rename_space_overlay(cx);
        let rename_group = self.rename_group_overlay(cx);

        div()
            .relative()
            .track_focus(&self.focus)
            .key_context(if self.rename_space.is_some() {
                "RenameSpace"
            } else if self.rename_group.is_some() {
                "RenameGroup"
            } else if self.command_palette_open {
                "CommandPalette"
            } else {
                "Chartr"
            })
            .size_full()
            .flex()
            .flex_col()
            .font(ui_font)
            .text_size(UI_TEXT_DEFAULT)
            .bg(background)
            .text_color(text)
            .on_drag_move::<chrome::DraggedSidebar>(cx.listener(
                |this, event: &DragMoveEvent<chrome::DraggedSidebar>, _, cx| {
                    this.sidebar_width = (event.event.position.x / px(1.))
                        .clamp(chrome::sidebar::MIN_WIDTH, chrome::sidebar::MAX_WIDTH);
                    cx.notify();
                },
            ))
            .on_drag_move::<chrome::DraggedSpace>(cx.listener(
                |this, event: &DragMoveEvent<chrome::DraggedSpace>, window, cx| {
                    let dragged = event.drag(cx).0;
                    let order = this.spaces.iter().map(|space| space.entity_id()).collect();
                    if this.space_sorter.drag_move(
                        dragged,
                        order,
                        event.event.position,
                        window.rem_size(),
                        cx.background_executor().now(),
                        cx.reduce_motion(),
                    ) {
                        cx.notify();
                    }
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    this.finish_space_drag(event.position.y, window, cx)
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    this.finish_space_drag(event.position.y, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &actions::pane::CloseActiveItem, _, cx| {
                this.close_active_item(cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::CloseAllItems, window, cx| {
                this.request_close_active_pane(window, cx)
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
            .on_action(cx.listener(|this, _: &actions::workspace::NewTerminal, window, cx| {
                this.act(Action::New, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::settings::Open, window, cx| {
                this.open_settings(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::command_palette::Toggle, window, cx| {
                this.toggle_command_palette(window, cx)
            }))
            .on_key_down(cx.listener(|this, event, window, cx| this.on_key(event, window, cx)))
            .child(title_bar)
            .child(body)
            .children(command_palette)
            .children(rename_space)
            .children(rename_group)
    }
}

/// Seats every space the current registry/state can recover in the last saved
/// full-vector order. Registry-only additions retain file order at the end;
/// an older snapshot that predates the synthetic Free entry gets that entry at
/// the front instead of unexpectedly moving it behind every recovered folder.
fn restore_space_order(
    mut spaces: Vec<Entity<Space>>,
    saved: &[crate::persistence::PersistedSpace],
    cx: &App,
) -> Vec<Entity<Space>> {
    if saved.is_empty() {
        return spaces;
    }
    let mut ordered = Vec::with_capacity(spaces.len());
    for saved in saved {
        let Some(index) = spaces.iter().position(|space| space.read(cx).key() == saved.key) else {
            continue;
        };
        ordered.push(spaces.remove(index));
    }
    if !saved.iter().any(|space| space.kind == PersistedSpaceKind::AdHoc)
        && let Some(index) =
            spaces.iter().position(|space| space.read(cx).kind() == SpaceKind::AdHoc)
    {
        ordered.insert(0, spaces.remove(index));
    }
    ordered.extend(spaces);
    ordered
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
            value
                .downcast_ref::<DraggedItem>()
                .is_some_and(|dragged| !dragged.grouped && dragged.space == space)
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
    let fit = item.fit.clone();
    let session = item.session.access();
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

    v_flex()
        .size_full()
        .p_2()
        .bg(theme.colors().terminal_background)
        .on_scroll_wheel(move |event, window, cx| {
            let Some(lines) = fit.wheel_lines(event) else {
                return;
            };
            if session.scroll(lines) {
                window.refresh();
            }
            cx.stop_propagation();
        })
        .child(TerminalElement::new(screen, colors, appearance, focused, item.fit.clone()))
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
        .child(Label::new(text.to_owned()).size(UI_LABEL_DEFAULT).color(Color::Muted))
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
