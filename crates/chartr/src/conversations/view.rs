use super::integrations::Setup;
use super::*;
use chartr_conversations::{Message, Request, Role};
use gpui::{AnyElement, ClipboardItem, MouseButton, SharedString, div, px};
use ui::{Button, ButtonSize, ButtonStyle, IconButton, Tooltip, prelude::*};

gpui::actions!(chartr_conversation, [Submit, FocusSearch]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        gpui::KeyBinding::new("enter", Submit, Some("ConversationComposer > Editor")),
        gpui::KeyBinding::new(
            "shift-enter",
            editor::actions::Newline,
            Some("ConversationComposer > Editor"),
        ),
        gpui::KeyBinding::new(
            if cfg!(target_os = "macos") { "cmd-enter" } else { "ctrl-enter" },
            Submit,
            Some("Conversations"),
        ),
        gpui::KeyBinding::new(
            if cfg!(target_os = "macos") { "cmd-f" } else { "ctrl-f" },
            FocusSearch,
            Some("Conversations"),
        ),
    ]);
}

impl Render for Conversations {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        for (id, text) in std::mem::take(&mut self.delivered_drafts) {
            if let Some(editor) = self.editors.get(&id) {
                if editor.read(cx).text(cx) == text {
                    editor.update(cx, |editor, cx| editor.set_text("", window, cx));
                }
            }
        }
        let colors = cx.theme().colors().clone();
        let selected = self
            .selected
            .as_ref()
            .and_then(|id| self.rows.iter().find(|r| &r.id == id && self.in_scope(r)))
            .cloned();
        let rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| {
                self.in_scope(row)
                    && row.archived == self.show_archived
                    && (row.matches(&self.query)
                        || self
                            .space_label(row)
                            .to_lowercase()
                            .contains(&self.query.trim().to_lowercase()))
            })
            .cloned()
            .collect();
        let sidebar = v_flex()
            .id("conversation-history")
            .w(px(296.))
            .min_w(px(220.))
            .flex_none()
            .h_full()
            .bg(colors.panel_background)
            .border_r_1()
            .border_color(colors.border)
            .child(
                h_flex()
                    .h(px(42.))
                    .px_3()
                    .gap_2()
                    .flex_none()
                    .child(
                        Icon::new(IconName::HistoryRerun).size(IconSize::Small).color(Color::Muted),
                    )
                    .child(Label::new("History").size(LabelSize::Default))
                    .child(div().flex_1())
                    .child(
                        IconButton::new("history-archive", IconName::Archive)
                            .icon_size(IconSize::Small)
                            .icon_color(if self.show_archived {
                                Color::Default
                            } else {
                                Color::Muted
                            })
                            .tooltip(Tooltip::text(if self.show_archived {
                                "Show recent conversations"
                            } else {
                                "Show archived conversations"
                            }))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_archived = !this.show_archived;
                                cx.notify();
                            })),
                    )
                    .child(self.agent_picker(false, cx)),
            )
            .child(div().px_3().pb_3().pt_1().child(crate::components::input_field(
                "history-search",
                self.search.clone(),
                cx,
            )))
            .child(
                h_flex().px_3().h(px(28.)).child(
                    Label::new(if self.show_archived { "Archived" } else { "Recent" })
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .child(
                v_flex()
                    .id("history-rows")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(rows.iter().map(|row| {
                        let id = row.id.clone();
                        let title = row.display_title().to_owned();
                        let space_label = self.space_label(row);
                        let detail = format!(
                            "{} · {}",
                            row.provider.name(),
                            row.cwd
                                .as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "Free sessions".to_owned())
                        );
                        let is_selected = self.selected.as_deref() == Some(&id);
                        h_flex()
                            .id(SharedString::from(format!("history-{id}")))
                            .h(px(46.))
                            .px_3()
                            .gap_2()
                            .w_full()
                            .min_w_0()
                            .flex_none()
                            .cursor_pointer()
                            .bg(if is_selected {
                                colors.element_selected
                            } else {
                                colors.panel_background
                            })
                            .hover(|row| row.bg(colors.element_hover))
                            .tooltip(Tooltip::text(detail))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, cx| {
                                    this.new_agent = None;
                                    this.new_space = None;
                                    this.launch_runtime = None;
                                    this.selected = Some(id.clone());
                                    this.focus_composer = true;
                                    this.pending_runtime = None;
                                    this.problem = None;
                                    this.focus.focus(window, cx);
                                    cx.emit(Event::SelectionChanged);
                                    cx.notify();
                                }),
                            )
                            .child(
                                Icon::new(IconName::Chat).size(IconSize::Small).color(Color::Muted),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Label::new(title).size(LabelSize::Default).truncate())
                                    .child(
                                        Label::new(space_label)
                                            .size(LabelSize::Small)
                                            .color(Color::Muted)
                                            .truncate(),
                                    ),
                            )
                            .when(row.runtime.is_some(), |item| {
                                item.child(div().size(px(5.)).rounded_full().bg(match row.status {
                                    Status::Working => cx.theme().status().info,
                                    Status::Waiting => cx.theme().status().warning,
                                    _ => cx.theme().status().success,
                                }))
                            })
                            .child(
                                Label::new(relative_time(row.updated))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                    }))
                    .when(rows.is_empty(), |list| {
                        list.child(div().px_3().py_4().text_color(colors.text_muted).child(
                            if self.query.is_empty() {
                                "Conversations will appear here as you use your agents."
                            } else {
                                "No matching conversations."
                            },
                        ))
                    }),
            );

        let content = if let Some(name) = self.new_agent.clone() {
            self.new_conversation(&name, window, cx)
        } else if let Some(row) = selected {
            self.conversation(&row, window, cx)
        } else {
            v_flex().flex_1().size_full().items_center().justify_center().gap_3()
                .child(Icon::new(IconName::Chat).size(IconSize::Medium).color(Color::Muted))
                .child(Label::new("Pick up a conversation").size(LabelSize::Large))
                .child(div().max_w(px(380.)).text_center().text_color(colors.text_muted)
                    .child("Choose a thread from History, or start a new conversation. Your terminals keep running."))
                .child(self.agent_picker(true, cx))
                .into_any_element()
        };

        v_flex()
            .id("conversations-mode")
            .key_context("Conversations")
            .track_focus(&self.focus)
            .size_full()
            .min_h_0()
            .text_size(rems(1.))
            .bg(colors.editor_background)
            .on_action(cx.listener(|this, _: &Submit, window, cx| {
                cx.stop_propagation();
                this.send(window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                cx.stop_propagation();
                this.search.focus_handle(cx).focus(window, cx);
            }))
            .child(h_flex().flex_1().min_h_0().w_full().child(sidebar).child(content))
            .when_some(self.problem.clone(), |view, problem| {
                view.child(
                    h_flex()
                        .px_3()
                        .py_2()
                        .gap_2()
                        .border_t_1()
                        .border_color(colors.border)
                        .bg(colors.panel_background)
                        .child(div().flex_1().text_size(rems(0.875)).child(problem))
                        .child(
                            IconButton::new("dismiss-conversation-error", IconName::Close)
                                .icon_size(IconSize::Small)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.problem = None;
                                    cx.notify();
                                })),
                        ),
                )
            })
    }
}

