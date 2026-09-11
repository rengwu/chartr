//! The window-level workspace.
//!
//! This follows Zed's `MultiWorkspace` ownership boundary: the window owns an
//! ordered collection of independently stateful space entities, keeps one
//! active, observes each child, and renders only the active one. A space owns
//! its sessions and selection; switching spaces therefore never moves or
//! recreates a session.

mod backend;
mod bundled_plugins;
mod command_palette;
mod companion;
mod conversations;
mod pane_drop_preview;
mod panes;
mod persistence;
mod plugins;
mod rename;
mod settings_bridge;
mod shortcuts;
mod status_bar;
mod terminal_search;
#[cfg(test)]
mod tests;
mod view;
mod window_chrome;

use bundled_plugins::load_plugin_catalog;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use chartr_herdr::{Namespace, Sidecar, WorkspaceId, control::Client};
use chartr_plugin::{InstanceContext, manifest::Multiplicity};
use chartr_plugin_host::{Catalog, FileBroker, HostedSurface, PaneSource, Paths, SettingsSource};
use gpui::{
    Anchor, AnyView, AnyWindowHandle, ClickEvent, DragMoveEvent, Entity, EntityId, FocusHandle,
    Focusable, MouseButton, PathPromptOptions, Role, WeakEntity,
};
use ui::{
    Banner, ButtonLike, ButtonSize, IconButtonShape, IconPosition, ListItem, ListItemSpacing,
    Severity, TabBar, Tooltip, prelude::*,
};

