//! Sidebar mode: the session list down the left.
//!
//! The mode for many long-lived sessions. There is room here for the things a
//! tab cannot hold — the agent's name under the title, and a close button that
//! is not fighting the title for space — so this chrome shows them.

use gpui::{MouseButton, Role, deferred};
use ui::{Tooltip, prelude::*};

use super::Emit;

use super::{Action, DraggedItem, DraggedSidebar, Entry, SpaceEntries, status_dot};

/// The sidebar's width. Fixed rather than draggable: a resizable sidebar is a
/// preference to persist, a drag handle to hit-test, and a minimum to enforce,
/// and none of that is what makes this mode useful.
pub const DEFAULT_WIDTH: f32 = 280.;
pub const MIN_WIDTH: f32 = 180.;
pub const MAX_WIDTH: f32 = 480.;

pub fn render(
    spaces: &[SpaceEntries],
    space_switcher: AnyElement,
    new_item: AnyElement,
    on: Emit,
    width: f32,
    cx: &App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
    let mut groups = Vec::new();
    let mut index = 0;
    for space in spaces {
        let add = on.clone();
        let close = on.clone();
        let rename = on.clone();
        let locate = on.clone();
        let space_id = space.id;
        let close_space = space.id;
        let rename_space = space.id;
        let locate_space = space.id;
        groups.push(
            h_flex()
                .group("space-heading")
                .px_2()
                .pt_2()
                .pb_1()
                .justify_between()
                .child(Label::new(space.name.clone()).size(LabelSize::XSmall).color(Color::Muted))
                .child(
                    h_flex()
                        .gap_px()
                        .child(
                            IconButton::new(("new-in-space", index), IconName::Plus)
                                .icon_size(IconSize::XSmall)
                                .tooltip(Tooltip::text("New session in this space"))
                                .on_click(move |_, window, cx| {
                                    add(Action::NewInSpace { space: space_id }, window, cx)
                                }),
                        )
                        .when(space.removable, |controls| {
                            controls.child(
                                IconButton::new(("rename-space", index), IconName::Pencil)
                                    .icon_size(IconSize::XSmall)
                                    .tooltip(Tooltip::text("Rename Space"))
                                    .on_click(move |_, window, cx| {
                                        rename(
                                            Action::RenameSpace { space: rename_space },
                                            window,
                                            cx,
                                        )
                                    }),
                            )
                        })
                        .when(!space.available, |controls| {
                            controls.child(
                                IconButton::new(("locate-space", index), IconName::FolderOpen)
                                    .icon_size(IconSize::XSmall)
                                    .tooltip(Tooltip::text("Locate Space Folder"))
                                    .on_click(move |_, window, cx| {
                                        locate(
                                            Action::LocateSpace { space: locate_space },
                                            window,
                                            cx,
                                        )
                                    }),
                            )
                        })
                        .when(space.removable, |controls| {
                            controls.child(
                                IconButton::new(("close-space", index), IconName::Close)
                                    .icon_size(IconSize::XSmall)
                                    .tooltip(Tooltip::text("Close Space"))
                                    .on_click(move |_, window, cx| {
                                        close(Action::CloseSpace { space: close_space }, window, cx)
                                    }),
                            )
                        }),
                )
                .into_any_element(),
        );
        if space.panes.is_empty() {
            groups.push(
                div()
                    .px_2()
                    .py_1()
                    .child(Label::new("No open tabs").size(LabelSize::XSmall).color(Color::Muted))
                    .into_any_element(),
            );
            continue;
        }
        for pane in &space.panes {
            let count = pane.entries.len();
            let close = on.clone();
            let move_item = on.clone();
            let close_space = space.id;
            let move_space = space.id;
            let pane_id = pane.id;
            groups.push(
                h_flex()
                    .id(format!("sidebar-pane-drop-{}-{}", index, pane.id.get()))
                    .group("sidebar-pane-heading")
                    .px_2()
                    .py_1()
                    .justify_between()
                    .border_1()
                    .border_color(colors.border_variant)
                    .rounded_sm()
                    .on_drop(move |dragged: &DraggedItem, window, cx| {
                        if dragged.space_entity != Some(move_space) || dragged.pane == pane_id {
                            return;
                        }
                        move_item(
                            Action::MoveToPane {
                                space: move_space,
                                item: dragged.item,
                                source: dragged.pane,
                                target: pane_id,
                            },
                            window,
                            cx,
                        )
                    })
                    .child(
                        Label::new(format!("Pane {}", pane.id.get()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Label::new(format!(
                                    "{count} tab{}",
                                    if count == 1 { "" } else { "s" }
                                ))
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                            )
                            .child(
                                div().visible_on_hover("sidebar-pane-heading").child(
                                    IconButton::new(
                                        ("close-sidebar-pane", pane.id.get()),
                                        IconName::Close,
                                    )
                                    .icon_size(IconSize::XSmall)
                                    .tooltip(Tooltip::text("Close All in Pane"))
                                    .on_click(
                                        move |_, window, cx| {
                                            close(
                                                Action::ClosePane {
                                                    space: close_space,
                                                    pane: pane_id,
                                                },
                                                window,
                                                cx,
                                            )
                                        },
                                    ),
                                ),
                            ),
                    )
                    .into_any_element(),
            );
            if pane.entries.is_empty() {
                groups.push(
                    div()
                        .px_3()
                        .py_1()
                        .child(
                            Label::new("Empty pane — drop a tab here")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                        .into_any_element(),
                );
            }
            for entry in &pane.entries {
                groups.push(row(index, entry, on.clone(), cx).into_any_element());
                index += 1;
            }
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
        .child(header(space_switcher, new_item, on.clone()))
        .child(v_flex().id("sessions").flex_1().overflow_y_scroll().p_1().gap_px().children(groups))
        .child(deferred(
            div()
                .id("sidebar-resize-handle")
                .absolute()
                .right(px(-3.))
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

fn header(space_switcher: AnyElement, new_item: AnyElement, on: Emit) -> impl IntoElement {
    let toggle = on.clone();
    let scope = on.clone();
    h_flex()
        .h(px(36.))
        .px_2()
        .gap_1()
        .justify_between()
        .child(div().min_w_0().flex_1().child(space_switcher))
        .child(
            h_flex()
                .gap_px()
                .child(new_item)
                .child(
                    IconButton::new("toggle-space-scope", IconName::ListTree)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Show all or active space"))
                        .on_click(move |_, window, cx| {
                            scope(Action::ToggleSidebarScope, window, cx)
                        }),
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

    let select = entry.key;
    let close_key = entry.key;
    let space = entry.space;
    let close_space = entry.space;
    let dragged = DraggedItem {
        space: entry.space_key.clone(),
        space_entity: Some(entry.space),
        pane: entry.pane,
        item: entry.key,
        title: entry.title.clone(),
    };
    h_flex()
        .id(("session", index))
        .role(Role::Tab)
        .aria_label(entry.title.clone())
        .aria_selected(entry.selected)
        .group("session")
        .h(px(38.))
        .px_2()
        .gap_2()
        .rounded_sm()
        .when(entry.selected, |row| row.bg(colors.element_selected))
        .when(!entry.selected, |row| row.hover(|row| row.bg(colors.element_hover)))
        .on_click(move |_, window, cx| {
            on(Action::Select { space: Some(space), item: select }, window, cx)
        })
        .on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
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
        .when(entry.closable, |row| {
            row.child(
                // Revealed on hover so a list of ten sessions is ten titles rather
                // than ten titles and ten buttons.
                div().visible_on_hover("session").child(
                    IconButton::new(("close", index), IconName::Close)
                        .icon_size(IconSize::XSmall)
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            close(
                                Action::Close { space: Some(close_space), item: close_key },
                                window,
                                cx,
                            )
                        }),
                ),
            )
        })
}
