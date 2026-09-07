//! Sidebar mode: standalone tabs and pane groups down the left.
//!
//! The mode for many long-lived sessions. There is room here for the things a
//! tab cannot hold — the agent's name under the title, and a close button that
//! is not fighting the title for space — so this chrome shows them.

use gpui::{
    Bounds, BoxShadow, ContentMask, EntityId, MouseButton, Rems, Role, canvas, deferred, fill,
    linear_color_stop, linear_gradient, point, px, relative, size, transparent_black,
};
use ui::{IconButtonShape, ScrollAxes, Scrollbars, Tooltip, WithScrollbar, prelude::*};

use super::Emit;
use super::tab_sorter::{SortableTab, SortableTabList};
use crate::components::SortAxis;
use crate::components::popup_right_click_menu;

use super::{
    Action, DraggedItem, DraggedSidebar, DraggedSpace, Entry, SpaceEntries, item_indicator,
    new_plugin_pane_button,
};
use crate::components::{ContextMenu, SelectionRowBackgrounds, selection_list, selection_row};
use crate::fonts::{UI_LABEL_DEFAULT, UI_LABEL_SMALL};
use crate::settings::{SettingsStore, sidebar_theme_colors};

/// Limits for the resizable sidebar.
pub const MIN_WIDTH: f32 = 108.;
pub const MAX_WIDTH: f32 = 480.;

/// One shared value for layout and FLIP arithmetic. `gap_2` is half a rem;
/// spelling that out keeps card travel equal to the distance layout actually
/// moved it.
pub(crate) const CARD_GAP: Rems = Rems(0.5);
pub type SpaceSorter = crate::components::ListSorter<EntityId>;

pub(super) fn scrollbar_thumb_colors(colors: &theme::ThemeColors) -> [gpui::Hsla; 3] {
    [
        colors.panel_background.blend(colors.text.alpha(0.7)).alpha(1.),
        colors.text.alpha(1.),
        colors.text.alpha(1.),
    ]
}

