//! Sidebar mode: standalone tabs and pane groups down the left.
//!
//! The mode for many long-lived sessions. There is room here for the things a
//! tab cannot hold — the agent's name under the title, and a close button that
//! is not fighting the title for space — so this chrome shows them.

use gpui::{Anchor, MouseButton, Role, deferred};
use ui::{ContextMenu, PopoverMenu, Tooltip, prelude::*};

use super::Emit;

use super::{
    Action, DraggedItem, DraggedSidebar, Entry, SpaceEntries, dragged_item_preview,
    status_indicator,
};
use crate::components::{selection_list, selection_row};
use crate::fonts::{UI_LABEL_DEFAULT, UI_LABEL_SMALL};

/// The sidebar's width. Fixed rather than draggable: a resizable sidebar is a
/// preference to persist, a drag handle to hit-test, and a minimum to enforce,
/// and none of that is what makes this mode useful.
pub const DEFAULT_WIDTH: f32 = 280.;
pub const MIN_WIDTH: f32 = 180.;
pub const MAX_WIDTH: f32 = 480.;

pub fn render(
    spaces: &[SpaceEntries],
    space_switcher: AnyElement,
    on: Emit,
    width: f32,
    cx: &App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
    let mut groups = Vec::new();
    let mut index = 0;
    for (space_index, space) in spaces.iter().enumerate() {
        let add = on.clone();
        let actions = on.clone();
        let space_id = space.id;
        let action_space = space.id;
        let removable = space.removable;
        let available = space.available;
        groups.push(
            h_flex()
                .group("space-heading")
                .px_2()
                .pt_2()
                .pb_1()
                .justify_between()
                .child(Label::new(space.name.clone()).size(UI_LABEL_SMALL).color(Color::Muted))
                .child(
                    h_flex()
                        .gap_px()
                        .child(
                            IconButton::new(("new-in-space", space_index), IconName::Plus)
                                .icon_size(IconSize::XSmall)
                                .tooltip(Tooltip::text("New session in this space"))
                                .on_click(move |_, window, cx| {
                                    add(Action::NewInSpace { space: space_id }, window, cx)
                                }),
                        )
                        .when(removable || !available, |controls| {
                            controls.child(
                                PopoverMenu::new(format!("space-actions-{space_index}"))
                                    .trigger_with_tooltip(
                                        IconButton::new(
                                            ("space-actions-trigger", space_index),
                                            IconName::Ellipsis,
                                        )
                                        .icon_size(IconSize::XSmall),
                                        Tooltip::text("Space Actions"),
                                    )
                                    .anchor(Anchor::TopRight)
                                    .menu(move |window, cx| {
                                        let rename = actions.clone();
                                        let locate = actions.clone();
                                        let close = actions.clone();
                                        Some(ContextMenu::build(window, cx, move |menu, _, _| {
                                            let menu = menu.when(!available, |menu| {
                                                menu.entry(
                                                    "Locate Space Folder",
                                                    None,
                                                    move |window, cx| {
                                                        locate(
                                                            Action::LocateSpace {
                                                                space: action_space,
                                                            },
                                                            window,
                                                            cx,
                                                        )
                                                    },
                                                )
                                            });
                                            menu.when(removable, |menu| {
                                                let menu = menu.entry(
                                                    "Rename Space",
                                                    None,
                                                    move |window, cx| {
                                                        rename(
                                                            Action::RenameSpace {
                                                                space: action_space,
                                                            },
                                                            window,
                                                            cx,
                                                        )
                                                    },
                                                );
                                                menu.separator().entry(
                                                    "Close Space",
                                                    None,
                                                    move |window, cx| {
                                                        close(
                                                            Action::CloseSpace {
                                                                space: action_space,
                                                            },
                                                            window,
                                                            cx,
                                                        )
                                                    },
                                                )
                                            })
                                        }))
                                    }),
                            )
                        }),
                )
                .into_any_element(),
        );
        for (target_index, entry) in space.entries.iter().enumerate() {
            groups.push(
                row(
                    index,
                    target_index,
                    entry,
                    space.active && entry.selected,
                    entry.grouped,
                    on.clone(),
                    cx,
                )
                .into_any_element(),
            );
            index += 1;
        }
    }

    v_flex()
        .id("spaces-sidebar")
        .relative()
        .w(px(width))
        .flex_none()
        .h_full()
        .bg(colors.panel_background)
        .border_r_1()
        .border_color(colors.border)
        .child(header(space_switcher, on.clone()))
        .child(
            selection_list()
                .id("sessions")
                .flex_1()
                .overflow_y_scroll()
                .py_1()
                .px_1()
                .children(groups),
        )
        .child(deferred(
            div()
                .id("sidebar-resize-handle")
                .absolute()
                // Keep the resize target fully outside the sidebar so it
                // cannot occlude a trailing row action at the panel boundary.
                .right(px(-6.))
                .top_0()
                .h_full()
                .w(px(6.))
                .cursor_col_resize()
                .on_drag(DraggedSidebar, |dragged, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| dragged.clone())
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
        ))
}

