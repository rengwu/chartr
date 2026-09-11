use super::scope::choice_label;
use super::*;
use chartr_plugin::services::{AGENT_SERVICE, Agents, Services};
use gpui::{Anchor, AnyElement, SharedString, div, px};
use ui::{Button, ButtonStyle, IconButton, Tooltip, prelude::*};

impl Conversations {
    pub fn set_agent_services(&mut self, services: Services) {
        self.services = services;
    }

    pub(super) fn agent_picker(&self, prominent: bool, cx: &Context<Self>) -> AnyElement {
        let weak = cx.weak_entity();
        let popup = ui::PopoverMenu::new(if prominent {
            "empty-conversation-launcher"
        } else {
            "conversation-launcher"
        });
        let popup = if prominent {
            popup.trigger(
                Button::new("empty-new-conversation", "New conversation")
                    .style(ButtonStyle::Outlined)
                    .disabled(self.busy),
            )
        } else {
            popup.trigger(
                IconButton::new("new-conversation", IconName::Plus)
                    .icon_size(IconSize::Small)
                    .disabled(self.busy)
                    .aria_label("New conversation")
                    .tooltip(Tooltip::text("New conversation")),
            )
        };
        popup
            .anchor(Anchor::TopLeft)
            .menu(move |_, cx| {
                let owner = weak.upgrade()?;
                Some(cx.new(|cx| LaunchPanel::new(owner, cx)))
            })
            .into_any_element()
    }

    pub(super) fn new_editor_key(&self, name: &str) -> String {
        format!("{}\0{name}", self.new_space.as_deref().unwrap_or_default())
    }

    fn begin_conversation(
        &mut self,
        name: String,
        space: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if self.busy {
            return Err("Another conversation is starting.".into());
        }
        if !self.connected {
            return Err("Wait for the terminal service to connect.".into());
        }
        if !self.spaces.iter().any(|candidate| candidate.key == space) {
            return Err("The selected space is no longer available.".into());
        }
        if self.scope.as_ref().is_some_and(|scope| scope != &space) {
            return Err("The current space changed. Open the launcher again.".into());
        }
        let names = self
            .services
            .get::<Agents>(AGENT_SERVICE)
            .ok_or("Enable Agent and register an agent first.")?
            .list(cx)?;
        if !names.contains(&name) {
            return Err("The selected agent is no longer registered.".into());
        }
        self.last_launch_agent = Some(name.clone());
        self.last_launch_space = Some(space.clone());
        self.new_agent = Some(name.clone());
        self.new_space = Some(space);
        self.launch_runtime = None;
        self.pending_runtime = None;
        self.selected = None;
        self.problem = None;
        let editor = self.new_editor(&name, window, cx);
        editor.focus_handle(cx).focus(window, cx);
        cx.notify();
        Ok(())
    }

