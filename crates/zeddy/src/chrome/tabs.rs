//! Tabs mode: standalone tabs and pane groups beside the active space name.
//!
//! The mode for a handful of sessions you are switching between quickly. A tab
//! has no second line, so the agent's name is dropped here rather than
//! squeezed in — the dot still carries the state, and the title carries the
//! identity.

use gpui::Role;
use ui::{Tab, TabPosition, Tooltip, prelude::*};

use super::Emit;

use super::{Action, DraggedItem, Entry, dragged_item_preview, status_dot};

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
    let select_item = on.clone();
    let move_tab = on;
    let close_key = entry.key;
    let close_tab = entry.tab;
    let grouped = entry.grouped;
    let space = entry.space;
    let close_space = entry.space;
    let target_index = index;
    let target_space_key = entry.space_key.clone();
    let dragged = DraggedItem {
        space: entry.space_key.clone(),
        tab: entry.tab,
        pane: entry.pane,
        index,
        item: entry.key,
        title: entry.title.clone(),
        selected: entry.selected,
        top_level: true,
    };
    let close_slot: Option<AnyElement> = entry.closable.then(|| {
        IconButton::new(("close", index), IconName::Close)
            .icon_size(IconSize::XSmall)
            .tooltip(Tooltip::text("Close"))
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                close(
                    if grouped {
                        Action::CloseGroup { space: close_space, tab: close_tab }
                    } else {
                        Action::Close { space: Some(close_space), item: close_key }
                    },
                    window,
                    cx,
                )
            })
            .into_any_element()
    });
    Tab::new(("tab", index))
        .role(Role::Tab)
        .aria_label(if entry.grouped {
            format!("Pane group: {}", entry.title)
        } else {
            entry.title.clone()
        })
        .aria_selected(entry.selected)
        .position(position)
        .toggle_state(entry.selected)
        .on_click(move |_, window, cx| {
            select_item(Action::Select { space: Some(space), item: select }, window, cx)
        })
        .when(!entry.grouped, |tab| {
            tab.on_drag(dragged, |dragged, offset, _, cx| dragged_item_preview(dragged, offset, cx))
        })
        .can_drop(move |value, _, _| {
            value
                .downcast_ref::<DraggedItem>()
                .is_some_and(|dragged| dragged.space == target_space_key && dragged.top_level)
        })
        .drag_over::<DraggedItem>(move |tab, dragged, _, cx| {
            let mut tab = tab
                .bg(cx.theme().colors().drop_target_background)
                .border_color(cx.theme().colors().drop_target_border)
                .border_0();
            if target_index < dragged.index {
                tab = tab.border_l_2();
            } else if target_index > dragged.index {
                tab = tab.border_r_2();
            }
            tab
        })
        .on_drop(move |dragged: &DraggedItem, window, cx| {
            move_tab(
                Action::MoveWorkspaceTab { space, tab: dragged.tab, target_index },
                window,
                cx,
            );
        })
        .start_slot(status_dot(entry, cx))
        .end_slot::<AnyElement>(close_slot)
        .child(Label::new(entry.title.clone()).size(LabelSize::Small).truncate())
}