use crate::{
    actions,
    chrome::{self, Action, DraggedItem, Entry, SpaceEntries},
    components::{ContextMenu, PopupMenu, SegmentedControl, SegmentedControlOption},
    fonts::{Fonts, UI_LABEL_DEFAULT, UI_LABEL_LARGE, UI_LABEL_SMALL, UI_TEXT_DEFAULT},
    item::{PluginItem, PluginView},
    mode::Mode,
    persistence::{Snapshot, SpaceKind as PersistedSpaceKind, StateStore, WindowState},
    settings::{PluginSettingsContent, SettingsStore},
    space::{Kind as SpaceKind, Space, SpaceEvent, name_for},
    spaces::{self, Registry},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ErrorSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ErrorNoticeKey {
    source: String,
    message: String,
    severity: ErrorSeverity,
}

#[derive(Debug, Clone)]
struct ErrorNotice {
    key: ErrorNoticeKey,
    first_seen: Instant,
}

#[derive(Clone)]
pub(crate) struct SettingsPluginDescriptor {
    pub manifest: chartr_plugin::manifest::Manifest,
    pub installation: Option<chartr_plugin_host::Installation>,
    pub prerequisite_error: Option<String>,
    pub bundled: bool,
    pub enabled: bool,
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
    NewTerminalPane,
    NewSurface,
    NewSurfacePane,
    Ungroup,
    SidebarMode,
    TabbedMode,
    ConversationMode,
    CycleViewMode,
    ToggleStatusBar,
    NewSpace,
    CloseSpace,
    ZoomIn,
    ZoomOut,
    TerminalZoomIn,
    TerminalZoomOut,
    NewFreeTerminal,
    NewFreeSurface,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenameKind {
    Space,
    Group,
}

impl PaletteCommand {
    const ALL: [(Self, &'static str, &'static str); 30] = [
        (Self::NewTerminal, "Workspace: New Terminal", "Ctrl+~"),
        (Self::NewTerminalPane, "Workspace: New terminal pane", ""),
        (Self::NewSurface, "Workspace: New surface tab", ""),
        (Self::NewSurfacePane, "Workspace: New surface pane", ""),
        (Self::Ungroup, "Workspace: Ungroup current group", ""),
        (Self::SidebarMode, "Workspace: Switch to sidebar mode", ""),
        (Self::TabbedMode, "Workspace: Switch to tabbed mode", ""),
        (Self::ConversationMode, "Workspace: Switch to inbox view", ""),
        (Self::CycleViewMode, "Workspace: Cycle view modes", ""),
        (Self::ToggleStatusBar, "Workspace: Toggle status bar", ""),
        (Self::NewSpace, "Workspace: Open new space", ""),
        (Self::CloseSpace, "Workspace: Close current space", ""),
        (Self::ZoomIn, "Workspace: Zoom in interface", ""),
        (Self::ZoomOut, "Workspace: Zoom out interface", ""),
        (Self::TerminalZoomIn, "Workspace: Zoom in terminal", ""),
        (Self::TerminalZoomOut, "Workspace: Zoom out terminal", ""),
        (Self::NewFreeTerminal, "Workspace: New free terminal session", ""),
        (Self::NewFreeSurface, "Workspace: New free surface", ""),
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
        (Self::OpenSettings, "chartr: Open Settings", "Cmd/Ctrl+,"),
    ];
}

impl Render for DraggedPaneDivider {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// The root view, analogous to Zed's `MultiWorkspace`.
pub struct WorkspaceWindow {
    client: Option<Client>,
    backend: Backend,
    backend_ready_since: Option<Instant>,
    backend_restart_spent: bool,
    supervision_started: bool,
    registry: Option<Registry>,
    spaces: Vec<Entity<Space>>,
    space_sorter: chrome::sidebar::SpaceSorter,
    pane_drop_preview: pane_drop_preview::PaneDropPreview,
    active_pane_size: Rc<std::cell::Cell<Option<shortcuts::MeasuredPane>>>,
    active: Option<Entity<Space>>,
    mode: Mode,
    terminal_mode: Mode,
    mode_focus_pending: bool,
    conversations: Entity<crate::conversations::Conversations>,
    conversation_all_spaces: bool,
    mode_transition: crate::mode::ModeTransition,
    catalog: Catalog,
    background_statuses: Vec<(String, chartr_plugin::BackgroundStatus)>,
    companion_bridge: Option<crate::companion_plugin::Bridge>,
    companion_history: chartr_companion::History,
    companion_leases: HashMap<String, companion::MobileLease>,
    plugins_restored: bool,
    settings: SettingsStore,
    command_palette_window: Option<AnyWindowHandle>,
    terminal_search_open: bool,
    terminal_search_input: Entity<TextInput>,
    terminal_search_query: String,
    terminal_search_matches: Vec<terminal::Range>,
    terminal_search_active: Option<usize>,
    terminal_search_generation: u64,
    terminal_search_target: Option<Entity<terminal::Terminal>>,
    rename_space: Option<EntityId>,
    rename_group: Option<(EntityId, WorkspaceTabId)>,
    rename_window: Option<AnyWindowHandle>,
    rename_input: Entity<TextInput>,
    rename_query: String,
    show_space_picker: bool,
    sidebar_width: f32,
    window_bounds: Option<crate::persistence::WindowBounds>,
    state: Option<crate::persistence::StateWriter>,
    persistence_dirty: bool,
    persistence_task: Option<gpui::Task<()>>,
    title_bar: Entity<crate::title_bar::TitleBar>,
    focus: FocusHandle,
    problem: Option<String>,
    error_seen_at: HashMap<ErrorNoticeKey, Instant>,
    dismissed_errors: HashSet<ErrorNoticeKey>,
}

impl WorkspaceWindow {
    pub fn new(
        cwd: PathBuf,
        opened_path: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::observe_background_status(cx);
        let focus = cx.focus_handle();
        cx.on_focus_in(&focus, window, |this, window, cx| {
            this.focus_active_terminal(window, cx);
        })
        .detach();
        cx.observe_window_bounds(window, |this, window, cx| {
            this.capture_window_bounds(window);
            this.schedule_persistence(cx);
            if let Some(palette) = this.command_palette_window {
                let modal_size = window.viewport_size();
                let _ = palette.update(cx, |_, window, _| window.resize(modal_size));
            }
            if let Some(rename_window) = this.rename_window {
                let modal_size = window.viewport_size();
                let _ = rename_window.update(cx, |_, window, _| window.resize(modal_size));
            }
        })
        .detach();
        let settings = cx.global::<SettingsStore>().clone();
        cx.observe_global::<SettingsStore>(|this, cx| {
            let previous = this.settings.resolved().clone();
            this.settings = cx.global::<SettingsStore>().clone();
            this.sync_plugin_settings(&previous, cx);
            cx.set_reduce_motion(this.settings.resolved().reduce_motion);
            cx.notify();
        })
        .detach();
        let terminal_search_input = cx.new(|cx| TextInput::new("Find in terminal…", cx));
        let rename_input = cx.new(|cx| TextInput::new("Type a name…", cx));
        let title_bar =
            cx.new(|_| crate::title_bar::TitleBar::new("workspace-title-bar").borderless());
        cx.subscribe(&terminal_search_input, |this, input, _: &InputEvent, cx| {
            this.terminal_search_query = input.read(cx).text().to_owned();
            this.start_terminal_search(cx);
        })
        .detach();
        cx.subscribe(&rename_input, |this, input, _: &InputEvent, cx| {
            this.rename_query = input.read(cx).text().to_owned();
            cx.notify();
        })
        .detach();
        let (mut state, mut saved, mut state_problem) =
            match crate::persistence::state_file().and_then(StateStore::open) {
                Ok(store) => match store.load() {
                    Ok(saved) => (Some(store), saved, None),
                    Err(error) => (Some(store), Snapshot::default(), Some(error.to_string())),
                },
                Err(error) => (None, Snapshot::default(), Some(error.to_string())),
            };
        let persisted = saved.clone();
        let conversations = cx.new(|cx| {
            crate::conversations::Conversations::new(saved.window.selected_conversation.clone(), cx)
        });
        cx.subscribe_in(&conversations, window, |this, _, event, window, cx| {
            this.conversation_event(event, window, cx)
        })
        .detach();
        cx.observe(&conversations, |_, _, cx| cx.notify()).detach();
        let mut this = Self {
            active_pane_size: Rc::default(),
            client: None,
            backend: Backend::Starting,
            backend_ready_since: None,
            backend_restart_spent: false,
            supervision_started: false,
            registry: None,
            spaces: Vec::new(),
            space_sorter: chrome::sidebar::SpaceSorter::new(chrome::sidebar::CARD_GAP),
            pane_drop_preview: pane_drop_preview::PaneDropPreview::default(),
            active: None,
            mode: Mode::default(),
            terminal_mode: saved.window.terminal_mode,
            mode_focus_pending: true,
            conversations,
            conversation_all_spaces: saved.window.conversation_all_spaces,
            mode_transition: crate::mode::ModeTransition::default(),
            catalog: Catalog::default(),
            background_statuses: Vec::new(),
            companion_bridge: None,
            companion_history: chartr_companion::History::default(),
            companion_leases: HashMap::new(),
            plugins_restored: false,
            settings,
            command_palette_window: None,
            terminal_search_open: false,
            terminal_search_input,
            terminal_search_query: String::new(),
            terminal_search_matches: Vec::new(),
            terminal_search_active: None,
            terminal_search_generation: 0,
            terminal_search_target: None,
            rename_space: None,
            rename_group: None,
            rename_window: None,
            rename_input,
            rename_query: String::new(),
            show_space_picker: saved.window.show_space_picker,
            sidebar_width: saved.window.sidebar_width,
            window_bounds: saved.window.bounds,
            state: None,
            persistence_dirty: false,
            persistence_task: None,
            title_bar,
            focus,
            problem: state_problem.clone(),
            error_seen_at: HashMap::new(),
            dismissed_errors: HashSet::new(),
        };
        let client = match Sidecar::beside_current_exe()
            .map(|sidecar| Client::new(sidecar, Namespace::private()))
        {
            Ok(client) => client,
            Err(error) => {
                this.backend = Backend::Failed(error.to_string());
                this.plugins_restored = true;
                // No spaces were restored: keep the saved workspace intact.
                this.problem = Some(state_problem.unwrap_or_else(|| error.to_string()));
                return this;
            }
        };

        let (mut registry, mut registry_problem) = load_registry(opened_path.as_deref());
        if opened_path.is_none() && cwd.parent().is_none() {
            let cleanup_pending = match state.as_ref() {
                Some(state) => match state.implicit_root_cleanup_pending() {
                    Ok(pending) => pending,
                    Err(error) => {
                        state_problem = Some(error.to_string());
                        false
                    }
                },
                None => true,
            };
            if cleanup_pending {
                let cleanup = registry
                    .as_mut()
                    .map(|registry| cleanup_empty_implicit_root(registry, &mut saved, &cwd))
                    .transpose();
                match cleanup {
                    Ok(Some(changed)) => {
                        if let Some(state) = state.as_mut() {
                            let result = if changed { state.save(&saved) } else { Ok(()) }
                                .and_then(|_| state.complete_implicit_root_cleanup());
                            if let Err(error) = result {
                                state_problem = Some(error.to_string());
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(error) => registry_problem = Some(error.to_string()),
                }
            }
        }
        let mut descriptors = Vec::new();
        let home = this
            .settings
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
        let spaces: Vec<_> = descriptors
            .into_iter()
            .map(|(name, path, kind)| {
                let space = cx.new(|cx| Space::new(name, path, kind, client.clone(), cx));
                Self::subscribe_to_space(&space, window, cx);
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
        let active = opened_path
            .as_ref()
            .and_then(|opened_path| {
                spaces
                    .iter()
                    .find(|space| {
                        let space = space.read(cx);
                        space.kind() == SpaceKind::Registered
                            && spaces::same_path(space.path(), opened_path)
                    })
                    .cloned()
            })
            .or_else(|| {
                saved.window.active_space.as_ref().and_then(|key| {
                    spaces.iter().find(|space| space.read(cx).persisted().key == *key).cloned()
                })
            })
            .or_else(|| spaces.first().cloned());

        this.bind_companion(window, cx);
        let catalog = load_plugin_catalog(&this.settings, cx);
        this.client = Some(client);
        this.registry = registry;
        this.spaces = spaces;
        this.active = active;
        this.mode = saved.window.chrome;
        this.catalog = catalog;
        this.conversations
            .update(cx, |view, _| view.set_agent_services(this.catalog.services.clone()));
        this.state = state.map(|state| crate::persistence::StateWriter::new(state, persisted));
        this.capture_window_bounds(window);
        cx.observe_self(|this, cx| this.schedule_persistence(cx)).detach();
        cx.on_release(|this, cx| this.flush_state(cx)).detach();
        this.schedule_persistence(cx);
        this.problem = state_problem.or(registry_problem);
        this.connect(cx);
        this
    }

    fn subscribe_to_space(space: &Entity<Space>, window: &mut Window, cx: &mut Context<Self>) {
        cx.observe_in(space, window, |this, space, window, cx| {
            if this.active.as_ref() == Some(&space)
                && this.focus.is_focused(window)
                && this.command_palette_window.is_none()
                && !this.terminal_search_open
                && this.rename_space.is_none()
                && this.rename_group.is_none()
            {
                this.focus_active_terminal(window, cx);
            }
            cx.notify();
        })
        .detach();
        cx.subscribe_in(space, window, |this, space, event, window, cx| {
            let SpaceEvent::TerminalReady(id) = *event;
            let Some(terminal) = space
                .read(cx)
                .item(id)
                .and_then(crate::item::Item::as_session)
                .map(|item| item.session.terminal())
            else {
                return;
            };
            let view = crate::terminal_host::new_view(terminal.clone(), window, cx);
            space.update(cx, |space, _| space.install_terminal_view(id, view.clone()));
            cx.subscribe(&terminal, |_, _, event, cx| {
                let terminal::Event::Open(terminal::MaybeNavigationTarget::PathLike(target)) =
                    event
                else {
                    return;
                };
                if let Some(path) = resolve_terminal_path(target)
                    && let Ok(url) = url::Url::from_file_path(path)
                {
                    cx.open_url(url.as_str());
                }
            })
            .detach();
            let observed_space = space.clone();
            cx.observe(&view, move |_, view, cx| {
                let bell = view.read(cx).has_bell();
                if observed_space.update(cx, |space, _| space.set_terminal_bell(id, bell)) {
                    cx.notify();
                }
            })
            .detach();

            if this.active.as_ref() == Some(space)
                && space.read(cx).active() == Some(id)
                && this.command_palette_window.is_none()
                && this.rename_space.is_none()
                && this.rename_group.is_none()
            {
                this.focus_active_terminal(window, cx);
            }
            cx.notify();
        })
        .detach();
    }

    fn focus_active_terminal(&self, window: &mut Window, cx: &mut App) -> bool {
        if self.mode == Mode::Inbox {
            self.conversations.focus_handle(cx).focus(window, cx);
            return true;
        }
        let Some(view) = self
            .active
            .as_ref()
            .and_then(|space| {
                let space = space.read(cx);
                space.active().and_then(|id| space.item(id))
            })
            .and_then(crate::item::Item::as_session)
            .filter(|item| !self.companion_leases.contains_key(&item.session.id().0))
            .and_then(crate::item::SessionItem::terminal_view)
        else {
            return false;
        };
        window.focus(&view.read(cx).focus_handle(cx), cx);
        true
    }

    fn active_space(&self) -> Option<&Entity<Space>> {
        self.active.as_ref()
    }

    fn snapshot(&self, cx: &App) -> Snapshot {
        Snapshot {
            window: WindowState {
                chrome: self.mode,
                terminal_mode: self.terminal_mode,
                selected_conversation: self.conversations.read(cx).selected().map(str::to_owned),
                conversation_all_spaces: self.conversation_all_spaces,
                show_space_picker: self.show_space_picker,
                sidebar_width: self.sidebar_width,
                active_space: self.active.as_ref().map(|space| space.read(cx).key()),
                bounds: self.window_bounds,
                ..WindowState::default()
            },
            spaces: self.spaces.iter().map(|space| space.read(cx).persisted()).collect(),
        }
    }

    fn activate(&mut self, space: Entity<Space>, window: &mut Window, cx: &mut Context<Self>) {
        if self.active.as_ref() == Some(&space) {
            window.focus(&self.focus, cx);
            return;
        }
        self.active = Some(space.clone());
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn pick_a_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Add".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let outcome = chosen.await;
            let _ = this.update_in(cx, |this, window, cx| match outcome {
                Ok(Ok(Some(paths))) => {
                    for path in paths {
                        this.register(path, window, cx);
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

    fn register(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
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
        Self::subscribe_to_space(&space, window, cx);
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
                Err(error) => {
                    this.conversations.update(cx, |view, cx| view.disconnected(cx));
                    this.problem = Some(error.to_string());
                }
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
        self.spaces
            .iter()
            .map(|space| {
                let read = space.read(cx);
                SpaceEntries {
                    id: space.entity_id(),
                    name: read.name().to_owned(),
                    is_free: read.kind() == SpaceKind::AdHoc,
                    active: self.active.as_ref() == Some(space),
                    removable: read.kind() == SpaceKind::Registered,
                    available: read.available(),
                    entries: read.entries(space.entity_id()),
                }
            })
            .collect()
    }

    fn act(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Mode::Inbox
            && matches!(
                &action,
                Action::New
                    | Action::NewPluginPane
                    | Action::NewInSpace { .. }
                    | Action::NewPluginPaneInSpace { .. }
            )
        {
            self.settings_set_mode(self.terminal_mode, cx);
        }
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
            Action::SwitchToConversations => self.settings_set_mode(Mode::Inbox, cx),
            Action::OpenSettings => self.open_settings(window, cx),
            Action::NewSpace => self.pick_a_folder(window, cx),
            Action::New => {
                if matches!(self.backend, Backend::Ready)
                    && let Some(space) = self.active.clone()
                {
                    space.update(cx, |space, cx| space.start_session(cx));
                }
            }
            Action::NewPluginPane => {
                if let Some(space) = self.active.clone() {
                    space.update(cx, |space, cx| {
                        space.open_plugin_launcher(cx);
                    });
                }
            }
            Action::NewPluginPaneInSpace { space } => {
                if let Some(target) =
                    self.spaces.iter().find(|candidate| candidate.entity_id() == space).cloned()
                {
                    self.activate(target.clone(), window, cx);
                    target.update(cx, |space, cx| {
                        space.open_plugin_launcher(cx);
                    });
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
                    self.open_rename_window(RenameKind::Group, window, cx);
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
                    self.open_rename_window(RenameKind::Space, window, cx);
                }
            }
            Action::OpenSpaceFolder { space } => self.open_space_folder(space, cx),
            Action::LocateSpace { space } => self.locate_space(space, cx),
            action @ (Action::Select { .. } | Action::Close { .. }) => {
                let selecting = matches!(action, Action::Select { .. });
                let target = match &action {
                    Action::Select { space, .. } | Action::Close { space, .. } => space
                        .and_then(|id| self.spaces.iter().find(|space| space.entity_id() == id))
                        .cloned()
                        .or_else(|| self.active.clone()),
                    _ => None,
                };
                if let Some(space) = target {
                    if selecting {
                        self.activate(space.clone(), window, cx);
                    }
                    space.update(cx, |space, cx| space.act(action, cx));
                    if selecting {
                        self.focus_active_terminal(window, cx);
                    }
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
        if self.spaces[from].read(cx).kind() == SpaceKind::AdHoc {
            return false;
        }

        // The sorter only sees folder-backed cards; Free sessions is outside
        // its scroll container. Translate that card-relative target back to
        // the complete model without changing where the synthetic space is
        // stored.
        let movable_count = self
            .spaces
            .iter()
            .filter(|candidate| candidate.read(cx).kind() != SpaceKind::AdHoc)
            .count();
        let target = target.min(movable_count.saturating_sub(1));
        let from_movable = self.spaces[..from]
            .iter()
            .filter(|candidate| candidate.read(cx).kind() != SpaceKind::AdHoc)
            .count();
        if from_movable == target {
            return true;
        }

        let mut candidate = self.spaces.clone();
        let moved = candidate.remove(from);
        let movable_positions: Vec<_> = candidate
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                (candidate.read(cx).kind() != SpaceKind::AdHoc).then_some(index)
            })
            .collect();
        let insertion_index = movable_positions
            .get(target)
            .copied()
            .or_else(|| movable_positions.last().map(|index| index + 1))
            .unwrap_or(candidate.len());
        candidate.insert(insertion_index, moved);

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
        if let Some(palette) = self.command_palette_window.take() {
            let _ = palette.update(cx, |_, window, _| window.remove_window());
            cx.notify();
            return;
        }
        if self.mode == Mode::Inbox {
            self.conversations.update(cx, |view, cx| view.archive_selected(cx));
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
        if self.mode == Mode::Inbox {
            return;
        }
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
        if self.mode == Mode::Inbox {
            return;
        }
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
        let (terminal_targets, immediate): (Vec<_>, Vec<_>) =
            targets.into_iter().partition(|(_, backend)| backend.is_some());
        let immediate: Vec<_> = immediate.into_iter().map(|(item, _)| item).collect();
        space.update(cx, |space, _| space.finish_bulk_close(&immediate));

        if terminal_targets.is_empty() {
            if remove_space {
                self.remove_space_after_close(&space, cx);
            }
            cx.notify();
            return;
        }

        let client = self.client.clone();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let results = executor
                .spawn(async move {
                    terminal_targets
                        .into_iter()
                        .map(|(item, backend)| {
                            let result = match (&client, backend) {
                                (_, None) => Ok(()),
                                (Some(client), Some(backend)) => client.close_session(&backend),
                                (None, Some(_)) => Err(chartr_herdr::Error::Protocol(
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

    fn open_space_folder(&mut self, id: EntityId, cx: &mut Context<Self>) {
        let Some(path) = self
            .spaces
            .iter()
            .find(|space| space.entity_id() == id)
            .map(|space| space.read(cx).path().clone())
        else {
            return;
        };
        match url::Url::from_directory_path(&path) {
            Ok(url) => cx.open_url(url.as_str()),
            Err(()) => {
                self.problem = Some(format!("Could not open the folder {}.", path.display()));
                cx.notify();
            }
        }
    }

    fn move_active_to_pane(&mut self, direction: SplitDirection, cx: &mut Context<Self>) {
        if self.mode == Mode::Inbox {
            return;
        }
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
        if self.mode == Mode::Inbox {
            return;
        }
        if let Some(space) = self.active.clone() {
            let moved = space.update(cx, |space, _| space.activate_pane_in_direction(direction));
            if moved {
                // Focus the destination directly. Focusing the workspace root first relies on a
                // re-entrant focus callback, which can leave the terminal without keyboard focus.
                if !self.focus_active_terminal(window, cx) {
                    window.focus(&self.focus, cx);
                }
                cx.notify();
            }
        }
    }

    fn join_active_into_next(&mut self, cx: &mut Context<Self>) {
        if self.mode == Mode::Inbox {
            return;
        }
        if let Some(space) = self.active.clone() {
            space.update(cx, |space, _| space.join_active_into_next());
            cx.notify();
        }
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
    }
}

impl Focusable for WorkspaceWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

fn error_menu_header(notice_count: usize, clear_button: AnyElement) -> AnyElement {
    h_flex()
        .w_full()
        .py_1()
        .justify_between()
        .gap_2()
        .child(
            Label::new(format!("Problems ({notice_count})"))
                .size(UI_LABEL_SMALL)
                .color(Color::Muted),
        )
        .child(clear_button)
        .into_any_element()
}

fn error_notice_row(
    notice: &ErrorNotice,
    now: Instant,
    dismiss_button: AnyElement,
    cx: &App,
) -> AnyElement {
    let (icon, color) = match notice.key.severity {
        ErrorSeverity::Warning => (IconName::Warning, Color::Warning),
        ErrorSeverity::Error => (IconName::XCircle, Color::Error),
    };
    h_flex()
        .w_full()
        .min_w_0()
        .items_start()
        .gap_2()
        .mb_1()
        .p_2()
        .rounded_md()
        .bg(crate::settings::sidebar_theme_colors(cx.theme()).card_inactive)
        .child(div().flex_none().child(Icon::new(icon).size(IconSize::Small).color(color)))
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_1()
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .gap_2()
                        .child(
                            Label::new(notice.key.source.clone())
                                .size(UI_LABEL_SMALL)
                                .weight(gpui::FontWeight::MEDIUM),
                        )
                        .child(
                            h_flex()
                                .flex_none()
                                .gap_1()
                                .child(
                                    Label::new(relative_error_time(notice.first_seen, now))
                                        .size(UI_LABEL_SMALL)
                                        .color(Color::Muted),
                                )
                                .child(dismiss_button),
                        ),
                )
                .child(Label::new(notice.key.message.clone()).size(UI_LABEL_DEFAULT)),
        )
        .into_any_element()
}

fn reconcile_error_notices(
    current: Vec<ErrorNoticeKey>,
    seen_at: &mut HashMap<ErrorNoticeKey, Instant>,
    dismissed: &mut HashSet<ErrorNoticeKey>,
    now: Instant,
) -> Vec<ErrorNotice> {
    seen_at.retain(|key, _| current.contains(key));
    dismissed.retain(|key| current.contains(key));
    for key in &current {
        seen_at.entry(key.clone()).or_insert(now);
    }
    let mut notices = current
        .into_iter()
        .filter(|key| !dismissed.contains(key))
        .map(|key| ErrorNotice { first_seen: seen_at[&key], key })
        .collect::<Vec<_>>();
    notices.sort_by(|a, b| b.first_seen.cmp(&a.first_seen));
    notices
}

fn relative_error_time(first_seen: Instant, now: Instant) -> String {
    let elapsed = now.saturating_duration_since(first_seen).as_secs();
    match elapsed {
        0..60 => "now".to_owned(),
        60..3600 => format!("{}m ago", elapsed / 60),
        3600..86400 => format!("{}h ago", elapsed / 3600),
        _ => format!("{}d ago", elapsed / 86400),
    }
}

fn regex_escape_literal(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(
            character,
            '.' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\'
        ) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn resolve_terminal_path(target: &terminal::PathLikeTarget) -> Option<PathBuf> {
    let base = target.working_directory.as_deref();
    let resolve = |text: &str| {
        let path = PathBuf::from(text);
        if path.is_absolute() {
            path
        } else if let Some(base) = base {
            base.join(path)
        } else {
            path
        }
    };
    let direct = resolve(&target.maybe_path);
    if direct.exists() {
        return Some(direct);
    }

    let mut path = target.maybe_path.as_str();
    for _ in 0..2 {
        let (candidate, suffix) = path.rsplit_once(':')?;
        if suffix.parse::<u32>().is_err() {
            return None;
        }
        path = candidate;
        let resolved = resolve(path);
        if resolved.exists() {
            return Some(resolved);
        }
    }
    None
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

fn pane_drop_direction_for_drag<T: 'static>(
    event: &DragMoveEvent<T>,
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

fn drop_target(
    direction: Option<SplitDirection>,
    group: String,
    space: String,
    new_item_space: EntityId,
    preview: &pane_drop_preview::PaneDropPreview,
) -> Div {
    div()
        .invisible()
        .absolute()
        .can_drop(move |value, _, _| {
            value
                .downcast_ref::<DraggedItem>()
                .is_some_and(|dragged| !dragged.grouped && dragged.space == space)
                || value
                    .downcast_ref::<chrome::DraggedNewItem>()
                    .is_some_and(|dragged| dragged.space == new_item_space)
        })
        .group_drag_over::<chrome::DraggedNewItem>(group.clone(), |style| style.visible())
        .group_drag_over::<DraggedItem>(group, |style| style.visible())
        .map(|target| match direction {
            None => target.top_0().right_0().bottom_0().left_0(),
            Some(SplitDirection::Up) => target.top_0().left_0().right_0().h(relative(0.5)),
            Some(SplitDirection::Down) => target.bottom_0().left_0().right_0().h(relative(0.5)),
            Some(SplitDirection::Left) => target.top_0().left_0().bottom_0().w(relative(0.5)),
            Some(SplitDirection::Right) => target.top_0().right_0().bottom_0().w(relative(0.5)),
        })
        .child(preview.target())
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

fn load_registry(opened_path: Option<&std::path::Path>) -> (Option<Registry>, Option<String>) {
    let file = match spaces::spaces_file() {
        Ok(file) => file,
        Err(error) => return (None, Some(error.to_string())),
    };
    let mut registry = match Registry::load(file) {
        Ok(registry) => registry,
        Err(error) => return (None, Some(error.to_string())),
    };
    // Only an explicit command-line path is equivalent to Zed's `zed <path>`.
    // The inherited process working directory belongs to the desktop launcher.
    if let Some(opened_path) = opened_path {
        let is_ad_hoc_home =
            std::env::home_dir().is_some_and(|home| spaces::same_path(&home, opened_path));
        if !is_ad_hoc_home
            && !registry.spaces().iter().any(|space| spaces::same_path(space.path(), opened_path))
            && let Err(error) = registry.register(opened_path)
        {
            return (Some(registry), Some(error.to_string()));
        }
    }
    (Some(registry), None)
}

/// Remove the legacy root row only when it cannot own anything. An explicit
/// `chartr /` launch bypasses this migration, and a root space with items is
/// retained so cleanup can never orphan a live terminal or plugin.
fn cleanup_empty_implicit_root(
    registry: &mut Registry,
    saved: &mut Snapshot,
    root: &std::path::Path,
) -> Result<bool, spaces::Error> {
    let saved_root_has_items = saved.spaces.iter().any(|space| {
        space.kind == PersistedSpaceKind::Folder
            && space.path.as_deref().is_some_and(|path| spaces::same_path(path, root))
            && !space.items.is_empty()
    });
    if saved_root_has_items {
        return Ok(false);
    }

    let registry_changed = registry.remove(root)?;
    let previous_len = saved.spaces.len();
    saved.spaces.retain(|space| {
        space.kind != PersistedSpaceKind::Folder
            || !space.path.as_deref().is_some_and(|path| spaces::same_path(path, root))
    });
    let state_changed = saved.spaces.len() != previous_len;
    let root_key = format!("folder:{}", root.display());
    if state_changed && saved.window.active_space.as_deref() == Some(root_key.as_str()) {
        saved.window.active_space = Some("ad-hoc".to_owned());
    }
    Ok(registry_changed || state_changed)
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

pub(crate) fn plugin_paths() -> Paths {
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        });
    Paths::under(root.join("chartr"))
}