fn header(space_switcher: AnyElement, on: Emit) -> impl IntoElement {
    let settings = on;
    h_flex()
        .h(px(36.))
        .px_2()
        .gap_1()
        .justify_between()
        .child(h_flex().min_w_0().flex_1().child(space_switcher))
        .child(
            h_flex().gap_px().child(
                IconButton::new("open-settings", IconName::Settings)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Settings"))
                    .on_click(move |_, window, cx| settings(Action::OpenSettings, window, cx)),
            ),
        )
}

fn row(
    index: usize,
    target_index: usize,
    entry: &Entry,
    selected: bool,
    grouped: bool,
    on: Emit,
    cx: &App,
) -> impl IntoElement {
    let close = on.clone();
    let move_tab = on.clone();

    let select = entry.key;
    let close_key = entry.key;
    let close_tab = entry.tab;
    let space = entry.space;
    let close_space = entry.space;
    let target_space_key = entry.space_key.clone();
    let dragged = DraggedItem {
        space: entry.space_key.clone(),
        tab: entry.tab,
        pane: entry.pane,
        // A sidebar row is an outer workspace tab. Its drag index therefore
        // belongs to the space's outer list, not to the representative item's
        // position inside its pane.
        index: target_index,
        item: entry.key,
        top_level: true,
        grouped,
    };
    let close_button_width = IconSize::XSmall.rems() + DynamicSpacing::Base04.rems(cx) * 2.;
    let close_slot_width = close_button_width - DynamicSpacing::Base06.rems(cx);
    let end_slot = h_flex()
        .gap_1()
        .when(grouped, |slot| {
            slot.child(
                Label::new(format!("{} tabs", entry.item_count))
                    .size(UI_LABEL_SMALL)
                    .color(Color::Muted),
            )
        })
        .when(entry.closable, |slot| {
            // Reserve exactly the portion of the button not already covered
            // by ListItem's trailing Base06 inset. The real control is an
            // unclipped overlay at the wrapper level below.
            slot.child(div().w(close_slot_width).flex_none())
        });
    let close_button = entry.closable.then(|| {
        IconButton::new(("close", index), IconName::Close).icon_size(IconSize::XSmall).on_click(
            move |_, window, cx| {
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
            },
        )
    });

    // `ListItem` deliberately owns row visuals and click semantics. This thin
    // wrapper owns sidebar-tab dragging, which Zed's generic row does not.
    div()
        .id(("session-drag", index))
        .relative()
        .group("session")
        .w_full()
        .flex_none()
        .on_drag(dragged, |dragged, offset, _, cx| dragged_item_preview(dragged, offset, cx))
        // Like both earlier Chartr clients, sorting stays within the card/space
        // where the drag began. Pane-local tab drags are rejected as well: this
        // surface only reorders top-level workspace tabs.
        .can_drop(move |value, _, _| {
            value
                .downcast_ref::<DraggedItem>()
                .is_some_and(|dragged| dragged.space == target_space_key && dragged.top_level)
        })
        .drag_over::<DraggedItem>(move |wrapper, dragged, _, cx| {
            let mut wrapper = wrapper
                .bg(cx.theme().colors().drop_target_background)
                .border_color(cx.theme().colors().drop_target_border)
                .border_0();
            if target_index < dragged.index {
                wrapper = wrapper.border_t_2();
            } else if target_index > dragged.index {
                wrapper = wrapper.border_b_2();
            }
            wrapper
        })
        .on_drop(move |dragged: &DraggedItem, window, cx| {
            move_tab(Action::MoveWorkspaceTab { space, tab: dragged.tab, target_index }, window, cx)
        })
        .child(
            selection_row(("session", index), selected)
                .aria_role(Role::Tab)
                .aria_label(if grouped {
                    format!("Pane group: {}", entry.title)
                } else {
                    entry.title.clone()
                })
                .on_click(move |_, window, cx| {
                    on(Action::Select { space: Some(space), item: select }, window, cx)
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
                .child(Label::new(entry.title.clone()).size(UI_LABEL_DEFAULT).truncate())
                .end_slot(end_slot),
        )
        .when_some(close_button, |wrapper, close_button| {
            wrapper.child(
                div()
                    .absolute()
                    .right_0()
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .visible_on_hover("session")
                    // A press on Close belongs to the control, not the row's
                    // drag recognizer; the button handles the resulting click.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(close_button),
            )
        })
}
