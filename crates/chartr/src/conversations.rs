//! Inbox: durable agent history with the original session terminal.
mod launcher;
mod scope;
pub use scope::SpaceChoice;
mod view;
pub use view::init;

use crate::text_input::{InputEvent, TextInput};
use chartr_conversations::{Conversation, Observation, ProviderPaths, Status, Store};
use gpui::{App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable};
use std::sync::{Arc, Mutex};

pub enum Event {
    LaunchAgent { name: String, space: String },
    ManageAgents,
    SelectionChanged,
}

pub struct Conversations {
    store: Option<Arc<Mutex<Store>>>,
    rows: Vec<Conversation>,
    selected: Option<String>,
    search: Entity<TextInput>,
    query: String,
    title_input: Entity<TextInput>,
    renaming: Option<String>,
    show_archived: bool,
    connected: bool,
    refreshing: bool,
    pending: Option<Vec<Observation>>,
    pending_runtime: Option<String>,
    problem: Option<String>,
    busy: bool,
    services: chartr_plugin::services::Services,
    new_agent: Option<String>,
    new_space: Option<String>,
    spaces: Vec<SpaceChoice>,
    scope: Option<String>,
    active_space: Option<String>,
    last_launch_agent: Option<String>,
    last_launch_space: Option<String>,
    launch_runtime: Option<String>,
    focus: FocusHandle,
    focus_terminal: bool,
    terminal_view: Option<Entity<terminal_view::TerminalView>>,
    terminal_notice: Option<String>,
}

impl EventEmitter<Event> for Conversations {}
impl Focusable for Conversations {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.terminal_view
            .as_ref()
            .map(|view| view.focus_handle(cx))
            .unwrap_or_else(|| self.focus.clone())
    }
}

impl Conversations {
    pub fn new(selected: Option<String>, cx: &mut Context<Self>) -> Self {
        let loaded = crate::persistence::state_file().and_then(|path| {
            Store::open(
                &path.with_file_name("conversations.sqlite"),
                ProviderPaths::from_environment(),
            )
        });
        Self::with_store(selected, loaded, cx)
    }

