//! Sidebar mode: the session list down the left.
//!
//! The mode for many long-lived sessions. There is room here for the things a
//! tab cannot hold — the agent's name under the title, and a close button that
//! is not fighting the title for space — so this chrome shows them.

use ui::{Tooltip, prelude::*};

use super::Emit;

use super::{Action, Entry, status_dot};

/// The sidebar's width. Fixed rather than draggable: a resizable sidebar is a
/// preference to persist, a drag handle to hit-test, and a minimum to enforce,
/// and none of that is what makes this mode useful.
pub const WIDTH: Pixels = px(220.);

pub fn render(entries: &[Entry], on: Emit, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();

    v_flex()
        .w(WIDTH)
        .flex_none()
        .h_full()
        .bg(colors.panel_background)
        .border_r_1()
        .border_color(colors.border)
        .child(header(on.clone()))
        .child(v_flex().id("sessions").flex_1().overflow_y_scroll().p_1().gap_px().children(
            entries.iter().enumerate().map(|(index, entry)| row(index, entry, on.clone(), cx)),
        ))
}

fn header(on: Emit) -> impl IntoElement {
    let toggle = on.clone();
    h_flex()
        .h(px(36.))
        .px_2()
        .gap_1()
        .justify_between()
        .child(Label::new("Sessions").size(LabelSize::Small).color(Color::Muted))
        .child(
            h_flex()
                .gap_px()
                .child(
                    IconButton::new("new-session", IconName::Plus)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("New session"))
                        .on_click(move |_, window, cx| on(Action::New, window, cx)),
                )
                .child(
                    IconButton::new("toggle-mode", IconName::Tab)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Switch to tabs"))
                        .on_click(move |_, window, cx| toggle(Action::ToggleMode, window, cx)),
                ),
        )
}

fn row(index: usize, entry: &Entry, on: Emit, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    let close = on.clone();

    h_flex()
        .id(("session", index))
        .group("session")
        .h(px(38.))
        .px_2()
        .gap_2()
        .rounded_sm()
        .when(entry.selected, |row| row.bg(colors.element_selected))
        .when(!entry.selected, |row| row.hover(|row| row.bg(colors.element_hover)))
        .on_click(move |_, window, cx| on(Action::Select(index), window, cx))
        .child(status_dot(entry, cx))
        .child(
            v_flex()
                .flex_1()
                .overflow_hidden()
                .child(Label::new(entry.title.clone()).size(LabelSize::Small).truncate())
                .when_some(entry.agent.clone(), |column, agent| {
                    column.child(
                        Label::new(agent).size(LabelSize::XSmall).color(Color::Muted).truncate(),
                    )
                }),
        )
        .child(
            // Revealed on hover so a list of ten sessions is ten titles rather
            // than ten titles and ten buttons.
            div().visible_on_hover("session").child(
                IconButton::new(("close", index), IconName::Close)
                    .icon_size(IconSize::XSmall)
                    .on_click(move |_, window, cx| close(Action::Close(index), window, cx)),
            ),
        )
}