pub fn render(
    spaces: &[SpaceEntries],
    controls: Option<(AnyElement, AnyElement)>,
    on: Emit,
    sorter: &SpaceSorter,
    width: f32,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
    let [thumb, hovered_thumb, active_thumb] = scrollbar_thumb_colors(colors);
    let sidebar_colors = sidebar_theme_colors(cx.theme());
    let session_backgrounds = SelectionRowBackgrounds {
        hover: sidebar_colors.session_hover,
        selected: sidebar_colors.session_active,
    };
    let mut cards = Vec::with_capacity(spaces.len());
    let mut free_sessions = None;
    let movable_space_count = spaces.iter().filter(|space| !space.is_free).count();
    let add_space = on.clone();
    let spaces_header = h_flex()
        .id("spaces-header")
        .w_full()
        .px_2()
        .pb_2()
        .justify_between()
        .child(Label::new("Spaces").size(UI_LABEL_SMALL).color(Color::Muted))
        .child(
            IconButton::new("new-space", IconName::FolderAdd)
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .aria_label("New Space")
                .tooltip(Tooltip::text("New Space"))
                .on_click(move |_, window, cx| add_space(Action::NewSpace, window, cx)),
        );
    let header = controls
        .map(|(space_switcher, view_menu)| header(space_switcher, view_menu).into_any_element());
    let scroll_handle = sorter.scroll_handle().clone();
    let scroll_background = colors.panel_background;
    let mut index = 0;
    let now = cx.background_executor().now();
    let reduce_motion = cx.reduce_motion();
    for (space_index, space) in spaces.iter().enumerate() {
        let mut contents = Vec::with_capacity(space.entries.len() + 1);
        let activate = on.clone();
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
            .id(("space-drag", space_index))
            .group("space-heading")
            .w_full()
            .min_w_0()
            .pl_1()
            // Keep card actions clear of the overlaid scrollbar's hit target.
            .when(!space.is_free, |heading| heading.pr_2())
            .pt_0()
            .pb_1()
            .justify_between()
            .child(h_flex().min_w_0().flex_1().child(
                Label::new(space.name.clone()).size(UI_LABEL_SMALL).color(Color::Muted).truncate(),
            ))
            .child(
                h_flex()
                    .gap_px()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
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
            .when(!space.is_free && movable_space_count > 1, |handle| {
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
        contents.push(
            SortableTabList::new(
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
            )
            .into_any_element(),
        );

        if space.is_free {
            // Free sessions is a permanent footer, not a space card. Its top
            // border belongs to the sidebar itself and remains visible while
            // the folder-backed cards above it scroll independently.
            free_sessions = Some(
                selection_list()
                    .id("free-sessions")
                    .w_full()
                    .flex_none()
                    .px_1p5()
                    .py_1()
                    .border_t_1()
                    .border_color(colors.border)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| {
                        activate(Action::ActivateSpace { space: space_id }, window, cx)
                    })
                    .children(contents)
                    .into_any_element(),
            );
            continue;
        }

        // A space and its sessions are one object in the sidebar. Keep the
        // plate restrained so it separates neighbouring spaces without
        // turning every session into a nested card; the stronger row fill is
        // then free to keep meaning "selected session". A transparent resting
        // border reserves the active-space ring without changing geometry.
        let held = sorter.holds(space.id);
        let offset = sorter.offset_of(space.id, now, reduce_motion);
        let card = selection_list()
            .id(format!("space-card-{:?}", space.id))
            .relative()
            .w_full()
            .flex_none()
            .cursor_pointer()
            .p_1()
            .rounded_md()
            .border_1()
            .border_color(if space.active { colors.border_selected } else { transparent_black() })
            .bg(if space.active {
                sidebar_colors.card_active
            } else {
                sidebar_colors.card_inactive
            })
            .when(held, |card| card.border_color(colors.drop_target_border).shadow_md())
            .when(offset != px(0.), |card| card.top(offset))
            .on_click(move |_, window, cx| {
                activate(Action::ActivateSpace { space: space_id }, window, cx)
            })
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
        .w(px(width))
        .flex_none()
        .h_full()
        .bg(colors.panel_background)
        .border_r_1()
        .border_color(colors.border)
        .children(header)
        .child(spaces_header)
        .child(
            v_flex()
                .relative()
                .flex_1()
                .min_h_0()
                .child(
                    v_flex()
                        .id("sessions")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .track_scroll(sorter.scroll_handle())
                        .pb_2()
                        .px_1p5()
                        .gap(CARD_GAP)
                        .children(cards),
                )
                .child(
                    // Read the clamped offset at paint time. This passive overlay
                    // adds no hitbox, layout shift, timer, or scroll subscription.
                    canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| {
                            let strength = (-scroll_handle.offset().y / px(12.)).clamp(0., 1.);
                            if strength == 0. {
                                return;
                            }
                            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                                // A small, theme-colored blurred veil gives the
                                // edge a frosted appearance. GPUI has no element
                                // backdrop blur; this needs no offscreen capture.
                                let veil = Bounds::new(
                                    point(bounds.left(), bounds.top() - px(8.)),
                                    size(bounds.size.width, px(8.)),
                                );
                                window.paint_drop_shadows(
                                    veil,
                                    Default::default(),
                                    &[BoxShadow::new(
                                        px(0.),
                                        px(4.),
                                        scroll_background.opacity(0.35 * strength),
                                    )
                                    .blur_radius(px(12.))],
                                );
                                window.paint_quad(fill(
                                    bounds,
                                    linear_gradient(
                                        180.,
                                        linear_color_stop(scroll_background.opacity(strength), 0.),
                                        linear_color_stop(scroll_background.opacity(0.), 1.),
                                    ),
                                ));
                            });
                        },
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(28.))
                    .max_h(relative(1.)),
                )
                .custom_scrollbars(
                    Scrollbars::on_hover(ScrollAxes::Vertical)
                        .id("spaces-scrollbar")
                        .thumb_colors(thumb, hovered_thumb, active_thumb)
                        .tracked_scroll_handle(sorter.scroll_handle())
                        .notify_content(),
                    window,
                    cx,
                ),
        )
        .children(free_sessions)
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

fn header(space_switcher: AnyElement, view_menu: AnyElement) -> impl IntoElement {
    h_flex()
        .h(px(36.))
        .px_2()
        .gap_1()
        .justify_between()
        .child(h_flex().min_w_0().flex_1().child(space_switcher))
        .child(h_flex().gap_px().child(view_menu))
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
        // Like both earlier Chartr clients, sorting stays within the card/space
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
                .start_slot(item_indicator(
                    entry.activity(),
                    entry.icon_path.clone(),
                    entry.grouped,
                    &entry.space_key,
                    entry.key,
                    cx,
                ))
                .child(Label::new(entry.title.clone()).size(UI_LABEL_DEFAULT).truncate())
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
