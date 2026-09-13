//! Selectable rows and segmented controls.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, ElementId, Hsla, IntoElement, ParentElement,
    RenderOnce, Role, SharedString, Window, canvas, point, px, size,
};
use ui::{ButtonSize, prelude::*};

/// A vertical collection of selectable rows. The inter-row gap is part of the
/// collection rather than any individual row, so adjacent state backgrounds
/// are always separated consistently.
pub fn selection_list() -> Div {
    v_flex().gap_px()
}

/// chartr's common selectable-row treatment. This mirrors Zed's sparse
/// `ListItem`, with one pixel removed from each vertical side. Zed only exposes
/// dense and sparse presets, so keeping the intermediate density here ensures
/// every chartr list uses the same geometry and full-row hit target.
pub fn selection_row(id: impl Into<ElementId>, selected: bool) -> SelectionRow {
    SelectionRow::new(id, selected)
}

/// One mutually exclusive choice inside a [`SegmentedControl`].
pub struct SegmentedControlOption {
    id: ElementId,
    label: SharedString,
    selected: bool,
    on_click: Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

impl SegmentedControlOption {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self { id: id.into(), label: label.into(), selected, on_click: Box::new(on_click) }
    }
}

/// A radio-like picker with one inset selection pill inside a shared outline.
/// The pill follows the selected option's measured position and width.
#[derive(IntoElement)]
pub struct SegmentedControl {
    label: SharedString,
    options: Vec<SegmentedControlOption>,
    disabled: bool,
    full_width: bool,
    list_row: bool,
}

