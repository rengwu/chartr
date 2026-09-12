//! Sidebar mode: collapsible spaces with indented, sortable session rows.

use gpui::{EntityId, MouseButton, Rems, Role, deferred, px, transparent_black};
use ui::{IconButtonShape, Tooltip, prelude::*};

use super::Emit;
use super::tab_sorter::{SortableTab, SortableTabList};
use crate::components::SortAxis;
use crate::components::popup_right_click_menu;

use super::{
    Action, DraggedItem, DraggedSpace, Entry, SpaceEntries, item_indicator, new_plugin_pane_button,
};
use crate::components::{ContextMenu, SelectionRowBackgrounds, selection_list, selection_row};
use crate::fonts::{UI_LABEL_DEFAULT, UI_LABEL_SMALL};
use crate::settings::{SettingsStore, sidebar_theme_colors};

/// Keep the compact space gap shared by layout and drag-sort animation.
pub(crate) const CARD_GAP: Rems = Rems(0.125);
pub type SpaceSorter = crate::components::ListSorter<EntityId>;

pub fn render(
    spaces: &[SpaceEntries],
    on: Emit,
    sorter: &SpaceSorter,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
    let sidebar_colors = sidebar_theme_colors(cx.theme());
    let session_backgrounds = SelectionRowBackgrounds {
        hover: sidebar_colors.session_hover,
        selected: sidebar_colors.session_active,
    };
    let mut cards = Vec::with_capacity(spaces.len());
    let movable_space_count = spaces.len();
    let add_space = on.clone();
    let spaces_header = h_flex()
        .id("spaces-header")
        .w_full()
        .px_2()
        .pb_2()
        .justify_between()
        .child(Label::new("Spaces").size(UI_LABEL_SMALL).color(Color::Muted))
        .child(
            IconButton::new("new-space", IconName::Plus)
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .aria_label("New Space")
                .tooltip(Tooltip::text("New Space"))
                .on_click(move |_, window, cx| add_space(Action::NewSpace, window, cx)),
        );
    let mut index = 0;
    let now = cx.background_executor().now();
    let reduce_motion = cx.reduce_motion();
    for (space_index, space) in spaces.iter().enumerate() {
        let mut contents = Vec::with_capacity(space.entries.len() + 1);
        let toggle = on.clone();
        let add = on.clone();
        let add_plugin = on.clone();
        let actions = on.clone();
        let space_id = space.id;
        let action_space = space.id;
        let removable = space.removable;
        let available = space.available;
        let space_drag = DraggedSpace(space.id);
        let begin_drag = on.clone();
        let dragging = cx.has_active_drag();
        let title_bar = h_flex()
            .id(format!("space-heading-{space_id:?}"))
            .group("space-heading")
            .w_full()
            .min_w_0()
            .pl_1()
            // Keep heading actions clear of the overlaid scrollbar's hit target.
            .pr_2()
            .py_1()
            .rounded_sm()
            .role(Role::TreeItem)
            .aria_label(space.name.clone())
            .aria_expanded(!space.collapsed)
            .hover(|style| style.bg(session_backgrounds.hover))
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                toggle(Action::ToggleSpaceCollapsed { space: space_id }, window, cx)
            })
            .justify_between()
            .child(
                h_flex()
                    .min_w_0()
                    .flex_1()
                    .gap_1()
                    .child(
                        Icon::new(if space.collapsed {
                            IconName::ChevronRight
                        } else {
                            IconName::ChevronDown
                        })
                        .size(IconSize::Small)
                        .color(Color::Muted),
                    )
                    .child(
                        Label::new(space.name.clone())
                            .size(UI_LABEL_DEFAULT)
                            .weight(gpui::FontWeight::MEDIUM)
                            .truncate(),
                    ),
            )
            .child(
                h_flex()
                    .id(format!("space-heading-actions-{space_id:?}"))
                    .gap_px()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(super::new_item_drag_handle(
                        ("new-in-space", space_index),
                        Some(space_id),
                        super::NewItemKind::Terminal,
                        IconButton::new(("new-in-space", space_index), IconName::Plus)
                            .icon_size(IconSize::XSmall)
                            .tooltip(Tooltip::text("New terminal session"))
                            .on_click(move |_, window, cx| {
                                add(Action::NewInSpace { space: space_id }, window, cx)
                            }),
                    ))
                    .child(super::new_item_drag_handle(
                        ("new-plugin-pane-in-space", space_index),
                        Some(space_id),
                        super::NewItemKind::Plugin,
                        new_plugin_pane_button(
                            ("new-plugin-pane-in-space", space_index),
                            IconSize::XSmall,
                        )
                        .on_click(move |_, window, cx| {
                            add_plugin(Action::NewPluginPaneInSpace { space: space_id }, window, cx)
                        }),
                    )),
            )
            .when(movable_space_count > 1, |handle| {
                handle
                    .when(!dragging, |handle| handle.cursor_grab())
                    .when(dragging, |handle| handle.cursor_grabbing())
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        begin_drag(Action::BeginSpaceDrag { at: event.position.y }, window, cx)
                    })
                    .on_drag(space_drag, |dragged, _, _, cx| {
                        let dragged = *dragged;
                        cx.new(move |_| dragged)
                    })
            });
        let title_bar = if removable || !available {
            popup_right_click_menu(format!("space-actions-{space_index}"))
                .trigger(move |_, _, _| title_bar)
                .menu(move |window, cx| {
                    let rename = actions.clone();
                    let open = actions.clone();
                    let locate = actions.clone();
                    let close = actions.clone();
                    ContextMenu::build_popup(window, cx, move |menu| {
                        let menu = menu
                            .when(available, |menu| {
                                menu.entry("Open Folder", None, move |window, cx| {
                                    open(
                                        Action::OpenSpaceFolder { space: action_space },
                                        window,
                                        cx,
                                    )
                                })
                            })
                            .when(!available, |menu| {
                                menu.entry("Locate Space Folder", None, move |window, cx| {
                                    locate(Action::LocateSpace { space: action_space }, window, cx)
                                })
                            });
                        menu.when(removable, |menu| {
                            let menu = menu.entry("Rename Space", None, move |window, cx| {
                                rename(Action::RenameSpace { space: action_space }, window, cx)
                            });
                            menu.danger_entry("Close Space", move |window, cx| {
                                close(Action::CloseSpace { space: action_space }, window, cx)
                            })
                        })
                    })
                })
                .into_any_element()
        } else {
            title_bar.into_any_element()
        };
        contents.push(title_bar);
        if !space.collapsed {
            let mut tabs = Vec::with_capacity(space.entries.len());
            for (target_index, entry) in space.entries.iter().enumerate() {
                let dragged = DraggedItem {
                    space: entry.space_key.clone(),
                    tab: entry.tab,
                    pane: entry.pane,
                    index: target_index,
                    item: entry.key,
                    top_level: true,
                    grouped: entry.grouped,
                };
                let row = row(
                    index,
                    target_index,
                    entry,
                    space.active && entry.selected,
                    entry.grouped,
                    session_backgrounds,
                    on.clone(),
                    cx,
                );
                tabs.push(SortableTab::new(dragged, entry.selected, move |_, _| row));
                index += 1;
            }
            let move_tab = on.clone();
            let sessions = SortableTabList::new(
                format!("sidebar-tab-sorter-{space_id:?}"),
                v_flex().id(format!("sidebar-tab-list-{space_id:?}")).w_full().flex_none(),
                SortAxis::Vertical,
                gpui::rems(1. / f32::from(window.rem_size())),
                tabs,
                move |dragged, target_index, window, cx| {
                    move_tab(
                        Action::MoveWorkspaceTab {
                            space: space_id,
                            tab: dragged.tab,
                            target_index,
                        },
                        window,
                        cx,
                    );
                },
            );
            contents.push(
                v_flex()
                    // Rows fill the tree; only their contents are indented.
                    .relative()
                    .w_full()
                    .when(space.entries.is_empty(), |tree| {
                        tree.child(
                            div().ml(px(9.)).child(
                                div().px_1p5().py_1().child(
                                    Label::new("Space is empty")
                                        .size(UI_LABEL_SMALL)
                                        .color(Color::Muted)
                                        .truncate(),
                                ),
                            ),
                        )
                    })
                    .when(!space.entries.is_empty(), |tree| tree.child(sessions))
                    // A passive guide paints over the full-width row backgrounds.
                    .child(
                        div()
                            .absolute()
                            .left(px(8.))
                            .top_0()
                            .bottom_0()
                            .w(px(1.))
                            .bg(colors.border_variant),
                    )
                    .into_any_element(),
            );
        }

        // Move the entire expanded or collapsed tree with its heading.
        let held = sorter.holds(space.id);
        let offset = sorter.offset_of(space.id, now, reduce_motion);
        let card = selection_list()
            .id(format!("space-card-{:?}", space.id))
            .relative()
            .w_full()
            .flex_none()
            .rounded_sm()
            .when(held, |tree| tree.bg(colors.panel_background).shadow_md())
            .when(offset != px(0.), |tree| tree.top(offset))
            .children(contents);
        cards.push(
            div()
                .id(format!("space-slot-{:?}", space.id))
                .relative()
                .w_full()
                .flex_none()
                .child(if held {
                    deferred(card).into_any_element()
                } else {
                    card.into_any_element()
                })
                .into_any_element(),
        );
    }

    v_flex()
        .id("spaces-sidebar")
        .relative()
        .w_full()
        .min_h_0()
        .h_full()
        .bg(colors.panel_background)
        .child(spaces_header)
        .child(crate::components::scrolling_list(
            "spaces-scrollbar",
            v_flex().id("sessions").pb_2().px_1p5().gap(CARD_GAP).children(cards),
            sorter.scroll_handle(),
            window,
            cx,
        ))
}

