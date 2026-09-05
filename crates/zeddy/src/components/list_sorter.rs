//! Shared live list sorting with interruptible FLIP animation and edge scrolling.

use gpui::{Bounds, Pixels, Point, Rems, ScrollHandle, point, px};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

const FLIP_DURATION: Duration = Duration::from_millis(150);
const AUTOSCROLL_EDGE: Pixels = px(32.);

fn flip_progress(elapsed: Duration) -> f32 {
    let t = (elapsed.as_secs_f32() / FLIP_DURATION.as_secs_f32()).clamp(0., 1.);
    1. - (1. - t).powi(5)
}

fn crossed_index<K: Clone + Eq>(
    item: K,
    order: &[K],
    geometry: &[(K, Bounds<Pixels>)],
    pointer: Pixels,
) -> Option<usize> {
    let (first, last) = (geometry.first()?, geometry.last()?);
    let target = if pointer < first.1.top() {
        first.0.clone()
    } else if pointer > last.1.bottom() {
        last.0.clone()
    } else {
        geometry
            .iter()
            .find(|(_, bounds)| pointer >= bounds.top() && pointer <= bounds.bottom())?
            .0
            .clone()
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

fn closest_index<K: Eq>(item: K, geometry: &[(K, Bounds<Pixels>)], pointer: Pixels) -> usize {
    geometry
        .iter()
        .filter(|(id, bounds)| *id != item && pointer >= bounds.top() + bounds.size.height * 0.5)
        .count()
}

struct HeldItem<K> {
    item: K,
    order: Vec<K>,
    anchor: Pixels,
    pointer: Pixels,
    slot_moved: Pixels,
    scroll_y: Pixels,
}

impl<K> HeldItem<K> {
    fn carried(&self) -> Pixels {
        (self.pointer - self.anchor) - self.slot_moved
    }
}

struct Slide {
    from: Pixels,
    at: Instant,
}

/// Window-only state for sorting variable-height rows and cards.
///
/// The model order is not changed until release. While a drag is active this
/// owns the temporary id order, measured card heights, interruptible FLIP
/// offsets, and the tracked scroll geometry needed to keep sorting alive after
/// the pointer leaves the list horizontally.
pub struct ListSorter<K> {
    gap: Rems,
    sizes: HashMap<K, Pixels>,
    slides: HashMap<K, Slide>,
    held: Option<HeldItem<K>>,
    pressed_at: Option<Pixels>,
    scroll: ScrollHandle,
    last_tick: Option<Instant>,
}

impl<K: Clone + Eq + std::hash::Hash> Default for ListSorter<K> {
    fn default() -> Self {
        Self {
            gap: Rems(0.5),
            sizes: HashMap::new(),
            slides: HashMap::new(),
            held: None,
            pressed_at: None,
            scroll: ScrollHandle::new(),
            last_tick: None,
        }
    }
}

impl<K: Clone + Eq + std::hash::Hash> ListSorter<K> {
    pub fn new(gap: Rems) -> Self {
        Self { gap, ..Self::default() }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub fn press(&mut self, at: Pixels) {
        self.pressed_at = Some(at);
    }

    pub fn holds(&self, item: K) -> bool {
        self.held.as_ref().is_some_and(|held| held.item == item)
    }

    pub fn is_dragging(&self) -> bool {
        self.held.is_some()
    }

    pub fn arrange<T>(&self, items: &mut [T], id: impl Fn(&T) -> K) {
        let Some(held) = &self.held else {
            return;
        };
        items.sort_by_key(|item| {
            let id = id(item);
            held.order.iter().position(|known| *known == id).unwrap_or(usize::MAX)
        });
    }

    pub fn offset_of(&self, item: K, now: Instant, reduce_motion: bool) -> Pixels {
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
        dragged: K,
        model_order: Vec<K>,
        pointer: Point<Pixels>,
        rem: Pixels,
        now: Instant,
        reduce_motion: bool,
    ) -> bool {
        if self.held.is_none() {
            if !model_order.contains(&dragged) {
                return false;
            }
            self.held = Some(HeldItem {
                item: dragged.clone(),
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
    ) -> Option<(K, usize)> {
        let held = self.held.as_mut()?;
        held.pointer = pointer_y;
        self.sync_scroll();
        if let Some(to) = self.nearest_index(pointer_y) {
            self.reorder_held(to, rem, now, reduce_motion);
        }
        let held = self.held.as_ref()?;
        let to = held.order.iter().position(|id| *id == held.item)?;
        Some((held.item.clone(), to))
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

    fn geometry(&mut self) -> Option<Vec<(K, Bounds<Pixels>)>> {
        let order = self.held.as_ref()?.order.clone();
        if self.scroll.children_count() != order.len() {
            return None;
        }
        let scroll_y = self.scroll.offset().y;
        let mut geometry = Vec::with_capacity(order.len());
        for (index, id) in order.into_iter().enumerate() {
            let mut bounds = self.scroll.bounds_for_item(index)?;
            bounds.origin.y += scroll_y;
            self.sizes.insert(id.clone(), bounds.size.height);
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
        let Some(to) = crossed_index(held.item.clone(), &held.order, &geometry, pointer) else {
            return false;
        };
        self.reorder_held(to, rem, now, reduce_motion)
    }

    fn nearest_index(&mut self, pointer: Pixels) -> Option<usize> {
        let item = self.held.as_ref()?.item.clone();
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

    fn layout(&self, order: &[K], gap: Pixels) -> Option<HashMap<K, Pixels>> {
        let mut y = px(0.);
        let mut tops = HashMap::with_capacity(order.len());
        for item in order {
            tops.insert(item.clone(), y);
            y += *self.sizes.get(item)? + gap;
        }
        Some(tops)
    }

    fn start_slides(
        &mut self,
        before: &[K],
        after: &[K],
        rem: Pixels,
        now: Instant,
        reduce_motion: bool,
    ) {
        let gap = self.gap.to_pixels(rem);
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
            if self.holds(item.clone()) {
                continue;
            }
            let (Some(was), Some(current)) = (was.get(item), current.get(item)) else {
                continue;
            };
            let from = ((*was - *current) + self.offset_of(item.clone(), now, false))
                .clamp(-column, column);
            if from == px(0.) {
                self.slides.remove(item);
            } else {
                self.slides.insert(item.clone(), Slide { from, at: now });
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

#[cfg(test)]
mod space_sorter_tests {
    use super::*;
    use gpui::{Context, EntityId, Render, TestAppContext, Window, size};
    use ui::prelude::*;
    type SpaceSorter = ListSorter<EntityId>;
    const CARD_GAP: Rems = Rems(0.5);

    fn id(value: u64) -> EntityId {
        value.into()
    }

    fn reordered_sorter(reduce_motion: bool) -> (SpaceSorter, Instant) {
        let now = Instant::now();
        let mut sorter = SpaceSorter::default();
        sorter.sizes = [(id(1), px(40.)), (id(2), px(80.)), (id(3), px(20.))].into_iter().collect();
        sorter.held = Some(HeldItem {
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
                harness.sorter.held = Some(HeldItem {
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