impl Conversations {
    fn conversation(
        &mut self,
        row: &Conversation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = cx.theme().colors().clone();
        let editor = self.editor(row, window, cx);
        if self.focus_composer {
            self.focus_composer = false;
            editor.focus_handle(cx).focus(window, cx);
        }
        let disabled = !self.connected
            || !row.can_send()
            || self.sending.contains(&row.id)
            || self.pending_deliveries.contains_key(&row.id);
        let scroll = self
            .scrolls
            .entry(row.id.clone())
            .or_insert_with(|| {
                let handle = ScrollHandle::new();
                handle.scroll_to_bottom();
                handle
            })
            .clone();
        let terminal = row.runtime.clone();
        let provider = row.provider;
        let setup = self.integrations.get(provider);
        let context = self.space_label(row);
        let messages = row
            .messages
            .iter()
            .map(|message| self.message(message, row, window, cx))
            .collect::<Vec<_>>();
        let status_text = if !self.connected {
            "Reconnecting to terminal service…"
        } else if row.delivery.is_some() {
            "Checking the last delivery. Continue in terminal until it is confirmed."
        } else if row.runtime.is_none() {
            "This conversation has ended. Its history is kept here."
        } else if row.native.is_none() {
            "Waiting for the agent's conversation identity."
        } else if row.provider.transport() == chartr_agent::MessageTransport::TerminalOnly
            || (row.provider.transport() == chartr_agent::MessageTransport::OpenCodeApi
                && row.endpoint.is_none())
        {
            if row.messages.is_empty() {
                "This agent needs terminal mode."
            } else {
                "Continue in terminal to reply."
            }
        } else if row.status == Status::Waiting {
            "Waiting for your response."
        } else if row.status == Status::Working {
            "Working…"
        } else {
            ""
        };

        v_flex()
            .id("selected-conversation")
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                h_flex()
                    .h(px(42.))
                    .px_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(colors.border)
                    .flex_none()
                    .child(Icon::new(IconName::Chat).size(IconSize::Small).color(Color::Muted))
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(row.display_title().to_owned())
                                .size(LabelSize::Default)
                                .truncate(),
                        ),
                    )
                    .child(Label::new(context).size(LabelSize::Small).color(Color::Muted))
                    .child(
                        IconButton::new("rename-conversation", IconName::Pencil)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text("Rename conversation"))
                            .on_click(cx.listener({
                                let id = row.id.clone();
                                let title = row.display_title().to_owned();
                                move |this, _, window, cx| {
                                    this.renaming = Some(id.clone());
                                    this.title_input.update(cx, |input, cx| {
                                        input.set_text(title.clone(), true, cx)
                                    });
                                    this.title_input.focus_handle(cx).focus(window, cx);
                                    cx.notify();
                                }
                            })),
                    )
                    .child(
                        IconButton::new("archive-conversation", IconName::Archive)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text(if row.archived {
                                "Restore conversation"
                            } else {
                                "Archive conversation; keep agent running"
                            }))
                            .on_click(cx.listener(|this, _, _, cx| this.archive_selected(cx))),
                    )
                    .when_some(terminal.clone(), |header, runtime| {
                        header.child(
                            IconButton::new("show-conversation-terminal", IconName::Terminal)
                                .icon_size(IconSize::Small)
                                .tooltip(Tooltip::text("Show in terminal"))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.emit(Event::ShowTerminal(runtime.clone()))
                                })),
                        )
                    }),
            )
            .when(self.renaming.as_deref() == Some(&row.id), |view| {
                view.child(
                    h_flex()
                        .px_4()
                        .py_2()
                        .gap_2()
                        .child(div().flex_1().child(crate::components::input_field(
                            "conversation-title",
                            self.title_input.clone(),
                            cx,
                        )))
                        .child(
                            Button::new("save-conversation-title", "Save")
                                .size(ButtonSize::Compact)
                                .on_click(cx.listener(|this, _, _, cx| this.save_title(cx))),
                        )
                        .child(
                            Button::new("cancel-conversation-title", "Cancel")
                                .size(ButtonSize::Compact)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.renaming = None;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .child(
                v_flex()
                    .id(SharedString::from(format!("transcript-{}", row.id)))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .items_center()
                    .px_5()
                    .py_6()
                    .gap_6()
                    .children(messages)
                    .children(
                        row.requests
                            .iter()
                            .enumerate()
                            .map(|(index, request)| self.request_card(row, index, request, cx)),
                    )
                    .when(row.messages.len() >= 300, |body| {
                        body.child(
                            Label::new(
                                "Showing recent messages. Full history remains with your agent.",
                            )
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                        )
                    })
                    .when(row.messages.is_empty(), |body| {
                        body.child(
                            v_flex()
                                .w_full()
                                .max_w(px(820.))
                                .flex_none()
                                .min_w_0()
                                .gap_2()
                                .child(
                                    Label::new(if row.native.is_none() {
                                        "Connect this conversation"
                                    } else {
                                        "No messages yet"
                                    })
                                    .size(LabelSize::Default),
                                )
                                .when_some(row.problem.clone().filter(|_| row.native.is_some()), |body, text| {
                                    body.child(
                                        div()
                                            .text_color(colors.text_muted)
                                            .text_size(rems(0.9))
                                            .child(text),
                                    )
                                })
                                .when(row.native.is_none(), |body| {
                                    let (message, button, disabled) = match &setup {
                                        Setup::Checking => ("Checking the installed integration…".to_owned(), None, true),
                                        Setup::Installing => ("Installing the agent integration…".to_owned(), Some("Installing…".to_owned()), true),
                                        Setup::Enabled => (format!("{} integration is enabled. Open a new terminal tab, then start or resume your conversation there. Existing processes keep their previous connection settings.", provider.name()), None, true),
                                        Setup::Failed(error) => (format!("Could not enable {} integration: {error}", provider.name()), Some("Retry integration".to_owned()), false),
                                        Setup::Outdated => ("Update the integration, then restart the agent to load it.".to_owned(), Some(format!("Update {} integration", provider.name())), false),
                                        Setup::Available => ("Enable the integration, then start or resume the agent in a new terminal tab.".to_owned(), Some(format!("Enable {} integration", provider.name())), false),
                                    };
                                    body.child(div().w_full().text_color(colors.text_muted).text_size(rems(0.9)).child(message))
                                    .when_some(button, |body, label| body.child(
                                        h_flex().flex_none().mt_1().child(
                                        Button::new(
                                            "enable-conversation-integration",
                                            label,
                                        )
                                        .style(ButtonStyle::Outlined)
                                        .size(ButtonSize::Compact)
                                        .disabled(disabled || self.busy || !self.connected)
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                this.busy = true;
                                                this.integrations.installing(provider);
                                                cx.emit(Event::EnableIntegration(provider));
                                                cx.notify();
                                            }),
                                        ),
                                    )))
                                }),
                        )
                    }),
            )
            .child(
                v_flex()
                    .px_4()
                    .pb_4()
                    .pt_2()
                    .gap_2()
                    .flex_none()
                    .when(!status_text.is_empty(), |footer| {
                        footer.child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(rems(0.85))
                                        .text_color(colors.text_muted)
                                        .child(status_text),
                                )
                                .when_some(
                                    terminal.filter(|_| {
                                        !row.can_send() && row.status != Status::Working
                                    }),
                                    |line, runtime| {
                                        line.child(
                                            Button::new(
                                                "continue-conversation-terminal",
                                                "Continue in terminal",
                                            )
                                            .size(ButtonSize::Compact)
                                            .on_click(
                                                cx.listener(move |_, _, _, cx| {
                                                    cx.emit(Event::ShowTerminal(runtime.clone()))
                                                }),
                                            ),
                                        )
                                    },
                                ),
                        )
                    })
                    .when(row.delivery.is_some(), |footer| {
                        let id = row.id.clone();
                        footer.child(
                            Button::new(
                                "reset-uncertain-delivery",
                                "I checked in terminal — unlock composer",
                            )
                            .size(ButtonSize::Compact)
                            .tooltip(Tooltip::text(
                                "Keep your draft. Nothing is sent automatically.",
                            ))
                            .on_click(cx.listener(
                                move |this, _, window, cx| this.reset_delivery(id.clone(), window, cx),
                            )),
                        )
                    })
                    .child(
                        v_flex()
                            .key_context("ConversationComposer")
                            .px_3()
                            .pt_3()
                            .pb_2()
                            .gap_3()
                            .rounded_lg()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.element_background)
                            .child(editor.clone())
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Label::new(row.provider.name())
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child(div().flex_1())
                                    .when(
                                        row.status == Status::Working && row.endpoint.is_some(),
                                        |line| {
                                            let row = row.clone();
                                            line.child(
                                                Button::new("interrupt-conversation", "Stop")
                                                    .size(ButtonSize::Compact)
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.control(&row, None, cx)
                                                        },
                                                    )),
                                            )
                                        },
                                    )
                                    .child(
                                        IconButton::new(
                                            "send-conversation-message",
                                            IconName::ArrowUp,
                                        )
                                        .icon_size(IconSize::Small)
                                        .disabled(
                                            disabled || editor.read(cx).text(cx).trim().is_empty(),
                                        )
                                        .tooltip(Tooltip::text("Send message · Enter"))
                                        .on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.send(window, cx)
                                            }),
                                        ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn request_card(
        &mut self,
        row: &Conversation,
        index: usize,
        request: &Request,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (title, choices) = match request {
            Request::Question { prompt, options, .. } => (prompt.clone(), options.clone()),
            Request::Permission { permission, patterns, .. } => (
                format!("Allow {permission}?\n{}", patterns.join("\n")),
                vec![("Allow once".into(), "once".into()), ("Deny".into(), "reject".into())],
            ),
            Request::TerminalRequired => {
                return Label::new("Continue this question in terminal.")
                    .color(Color::Muted)
                    .into_any_element();
            }
        };
        v_flex()
            .w_full()
            .max_w(px(820.))
            .p_3()
            .gap_2()
            .border_1()
            .border_color(cx.theme().colors().border)
            .rounded_md()
            .child(title)
            .children(choices.into_iter().enumerate().map(|(choice, (label, detail))| {
                let answer = if matches!(request, Request::Permission { .. }) {
                    detail.clone()
                } else {
                    label.clone()
                };
                let request = request.clone();
                let row = row.clone();
                Button::new(SharedString::from(format!("request-{index}-{choice}")), label)
                    .size(ButtonSize::Compact)
                    .style(ButtonStyle::Outlined)
                    .disabled(!self.connected || self.sending.contains(&row.id))
                    .tooltip(Tooltip::text(detail))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.control(&row, Some((request.clone(), answer.clone())), cx)
                    }))
            }))
            .into_any_element()
    }

    fn message(
        &mut self,
        message: &Message,
        row: &Conversation,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = cx.theme().colors().clone();
        let key = format!("{}:{}", row.id, message.id);
        if message.role == Role::Tool {
            let expanded = self.expanded_tools.contains(&key);
            let title = message.text.lines().next().unwrap_or("Tool").to_owned();
            return v_flex()
                .w_full()
                .max_w(px(820.))
                .gap_2()
                .child(
                    Button::new(SharedString::from(key.clone()), title)
                        .size(ButtonSize::Compact)
                        .start_icon(
                            Icon::new(if expanded {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .size(IconSize::Small),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if !this.expanded_tools.remove(&key) {
                                this.expanded_tools.insert(key.clone());
                            }
                            cx.notify();
                        })),
                )
                .when(expanded, |view| {
                    view.child(
                        div()
                            .p_3()
                            .rounded_md()
                            .bg(colors.element_background)
                            .text_size(rems(0.875))
                            .child(message.text.clone()),
                    )
                })
                .into_any_element();
        }
        let copied = message.text.clone();
        v_flex()
            .w_full()
            .max_w(px(820.))
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(if message.role == Role::User {
                            "You"
                        } else {
                            row.provider.name()
                        })
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                    .child(div().flex_1())
                    .child(
                        IconButton::new(SharedString::from(format!("copy-{key}")), IconName::Copy)
                            .icon_size(IconSize::XSmall)
                            .tooltip(Tooltip::text("Copy message"))
                            .on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()))
                            }),
                    ),
            )
            .child(super::markdown::render(&message.text, window, cx))
            .into_any_element()
    }
}

fn relative_time(updated: u64) -> String {
    let minutes = chartr_conversations::now_millis().saturating_sub(updated) / 60_000;
    match minutes {
        0 => "now".into(),
        1..60 => format!("{minutes}m"),
        60..1440 => format!("{}h", minutes / 60),
        _ => format!("{}d", minutes / 1440),
    }
}