impl SegmentedControl {
    pub fn new(
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = SegmentedControlOption>,
    ) -> Self {
        Self {
            label: label.into(),
            options: options.into_iter().collect(),
            disabled: false,
            full_width: false,
            list_row: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Match a bordered selection row's height and title typography.
    pub fn list_row(mut self) -> Self {
        self.list_row = true;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
}

const PILL_INSET: f32 = 2.;
const PILL_DURATION: Duration = Duration::from_millis(250);

#[derive(Default)]
struct PillState {
    motion: Option<PillMotion>,
}

struct PillMotion {
    from: Bounds<Pixels>,
    to: Bounds<Pixels>,
    started: Instant,
}

impl PillMotion {
    fn sample(&self, now: Instant) -> (Bounds<Pixels>, bool) {
        let progress = (now.saturating_duration_since(self.started).as_secs_f32()
            / PILL_DURATION.as_secs_f32())
        .min(1.);
        let eased = 1. - (1. - progress).powi(3);
        (
            Bounds::new(
                self.from.origin + (self.to.origin - self.from.origin) * eased,
                size(
                    self.from.size.width + (self.to.size.width - self.from.size.width) * eased,
                    self.from.size.height + (self.to.size.height - self.from.size.height) * eased,
                ),
            ),
            progress < 1. && self.from != self.to,
        )
    }
}

impl PillState {
    fn advance(
        &mut self,
        target: Option<Bounds<Pixels>>,
        now: Instant,
        reduce_motion: bool,
    ) -> Option<(Bounds<Pixels>, bool)> {
        let Some(target) = target else {
            self.motion = None;
            return None;
        };
        if reduce_motion || self.motion.as_ref().is_none_or(|motion| motion.to != target) {
            let from = if reduce_motion {
                target
            } else {
                self.motion.as_ref().map_or(target, |motion| motion.sample(now).0)
            };
            self.motion = Some(PillMotion { from, to: target, started: now });
        }
        Some(self.motion.as_ref().unwrap().sample(now))
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let inset = px(PILL_INSET);
        let vertical_padding = (selection_row_vertical_padding(window) - inset).max(px(0.));
        let compact_height = ButtonSize::Default.rems().to_pixels(window.rem_size()) - inset * 2.;
        let horizontal_padding = gpui::rems(if self.list_row { 0.625 } else { 0.75 })
            .to_pixels(window.rem_size())
            - inset;
        let label_size =
            if self.list_row { crate::fonts::UI_LABEL_DEFAULT } else { LabelSize::Small };
        let motion = window.use_keyed_state(self.label.clone(), cx, |_, _| {
            Rc::new(RefCell::new(PillState::default()))
        });
        let motion = motion.read(cx).clone();
        let hovered_option =
            window.use_keyed_state(format!("segmented-hover-{}", self.label), cx, |_, _| {
                Option::<ElementId>::None
            });
        // Layout probes do not paint or advance the motion. Keep targets local
        // to this render and positions relative to the control, so moving the
        // whole picker cannot make its pill lag behind.
        let origin = Rc::new(Cell::new(point(px(0.), px(0.))));
        let target = Rc::new(Cell::new(None));
        let measured_origin = origin.clone();
        let painted_target = target.clone();
        let colors = cx.theme().colors();
        let border = colors.border.opacity(0.8);
        let selected_background = colors.ghost_element_selected;

        h_flex()
            .id(self.label.clone())
            .relative()
            .role(Role::RadioGroup)
            .aria_label(self.label)
            .rounded_md()
            .overflow_hidden()
            .border_1()
            .border_color(border)
            .p(inset)
            .gap(inset)
            .when(self.full_width, |control| control.w_full())
            .when(self.disabled, |control| control.opacity(0.5))
            .child(
                canvas(
                    move |bounds, _, _| measured_origin.set(bounds.origin),
                    move |bounds, _, window, cx| {
                        if let Some((mut pill, animating)) = motion.borrow_mut().advance(
                            painted_target.get(),
                            cx.background_executor().now(),
                            cx.reduce_motion(),
                        ) {
                            pill.origin += bounds.origin;
                            let mut quad = gpui::fill(pill, selected_background);
                            quad.corner_radii = px(4.).into();
                            window.paint_quad(quad);
                            if animating {
                                window.request_animation_frame();
                            }
                        }
                    },
                )
                .absolute()
                .inset_0(),
            )
            .children(self.options.into_iter().map(|option| {
                let selected = option.selected;
                let hovered = hovered_option.read(cx).as_ref() == Some(&option.id);
                let hover_state = hovered_option.clone();
                let hover_id = option.id.clone();
                let origin = origin.clone();
                let target = target.clone();
                h_flex()
                    .id(option.id)
                    .relative()
                    .role(Role::RadioButton)
                    .aria_label(option.label.clone())
                    .aria_selected(selected)
                    .when_else(
                        self.list_row,
                        |item| item.py(vertical_padding),
                        |item| item.h(compact_height),
                    )
                    .px(horizontal_padding)
                    .when(self.full_width, |item| item.flex_1().min_w_0().justify_center())
                    .when(selected, |item| {
                        item.child(
                            canvas(
                                move |bounds, _, _| {
                                    target.set(Some(Bounds::new(
                                        bounds.origin - origin.get(),
                                        bounds.size,
                                    )));
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        )
                    })
                    .on_hover(move |hovered, _, cx| {
                        hover_state.update(cx, |current, cx| {
                            let next = if *hovered {
                                Some(hover_id.clone())
                            } else if current.as_ref() == Some(&hover_id) {
                                None
                            } else {
                                current.clone()
                            };
                            if *current != next {
                                *current = next;
                                cx.notify();
                            }
                        });
                    })
                    .when_else(
                        self.disabled,
                        |item| item.cursor_not_allowed(),
                        |item| item.cursor_pointer().on_click(option.on_click),
                    )
                    .child(
                        Label::new(option.label)
                            .size(label_size)
                            .when(self.full_width, |label| label.truncate())
                            .when(!selected, |label| {
                                label.color(if self.disabled {
                                    Color::Disabled
                                } else if hovered {
                                    Color::Default
                                } else {
                                    Color::Muted
                                })
                            }),
                    )
            }))
    }
}

/// Optional state surfaces for a selection row embedded on a custom ground.
#[derive(Debug, Clone, Copy)]
pub struct SelectionRowBackgrounds {
    pub hover: Hsla,
    pub selected: Hsla,
}

#[derive(IntoElement)]
pub struct SelectionRow {
    id: ElementId,
    selected: bool,
    aria_role: Option<Role>,
    aria_label: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
    backgrounds: Option<SelectionRowBackgrounds>,
    children: Vec<AnyElement>,
}

impl SelectionRow {
    fn new(id: impl Into<ElementId>, selected: bool) -> Self {
        Self {
            id: id.into(),
            selected,
            aria_role: None,
            aria_label: None,
            on_click: None,
            start_slot: None,
            end_slot: None,
            backgrounds: None,
            children: Vec::new(),
        }
    }

    pub fn aria_role(mut self, role: Role) -> Self {
        self.aria_role = Some(role);
        self
    }

    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn start_slot<E: IntoElement>(mut self, slot: impl Into<Option<E>>) -> Self {
        self.start_slot = slot.into().map(IntoElement::into_any_element);
        self
    }

    pub fn end_slot<E: IntoElement>(mut self, slot: impl Into<Option<E>>) -> Self {
        self.end_slot = slot.into().map(IntoElement::into_any_element);
        self
    }

    pub fn backgrounds(mut self, backgrounds: SelectionRowBackgrounds) -> Self {
        self.backgrounds = Some(backgrounds);
        self
    }
}

impl ParentElement for SelectionRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

fn selection_row_vertical_padding(window: &Window) -> gpui::Pixels {
    (window.rem_size() * 0.25 - px(1.)).max(px(0.))
}

impl RenderOnce for SelectionRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let vertical_padding = selection_row_vertical_padding(window);
        let has_end_slot = self.end_slot.is_some();
        let colors = cx.theme().colors();
        let (selected_background, hover_background, active_background) = if let Some(backgrounds) =
            self.backgrounds
        {
            let interaction = if self.selected { backgrounds.selected } else { backgrounds.hover };
            (backgrounds.selected, interaction, interaction)
        } else {
            (colors.ghost_element_selected, colors.ghost_element_hover, colors.ghost_element_active)
        };

        h_flex()
            .id(self.id)
            .group("list_item")
            .w_full()
            .relative()
            .gap_1()
            .px(DynamicSpacing::Base06.rems(cx))
            .py(vertical_padding)
            .rounded_sm()
            .when_some(self.aria_role, |row, role| row.role(role).aria_selected(self.selected))
            .when_some(self.aria_label, |row, label| row.aria_label(label))
            .when(self.selected, |row| row.bg(selected_background))
            .hover(|style| style.bg(hover_background))
            .active(|style| style.bg(active_background))
            .when_some(self.on_click, |row, on_click| row.cursor_pointer().on_click(on_click))
            .child(
                h_flex()
                    .flex_grow_1()
                    .flex_shrink_0()
                    .flex_basis(relative(0.25))
                    .gap(DynamicSpacing::Base06.rems(cx))
                    .overflow_hidden()
                    .children(self.start_slot)
                    .children(self.children),
            )
            .when(has_end_slot, |row| row.justify_between())
            .when_some(self.end_slot, |row, end_slot| {
                row.child(h_flex().flex_shrink_1().overflow_hidden().child(end_slot))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext};

    #[test]
    fn pill_retargets_from_its_current_bounds_and_respects_reduced_motion() {
        let now = Instant::now();
        let first = Bounds::new(point(px(2.), px(2.)), size(px(44.), px(20.)));
        let second = Bounds::new(point(px(50.), px(2.)), size(px(100.), px(20.)));
        let mut state = PillState::default();
        assert_eq!(state.advance(Some(first), now, false), Some((first, false)));
        assert_eq!(state.advance(Some(second), now, false), Some((first, true)));
        let midway = now + PILL_DURATION / 2;
        let (moving, active) = state.advance(Some(second), midway, false).unwrap();
        assert!(active && moving.left() > first.left() && moving.left() < second.left());
        assert!(moving.size.width > first.size.width && moving.size.width < second.size.width);
        assert_eq!(state.advance(Some(first), midway, false), Some((moving, true)));
        assert_eq!(state.advance(Some(first), midway, false), Some((moving, true)));
        assert_eq!(state.advance(Some(first), midway + PILL_DURATION, false), Some((first, false)),);
        assert_eq!(
            state.advance(Some(second), midway + PILL_DURATION, true),
            Some((second, false))
        );
        assert_eq!(state.advance(None, midway + PILL_DURATION, false), None);
        assert_eq!(state.advance(Some(first), midway + PILL_DURATION, false), Some((first, false)));
    }

    struct AnimatedPickerHarness {
        selected: usize,
        disabled: bool,
    }

    impl Render for AnimatedPickerHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            h_flex().child(
                SegmentedControl::new(
                    "Animated picker",
                    ["Tabs", "A much longer choice"].into_iter().enumerate().map(
                        |(index, label)| {
                            SegmentedControlOption::new(
                                label,
                                label,
                                self.selected == index,
                                cx.listener(move |this, _, _, cx| {
                                    this.selected = index;
                                    cx.notify();
                                }),
                            )
                        },
                    ),
                )
                .disabled(self.disabled),
            )
        }
    }

    fn painted_pill(cx: &mut gpui::VisualTestContext) -> Bounds<Pixels> {
        cx.update(|window, cx| {
            let pills: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == cx.theme().colors().ghost_element_selected.into())
                .collect();
            assert_eq!(pills.len(), 1, "one continuous selection pill");
            let bounds = pills[0].bounds;
            let scale = window.scale_factor();
            Bounds::new(
                point(px(bounds.origin.x.as_f32() / scale), px(bounds.origin.y.as_f32() / scale)),
                size(
                    px(bounds.size.width.as_f32() / scale),
                    px(bounds.size.height.as_f32() / scale),
                ),
            )
        })
    }

    #[gpui::test]
    fn picker_paints_a_sliding_resizing_pill_and_keeps_option_clicks(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
        let (view, cx) =
            cx.add_window_view(|_, _| AnimatedPickerHarness { selected: 0, disabled: false });
        cx.run_until_parked();
        let first = painted_pill(cx);
        let outer = cx.update(|window, cx| {
            let bounds = window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.border_color == cx.theme().colors().border.opacity(0.8))
                .unwrap()
                .bounds;
            let scale = window.scale_factor();
            Bounds::new(
                point(px(bounds.origin.x.as_f32() / scale), px(bounds.origin.y.as_f32() / scale)),
                size(
                    px(bounds.size.width.as_f32() / scale),
                    px(bounds.size.height.as_f32() / scale),
                ),
            )
        });
        assert!(first.top() >= px(PILL_INSET) && first.left() >= px(PILL_INSET));
        let resting_quads = cx.update(|window, _| window.painted_quads().len());
        cx.simulate_mouse_move(
            point(first.right() + px(25.), first.center().y),
            None,
            gpui::Modifiers::none(),
        );
        cx.run_until_parked();
        assert_eq!(
            cx.update(|window, _| window.painted_quads().len()),
            resting_quads,
            "hover changes text emphasis without painting a background",
        );
        cx.simulate_click(
            point(first.right() + px(25.), first.center().y),
            gpui::Modifiers::none(),
        );
        cx.run_until_parked();
        assert_eq!(view.read_with(cx, |view, _| view.selected), 1);
        assert_eq!(painted_pill(cx), first, "selection starts from the previous pill");
        cx.executor().advance_clock(PILL_DURATION / 2);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        let midway = painted_pill(cx);
        cx.executor().advance_clock(PILL_DURATION);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        let second = painted_pill(cx);
        assert!(first.left() < midway.left() && midway.left() < second.left());
        assert!(first.size.width < midway.size.width && midway.size.width < second.size.width);
        assert_eq!(first.size.height, second.size.height);
        assert_eq!(second.left() - first.right(), px(PILL_INSET));
        assert_eq!(first.top() - outer.top() - px(1.), px(PILL_INSET));
        assert_eq!(outer.bottom() - first.bottom() - px(1.), px(PILL_INSET));
        assert_eq!(first.left() - outer.left() - px(1.), px(PILL_INSET));

        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.simulate_click(first.center(), gpui::Modifiers::none());
        cx.run_until_parked();
        assert_eq!(view.read_with(cx, |view, _| view.selected), 0);
        assert_eq!(painted_pill(cx), first, "reduced motion settles immediately");
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        cx.run_until_parked();
        cx.simulate_click(second.center(), gpui::Modifiers::none());
        assert_eq!(view.read_with(cx, |view, _| view.selected), 0);
    }

    struct RowSizingHarness {
        rem_size: gpui::Pixels,
    }

    impl Render for RowSizingHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(self.rem_size);
            v_flex()
                .w(px(200.))
                .font(theme::theme_settings(cx).ui_font(cx).clone())
                .child(
                    div().debug_selector(|| "HISTORY_PICKER".into()).child(
                        SegmentedControl::new(
                            "Conversation history",
                            ["Inbox", "Archive"].map(|label| {
                                SegmentedControlOption::new(
                                    label,
                                    label,
                                    label == "Inbox",
                                    |_, _, _| {},
                                )
                            }),
                        )
                        .list_row()
                        .full_width(),
                    ),
                )
                .child(
                    div().debug_selector(|| "CHAT_ROW".into()).border_1().child(
                        selection_row("chat", true)
                            .start_slot(div().size(px(12.)))
                            .child(Label::new("hello!").size(crate::fonts::UI_LABEL_DEFAULT))
                            .end_slot(Label::new("7h").size(crate::fonts::UI_LABEL_SMALL)),
                    ),
                )
        }
    }

    #[gpui::test]
    fn history_picker_matches_chat_row_height_at_each_ui_scale(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
        let (view, cx) = cx.add_window_view(|_, _| RowSizingHarness { rem_size: px(14.) });
        for rem_size in [12., 14., 18., 24.] {
            view.update(cx, |view, cx| {
                view.rem_size = px(rem_size);
                cx.notify();
            });
            cx.run_until_parked();
            assert_eq!(
                cx.debug_bounds("HISTORY_PICKER").unwrap().size.height,
                cx.debug_bounds("CHAT_ROW").unwrap().size.height,
                "picker and row must have the same outer height at {rem_size}px",
            );
        }
    }
}