    fn with_store(
        selected: Option<String>,
        loaded: anyhow::Result<Store>,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| TextInput::new("Search…", cx));
        cx.subscribe(&search, |this, input, _: &InputEvent, cx| {
            this.query = input.read(cx).text().to_owned();
            cx.notify();
        })
        .detach();
        let (store, rows, problem) = match loaded {
            Ok(store) => {
                let rows = store.list();
                (Some(Arc::new(Mutex::new(store))), rows, None)
            }
            Err(error) => (None, Vec::new(), Some(error.to_string())),
        };
        let title_input = cx.new(|cx| TextInput::new("Conversation title", cx));
        Self {
            title_input,
            renaming: None,
            store,
            rows,
            selected,
            search,
            query: String::new(),
            show_archived: false,
            connected: false,
            refreshing: false,
            pending: None,
            pending_runtime: None,
            problem,
            busy: false,
            services: Default::default(),
            new_agent: None,
            new_space: None,
            spaces: Vec::new(),
            scope: None,
            active_space: None,
            last_launch_agent: None,
            last_launch_space: None,
            launch_runtime: None,
            focus: cx.focus_handle(),
            focus_terminal: false,
            terminal_view: None,
            terminal_notice: None,
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn selected_row(&self) -> Option<&Conversation> {
        self.selected
            .as_ref()
            .and_then(|id| self.rows.iter().find(|row| &row.id == id && self.in_scope(row)))
    }

    pub fn terminal(&self, cx: &App) -> Option<Entity<terminal::Terminal>> {
        self.terminal_view
            .as_ref()
            .filter(|_| self.connected)
            .map(|view| view.read(cx).terminal().clone())
    }

    pub fn selected_runtime(&self) -> Option<&str> {
        self.selected_row()
            .and_then(|row| row.runtime.as_deref())
            .or(self.launch_runtime.as_deref())
    }

    pub fn set_terminal(
        &mut self,
        view: Option<Entity<terminal_view::TerminalView>>,
        notice: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let view = view.filter(|_| self.connected);
        if self.terminal_view != view || self.terminal_notice != notice {
            self.terminal_view = view;
            self.terminal_notice = notice;
            cx.notify();
        }
    }

    fn clear_terminal(&mut self) {
        self.terminal_view = None;
        self.terminal_notice = None;
    }

    pub fn select_runtime(&mut self, runtime: &str, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter().find(|row| row.runtime.as_deref() == Some(runtime)) {
            self.new_agent = None;
            self.pending_runtime = None;
            self.launch_runtime = None;
            self.new_space = None;
            self.selected = Some(row.id.clone());
            self.focus_terminal = true;
            self.show_archived = row.archived;
            self.query.clear();
            self.search.update(cx, |input, cx| input.clear(cx));
            cx.emit(Event::SelectionChanged);
        } else {
            self.pending_runtime = Some(runtime.to_owned());
        }
        cx.notify();
    }

    pub fn report(&mut self, result: Result<(), String>, cx: &mut Context<Self>) {
        self.busy = false;
        self.problem = result.err();
        cx.notify();
    }

    pub fn disconnected(&mut self, cx: &mut Context<Self>) {
        self.connected = false;
        self.clear_terminal();
        for row in &mut self.rows {
            row.status = Status::Unknown;
        }
        cx.notify();
    }

    pub fn observe(&mut self, observations: Vec<Observation>, cx: &mut Context<Self>) {
        self.connected = true;
        self.pending = Some(observations);
        if self.refreshing {
            return;
        }
        let Some(store) = self.store.clone() else { return };
        self.refreshing = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            loop {
                let Ok(Some((observations, selected))) = this.update(cx, |this, _| {
                    let pending = this.pending.take();
                    if pending.is_none() {
                        this.refreshing = false;
                    }
                    pending.map(|observations| (observations, this.selected.clone()))
                }) else {
                    break;
                };
                let store = store.clone();
                let result = executor
                    .spawn(async move {
                        let mut store = store
                            .lock()
                            .map_err(|_| "Conversation store unavailable".to_owned())?;
                        store
                            .reconcile(observations, chartr_conversations::now_millis())
                            .map_err(|e| e.to_string())?;
                        Ok::<_, String>((
                            store.list(),
                            selected.clone(),
                            selected.map(|id| store.resolve_id(&id)),
                        ))
                    })
                    .await;
                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok((mut rows, expected, resolved)) => {
                            if !this.connected {
                                for row in &mut rows {
                                    row.status = Status::Unknown;
                                }
                            }
                            // Resolve a provisional row without changing a newer user selection.
                            if let (Some(old), Some(new)) = (this.selected.clone(), resolved)
                                && Some(&old) == expected.as_ref()
                                && old != new
                                && !rows.iter().any(|r| r.id == old)
                            {
                                this.selected = Some(new);
                                cx.emit(Event::SelectionChanged);
                            }
                            this.rows = rows;
                            if let Some(runtime) = this.pending_runtime.take() {
                                this.select_runtime(&runtime, cx);
                            }
                        }
                        Err(error) => this.problem = Some(error),
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn archive_selected(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected.clone() else { return };
        let archived = !self.rows.iter().find(|r| r.id == id).is_some_and(|r| r.archived);
        let Some(store) = self.store.clone() else { return };
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let expected = id.clone();
            let result = executor
                .spawn(async move {
                    let mut store =
                        store.lock().map_err(|_| "Conversation store unavailable".to_owned())?;
                    store.archive(&id, archived).map_err(|e| e.to_string())?;
                    Ok::<_, String>(store.list())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(rows) => {
                        this.rows = rows;
                        if this.selected.as_ref() == Some(&expected) {
                            this.selected = None;
                            this.renaming = None;
                            this.clear_terminal();
                            cx.emit(Event::SelectionChanged);
                        }
                    }
                    Err(error) => this.problem = Some(error),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn save_title(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.renaming.take() else { return };
        let title = self.title_input.read(cx).text().to_owned();
        let Some(store) = self.store.clone() else { return };
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    let mut store =
                        store.lock().map_err(|_| "Conversation store unavailable".to_owned())?;
                    store.rename(&id, title).map_err(|e| e.to_string())?;
                    Ok::<_, String>(store.list())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(rows) => this.rows = rows,
                    Err(error) => this.problem = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chrome::sidebar_pane, mode::Mode};
    use chartr_conversations::Provider;
    use gpui::{DragMoveEvent, Modifiers, MouseButton, Window, point, px, size};
    use terminal::{
        TerminalBuilder,
        terminal_settings::{AlternateScroll, CursorShape},
    };
    use ui::prelude::*;
    use util::paths::PathStyle;

    struct InboxHarness {
        inbox: Entity<Conversations>,
        sidebar: sidebar_pane::SidebarPane,
    }

    impl Render for InboxHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let contents = self.inbox.update(cx, |inbox, cx| inbox.render_sidebar(cx));
            h_flex()
                .size_full()
                .child(sidebar_pane::render(self.sidebar.width(), None, contents, cx))
                .child(div().flex_1().min_w_0().h_full().child(self.inbox.clone()))
                .on_drag_move::<crate::chrome::DraggedSidebar>(cx.listener(
                    |this, event: &DragMoveEvent<crate::chrome::DraggedSidebar>, _, cx| {
                        this.sidebar.resize(event.event.position.x / px(1.), Mode::Inbox);
                        cx.notify();
                    },
                ))
        }
    }

    #[gpui::test]
    fn inbox_mounts_and_focuses_the_original_terminal(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            view::init(cx);
        });
        let dir = tempfile::tempdir().unwrap();
        let paths = ProviderPaths {
            codex: dir.path().join("codex"),
            claude: dir.path().join("claude"),
            opencode: dir.path().join("opencode"),
            pi: dir.path().join("pi"),
        };
        let mut store = Store::open(&dir.path().join("history.sqlite"), paths).unwrap();
        store
            .reconcile(
                vec![Observation {
                    runtime: "pane".into(),
                    terminal: "pty".into(),
                    provider: Provider::Codex,
                    native: Some(chartr_conversations::NativeSession {
                        id: "fixture".into(),
                        path: None,
                    }),
                    cwd: None,
                    space: None,
                    title: Some("A terminal session".into()),
                    status: Status::Idle,
                    pid: None,
                }],
                1,
            )
            .unwrap();
        let selected = store.for_runtime("pane").unwrap().to_owned();
        let terminal = cx.new(|cx| {
            TerminalBuilder::new_display_only(
                CursorShape::default(),
                AlternateScroll::On,
                None,
                0,
                cx.background_executor(),
                PathStyle::local(),
            )
            .subscribe(cx)
        });
        let model = terminal.clone();
        let (harness, cx) = cx.add_window_view(move |window: &mut Window, cx| {
            let terminal_view = crate::terminal_host::new_view(model, window, cx);
            let inbox = cx.new(|cx| {
                let mut inbox = Conversations::with_store(Some(selected), Ok(store), cx);
                inbox.connected = true;
                inbox.focus_terminal = true;
                inbox.set_terminal(Some(terminal_view), None, cx);
                inbox
            });
            cx.observe(&inbox, |_, _, cx| cx.notify()).detach();
            InboxHarness { inbox, sidebar: sidebar_pane::SidebarPane::new(320., Mode::Inbox) }
        });
        let inbox = harness.read_with(cx, |harness, _| harness.inbox.clone());
        terminal.update(cx, |terminal, cx| terminal.write_output(b"Inbox terminal content", cx));
        cx.simulate_resize(size(px(1000.), px(600.)));
        cx.run_until_parked();
        inbox.update_in(cx, |inbox, window, cx| {
            assert_eq!(inbox.terminal(cx), Some(terminal.clone()));
            assert!(inbox.focus_handle(cx).is_focused(window));
        });
        let first_width = terminal
            .read_with(cx, |terminal, _| terminal.last_content().terminal_bounds.bounds.size.width);
        assert!(first_width > px(600.) && first_width < px(710.));
        cx.simulate_resize(size(px(1200.), px(600.)));
        cx.run_until_parked();
        let next_width = terminal
            .read_with(cx, |terminal, _| terminal.last_content().terminal_bounds.bounds.size.width);
        assert!(
            next_width > first_width + px(180.),
            "Inbox must resize the existing terminal model"
        );

        // The shared divider must receive drags above the terminal and obey Inbox's minimum.
        let divider = point(px(323.), px(250.));
        let wider = point(px(440.), px(250.));
        cx.simulate_mouse_down(divider, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(wider, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(wider, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(wider, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
        assert_eq!(harness.read_with(cx, |harness, _| harness.sidebar.width()), 440.);
        let narrower_terminal = terminal
            .read_with(cx, |terminal, _| terminal.last_content().terminal_bounds.bounds.size.width);
        assert!(narrower_terminal < next_width - px(100.));
        let divider = point(px(443.), px(250.));
        let narrower = point(px(50.), px(250.));
        cx.simulate_mouse_down(divider, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(narrower, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(narrower, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(narrower, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            harness.read_with(cx, |harness, _| harness.sidebar.width()),
            sidebar_pane::INBOX_MIN_WIDTH,
        );

        // Search still routes from the terminal to the separately mounted sidebar.
        inbox.update_in(cx, |inbox, window, cx| inbox.focus_handle(cx).focus(window, cx));
        cx.simulate_keystrokes(if cfg!(target_os = "macos") { "cmd-f" } else { "ctrl-f" });
        inbox.update_in(cx, |inbox, window, cx| {
            assert!(inbox.search.focus_handle(cx).is_focused(window));
        });
        inbox.update(cx, |inbox, cx| inbox.disconnected(cx));
        cx.run_until_parked();
        assert!(inbox.read_with(cx, |inbox, cx| inbox.terminal(cx)).is_none());
    }
}
