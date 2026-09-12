use super::*;
use gpui::{AnyElement, Role, Window, div, px};
use ui::{Button, ButtonSize, IconButton, Tooltip, prelude::*};

use crate::components::{
    SegmentedControl, SegmentedControlOption, SelectionRowBackgrounds, selection_list,
    selection_row,
};
use crate::fonts::{UI_LABEL_DEFAULT, UI_LABEL_SMALL, UI_TEXT_DEFAULT};
use crate::settings::sidebar_theme_colors;

impl Render for Conversations {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors().clone();
        let selected = self
            .selected
            .as_ref()
            .and_then(|id| self.rows.iter().find(|r| &r.id == id && self.in_scope(r)))
            .cloned();

        if self.focus_terminal
            && let Some(terminal) = self.terminal_view.as_ref()
        {
            self.focus_terminal = false;
            terminal.focus_handle(cx).focus(window, cx);
        }
        let content = if let Some(row) = selected {
            self.conversation(&row, cx)
        } else if let Some(name) = self.new_agent.clone() {
            v_flex()
                .flex_1()
                .min_w_0()
                .h_full()
                .child(
                    h_flex()
                        .h(px(42.))
                        .px_4()
                        .gap_2()
                        .flex_none()
                        .border_b_1()
                        .border_color(colors.border)
                        .child(Icon::new(IconName::Chat).size(IconSize::Small).color(Color::Muted))
                        .child(Label::new(format!("New {name} conversation"))),
                )
                .child(self.terminal_content(
                    if self.busy {
                        "Starting agent…"
                    } else {
                        "The agent has not connected yet."
                    },
                    cx,
                ))
                .into_any_element()
        } else {
            v_flex()
                .flex_1()
                .size_full()
                .items_center()
                .justify_center()
                .gap_3()
                .child(Icon::new(IconName::Chat).size(IconSize::Medium).color(Color::Muted))
                .child(Label::new("Pick up a conversation").size(LabelSize::Large))
                .child(
                    div().max_w(px(380.)).text_center().text_color(colors.text_muted).child(
                        "Choose a conversation from Inbox, or launch an agent to get started.",
                    ),
                )
                .child(self.agent_picker(true, cx))
                .into_any_element()
        };

