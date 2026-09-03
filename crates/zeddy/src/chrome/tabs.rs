//! Tabs mode: standalone tabs and pane groups in a horizontal strip.
//!
//! The mode for a handful of sessions you are switching between quickly. A tab
//! has no second line, so the agent's name is dropped here rather than
//! squeezed in — the dot still carries the state, and the title carries the
//! identity.

use ui::{ButtonSize, IconButtonShape, TabBar, Tooltip, prelude::*};

use super::Emit;

use super::{
    Action, DraggedItem, Entry, ItemTab, dragged_item_preview, new_item_cell, tab_position,
};
use crate::components::{ContextMenu, popup_right_click_menu};
use crate::settings::SettingsStore;
const SPACE_SWITCHER_MAX_WIDTH: f32 = 200.;

pub fn render(
    entries: &[Entry],
    controls: Option<(AnyElement, AnyElement)>,
    new_item: AnyElement,
    on: Emit,
    cx: &App,
) -> impl IntoElement {
    let active_index = entries.iter().position(|entry| entry.selected);
    let selected_is_grouped = entries.iter().any(|entry| entry.selected && entry.grouped);
    let tabs_with_pinned_new_item = h_flex()
        .w_full()
        .min_w_0()
        .h_full()
        .child(
            h_flex()
                .id("workspace-tab-list")
                .min_w_0()
                .flex_shrink_1()
                .overflow_x_scroll()
                .children(entries.iter().enumerate().map(|(index, entry)| {
                    tab(index, entries.len(), active_index, entry, on.clone(), cx)
                })),
        )
        .child(new_item_cell(new_item, cx));

    let tab_bar = TabBar::new("workspace-tabs").child(tabs_with_pinned_new_item);
    let tab_bar = match controls {
        Some((space_switcher, view_menu)) => tab_bar
            .start_child(
                h_flex().flex_none().max_w(px(SPACE_SWITCHER_MAX_WIDTH)).child(space_switcher),
            )
            .end_child(view_menu)
            .into_any_element(),
        None => tab_bar.into_any_element(),
    };

    div().relative().w_full().flex_none().child(tab_bar).when(selected_is_grouped, |tab_bar| {
        // Selected Zed tabs omit their bottom border to join their content.
        // A group opens onto another tab strip instead, so paint over that
        // gap after the tabs while sharing the existing strip border's edge.
        tab_bar.child(
            div().absolute().left_0().right_0().bottom_0().h(px(1.)).bg(cx.theme().colors().border),
        )
    })
}

fn tab(
    index: usize,
    count: usize,
    active_index: Option<usize>,
    entry: &Entry,
    on: Emit,
    cx: &App,
) -> AnyElement {
    let close = on.clone();
    let middle_close = on.clone();
    let middle_click_closes_tab = cx.global::<SettingsStore>().resolved().middle_click_closes_tab;
    let position = tab_position(index, count, active_index);
    let select = entry.key;
    let select_item = on.clone();
    let ungroup = on.clone();
    let rename = on.clone();
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
        top_level: true,
        grouped: entry.grouped,
    };
    let close_slot: Option<AnyElement> = entry.closable.then(|| {
        IconButton::new(("close", index), IconName::Close)
            .shape(IconButtonShape::Square)
            .size(ButtonSize::None)
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
    let aria_label =
        if entry.grouped { format!("Pane group: {}", entry.title) } else { entry.title.clone() };
    let tab = ItemTab::new(
        ("tab", index),
        entry.title.clone(),
        entry.selected,
        position,
        &entry.space_key,
        entry.key,
    )
    .aria_label(aria_label)
    .activity(entry.activity())
    .grouped(entry.grouped)
    .close_slot(close_slot)
    .build(cx)
    .on_click(move |_, window, cx| {
        select_item(Action::Select { space: Some(space), item: select }, window, cx)
    })
    .when(entry.closable && middle_click_closes_tab, |tab| {
        tab.on_aux_click(move |event, window, cx| {
            if event.is_middle_click() {
                cx.stop_propagation();
                middle_close(
                    if grouped {
                        Action::CloseGroup { space: close_space, tab: close_tab }
                    } else {
                        Action::Close { space: Some(close_space), item: close_key }
                    },
                    window,
                    cx,
                );
            }
        })
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
        move_tab(Action::MoveWorkspaceTab { space, tab: dragged.tab, target_index }, window, cx);
    });

    if grouped {
        popup_right_click_menu(format!("group-tab-menu-{space:?}-{}", close_tab.get()))
            .trigger(move |_, _, _| tab)
            .menu(move |window, cx| {
                let ungroup = ungroup.clone();
                let rename = rename.clone();
                ContextMenu::build_popup(window, cx, move |menu| {
                    menu.entry("Rename", None, move |window, cx| {
                        rename(Action::RenameGroup { space, tab: close_tab }, window, cx)
                    })
                    .entry("Ungroup", None, move |window, cx| {
                        ungroup(Action::UngroupPane { space, tab: close_tab }, window, cx)
                    })
                })
            })
            .into_any_element()
    } else {
        tab.into_any_element()
    }
}
