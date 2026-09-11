//! Inbox: durable agent history with the original session terminal.
mod integrations;
mod launcher;
mod scope;
pub use scope::SpaceChoice;
mod view;
pub use view::init;

use crate::text_input::{InputEvent, TextInput};
use chartr_conversations::{Conversation, Observation, Provider, ProviderPaths, Status, Store};
use gpui::{App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, Window};
use std::sync::{Arc, Mutex};

pub enum Event {
    LaunchAgent { name: String, space: String },
    ManageAgents,
    EnableIntegration(Provider),
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
    integrations: integrations::Integrations,
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
        let search = cx.new(|cx| TextInput::new("Search conversations…", cx));
        cx.subscribe(&search, |this, input, _: &InputEvent, cx| {
            this.query = input.read(cx).text().to_owned();
            cx.notify();
        })
        .detach();
        let loaded = crate::persistence::state_file().and_then(|path| {
            Store::open(
                &path.with_file_name("conversations.sqlite"),
                ProviderPaths::from_environment(),
            )
        });
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
            integrations: Default::default(),
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

    pub fn needs_integration_check(&mut self) -> bool {
        self.integrations.begin_check()
    }

    pub fn integrations_checked(
        &mut self,
        result: Result<Vec<chartr_herdr::protocol::IntegrationInfo>, String>,
        cx: &mut Context<Self>,
    ) {
        self.integrations.checked(result);
        cx.notify();
    }

    pub fn integration_installed(
        &mut self,
        provider: Provider,
        result: Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        self.busy = false;
        self.integrations.installed(provider, result);
        cx.notify();
    }

    pub fn disconnected(&mut self, cx: &mut Context<Self>) {
        self.connected = false;
        self.clear_terminal();
        self.integrations.reset();
        for row in &mut self.rows {
            row.endpoint = None;
            row.requests.clear();
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
                                    row.endpoint = None;
                                    row.requests.clear();
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
                        this.selected = None;
                        cx.emit(Event::SelectionChanged);
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