        v_flex()
            .id("inbox-mode")
            .key_context("Inbox")
            .track_focus(&self.focus)
            .size_full()
            .min_h_0()
            .text_size(rems(1.))
            .bg(colors.editor_background)
            .child(h_flex().flex_1().min_h_0().w_full().child(content))
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
    pub(crate) fn render_sidebar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors().clone();
        let rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| self.in_scope(row) && row.archived == self.show_archived)
            .cloned()
            .collect();
        v_flex()
            .id("conversation-history")
            .key_context("Inbox")
            .size_full()
            .min_h_0()
            .min_w_0()
            .text_size(UI_TEXT_DEFAULT)
            .bg(colors.panel_background)
            .child(
                div().w_full().px_1p5().pb_2().flex_none().child(
                    SegmentedControl::new(
                        "Conversation history",
                        [("history-inbox", "Inbox", false), ("history-archive", "Archive", true)]
                            .map(|(id, label, archived)| {
                                SegmentedControlOption::new(
                                    id,
                                    label,
                                    self.show_archived == archived,
                                    cx.listener(move |this, _, _, cx| {
                                        this.show_archived = archived;
                                        cx.notify();
                                    }),
                                )
                            }),
                    )
                    .full_width(),
                ),
            )
            .child(self.flat_history(&rows, cx))
            .into_any_element()
    }

    fn flat_history(&self, rows: &[Conversation], cx: &Context<Self>) -> AnyElement {
        selection_list()
            .id("history-rows")
            .flex_1()
            .min_h_0()
            .w_full()
            .px_1p5()
            .pb_2()
            .overflow_y_scroll()
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .h(ButtonSize::Compact.rems())
                    .px_1()
                    .pb_1()
                    .gap_1()
                    .flex_none()
                    .child(
                        Label::new(if self.show_archived {
                            "Archived chats"
                        } else {
                            "Recent chats"
                        })
                        .size(UI_LABEL_SMALL)
                        .color(Color::Muted)
                        .flex_1()
                        .truncate(),
                    )
                    .when(!self.show_archived, |heading| {
                        heading.child(self.agent_picker(false, cx))
                    }),
            )
            .children(rows.iter().map(|row| self.history_row(row, cx)))
            .when(rows.is_empty(), |list| list.child(self.empty_history(cx)))
            .into_any_element()
    }

    fn empty_history(&self, cx: &App) -> AnyElement {
        div()
            .px_1()
            .py_2()
            .text_color(cx.theme().colors().text_muted)
            .child(if self.show_archived {
                "No archived chats."
            } else {
                "Conversations will appear here as you use your agents."
            })
            .into_any_element()
    }

    fn history_row(&self, row: &Conversation, cx: &Context<Self>) -> AnyElement {
        let colors = cx.theme().colors();
        let sidebar_colors = sidebar_theme_colors(cx.theme());
        let row_backgrounds = SelectionRowBackgrounds {
            hover: sidebar_colors.session_hover,
            selected: sidebar_colors.session_active,
        };
        let id = row.id.clone();
        let title = row.display_title().to_owned();
        let space_label = self.space_label(row);
        let detail = format!(
            "{}\n{} · {}\n{}",
            title,
            row.provider.name(),
            space_label,
            row.cwd
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Free sessions".to_owned())
        );
        let is_selected = self.selected.as_deref() == Some(&id);
        let status_color = if row.runtime.is_some() {
            match row.status {
                Status::Working => cx.theme().status().info,
                Status::Waiting => cx.theme().status().warning,
                _ => cx.theme().status().success,
            }
        } else {
            colors.text_muted.opacity(0.45)
        };
        div()
            .id(format!("history-row-{id}"))
            .w_full()
            .min_w_0()
            .flex_none()
            .tooltip(Tooltip::text(detail))
            .child(
                selection_row(format!("history-{id}"), is_selected)
                    .backgrounds(row_backgrounds)
                    .aria_role(Role::Tab)
                    .aria_label(title.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.new_agent = None;
                        this.new_space = None;
                        this.launch_runtime = None;
                        this.selected = Some(id.clone());
                        this.renaming = None;
                        this.clear_terminal();
                        this.focus_terminal = true;
                        this.pending_runtime = None;
                        this.problem = None;
                        this.focus.focus(window, cx);
                        cx.emit(Event::SelectionChanged);
                        cx.notify();
                    }))
                    .start_slot(
                        h_flex()
                            .w(IconSize::XSmall.rems())
                            .flex_none()
                            .justify_center()
                            .child(div().size(px(5.)).rounded_full().bg(status_color)),
                    )
                    .child(Label::new(title).size(UI_LABEL_DEFAULT).flex_1().truncate())
                    .end_slot(
                        Label::new(relative_time(row.updated))
                            .size(UI_LABEL_SMALL)
                            .color(Color::Muted)
                            .flex_none(),
                    ),
            )
            .into_any_element()
    }

    fn conversation(&mut self, row: &Conversation, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors().clone();
        let context = self.space_label(row);
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
                    ),
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
            .child(self.terminal_content(
                if row.runtime.is_none() {
                    "Session ended. This conversation remains in Inbox."
                } else {
                    "Connecting to session terminal…"
                },
                cx,
            ))
            .into_any_element()
    }

    fn terminal_content(&self, empty: &str, cx: &App) -> AnyElement {
        if self.connected
            && let Some(view) = self.terminal_view.clone()
        {
            return div()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .overflow_hidden()
                .child(crate::terminal_host::element(view, cx.theme().colors().terminal_background))
                .into_any_element();
        }
        v_flex()
            .flex_1()
            .min_h_0()
            .w_full()
            .items_center()
            .justify_center()
            .gap_2()
            .p_4()
            .child(Icon::new(IconName::Terminal).size(IconSize::Medium).color(Color::Muted))
            .child(
                div()
                    .max_w(px(480.))
                    .text_center()
                    .text_color(cx.theme().colors().text_muted)
                    .child(
                        if !self.connected {
                            "Reconnecting to terminal service…"
                        } else {
                            self.terminal_notice.as_deref().unwrap_or(empty)
                        }
                        .to_owned(),
                    ),
            )
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
