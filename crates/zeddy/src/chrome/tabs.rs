//! Tabs mode: the session list across the top.
//!
//! The mode for a handful of sessions you are switching between quickly. A tab
//! has no second line, so the agent's name is dropped here rather than
//! squeezed in — the dot still carries the state, and the title carries the
//! identity.

use gpui::Role;
use ui::{Tab, TabPosition, Tooltip, prelude::*};

use super::Emit;

use super::{Action, Entry, status_dot};

pub fn render(
    entries: &[Entry],
    space_switcher: AnyElement,
    new_item: AnyElement,
    on: Emit,
    cx: &App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
    let toggle = on.clone();
    let active_index = entries.iter().position(|entry| entry.selected);

    h_flex()
        .h(Tab::container_height(cx))
        .flex_none()
        .w_full()
        .bg(colors.tab_bar_background)
        .border_b_1()
        .border_color(colors.border)
        .child(
            div()
                .w(px(super::sidebar::DEFAULT_WIDTH))
                .h_full()
                .flex_none()
                .border_r_1()
                .border_color(colors.border)
                .child(space_switcher),
        )
        .child(h_flex().id("tabs").flex_1().overflow_x_scroll().children(
            entries.iter().enumerate().map(|(index, entry)| {
                tab(index, entries.len(), active_index, entry, on.clone(), cx)
            }),
        ))
        .child(
            h_flex().px_1().gap_px().flex_none().child(new_item).child(
                IconButton::new("toggle-mode", IconName::Menu)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Switch to sidebar"))
                    .on_click(move |_, window, cx| toggle(Action::ToggleMode, window, cx)),
            ),
        )
}

fn tab(
    index: usize,
    count: usize,
    active_index: Option<usize>,
    entry: &Entry,
    on: Emit,
    cx: &App,
) -> impl IntoElement {
    let close = on.clone();
    let position = if index == 0 {
        TabPosition::First
    } else if index + 1 == count {
        TabPosition::Last
    } else {
        TabPosition::Middle(index.cmp(&active_index.unwrap_or(index)))
    };
    let select = entry.key;
    let close_key = entry.key;
    let space = entry.space;
    let close_space = entry.space;
    let close_slot: Option<AnyElement> = entry.closable.then(|| {
        IconButton::new(("close", index), IconName::Close)
            .icon_size(IconSize::XSmall)
            .tooltip(Tooltip::text("Close"))
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                close(Action::Close { space: Some(close_space), item: close_key }, window, cx)
            })
            .into_any_element()
    });
    Tab::new(("tab", index))
        .role(Role::Tab)
        .aria_label(entry.title.clone())
        .aria_selected(entry.selected)
        .position(position)
        .toggle_state(entry.selected)
        .on_click(move |_, window, cx| {
            on(Action::Select { space: Some(space), item: select }, window, cx)
        })
        .start_slot(status_dot(entry, cx))
        .end_slot::<AnyElement>(close_slot)
        .child(Label::new(entry.title.clone()).size(LabelSize::Small).truncate())
}
