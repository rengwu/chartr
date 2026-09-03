//! Sidebar mode: standalone tabs and pane groups down the left.
//!
//! The mode for many long-lived sessions. There is room here for the things a
//! tab cannot hold — the agent's name under the title, and a close button that
//! is not fighting the title for space — so this chrome shows them.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use gpui::{
    Bounds, EntityId, MouseButton, Pixels, Point, Rems, Role, ScrollHandle, deferred, point, px,
    transparent_black,
};
use ui::{IconButtonShape, Tooltip, prelude::*};

use super::Emit;
use crate::components::popup_right_click_menu;

use super::{
    Action, DraggedItem, DraggedSidebar, DraggedSpace, Entry, SpaceEntries, dragged_item_preview,
    item_indicator, new_plugin_pane_button,
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
const CARD_GAP: Rems = Rems(0.5);
const FLIP_DURATION: Duration = Duration::from_millis(150);
const AUTOSCROLL_EDGE: Pixels = px(32.);

fn flip_progress(elapsed: Duration) -> f32 {
    let t = (elapsed.as_secs_f32() / FLIP_DURATION.as_secs_f32()).clamp(0., 1.);
    1. - (1. - t).powi(5)
}

fn crossed_index(
    item: EntityId,
    order: &[EntityId],
    geometry: &[(EntityId, Bounds<Pixels>)],
    pointer: Pixels,
) -> Option<usize> {
    let (first, last) = (geometry.first()?, geometry.last()?);
    let target = if pointer < first.1.top() {
        first.0
    } else if pointer > last.1.bottom() {
        last.0
    } else {
        geometry.iter().find(|(_, bounds)| pointer >= bounds.top() && pointer <= bounds.bottom())?.0
    };
    if target == item {
        return None;
    }
    let from = order.iter().position(|id| *id == item)?;
    let to = order.iter().position(|id| *id == target)?;
    let bounds = geometry.iter().find(|(id, _)| *id == target)?.1;
    let beyond = (to == 0 && pointer < bounds.top())
        || (to + 1 == geometry.len() && pointer > bounds.bottom());
    let midpoint = bounds.top() + bounds.size.height * 0.5;
    (beyond || if to > from { pointer > midpoint } else { pointer < midpoint }).then_some(to)
}

fn closest_index(
    item: EntityId,
    geometry: &[(EntityId, Bounds<Pixels>)],
    pointer: Pixels,
) -> usize {
    geometry
        .iter()
        .filter(|(id, bounds)| *id != item && pointer >= bounds.top() + bounds.size.height * 0.5)
        .count()
}

struct HeldSpace {
    item: EntityId,
    order: Vec<EntityId>,
    anchor: Pixels,
    pointer: Pixels,
    slot_moved: Pixels,
    scroll_y: Pixels,
}

impl HeldSpace {
    fn carried(&self) -> Pixels {
        (self.pointer - self.anchor) - self.slot_moved
    }
}

struct Slide {
    from: Pixels,
    at: Instant,
}

/// Window-only state for sorting the sidebar's variable-height space cards.
///
/// The model order is not changed until release. While a drag is active this
/// owns the temporary id order, measured card heights, interruptible FLIP
/// offsets, and the tracked scroll geometry needed to keep sorting alive after
/// the pointer leaves the sidebar horizontally.
pub struct SpaceSorter {
    sizes: HashMap<EntityId, Pixels>,
    slides: HashMap<EntityId, Slide>,
    held: Option<HeldSpace>,
    pressed_at: Option<Pixels>,
    scroll: ScrollHandle,
    last_tick: Option<Instant>,
}

impl Default for SpaceSorter {
    fn default() -> Self {
        Self {
            sizes: HashMap::new(),
            slides: HashMap::new(),
            held: None,
            pressed_at: None,
            scroll: ScrollHandle::new(),
            last_tick: None,
        }
    }
}

impl SpaceSorter {
    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub fn press(&mut self, at: Pixels) {
        self.pressed_at = Some(at);
    }

    pub fn holds(&self, item: EntityId) -> bool {
        self.held.as_ref().is_some_and(|held| held.item == item)
    }

    pub fn arrange<T>(&self, items: &mut [T], id: impl Fn(&T) -> EntityId) {
        let Some(held) = &self.held else {
            return;
        };
        items.sort_by_key(|item| {
            let id = id(item);
            held.order.iter().position(|known| *known == id).unwrap_or(usize::MAX)
        });
    }

    pub fn offset_of(&self, item: EntityId, now: Instant, reduce_motion: bool) -> Pixels {
        if let Some(held) = &self.held
            && held.item == item
        {
            return held.carried();
        }
        if reduce_motion {
            return px(0.);
        }
        self.slides
            .get(&item)
            .map(|slide| slide.from * (1. - flip_progress(now.saturating_duration_since(slide.at))))
            .unwrap_or(px(0.))
    }

    pub fn drag_move(
        &mut self,
        dragged: EntityId,
        model_order: Vec<EntityId>,
        pointer: Point<Pixels>,
        rem: Pixels,
        now: Instant,
        reduce_motion: bool,
    ) -> bool {
        if self.held.is_none() {
            if !model_order.contains(&dragged) {
                return false;
            }
            self.held = Some(HeldSpace {
                item: dragged,
                order: model_order,
                anchor: self.pressed_at.take().unwrap_or(pointer.y),
                pointer: pointer.y,
                slot_moved: px(0.),
                scroll_y: self.scroll.offset().y,
            });
            self.last_tick = Some(now);
        }
        let Some(held) = self.held.as_mut() else {
            return false;
        };
        if held.item != dragged {
            return false;
        }
        let changed = held.pointer != pointer.y;
        held.pointer = pointer.y;
        self.sync_scroll();
        self.cross_midpoint(rem, now, reduce_motion) || changed
    }

    /// Advances both FLIP and edge autoscroll. Returns whether the window owes
    /// the sorter another animation frame.
    pub fn tick(&mut self, now: Instant, rem: Pixels, reduce_motion: bool) -> bool {
        if reduce_motion {
            self.slides.clear();
        } else {
            self.slides.retain(|_, slide| now.saturating_duration_since(slide.at) < FLIP_DURATION);
        }

        let mut scrolling = false;
        if self.held.is_some() {
            self.sync_scroll();
            scrolling = self.autoscroll(now);
            if scrolling {
                self.sync_scroll();
                self.cross_midpoint(rem, now, reduce_motion);
            }
        } else {
            self.last_tick = None;
        }
        scrolling || !self.slides.is_empty()
    }

    /// Resolves the release's final Y even if no last drag-move event reached
    /// the sidebar, then returns the model move that should be committed.
    pub fn drop_at(
        &mut self,
        pointer_y: Pixels,
        rem: Pixels,
        now: Instant,
        reduce_motion: bool,
    ) -> Option<(EntityId, usize)> {
        let held = self.held.as_mut()?;
        held.pointer = pointer_y;
        self.sync_scroll();
        if let Some(to) = self.nearest_index(pointer_y) {
            self.reorder_held(to, rem, now, reduce_motion);
        }
        let held = self.held.as_ref()?;
        let to = held.order.iter().position(|id| *id == held.item)?;
        Some((held.item, to))
    }

    pub fn accept_drop(&mut self, now: Instant, reduce_motion: bool) {
        let Some(held) = self.held.take() else {
            return;
        };
        let carried = held.carried();
        if !reduce_motion && carried != px(0.) {
            self.slides.insert(held.item, Slide { from: carried, at: now });
        }
        self.pressed_at = None;
        self.last_tick = None;
    }

    pub fn cancel(&mut self) {
        self.held = None;
        self.pressed_at = None;
        self.slides.clear();
        self.last_tick = None;
    }

    fn sync_scroll(&mut self) {
        let Some(held) = self.held.as_mut() else {
            return;
        };
        let scroll_y = self.scroll.offset().y;
        if scroll_y != held.scroll_y {
            held.slot_moved += scroll_y - held.scroll_y;
            held.scroll_y = scroll_y;
        }
    }

    fn geometry(&mut self) -> Option<Vec<(EntityId, Bounds<Pixels>)>> {
        let order = self.held.as_ref()?.order.clone();
        if self.scroll.children_count() != order.len() {
            return None;
        }
        let scroll_y = self.scroll.offset().y;
        let mut geometry = Vec::with_capacity(order.len());
        for (index, id) in order.into_iter().enumerate() {
            let mut bounds = self.scroll.bounds_for_item(index)?;
            bounds.origin.y += scroll_y;
            self.sizes.insert(id, bounds.size.height);
            geometry.push((id, bounds));
        }
        Some(geometry)
    }

    fn cross_midpoint(&mut self, rem: Pixels, now: Instant, reduce_motion: bool) -> bool {
        let Some(pointer) = self.held.as_ref().map(|held| held.pointer) else {
            return false;
        };
        let Some(geometry) = self.geometry() else {
            return false;
        };
        let Some(held) = self.held.as_ref() else {
            return false;
        };
        let Some(to) = crossed_index(held.item, &held.order, &geometry, pointer) else {
            return false;
        };
        self.reorder_held(to, rem, now, reduce_motion)
    }

    fn nearest_index(&mut self, pointer: Pixels) -> Option<usize> {
        let item = self.held.as_ref()?.item;
        let geometry = self.geometry()?;
        Some(closest_index(item, &geometry, pointer))
    }

    fn reorder_held(&mut self, to: usize, rem: Pixels, now: Instant, reduce_motion: bool) -> bool {
        let Some(held) = self.held.as_mut() else {
            return false;
        };
        let Some(from) = held.order.iter().position(|id| *id == held.item) else {
            return false;
        };
        let to = to.min(held.order.len().saturating_sub(1));
        if from == to {
            return false;
        }
        let before = held.order.clone();
        let item = held.order.remove(from);
        held.order.insert(to, item);
        let after = held.order.clone();
        self.start_slides(&before, &after, rem, now, reduce_motion);
        true
    }

    fn layout(&self, order: &[EntityId], gap: Pixels) -> Option<HashMap<EntityId, Pixels>> {
        let mut y = px(0.);
        let mut tops = HashMap::with_capacity(order.len());
        for item in order {
            tops.insert(*item, y);
            y += *self.sizes.get(item)? + gap;
        }
        Some(tops)
    }

    fn start_slides(
        &mut self,
        before: &[EntityId],
        after: &[EntityId],
        rem: Pixels,
        now: Instant,
        reduce_motion: bool,
    ) {
        let gap = CARD_GAP.to_pixels(rem);
        let (Some(was), Some(current)) = (self.layout(before, gap), self.layout(after, gap)) else {
            return;
        };
        if let Some(held) = self.held.as_mut()
            && let (Some(was), Some(current)) = (was.get(&held.item), current.get(&held.item))
        {
            held.slot_moved += *current - *was;
        }
        if reduce_motion {
            self.slides.clear();
            return;
        }
        let column = after
            .iter()
            .filter_map(|item| Some(*current.get(item)? + *self.sizes.get(item)?))
            .fold(px(0.), Pixels::max)
            .max(px(0.));
        for item in after {
            if self.holds(*item) {
                continue;
            }
            let (Some(was), Some(current)) = (was.get(item), current.get(item)) else {
                continue;
            };
            let from =
                ((*was - *current) + self.offset_of(*item, now, false)).clamp(-column, column);
            if from == px(0.) {
                self.slides.remove(item);
            } else {
                self.slides.insert(*item, Slide { from, at: now });
            }
        }
    }

    fn autoscroll(&mut self, now: Instant) -> bool {
        let Some(held) = self.held.as_ref() else {
            return false;
        };
        let viewport = self.scroll.bounds();
        if viewport.size.height <= px(0.) {
            return false;
        }
        let edge = AUTOSCROLL_EDGE.min(viewport.size.height / 3.);
        let top = viewport.top() + edge;
        let bottom = viewport.bottom() - edge;
        let pointer = held.pointer;
        let direction = if pointer < top {
            1.
        } else if pointer > bottom {
            -1.
        } else {
            self.last_tick = Some(now);
            return false;
        };
        let distance = if direction > 0. { top - pointer } else { pointer - bottom };
        // Zed's editor uses the same capped nonlinear curve for selection
        // autoscroll. Scale it by elapsed frames so speed is stable if a frame
        // is delayed.
        let speed: f32 = (distance.pow(1.2) / 100.).min(px(3.)).into();
        let elapsed = self
            .last_tick
            .replace(now)
            .map(|last| now.saturating_duration_since(last).as_secs_f32())
            .unwrap_or_default();
        let frame_scale = (elapsed * 60.).clamp(0., 3.);
        let offset = self.scroll.offset();
        let max = self.scroll.max_offset().y;
        let next = (offset.y + px(direction * speed * frame_scale)).clamp(-max, px(0.));
        if next == offset.y {
            return false;
        }
        self.scroll.set_offset(point(offset.x, next));
        true
    }
}

pub fn render(
    spaces: &[SpaceEntries],
    controls: Option<(AnyElement, AnyElement)>,
    on: Emit,
    sorter: &SpaceSorter,
    width: f32,
    cx: &App,
) -> impl IntoElement {
    let colors = cx.theme().colors();
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
                    .child(
                        IconButton::new(("new-in-space", space_index), IconName::Plus)
                            .icon_size(IconSize::XSmall)
                            .tooltip(Tooltip::text("New session in this space"))
                            .on_click(move |_, window, cx| {
                                add(Action::NewInSpace { space: space_id }, window, cx)
                            }),
                    )
                    .child(
                        new_plugin_pane_button(
                            ("new-plugin-pane-in-space", space_index),
                            IconSize::XSmall,
                        )
                        .on_click(move |_, window, cx| {
                            add_plugin(Action::NewPluginPaneInSpace { space: space_id }, window, cx)
                        }),
                    ),
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
                    let locate = actions.clone();
                    let close = actions.clone();
                    ContextMenu::build_popup(window, cx, move |menu| {
                        let menu = menu.when(!available, |menu| {
                            menu.entry("Locate Space Folder", None, move |window, cx| {
                                locate(Action::LocateSpace { space: action_space }, window, cx)
                            })
                        });
                        menu.when(removable, |menu| {
                            let menu = menu.entry("Rename Space", None, move |window, cx| {
                                rename(Action::RenameSpace { space: action_space }, window, cx)
                            });
                            menu.separator().entry("Close Space", None, move |window, cx| {
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
        for (target_index, entry) in space.entries.iter().enumerate() {
            contents.push(
                row(
                    index,
                    target_index,
                    entry,
                    space.active && entry.selected,
                    entry.grouped,
                    session_backgrounds,
                    on.clone(),
                    cx,
                )
                .into_any_element(),
            );
            index += 1;
        }

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

    // `ListItem` deliberately owns row visuals and click semantics. This thin
    // wrapper owns sidebar-tab dragging, which Zed's generic row does not.
    let row = div()
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

#[cfg(test)]
mod space_sorter_tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, Window, size};

    fn id(value: u64) -> EntityId {
        value.into()
    }

    fn reordered_sorter(reduce_motion: bool) -> (SpaceSorter, Instant) {
        let now = Instant::now();
        let mut sorter = SpaceSorter::default();
        sorter.sizes = [(id(1), px(40.)), (id(2), px(80.)), (id(3), px(20.))].into_iter().collect();
        sorter.held = Some(HeldSpace {
            item: id(2),
            order: vec![id(1), id(2), id(3)],
            anchor: px(100.),
            pointer: px(100.),
            slot_moved: px(0.),
            scroll_y: px(0.),
        });
        assert!(sorter.reorder_held(0, px(16.), now, reduce_motion));
        (sorter, now)
    }

    #[test]
    fn variable_height_reorder_keeps_the_held_card_under_the_pointer() {
        let (sorter, now) = reordered_sorter(false);
        let held = sorter.held.as_ref().unwrap();

        assert_eq!(held.order, vec![id(2), id(1), id(3)]);
        // The held card's new slot starts 48 px earlier (40 px card + 8 px
        // gap), so its transform adds exactly 48 px to keep it stationary.
        assert_eq!(sorter.offset_of(id(2), now, false), px(48.));
        // The displaced 40 px card now has an 80 px card and the gap ahead of
        // it, so FLIP initially draws it at its old position.
        assert_eq!(sorter.offset_of(id(1), now, false), px(-88.));
        assert_eq!(sorter.offset_of(id(3), now, false), px(0.));

        let mut drawn = vec![id(1), id(2), id(3)];
        sorter.arrange(&mut drawn, |item| *item);
        assert_eq!(drawn, held.order);
    }

    #[test]
    fn accepted_drop_settles_for_150ms_and_reduce_motion_skips_flip() {
        let (mut sorter, now) = reordered_sorter(false);
        sorter.accept_drop(now, false);
        assert!(sorter.held.is_none());
        assert_eq!(sorter.offset_of(id(2), now, false), px(48.));
        assert_eq!(sorter.offset_of(id(2), now + FLIP_DURATION, false), px(0.));

        let (mut reduced, now) = reordered_sorter(true);
        assert!(reduced.slides.is_empty());
        // Direct pointer carrying remains spatially correct; only the settle
        // and displaced-card animations are removed.
        assert_eq!(reduced.offset_of(id(2), now, true), px(48.));
        reduced.accept_drop(now, true);
        assert_eq!(reduced.offset_of(id(2), now, true), px(0.));
        assert!(reduced.slides.is_empty());
    }

    #[test]
    fn reversing_an_active_flip_does_not_jump() {
        let (mut sorter, started) = reordered_sorter(false);
        let reversed = started + FLIP_DURATION / 2;
        let before = px(88.) + sorter.offset_of(id(1), reversed, false);

        assert!(sorter.reorder_held(2, px(16.), reversed, false));
        let after = sorter.offset_of(id(1), reversed, false);
        assert!((f32::from(before - after)).abs() < f32::EPSILON);

        let held = sorter.held.as_ref().unwrap();
        assert_eq!(held.order, vec![id(1), id(3), id(2)]);
        // Moving the held slot twice still leaves its painted top at the
        // original 48 px while the pointer itself has not moved.
        assert_eq!(px(76.) + held.carried(), px(48.));
    }

    #[test]
    fn flip_curve_is_quintic_and_bounded() {
        assert_eq!(flip_progress(Duration::ZERO), 0.);
        assert!((flip_progress(FLIP_DURATION / 2) - 0.96875).abs() < f32::EPSILON);
        assert_eq!(flip_progress(FLIP_DURATION), 1.);
        assert_eq!(flip_progress(FLIP_DURATION * 2), 1.);
    }

    #[test]
    fn variable_height_midpoints_and_final_release_y_choose_legal_slots() {
        let geometry = vec![
            (id(1), Bounds::new(point(px(0.), px(10.)), gpui::size(px(200.), px(40.)))),
            (id(2), Bounds::new(point(px(0.), px(58.)), gpui::size(px(200.), px(80.)))),
            (id(3), Bounds::new(point(px(0.), px(146.)), gpui::size(px(200.), px(20.)))),
        ];
        let order = [id(1), id(2), id(3)];

        // The second card is 80 px high, so a card coming from above does not
        // displace it at 98 px exactly and does immediately after that point.
        assert_eq!(crossed_index(id(1), &order, &geometry, px(98.)), None);
        assert_eq!(crossed_index(id(1), &order, &geometry, px(98.1)), Some(1));
        // Empty space beyond the column belongs unambiguously to its end.
        assert_eq!(crossed_index(id(2), &order, &geometry, px(-100.)), Some(0));
        assert_eq!(crossed_index(id(2), &order, &geometry, px(500.)), Some(2));

        // Release uses Y alone. These coordinates can just as well have come
        // from the workspace to the right of the sidebar.
        assert_eq!(closest_index(id(2), &geometry, px(-100.)), 0);
        assert_eq!(closest_index(id(2), &geometry, px(100.)), 1);
        assert_eq!(closest_index(id(2), &geometry, px(500.)), 2);
    }

    struct ScrollHarness {
        sorter: SpaceSorter,
    }

    impl Render for ScrollHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            v_flex()
                .id("sorter-scroll-harness")
                .w(px(200.))
                .h(px(100.))
                .overflow_y_scroll()
                .track_scroll(self.sorter.scroll_handle())
                .gap(CARD_GAP)
                .children([
                    div().h(px(80.)).flex_none(),
                    div().h(px(80.)).flex_none(),
                    div().h(px(80.)).flex_none(),
                ])
        }
    }

    #[gpui::test]
    fn edge_autoscroll_uses_tracked_bounds_without_letting_the_card_drift(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(240.), px(140.)), |_, _| ScrollHarness {
            sorter: SpaceSorter::default(),
        });
        cx.run_until_parked();

        window
            .update(cx, |harness, _, _| {
                let viewport = harness.sorter.scroll.bounds();
                assert!(harness.sorter.scroll.max_offset().y > px(0.));
                let started = Instant::now();
                let pointer = viewport.bottom() + px(20.);
                harness.sorter.held = Some(HeldSpace {
                    item: id(1),
                    order: vec![id(1), id(2), id(3)],
                    anchor: pointer,
                    pointer,
                    slot_moved: px(0.),
                    scroll_y: px(0.),
                });
                harness.sorter.last_tick = Some(started);

                assert!(harness.sorter.tick(started + Duration::from_millis(16), px(16.), false));
                let scroll_y = harness.sorter.scroll.offset().y;
                assert!(scroll_y < px(0.), "a pointer below the viewport scrolls down");
                let held = harness.sorter.held.as_ref().unwrap();
                assert_eq!(held.carried(), -scroll_y);
            })
            .unwrap();
    }
}