fn row(
    index: usize,
    target_index: usize,
    entry: &Entry,
    selected: bool,
    grouped: bool,
    backgrounds: SelectionRowBackgrounds,
    on: Emit,
    cx: &App,
) -> AnyElement {
    let close = on.clone();
    let middle_close = on.clone();
    let ungroup = on.clone();
    let rename = on.clone();
    let move_tab = on.clone();

    let select = entry.key;
    let close_key = entry.key;
    let close_tab = entry.tab;
    let space = entry.space;
    let close_space = entry.space;
    let target_space_key = entry.space_key.clone();
    let settings = cx.global::<SettingsStore>().resolved();
    let middle_click_closes_tab = settings.middle_click_closes_tab
        && settings.middle_click_closes_sidebar_tab
        && entry.closable;
    let close_button_width = IconSize::XSmall.rems() + DynamicSpacing::Base04.rems(cx) * 2.;
    let close_slot_width = close_button_width - DynamicSpacing::Base06.rems(cx);
    let end_slot = h_flex().when(entry.closable, |slot| {
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

    // `ListItem` owns row visuals and click semantics. This wrapper supplies
    // the close overlay and fallback drop target; SortableTabList owns sorting.
    let row = div()
        .id(format!("session-drag-{space:?}-{}", entry.tab.get()))
        .relative()
        .group("session")
        .w_full()
        .flex_none()
        .rounded_sm()
        .border_1()
        .border_color(if selected {
            cx.theme().colors().border_selected
        } else {
            transparent_black()
        })
        // Like both earlier chartr clients, sorting stays within the card/space
        // where the drag began. Pane-local tab drags are rejected as well: this
        // surface only reorders top-level workspace tabs.
        .can_drop(move |value, _, _| {
            value
                .downcast_ref::<DraggedItem>()
                .is_some_and(|dragged| dragged.space == target_space_key && dragged.top_level)
        })
        .on_drop(move |dragged: &DraggedItem, window, cx| {
            move_tab(Action::MoveWorkspaceTab { space, tab: dragged.tab, target_index }, window, cx)
        })
        .child(
            selection_row(format!("session-{space:?}-{}", entry.tab.get()), selected)
                .backgrounds(backgrounds)
                .aria_role(Role::Tab)
                .aria_label(if grouped {
                    format!("Pane group: {}", entry.title)
                } else {
                    entry.title.clone()
                })
                .on_click(move |_, window, cx| {
                    on(Action::Select { space: Some(space), item: select }, window, cx)
                })
                .start_slot(div().ml(px(9.)).child(item_indicator(
                    super::Activity::default(),
                    entry.icon_path.clone().or_else(|| Some("icons/tool_terminal.svg".into())),
                    entry.grouped,
                    &entry.space_key,
                    entry.key,
                    cx,
                )))
                .child(Label::new(entry.title.clone()).size(UI_LABEL_DEFAULT).truncate())
                .when(
                    entry.status.is_some() || entry.ended || entry.bell || entry.process_running,
                    |row| {
                        row.child(item_indicator(
                            entry.activity(),
                            None,
                            false,
                            &entry.space_key,
                            entry.key,
                            cx,
                        ))
                    },
                )
                .end_slot(end_slot),
        )
        .when(middle_click_closes_tab, |row| {
            row.on_aux_click(move |event, window, cx| {
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
        .when_some(close_button, |wrapper, close_button| {
            wrapper.child(
                div()
                    .absolute()
                    .right_1()
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
        });

    if grouped {
        popup_right_click_menu(format!("group-row-menu-{space:?}-{}", close_tab.get()))
            .trigger(move |_, _, _| row)
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
        row.into_any_element()
    }
}
