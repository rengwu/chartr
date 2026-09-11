//! A conversation surface which observes host-owned sessions even while hidden.
mod input;
mod integrations;
mod launcher;
mod markdown;
mod scope;
pub use scope::SpaceChoice;
mod view;
pub use view::init;

use crate::text_input::{InputEvent, TextInput};
use chartr_conversations::{Conversation, Observation, Provider, ProviderPaths, Status, Store};
use editor::Editor;
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, ScrollHandle, Task,
    Window,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

pub enum Event {
    ShowTerminal(String),
    LaunchAgent { name: String, prompt: String, space: String },
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
    editors: HashMap<String, Entity<Editor>>,
    scrolls: HashMap<String, ScrollHandle>,
    expanded_tools: HashSet<String>,
    drafts: HashMap<String, String>,
    draft_version: u64,
    connected: bool,
    draft_task: Option<Task<()>>,
    refreshing: bool,
    pending: Option<Vec<Observation>>,
    pending_runtime: Option<String>,
    sending: HashSet<String>,
    pending_deliveries: HashMap<String, chartr_conversations::Delivery>,
    delivered_drafts: HashMap<String, String>,
    problem: Option<String>,
    busy: bool,
    integrations: integrations::Integrations,
    terminal_client: Option<chartr_herdr::control::Client>,
    services: chartr_plugin::services::Services,
    new_agent: Option<String>,
    new_space: Option<String>,
    spaces: Vec<SpaceChoice>,
    scope: Option<String>,
    active_space: Option<String>,
    last_launch_agent: Option<String>,
    last_launch_space: Option<String>,
    new_editors: HashMap<String, Entity<Editor>>,
    launch_runtime: Option<String>,
    focus: FocusHandle,
    focus_composer: bool,
}

impl EventEmitter<Event> for Conversations {}
impl Focusable for Conversations {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if let Some(editor) = self
            .new_agent
            .as_ref()
            .and_then(|name| self.new_editors.get(&self.new_editor_key(name)))
        {
            return editor.focus_handle(cx);
        }
        self.selected
            .as_ref()
            .and_then(|id| self.editors.get(id))
            .map(|editor| editor.focus_handle(cx))
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
        cx.on_release(|this, _| this.flush_drafts()).detach();
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
            editors: HashMap::new(),
            scrolls: HashMap::new(),
            expanded_tools: HashSet::new(),
            drafts: HashMap::new(),
            draft_version: 0,
            connected: false,
            draft_task: None,
            refreshing: false,
            pending: None,
            pending_runtime: None,
            sending: HashSet::new(),
            pending_deliveries: HashMap::new(),
            delivered_drafts: HashMap::new(),
            problem,
            busy: false,
            integrations: Default::default(),
            terminal_client: None,
            services: Default::default(),
            new_agent: None,
            new_space: None,
            spaces: Vec::new(),
            scope: None,
            active_space: None,
            last_launch_agent: None,
            last_launch_space: None,
            new_editors: HashMap::new(),
            launch_runtime: None,
            focus: cx.focus_handle(),
            focus_composer: false,
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn set_terminal_client(&mut self, client: Option<chartr_herdr::control::Client>) {
        self.terminal_client = client;
    }

