//! Tabs mode: standalone tabs and pane groups beside the active space name.
//!
//! The mode for a handful of sessions you are switching between quickly. A tab
//! has no second line, so the agent's name is dropped here rather than
//! squeezed in — the dot still carries the state, and the title carries the
//! identity.

use gpui::Role;
use ui::{
    ButtonSize, ContextMenu, IconButtonShape, Tab, TabBar, TabPosition, Tooltip, prelude::*,
    right_click_menu,
};

use super::Emit;

use super::{Action, DraggedItem, Entry, dragged_item_preview, status_indicator};
use crate::fonts::UI_LABEL_DEFAULT;

const SPACE_SWITCHER_MAX_WIDTH: f32 = 200.;

pub fn render(
    entries: &[Entry],
    space_switcher: AnyElement,
    new_item: AnyElement,
    on: Emit,
    cx: &App,
) -> impl IntoElement {
    let settings = on.clone();
    let active_index = entries.iter().position(|entry| entry.selected);

    TabBar::new("workspace-tabs")
        .start_child(h_flex().flex_none().max_w(px(SPACE_SWITCHER_MAX_WIDTH)).child(space_switcher))
        .children(
            entries.iter().enumerate().map(|(index, entry)| {
                tab(index, entries.len(), active_index, entry, on.clone(), cx)
            }),
        )
        .end_child(new_item)
        .end_child(
            IconButton::new("open-settings", IconName::Settings)
                .icon_size(IconSize::Small)
                .tooltip(Tooltip::text("Settings"))
                .on_click(move |_, window, cx| settings(Action::OpenSettings, window, cx)),
        )
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
    let position = if index == 0 {
        TabPosition::First
    } else if index + 1 == count {
        TabPosition::Last
    } else {
        TabPosition::Middle(index.cmp(&active_index.unwrap_or(index)))
    };
    let select = entry.key;
    let select_item = on.clone();
    let ungroup = on.clone();
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
    let tab = Tab::new(("tab", index))
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
        .start_slot(status_indicator(
            entry.status,
            entry.process_running,
            entry.ended,
            entry.grouped,
            &entry.space_key,
            entry.key,
            cx,
        ))
        .end_slot::<AnyElement>(close_slot)
        .child(Label::new(entry.title.clone()).size(UI_LABEL_DEFAULT).truncate());

    if grouped {
        right_click_menu(format!("group-tab-menu-{space:?}-{}", close_tab.get()))
            .trigger(move |_, _, _| tab)
            .menu(move |window, cx| {
                let ungroup = ungroup.clone();
                ContextMenu::build(window, cx, move |menu, _, _| {
                    menu.entry("Ungroup", None, move |window, cx| {
                        ungroup(Action::UngroupPane { space, tab: close_tab }, window, cx)
                    })
                })
            })
            .into_any_element()
    } else {
        tab.into_any_element()
    }
}