    fn new_editor(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<Editor> {
        if let Some(editor) = self.new_editors.get(&self.new_editor_key(name)) {
            return editor.clone();
        }
        let editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(3, 10, window, cx);
            editor.set_soft_wrap();
            editor.set_autoindent(false);
            editor.set_use_autoclose(false);
            editor.set_show_wrap_guides(false, cx);
            editor.set_show_indent_guides(false, cx);
            editor.set_placeholder_text(&format!("Message {name}…"), window, cx);
            editor
        });
        cx.observe(&editor, |_, _, cx| cx.notify()).detach();
        self.new_editors.insert(self.new_editor_key(name), editor.clone());
        editor
    }

    pub(super) fn new_conversation(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editor = self.new_editor(name, window, cx);
        let waiting = self.busy || self.launch_runtime.is_some();
        editor.update(cx, |editor, _| editor.set_read_only(waiting));
        let colors = cx.theme().colors().clone();
        v_flex().flex_1().min_w_0().h_full()
            .child(h_flex().h(px(42.)).px_4().gap_2().border_b_1().border_color(colors.border)
                .child(Icon::new(IconName::Chat).size(IconSize::Small).color(Color::Muted))
                .child(Label::new(format!("New {name} conversation")))
                .child(div().flex_1())
                .when_some(self.new_space.as_ref().and_then(|key| self.spaces.iter().find(|s| &s.key == key)).cloned(), |header, space|
                    header.child(Label::new(choice_label(&space, &self.spaces)).size(LabelSize::Small).color(Color::Muted))))
            .child(v_flex().flex_1().min_h_0().px_4().pt_4().gap_2()
                .child(Label::new(if self.busy { format!("Starting {name}…") }
                    else if self.launch_runtime.is_some() { format!("Waiting for {name} to connect…") }
                    else { "What would you like to work on?".into() }).color(Color::Muted))
                .when_some(self.launch_runtime.clone(), |view, runtime| view
                    .child(Label::new("If it needs setup or has no chat adapter, continue in its terminal.").size(LabelSize::Small).color(Color::Muted))
                    .child(Button::new("new-agent-terminal", "Open terminal")
                        .on_click(cx.listener(move |_, _, _, cx| cx.emit(Event::ShowTerminal(runtime.clone())))))))
            .child(div().px_4().pb_4().pt_2().child(v_flex()
                .key_context("ConversationComposer").px_3().pt_3().pb_2().gap_3()
                .rounded_lg().border_1().border_color(colors.border).bg(colors.element_background)
                .child(editor.clone())
                .child(h_flex().gap_2()
                    .child(Label::new(SharedString::from(name.to_owned())).size(LabelSize::Small).color(Color::Muted))
                    .child(div().flex_1())
                    .child(IconButton::new("start-agent-conversation", IconName::ArrowUp)
                        .icon_size(IconSize::Small).aria_label("Start conversation")
                        .disabled(waiting || !self.connected || editor.read(cx).text(cx).trim().is_empty())
                        .tooltip(Tooltip::text("Start conversation · Enter"))
                        .on_click(cx.listener(|this, _, window, cx| this.send(window, cx)))))))
            .into_any_element()
    }

    pub(super) fn launch_from_composer(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if self.busy || !self.connected || self.launch_runtime.is_some() {
            return;
        }
        let Some(name) = self.new_agent.clone() else { return };
        let Some(editor) = self.new_editors.get(&self.new_editor_key(&name)) else { return };
        let prompt = editor.read(cx).text(cx);
        if prompt.trim().is_empty() {
            return;
        }
        self.busy = true;
        self.problem = None;
        let Some(space) = self.new_space.clone() else {
            self.busy = false;
            return;
        };
        cx.emit(Event::LaunchAgent { name, prompt, space });
        cx.notify();
    }

    pub fn launch_allocated(&mut self, name: &str, runtime: &str, cx: &mut Context<Self>) {
        if self.new_agent.as_deref() == Some(name) {
            self.launch_runtime = Some(runtime.into());
            cx.notify();
        }
    }

    pub fn launch_started(&mut self, name: &str, runtime: &str, cx: &mut Context<Self>) {
        if self.new_agent.as_deref() == Some(name) {
            self.select_runtime(runtime, cx);
        }
    }
}

/// A compact form popover. Its two bounded submenus use the same stock picker
/// controls and never flatten the space × profile combinations into one list.
struct LaunchPanel {
    owner: Entity<Conversations>,
    space: Option<String>,
    agent: Option<String>,
    scope: Option<String>,
    focus: FocusHandle,
    space_menu: ui::PopoverMenuHandle<ui::ContextMenu>,
    agent_menu: ui::PopoverMenuHandle<ui::ContextMenu>,
    problem: Option<String>,
}

impl EventEmitter<gpui::DismissEvent> for LaunchPanel {}
impl Focusable for LaunchPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl LaunchPanel {
    fn new(owner: Entity<Conversations>, cx: &mut Context<Self>) -> Self {
        let view = owner.read(cx);
        let names = view
            .services
            .get::<Agents>(AGENT_SERVICE)
            .and_then(|s| s.list(cx).ok())
            .unwrap_or_default();
        let space = view
            .scope
            .clone()
            .or_else(|| {
                view.last_launch_space
                    .clone()
                    .filter(|key| view.spaces.iter().any(|s| &s.key == key))
            })
            .or_else(|| view.active_space.clone());
        let agent = view
            .last_launch_agent
            .clone()
            .filter(|name| names.contains(name))
            .or_else(|| names.first().cloned());
        Self {
            scope: view.scope.clone(),
            owner,
            space,
            agent,
            focus: cx.focus_handle(),
            space_menu: Default::default(),
            agent_menu: Default::default(),
            problem: None,
        }
    }

    fn launch(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.scope != self.owner.read(cx).scope {
            self.problem = Some("The current space changed. Open the launcher again.".into());
            cx.notify();
            return;
        }
        let (Some(space), Some(agent)) = (self.space.clone(), self.agent.clone()) else { return };
        let result =
            self.owner.update(cx, |owner, cx| owner.begin_conversation(agent, space, window, cx));
        match result {
            Ok(()) => cx.emit(gpui::DismissEvent),
            Err(error) => {
                self.problem = Some(error);
                cx.notify();
            }
        }
    }
}

