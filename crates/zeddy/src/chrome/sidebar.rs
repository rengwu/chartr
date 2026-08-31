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
        let panes: Vec<_> = space.panes.iter().filter(|pane| !pane.entries.is_empty()).collect();
        if panes.len() <= 1 {
            for entry in panes.into_iter().flat_map(|pane| &pane.entries) {
                groups.push(
                    row(index, entry, space.active && entry.selected, false, on.clone(), cx)
                        .into_any_element(),
                );
                index += 1;
            }
            continue;
        }
        let representative = panes
            .iter()
            .flat_map(|pane| &pane.entries)
            .find(|entry| entry.selected)
            .or_else(|| panes.iter().flat_map(|pane| &pane.entries).next());
        if let Some(entry) = representative {
            groups.push(row(index, entry, space.active, true, on.clone(), cx).into_any_element());
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

fn row(
    index: usize,
    entry: &Entry,
    selected: bool,
    grouped: bool,
    on: Emit,
    cx: &App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
    let close = on.clone();

    let select = entry.key;
    let close_key = entry.key;
    let space = entry.space;
    let close_space = entry.space;
    let dragged = DraggedItem {
        space: entry.space_key.clone(),
        pane: entry.pane,
        index: entry.index,
        item: entry.key,
        title: entry.title.clone(),
        selected,
    };
    h_flex()
        .id(("session", index))
        .role(Role::Tab)
        .aria_label(if grouped {
            format!("Pane group: {}", entry.title)
        } else {
            entry.title.clone()
        })
        .aria_selected(selected)
        .group("session")
        .h(px(38.))
        .px_2()
        .gap_2()
        .rounded_sm()
        .when(selected, |row| row.bg(colors.element_selected))
        .when(!selected, |row| row.hover(|row| row.bg(colors.element_hover)))
        .on_click(move |_, window, cx| {
            on(Action::Select { space: Some(space), item: select }, window, cx)
        })
        .when(!grouped, |row| row.on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone())))
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
                                if grouped {
                                    Action::CloseGroup { space: close_space }
                                } else {
                                    Action::Close { space: Some(close_space), item: close_key }
                                },
                                window,
                                cx,
                            )
                        }),
                ),
            )
        })
}