    pub fn select_runtime(&mut self, runtime: &str, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter().find(|row| row.runtime.as_deref() == Some(runtime)) {
            if let Some(name) = self.new_agent.take() {
                self.new_editors.remove(&self.new_editor_key(&name));
            }
            self.launch_runtime = None;
            self.new_space = None;
            self.selected = Some(row.id.clone());
            self.focus_composer = true;
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
                                if let Some(editor) = this.editors.remove(&old) {
                                    this.editors.insert(new.clone(), editor);
                                }
                                if let Some(draft) = this.drafts.remove(&old) {
                                    this.drafts.insert(new.clone(), draft);
                                }
                                this.selected = Some(new);
                                cx.emit(Event::SelectionChanged);
                            }
                            for row in &rows {
                                let previous = this
                                    .pending_deliveries
                                    .get(&row.id)
                                    .or_else(|| {
                                        this.rows
                                            .iter()
                                            .find(|old| old.id == row.id)
                                            .and_then(|old| old.delivery.as_ref())
                                    })
                                    .cloned();
                                if let Some(previous) = previous {
                                    if row.messages.iter().any(|m| previous.matches(m)) {
                                        this.pending_deliveries.remove(&row.id);
                                        this.delivered_drafts.insert(row.id.clone(), previous.text);
                                    }
                                }
                                if let Some(scroll) = this.scrolls.get(&row.id) {
                                    if scroll.max_offset().y + scroll.offset().y < gpui::px(64.) {
                                        scroll.scroll_to_bottom();
                                    }
                                }
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

    fn editor(
        &mut self,
        row: &Conversation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<Editor> {
        if let Some(editor) = self.editors.get(&row.id) {
            return editor.clone();
        }
        let initial = self.drafts.get(&row.id).unwrap_or(&row.draft).clone();
        let editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(3, 10, window, cx);
            editor.set_soft_wrap();
            editor.set_autoindent(false);
            editor.set_use_autoclose(false);
            editor.set_show_wrap_guides(false, cx);
            editor.set_show_indent_guides(false, cx);
            editor.set_placeholder_text(&format!("Message {}…", row.provider.name()), window, cx);
            editor.set_text(initial, window, cx);
            editor
        });
        let id = row.id.clone();
        cx.subscribe(&editor, move |this, editor, event: &editor::EditorEvent, cx| {
            if matches!(event, editor::EditorEvent::BufferEdited) {
                this.draft_version += 1;
                this.drafts.insert(id.clone(), editor.read(cx).text(cx));
                this.schedule_drafts(cx);
                cx.notify();
            }
        })
        .detach();
        self.editors.insert(row.id.clone(), editor.clone());
        editor
    }

    fn schedule_drafts(&mut self, cx: &mut Context<Self>) {
        if self.draft_task.is_some() {
            return;
        }
        let Some(store) = self.store.clone() else { return };
        let executor = cx.background_executor().clone();
        self.draft_task = Some(cx.spawn(async move |this, cx| {
            loop {
                executor.timer(Duration::from_millis(250)).await;
                let Ok((drafts, version)) = this.update(cx, |this, _| {
                    if this.drafts.is_empty() {
                        this.draft_task = None;
                    }
                    (this.drafts.clone(), this.draft_version)
                }) else {
                    break;
                };
                if drafts.is_empty() {
                    break;
                }
                let saved = drafts.clone();
                let store = store.clone();
                let result = executor
                    .spawn(async move {
                        let mut store = store
                            .lock()
                            .map_err(|_| "Conversation store unavailable".to_owned())?;
                        for (id, draft) in drafts {
                            store
                                .set_draft_version(&id, draft, version)
                                .map_err(|e| e.to_string())?;
                        }
                        Ok::<_, String>(())
                    })
                    .await;
                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(()) => {
                            for (id, text) in saved {
                                if this.drafts.get(&id) == Some(&text) {
                                    this.drafts.remove(&id);
                                }
                            }
                        }
                        Err(error) => this.problem = Some(error),
                    }
                    cx.notify();
                });
            }
        }));
    }

    pub fn flush_drafts(&mut self) {
        if let Some(store) = &self.store
            && let Ok(mut store) = store.lock()
        {
            for (id, draft) in std::mem::take(&mut self.drafts) {
                if let Err(error) = store.set_draft_version(&id, draft, self.draft_version) {
                    eprintln!("Could not save conversation draft: {error}");
                }
            }
        }
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

    fn reset_delivery(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(store) = self.store.clone() else { return };
        if let Some(delivery) =
            self.rows.iter().find(|row| row.id == id).and_then(|row| row.delivery.as_ref())
            && delivery.prior_user_messages.is_some()
            && let Some(editor) = self.editors.get(&id).cloned()
        {
            let draft = editor.read(cx).text(cx);
            if draft != delivery.text {
                let restored = if draft.is_empty() {
                    delivery.text.clone()
                } else {
                    format!("{}\n\n{draft}", delivery.text)
                };
                editor.update(cx, |editor, cx| editor.set_text(restored, window, cx));
            }
        }
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let target = id.clone();
            let result = executor
                .spawn(async move {
                    store
                        .lock()
                        .map_err(|_| "Conversation store unavailable".to_owned())?
                        .confirm_delivery(&target)
                        .map_err(|e| e.to_string())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.problem = Some(error);
                } else {
                    this.pending_deliveries.remove(&id);
                    if let Some(row) = this.rows.iter_mut().find(|r| r.id == id) {
                        row.delivery = None;
                    }
                    this.problem = None;
                }
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

    fn control(
        &mut self,
        row: &Conversation,
        request: Option<(chartr_conversations::Request, String)>,
        cx: &mut Context<Self>,
    ) {
        if !self.connected || self.sending.contains(&row.id) {
            return;
        }
        let Some(store) = self.store.clone() else { return };
        let id = row.id.clone();
        self.sending.insert(id.clone());
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let target = id.clone();
            let result = executor
                .spawn(async move {
                    let (client, native) = store
                        .lock()
                        .map_err(|_| "Conversation store unavailable".to_owned())?
                        .live_client(&target)
                        .map_err(|e| e.to_string())?;
                    match request {
                        Some((request, answer)) => client.answer(&native, &request, &answer),
                        None => client.interrupt(&native),
                    }
                    .map_err(|e| e.to_string())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.sending.remove(&id);
                this.problem = result.err();
                cx.notify();
            });
        })
        .detach();
    }

    fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.new_agent.is_some() {
            self.launch_from_composer(window, cx);
            return;
        }
        let Some(row) =
            self.selected.as_ref().and_then(|id| self.rows.iter().find(|r| &r.id == id)).cloned()
        else {
            return;
        };
        if !self.connected
            || !row.can_send()
            || self.sending.contains(&row.id)
            || self.pending_deliveries.contains_key(&row.id)
        {
            return;
        }
        let Some(editor) = self.editors.get(&row.id).cloned() else { return };
        let text = editor.read(cx).text(cx);
        if text.trim().is_empty() {
            return;
        }
        let Some(store) = self.store.clone() else { return };
        let id = row.id.clone();
        let message_id = chartr_conversations::OpenCode::message_id();
        let terminal_client = self.terminal_client.clone();
        self.sending.insert(id.clone());
        editor.update(cx, |editor, cx| editor.set_text("", window, cx));
        self.problem = None;
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |this, cx| {
            let target = id.clone();
            let submitted = text.clone();
            let sent_id = message_id.clone();
            let (result, receipt) = executor.spawn(async move {
                input::send(&store, &target, terminal_client, sent_id, submitted)
            }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.sending.remove(&id);
                let confirmed = receipt.as_ref().is_some_and(|delivery| {
                    this.rows.iter().find(|r| r.id == id)
                        .is_some_and(|row| row.messages.iter().any(|m| delivery.matches(m)))
                });
                if !confirmed && let Some(receipt) = receipt {
                    if let Some(row) = this.rows.iter_mut().find(|r| r.id == id) {
                        row.delivery = Some(receipt.clone());
                    }
                    this.pending_deliveries.insert(id.clone(), receipt);
                }
                match result {
                    Ok(()) => {
                        // New text typed after submission belongs to the next draft.
                        if let Some(row) = this.rows.iter_mut().find(|r| r.id == id) {
                            row.status = Status::Working;
                        }
                    }
                    Err(_) if confirmed => {
                        // A native transcript receipt can beat a transport timeout.
                        this.problem = None;
                    }
                    Err((uncertain, error)) => {
                        let next = editor.read(cx).text(cx);
                        let restored = if next.is_empty() { text.clone() } else { format!("{text}\n\n{next}") };
                        editor.update(cx, |editor, cx| editor.set_text(restored, window, cx));
                        this.problem = Some(if uncertain { format!("Delivery could not be confirmed: {error}. Check the conversation in terminal before sending again.") } else { error });
                    }
                }
                cx.notify();
            });
        }).detach();
        cx.notify();
    }
}
