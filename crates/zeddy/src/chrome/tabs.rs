//! Tabs mode: standalone tabs and pane groups in a horizontal strip.
//!
//! The mode for a handful of sessions you are switching between quickly. A tab
//! has no second line, so the agent's name is dropped here rather than
//! squeezed in — the dot still carries the state, and the title carries the
//! identity.

use gpui::Entity;
use ui::{ButtonSize, IconButtonShape, Tab, TabBar, Tooltip, prelude::*};

use super::Emit;

use super::tab_sorter::{SortableTab, SortableTabList};
use super::{Action, DraggedItem, Entry, ItemTab, tab_position};
use crate::components::SortAxis;
use crate::components::{ContextMenu, popup_right_click_menu};
use crate::settings::SettingsStore;
use crate::workspace::WorkspaceTabId;
const SPACE_SWITCHER_MAX_WIDTH: f32 = 200.;

type HoveredTab = Option<(gpui::EntityId, WorkspaceTabId)>;

pub fn render(
    entries: &[Entry],
    controls: Option<(AnyElement, AnyElement)>,
    new_item: AnyElement,
    new_plugin_pane: AnyElement,
    on: Emit,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    // Use stable workspace identities so hover cannot move to an unrelated
    // tab when entries are reordered, closed, or the current space changes.
    let hovered_tab = window.use_keyed_state("workspace-tab-hover", cx, |_, _| HoveredTab::None);
    let hovered = *hovered_tab.read(cx);
    let move_tab = on.clone();
    let space = entries.first().map(|entry| entry.space);
    let space_key = entries.first().map(|entry| entry.space_key.as_str()).unwrap_or_default();
    let tabs = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let dragged = DraggedItem {
                space: entry.space_key.clone(),
                tab: entry.tab,
                pane: entry.pane,
                index,
                item: entry.key,
                top_level: true,
                grouped: entry.grouped,
            };
            let entry = entry.clone();
            let on = on.clone();
            let hovered_tab = hovered_tab.clone();
            SortableTab::new(dragged, entry.selected, move |placement, cx| {
                let previous_selected = placement.index.checked_sub(1) == placement.active_index;
                let previous_hovered = placement.previous.is_some_and(|previous| {
                    hovered
                        .is_some_and(|(space, tab)| space == entry.space && tab.get() == previous)
                });
                let separator_visible = placement.previous.is_some()
                    && !previous_selected
                    && !entry.selected
                    && !previous_hovered
                    && hovered != Some((entry.space, entry.tab));
                let trailing_separator_visible = placement.index + 1 == placement.count
                    && !entry.selected
                    && hovered != Some((entry.space, entry.tab));
                h_flex()
                    .relative()
                    .child(tab(
                        placement.index,
                        placement.count,
                        placement.active_index,
                        &entry,
                        &hovered_tab,
                        on,
                        cx,
                    ))
                    .child(
                        div()
                            .absolute()
                            .left(gpui::rems(-0.125))
                            .w(px(1.))
                            .h(px(12.))
                            .bg(cx.theme().colors().border)
                            .opacity(if separator_visible { 1. } else { 0. }),
                    )
                    .when(trailing_separator_visible, |tab| {
                        tab.child(
                            div()
                                .absolute()
                                .right(gpui::rems(-0.125))
                                .w(px(1.))
                                .h(px(12.))
                                .bg(cx.theme().colors().border),
                        )
                    })
                    .into_any_element()
            })
        })
        .collect();
    let list = SortableTabList::new(
        format!("workspace-tab-sorter-{space_key}"),
        h_flex()
            .id("workspace-tab-list")
            .min_w_0()
            .h(Tab::container_height(cx))
            .px_1()
            .flex_shrink_1()
            .overflow_x_scroll(),
        SortAxis::Horizontal,
        gpui::rems(0.25),
        tabs,
        move |dragged, target_index, window, cx| {
            if let Some(space) = space {
                move_tab(
                    Action::MoveWorkspaceTab { space, tab: dragged.tab, target_index },
                    window,
                    cx,
                );
            }
        },
    );
    let tabs_with_pinned_new_item = h_flex().w_full().min_w_0().h_full().child(list).child(
        h_flex()
            .h(Tab::container_height(cx))
            .flex_none()
            .px(DynamicSpacing::Base04.rems(cx))
            .gap_px()
            .child(new_item)
            .child(new_plugin_pane),
    );

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

    // Inset tabs leave the strip's bottom border continuous for standalone
    // items and pane groups alike.
    div().relative().w_full().flex_none().child(tab_bar)
}

fn tab(
    index: usize,
    count: usize,
    active_index: Option<usize>,
    entry: &Entry,
    hovered_tab: &Entity<HoveredTab>,
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
    let hover_key = (entry.space, entry.tab);
    let hovered = *hovered_tab.read(cx) == Some(hover_key);
    let hovered_tab = hovered_tab.clone();
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
        format!("workspace-tab-{space:?}-{}", entry.tab.get()),
        entry.title.clone(),
        entry.selected,
        position,
        &entry.space_key,
        entry.key,
    )
    .aria_label(aria_label)
    .activity(entry.activity())
    .icon_path(entry.icon_path.clone())
    .grouped(entry.grouped)
    .close_slot(close_slot)
    .build_rounded(hovered, cx)
    .on_hover(move |is_hovered, window, cx| {
        hovered_tab.update(cx, |hovered, _| {
            let next = if *is_hovered {
                Some(hover_key)
            } else if *hovered == Some(hover_key) {
                None
            } else {
                *hovered
            };
            if *hovered != next {
                *hovered = next;
                window.refresh();
            }
        });
    })
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
    .can_drop(move |value, _, _| {
        value
            .downcast_ref::<DraggedItem>()
            .is_some_and(|dragged| dragged.space == target_space_key && dragged.top_level)
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