impl Render for LaunchPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let choose_space = self.scope.is_none();
        let owner = self.owner.read(cx);
        let spaces = owner.spaces.clone();
        let names = owner
            .services
            .get::<Agents>(AGENT_SERVICE)
            .ok_or_else(|| "Enable Agent to launch a conversation.".to_owned())
            .and_then(|service| service.list(cx));
        let connected = owner.connected && !owner.busy;
        let space_valid =
            self.space.as_ref().is_some_and(|key| spaces.iter().any(|s| &s.key == key));
        let agent_valid = self
            .agent
            .as_ref()
            .is_some_and(|name| names.as_ref().is_ok_and(|names| names.contains(name)));
        let selected_space = self.space.clone();
        let selected_agent = self.agent.clone();
        let selected_label = spaces
            .iter()
            .find(|s| Some(&s.key) == self.space.as_ref())
            .map(|s| choice_label(s, &spaces))
            .unwrap_or_else(|| "Choose a space".into());
        let weak = cx.weak_entity();
        let space_menu = ui::ContextMenu::build(window, cx, move |mut menu, _, _| {
            menu = menu.max_height(px(224.).into()).fixed_width(px(276.).into());
            for space in &spaces {
                let key = space.key.clone();
                let set = weak.clone();
                menu = menu.toggleable_entry(
                    choice_label(space, &spaces),
                    Some(&key) == selected_space.as_ref(),
                    ui::IconPosition::End,
                    None,
                    move |_, cx| {
                        let _ = set.update(cx, |this, cx| {
                            this.space = Some(key.clone());
                            this.problem = None;
                            cx.notify();
                        });
                    },
                );
            }
            menu
        });
        let problem =
            self.problem.clone().or_else(|| names.as_ref().err().cloned()).or_else(|| {
                names
                    .as_ref()
                    .ok()
                    .filter(|names| names.is_empty())
                    .map(|_| "Register an agent to get started.".into())
            });
        let weak = cx.weak_entity();
        let agent_menu = ui::ContextMenu::build(window, cx, move |mut menu, _, _| {
            menu = menu.max_height(px(224.).into()).fixed_width(px(276.).into());
            for name in names.unwrap_or_default() {
                let set = weak.clone();
                let value = name.clone();
                menu = menu.toggleable_entry(
                    name,
                    Some(&value) == selected_agent.as_ref(),
                    ui::IconPosition::End,
                    None,
                    move |_, cx| {
                        let _ = set.update(cx, |this, cx| {
                            this.agent = Some(value.clone());
                            this.problem = None;
                            cx.notify();
                        });
                    },
                );
            }
            menu
        });
        let space_picker = ui::DropdownMenu::new("launch-space", selected_label, space_menu)
            .style(ui::DropdownStyle::Outlined)
            .trigger_size(crate::components::FORM_CONTROL_SIZE)
            .full_width(true)
            .attach(Anchor::BottomLeft)
            .aria_label("Conversation space")
            .tab_index(0)
            .handle(self.space_menu.clone());
        let agent_picker = ui::DropdownMenu::new(
            "launch-agent",
            self.agent.clone().unwrap_or_else(|| "Choose an agent".into()),
            agent_menu,
        )
        .style(ui::DropdownStyle::Outlined)
        .trigger_size(crate::components::FORM_CONTROL_SIZE)
        .full_width(true)
        .attach(Anchor::BottomLeft)
        .aria_label("Conversation agent")
        .tab_index(if choose_space { 1 } else { 0 })
        .handle(self.agent_menu.clone());
        let colors = cx.theme().colors();
        v_flex()
            .id("conversation-launch-panel")
            .key_context("ConversationLaunchPanel")
            .track_focus(&self.focus)
            .w(px(320.))
            .p_3()
            .gap_3()
            .rounded_lg()
            .shadow_lg()
            .border_1()
            .border_color(colors.border)
            .bg(colors.elevated_surface_background)
            .on_action(cx.listener(|this, _: &super::view::Submit, window, cx| {
                cx.stop_propagation();
                this.launch(window, cx);
            }))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if !this.space_menu.is_deployed() && !this.agent_menu.is_deployed() {
                    cx.emit(gpui::DismissEvent);
                }
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if this.space_menu.is_deployed() || this.agent_menu.is_deployed() {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "escape" => {
                        cx.stop_propagation();
                        cx.emit(gpui::DismissEvent);
                    }
                    "enter" => {
                        cx.stop_propagation();
                        this.launch(window, cx);
                    }
                    _ => {}
                }
            }))
            .child(Label::new("New conversation"))
            .when(choose_space, |panel| {
                panel.child(
                    v_flex()
                        .gap_1()
                        .child(Label::new("Space").size(LabelSize::Small).color(Color::Muted))
                        .child(space_picker),
                )
            })
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("Agent").size(LabelSize::Small).color(Color::Muted))
                    .child(agent_picker),
            )
            .when_some(problem, |panel, error| {
                panel.child(Label::new(error).size(LabelSize::Small).color(Color::Error))
            })
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .child(
                        Button::new("manage-conversation-agents", "Manage agents…")
                            .size(ui::ButtonSize::Compact)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.owner.update(cx, |_, cx| cx.emit(Event::ManageAgents));
                                cx.emit(gpui::DismissEvent);
                            })),
                    )
                    .child(
                        Button::new("launch-conversation", "Launch")
                            .style(ButtonStyle::Filled)
                            .disabled(!connected || !space_valid || !agent_valid)
                            .on_click(cx.listener(|this, _, window, cx| this.launch(window, cx))),
                    ),
            )
    }
}
