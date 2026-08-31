//! Tabs mode: the session list across the top.
//!
//! The mode for a handful of sessions you are switching between quickly. A tab
//! has no second line, so the agent's name is dropped here rather than
//! squeezed in — the dot still carries the state, and the title carries the
//! identity.

use ui::{Tooltip, prelude::*};

use super::Emit;

use super::{Action, Entry, status_dot};

pub const HEIGHT: Pixels = px(32.);

pub fn render(entries: &[Entry], on: Emit, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    let new = on.clone();
    let toggle = on.clone();

    h_flex()
        .h(HEIGHT)
        .flex_none()
        .w_full()
        .bg(colors.tab_bar_background)
        .border_b_1()
        .border_color(colors.border)
        .child(h_flex().id("tabs").flex_1().overflow_x_scroll().children(
            entries.iter().enumerate().map(|(index, entry)| tab(index, entry, on.clone(), cx)),
        ))
        .child(
            h_flex()
                .px_1()
                .gap_px()
                .flex_none()
                .child(
                    IconButton::new("new-session", IconName::Plus)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("New session"))
                        .on_click(move |_, window, cx| new(Action::New, window, cx)),
                )
                .child(
                    IconButton::new("toggle-mode", IconName::Menu)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Switch to sidebar"))
                        .on_click(move |_, window, cx| toggle(Action::ToggleMode, window, cx)),
                ),
        )
}

fn tab(index: usize, entry: &Entry, on: Emit, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    let close = on.clone();

    h_flex()
        .id(("tab", index))
        .group("tab")
        .h_full()
        .px_2()
        .gap_1p5()
        .max_w(px(200.))
        .border_r_1()
        .border_color(colors.border)
        .when(entry.selected, |tab| tab.bg(colors.tab_active_background))
        .when(!entry.selected, |tab| {
            tab.bg(colors.tab_inactive_background).hover(|tab| tab.bg(colors.element_hover))
        })
        .on_click(move |_, window, cx| on(Action::Select(index), window, cx))
        .child(status_dot(entry, cx))
        .child(
            Label::new(entry.title.clone())
                .size(LabelSize::Small)
                .color(if entry.selected { Color::Default } else { Color::Muted })
                .truncate(),
        )
        .child(
            div().visible_on_hover("tab").child(
                IconButton::new(("close", index), IconName::Close)
                    .icon_size(IconSize::XSmall)
                    .on_click(move |_, window, cx| close(Action::Close(index), window, cx)),
            ),
        )
}
