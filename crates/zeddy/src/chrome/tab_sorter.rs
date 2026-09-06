//! Optimistic tab ordering shared by the outer strip, pane headers, and sidebar.

use gpui::{DragMoveEvent, Entity, MouseButton, RenderOnce, Stateful, deferred};
use ui::prelude::*;

use super::{DraggedItem, dragged_item_preview};
use crate::components::{ListSorter, SortAxis};

type Sorter = ListSorter<u64>;
type Commit = Box<dyn Fn(&DraggedItem, usize, &mut Window, &mut App)>;

pub(super) fn same_list(left: &DraggedItem, right: &DraggedItem) -> bool {
    left.space == right.space
        && left.top_level == right.top_level
        && (left.top_level || (left.tab == right.tab && left.pane == right.pane))
}

fn key(dragged: &DraggedItem) -> u64 {
    if dragged.top_level { dragged.tab.get() } else { dragged.item.get() }
}

pub(crate) struct Placement {
    pub index: usize,
    pub count: usize,
    pub active_index: Option<usize>,
    pub previous: Option<u64>,
}

pub(crate) struct SortableTab {
    dragged: DraggedItem,
    selected: bool,
    render: Box<dyn FnOnce(Placement, &mut App) -> AnyElement>,
}

impl SortableTab {
    pub fn new(
        dragged: DraggedItem,
        selected: bool,
        render: impl FnOnce(Placement, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self { dragged, selected, render: Box::new(render) }
    }
}

#[derive(IntoElement)]
pub(crate) struct SortableTabList {
    list: Stateful<Div>,
    id: String,
    axis: SortAxis,
    gap: gpui::Rems,
    tabs: Vec<SortableTab>,
    commit: Commit,
    drag_lane: Option<(Stateful<Div>, AnyElement)>,
}

impl SortableTabList {
    pub fn new(
        id: String,
        list: Stateful<Div>,
        axis: SortAxis,
        gap: gpui::Rems,
        tabs: Vec<SortableTab>,
        commit: impl Fn(&DraggedItem, usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self { list, id, axis, gap, tabs, commit: Box::new(commit), drag_lane: None }
    }

    /// Keep sorting over the entire strip, including pinned controls and empty
    /// space, while only the tab list participates in scrolling and geometry.
    pub fn drag_lane(mut self, lane: Stateful<Div>, end_slot: impl IntoElement) -> Self {
        self.drag_lane = Some((lane, end_slot.into_any_element()));
        self
    }
}

impl RenderOnce for SortableTabList {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let axis = self.axis;
        let sorter: Entity<Sorter> =
            window.use_keyed_state(self.id, cx, |_, _| ListSorter::with_axis(self.gap, axis));
        let order: Vec<_> = self.tabs.iter().map(|tab| key(&tab.dragged)).collect();
        let payloads: std::collections::HashMap<_, _> =
            self.tabs.iter().map(|tab| (key(&tab.dragged), tab.dragged.clone())).collect();
        let source = self.tabs.first().map(|tab| tab.dragged.clone());
        let now = cx.background_executor().now();
        let reduce_motion = cx.reduce_motion();
        let dragging = cx.has_active_drag();
        sorter.update(cx, |sorter, _| {
            sorter.reconcile(&order);
            if sorter.is_dragging() && !dragging {
                sorter.cancel();
            }
            if sorter.tick(now, window.rem_size(), reduce_motion) {
                window.request_animation_frame();
            }
        });
        sorter.read(cx).arrange(&mut self.tabs, |tab| key(&tab.dragged));
        let count = self.tabs.len();
        let active_index = self.tabs.iter().position(|tab| tab.selected);
        let mut previous = None;
        let children: Vec<_> = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(index, tab)| {
                let id = key(&tab.dragged);
                let placement = Placement { index, count, active_index, previous };
                previous = Some(id);
                let held = sorter.read(cx).holds(id);
                let offset = sorter.read(cx).offset_of(id, now, reduce_motion);
                let child = (tab.render)(placement, cx);
                let press = sorter.clone();
                let preview = sorter.downgrade();
                let surface = div()
                    .id(("sortable-tab-surface", id))
                    .relative()
                    .when(axis == SortAxis::Vertical, |tab| tab.w_full())
                    .when(axis == SortAxis::Horizontal, |tab| tab.left(offset))
                    .when(axis == SortAxis::Vertical, |tab| tab.top(offset))
                    .when(held, |tab| tab.shadow_md())
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        press.update(cx, |sorter, _| sorter.press(axis.coordinate(event.position)));
                    })
                    .on_drag(tab.dragged, move |dragged, offset, _, cx| {
                        dragged_item_preview(dragged, offset, Some(preview.clone()), cx)
                    })
                    .child(child);
                div()
                    .id(("sortable-tab-slot", id))
                    .relative()
                    .flex_none()
                    .when(axis == SortAxis::Vertical, |slot| slot.w_full())
                    .child(if held {
                        deferred(surface).into_any_element()
                    } else {
                        surface.into_any_element()
                    })
                    .into_any_element()
            })
            .collect();
        let scroll = sorter.read(cx).scroll_handle().clone();
        let moving = sorter.clone();
        let dropping = sorter.clone();
        let leaving = sorter;
        let list = self.list.gap(self.gap).track_scroll(&scroll).children(children);
        let lane = match self.drag_lane {
            Some((lane, end_slot)) => lane.child(list).child(end_slot),
            None => list,
        };
        lane.on_drag_move::<DraggedItem>(move |event: &DragMoveEvent<DraggedItem>, window, cx| {
            let dragged = event.drag(cx).clone();
            if !source.as_ref().is_some_and(|source| same_list(source, &dragged)) {
                return;
            }
            moving.update(cx, |sorter, cx| {
                // Leaving the strip restores the model order and allows the
                // existing pane/edge drop targets to take over immediately.
                let point = event.event.position;
                let in_lane = event.bounds.contains(&point);
                if !in_lane {
                    if sorter.is_dragging() {
                        sorter.suspend();
                        window.refresh();
                    }
                    return;
                }
                if sorter.drag_move(
                    key(&dragged),
                    order.clone(),
                    point,
                    window.rem_size(),
                    cx.background_executor().now(),
                    cx.reduce_motion(),
                ) {
                    window.refresh();
                }
            });
        })
        .capture_any_mouse_up(move |event, window, cx| {
            if event.button != MouseButton::Left || !dropping.read(cx).is_dragging() {
                return;
            }
            let now = cx.background_executor().now();
            let reduce_motion = cx.reduce_motion();
            let destination = dropping.update(cx, |sorter, _| {
                sorter.drop_at(
                    axis.coordinate(event.position),
                    window.rem_size(),
                    now,
                    reduce_motion,
                )
            });
            if let Some((id, index)) = destination {
                // The actual payload retains its source pane and item for
                // pane-local commits (including modifier-assisted cloning).
                if let Some(dragged) = payloads.get(&id) {
                    (self.commit)(dragged, index, window, cx);
                    dropping.update(cx, |sorter, _| sorter.accept_drop(now, reduce_motion));
                    cx.stop_active_drag(window);
                    cx.stop_propagation();
                    window.refresh();
                }
            }
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            leaving.update(cx, |sorter, _| {
                if sorter.is_dragging() {
                    sorter.cancel();
                    window.refresh();
                }
            });
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrome::{NewItemKind, new_item_drag_handle};
    use gpui::{Context, Modifiers, Render, TestAppContext, point};
    use std::{cell::RefCell, rc::Rc, time::Duration};

    struct Harness {
        axis: SortAxis,
        pane_tabs: bool,
        order: Vec<u64>,
        painted_order: Rc<RefCell<Vec<u64>>>,
        commits: usize,
        legacy_drops: usize,
        clicks: usize,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.painted_order.borrow_mut().clear();
            let tabs = self
                .order
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let id = *id;
                    let dragged = DraggedItem {
                        space: "test".into(),
                        tab: serde_json::from_value(serde_json::json!(if self.pane_tabs {
                            1
                        } else {
                            id
                        }))
                        .unwrap(),
                        pane: serde_json::from_value(serde_json::json!(1)).unwrap(),
                        item: serde_json::from_value(serde_json::json!(id)).unwrap(),
                        index,
                        top_level: !self.pane_tabs,
                        grouped: !self.pane_tabs && id == 2,
                    };
                    let painted = self.painted_order.clone();
                    let axis = self.axis;
                    let click = cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                        this.clicks += 1;
                        cx.notify();
                    });
                    let legacy_drop = cx.listener(|this, _: &DraggedItem, _, cx| {
                        this.legacy_drops += 1;
                        cx.notify();
                    });
                    SortableTab::new(dragged, id == 1, move |_, _| {
                        painted.borrow_mut().push(id);
                        div()
                            .id(("test-tab", id))
                            .debug_selector(move || format!("SORT_TAB_{id}"))
                            .w(px(if axis == SortAxis::Horizontal {
                                if id == 2 { 120. } else { 60. }
                            } else {
                                150.
                            }))
                            .h(px(if axis == SortAxis::Vertical {
                                if id == 2 { 70. } else { 30. }
                            } else {
                                30.
                            }))
                            .on_click(click)
                            .on_drop(legacy_drop)
                            .into_any_element()
                    })
                })
                .collect();
            let weak = cx.weak_entity();
            let list = match self.axis {
                SortAxis::Horizontal => h_flex().id("test-list").w(px(300.)).overflow_x_scroll(),
                SortAxis::Vertical => v_flex().id("test-list").w(px(150.)),
            };
            let sortable = SortableTabList::new(
                "test-sorter".into(),
                list,
                self.axis,
                gpui::rems(0.25),
                tabs,
                move |dragged, index, _, cx| {
                    weak.update(cx, |this, cx| {
                        let from = this.order.iter().position(|id| *id == key(dragged)).unwrap();
                        let id = this.order.remove(from);
                        this.order.insert(index, id);
                        this.commits += 1;
                        cx.notify();
                    })
                    .unwrap();
                },
            );
            let sortable = if self.axis == SortAxis::Horizontal {
                let controls = h_flex().flex_none().children(
                    [("NEW_TERMINAL", NewItemKind::Terminal), ("NEW_SURFACE", NewItemKind::Plugin)]
                        .into_iter()
                        .map(|(id, kind)| {
                            new_item_drag_handle(
                                id,
                                None,
                                kind,
                                div()
                                    .id(format!("{id}-button"))
                                    .debug_selector(move || id.into())
                                    .size(px(30.))
                                    .on_click(cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    })),
                            )
                        }),
                );
                sortable.drag_lane(
                    h_flex()
                        .id("test-strip")
                        .debug_selector(|| "SORT_STRIP".into())
                        .w(px(600.))
                        .h(px(30.))
                        .on_drop(cx.listener(|this, _: &DraggedItem, _, cx| {
                            this.legacy_drops += 1;
                            cx.notify();
                        })),
                    controls,
                )
            } else {
                sortable
            };
            div().size_full().child(sortable).child(
                div()
                    .id("foreign-drop")
                    .w(px(150.))
                    .h(px(100.))
                    .mt(px(100.))
                    .debug_selector(|| "FOREIGN_DROP".into())
                    .on_drop(cx.listener(|this, _: &DraggedItem, _, cx| {
                        this.legacy_drops += 1;
                        cx.notify();
                    })),
            )
        }
    }

    fn init(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
    }

    fn harness(axis: SortAxis, pane_tabs: bool) -> Harness {
        Harness {
            axis,
            pane_tabs,
            order: vec![1, 2, 3],
            painted_order: Rc::default(),
            commits: 0,
            legacy_drops: 0,
            clicks: 0,
        }
    }

    #[gpui::test]
    fn every_tab_variant_sorts_before_release_and_commits_once(cx: &mut TestAppContext) {
        init(cx);
        for (axis, pane_tabs) in [
            (SortAxis::Horizontal, false),
            (SortAxis::Horizontal, true),
            (SortAxis::Vertical, false),
        ] {
            let (view, cx) = cx.add_window_view(|_, _| harness(axis, pane_tabs));
            cx.run_until_parked();
            let source = cx.debug_bounds("SORT_TAB_2").unwrap().center();
            let first = cx.debug_bounds("SORT_TAB_1").unwrap();
            let target = first.origin + point(px(5.), px(5.));
            cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
            cx.simulate_mouse_move(
                source + point(px(8.), px(0.)),
                Some(MouseButton::Left),
                Modifiers::none(),
            );
            cx.simulate_mouse_move(target, Some(MouseButton::Left), Modifiers::none());
            cx.run_until_parked();
            assert_eq!(view.read_with(cx, |this, _| this.order.clone()), vec![1, 2, 3]);
            assert_eq!(
                view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                vec![2, 1, 3]
            );
            // FLIP paints the displaced neighbor at its original position on
            // the first frame, despite already having its new layout slot.
            assert_eq!(cx.debug_bounds("SORT_TAB_1").unwrap().origin, first.origin);
            cx.executor().advance_clock(Duration::from_millis(160));
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            assert!(
                axis.coordinate(cx.debug_bounds("SORT_TAB_1").unwrap().origin)
                    > axis.coordinate(first.origin)
            );
            cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
            cx.run_until_parked();
            assert_eq!(
                view.read_with(cx, |this, _| (
                    this.order.clone(),
                    this.commits,
                    this.legacy_drops,
                    this.clicks
                )),
                (vec![2, 1, 3], 1, 0, 0)
            );
            assert!(!cx.read(|cx| cx.has_active_drag()));
        }
    }

    #[gpui::test]
    fn buttons_and_empty_strip_space_keep_sorting_until_release(cx: &mut TestAppContext) {
        init(cx);
        for pane_tabs in [false, true] {
            // Outer item 2 is a group; exercise both kinds of outer drag payload.
            for dragged_id in [1, 2] {
                for release_at in 0..3 {
                    let (view, cx) =
                        cx.add_window_view(|_, _| harness(SortAxis::Horizontal, pane_tabs));
                    cx.run_until_parked();
                    let selector = if dragged_id == 1 { "SORT_TAB_1" } else { "SORT_TAB_2" };
                    let source = cx.debug_bounds(selector).unwrap().center();
                    let strip = cx.debug_bounds("SORT_STRIP").unwrap();
                    let targets = [
                        cx.debug_bounds("NEW_TERMINAL").unwrap().center(),
                        cx.debug_bounds("NEW_SURFACE").unwrap().center(),
                        point(strip.right() - px(10.), strip.center().y),
                    ];
                    let mut expected = vec![1, 2, 3];
                    expected.retain(|id| *id != dragged_id);
                    expected.push(dragged_id);
                    cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
                    cx.simulate_mouse_move(
                        source + point(px(8.), px(0.)),
                        Some(MouseButton::Left),
                        Modifiers::none(),
                    );
                    for target in &targets[..=release_at] {
                        cx.simulate_mouse_move(*target, Some(MouseButton::Left), Modifiers::none());
                        cx.run_until_parked();
                        assert_eq!(view.read_with(cx, |this, _| this.order.clone()), vec![1, 2, 3]);
                        assert_eq!(
                            view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                            expected
                        );
                        // The actual tab stays under the pointer instead of
                        // reverting to the compact pane-placement preview.
                        assert!(cx.debug_bounds(selector).unwrap().contains(target));
                        assert!(cx.read(|cx| cx.has_active_drag()));
                    }
                    cx.simulate_mouse_up(targets[release_at], MouseButton::Left, Modifiers::none());
                    cx.run_until_parked();
                    assert_eq!(
                        view.read_with(cx, |this, _| (
                            this.order.clone(),
                            this.commits,
                            this.legacy_drops,
                            this.clicks
                        )),
                        (expected, 1, 0, 0)
                    );
                    assert!(!cx.read(|cx| cx.has_active_drag()));
                }
            }
        }
    }

    #[gpui::test]
    fn leaving_and_reentering_the_full_strip_resumes_sorting(cx: &mut TestAppContext) {
        init(cx);
        for pane_tabs in [false, true] {
            for leave_right in [false, true] {
                let (view, cx) =
                    cx.add_window_view(|_, _| harness(SortAxis::Horizontal, pane_tabs));
                cx.run_until_parked();
                let source = cx.debug_bounds("SORT_TAB_2").unwrap().center();
                let strip = cx.debug_bounds("SORT_STRIP").unwrap();
                let target = point(strip.right() - px(10.), strip.center().y);
                cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
                cx.simulate_mouse_move(
                    source + point(px(8.), px(0.)),
                    Some(MouseButton::Left),
                    Modifiers::none(),
                );
                cx.simulate_mouse_move(target, Some(MouseButton::Left), Modifiers::none());
                cx.run_until_parked();
                assert_eq!(
                    view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                    vec![1, 3, 2]
                );
                let outside = if leave_right {
                    point(strip.right() + px(20.), target.y)
                } else {
                    point(target.x, strip.bottom() + px(20.))
                };
                cx.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::none());
                cx.run_until_parked();
                assert_eq!(
                    view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                    vec![1, 2, 3]
                );
                assert!(cx.read(|cx| cx.has_active_drag()));
                cx.simulate_mouse_move(target, Some(MouseButton::Left), Modifiers::none());
                cx.run_until_parked();
                assert_eq!(
                    view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                    vec![1, 3, 2]
                );
                assert!(cx.debug_bounds("SORT_TAB_2").unwrap().contains(&target));
                cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
                cx.run_until_parked();
                assert_eq!(
                    view.read_with(cx, |this, _| (
                        this.order.clone(),
                        this.commits,
                        this.legacy_drops,
                        this.clicks
                    )),
                    (vec![1, 3, 2], 1, 0, 0)
                );
            }
        }
    }

    #[gpui::test]
    fn leaving_the_strip_or_cancelling_restores_order_without_committing(cx: &mut TestAppContext) {
        init(cx);
        for cancel in [false, true] {
            let (view, cx) = cx.add_window_view(|_, _| harness(SortAxis::Horizontal, false));
            cx.run_until_parked();
            let source = cx.debug_bounds("SORT_TAB_2").unwrap().center();
            let target = point(px(5.), px(5.));
            cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
            cx.simulate_mouse_move(
                source + point(px(8.), px(0.)),
                Some(MouseButton::Left),
                Modifiers::none(),
            );
            cx.simulate_mouse_move(target, Some(MouseButton::Left), Modifiers::none());
            cx.run_until_parked();
            assert_eq!(
                view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                vec![2, 1, 3]
            );
            let outside = point(px(20.), px(100.));
            if cancel {
                cx.update(|window, cx| {
                    cx.stop_active_drag(window);
                    window.refresh();
                });
            } else {
                cx.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::none());
            }
            cx.run_until_parked();
            assert_eq!(
                view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
                vec![1, 2, 3]
            );
            cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::none());
            assert_eq!(
                view.read_with(cx, |this, _| (
                    this.order.clone(),
                    this.commits,
                    this.legacy_drops,
                    this.clicks
                )),
                (vec![1, 2, 3], 0, 0, 0)
            );
        }
    }
    #[gpui::test]
    fn a_tab_can_leave_its_preview_and_drop_into_another_pane(cx: &mut TestAppContext) {
        init(cx);
        let (view, cx) = cx.add_window_view(|_, _| harness(SortAxis::Horizontal, true));
        cx.run_until_parked();
        let source = cx.debug_bounds("SORT_TAB_2").unwrap().center();
        cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(
            source + point(px(8.), px(0.)),
            Some(MouseButton::Left),
            Modifiers::none(),
        );
        cx.simulate_mouse_move(point(px(5.), px(5.)), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
            vec![2, 1, 3]
        );
        let foreign = cx.debug_bounds("FOREIGN_DROP").unwrap().center();
        cx.simulate_mouse_move(foreign, Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        assert!(cx.read(|cx| cx.has_active_drag()));
        assert_eq!(
            view.read_with(cx, |this, _| this.painted_order.borrow().clone()),
            vec![1, 2, 3]
        );
        cx.simulate_mouse_up(foreign, MouseButton::Left, Modifiers::none());
        assert_eq!(
            view.read_with(cx, |this, _| (this.commits, this.legacy_drops, this.clicks)),
            (0, 1, 0)
        );
    }
}
