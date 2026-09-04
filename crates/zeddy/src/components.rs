//! Small Chartr defaults around Zed's reusable UI components.
//!
//! Content and behavior stay with their owning feature; only visual contracts
//! shared across features belong here.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    Action, Anchor, AnyElement, AnyView, AnyWindowHandle, App, AppContext as _, Bounds, ClickEvent,
    Context, DismissEvent, Div, ElementId, Entity, Focusable, Hsla, IntoElement, MouseButton,
    ParentElement, Pixels, Render, RenderOnce, Role, SharedString, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, canvas, div, point, px,
    relative, size,
};
use ui::{ButtonSize, ContextMenu as UiContextMenu, DynamicSpacing, IconPosition, prelude::*};

// Matches ui::ContextMenu's default minimum width.
const POPUP_CONTENT_WIDTH: Pixels = px(200.);
const POPUP_OUTSET: Pixels = px(8.);
const POPUP_GAP: Pixels = px(4.);

#[derive(Clone, Copy, Default)]
struct PopupMetrics {
    entries: usize,
    inter_item_gaps: usize,
    separators: usize,
    headers: usize,
    custom_rows_height: Pixels,
}

impl PopupMetrics {
    fn height(self, cx: &App) -> Pixels {
        let rem = theme::theme_settings(cx).ui_font_size(cx);
        let entry = rem * theme::BufferLineHeight::Comfortable.value();
        let inter_item_gap = rem * 0.25;
        let separator = px(1.) + DynamicSpacing::Base06.px(cx) * 2.;
        let header = rem * 1.25 + DynamicSpacing::Base04.px(cx);
        let list_padding = DynamicSpacing::Base04.px(cx) * 2.;
        let height = list_padding
            + entry * self.entries
            + inter_item_gap * self.inter_item_gaps
            + separator * self.separators
            + header * self.headers
            + self.custom_rows_height;
        px(height.as_f32().ceil().max(1.))
    }
}

/// A context menu prepared for rendering in an anchored child window.
pub struct AnchoredContextMenu {
    items: Vec<PopupItem>,
    target_window: AnyWindowHandle,
    height: Pixels,
    width: Pixels,
}

type PopupHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type PopupRowRenderer = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

struct PopupEntry {
    label: SharedString,
    toggle: Option<(IconPosition, bool)>,
    action: Option<Box<dyn Action>>,
    handler: PopupHandler,
}

enum PopupItem {
    Entry(PopupEntry),
    CustomRow(PopupRowRenderer),
    Gap,
    Separator,
    Header(SharedString),
}

/// Chartr's shared context-menu builder.
///
/// Zed's menu rows are flush by default. This wrapper inserts a small,
/// non-selectable gap between adjacent actions while leaving separators and
/// headers as distinct group boundaries.
pub struct ContextMenu {
    inner: Option<UiContextMenu>,
    popup_items: Vec<PopupItem>,
    has_item_in_group: bool,
    metrics: PopupMetrics,
    popup_width: Pixels,
}

impl ContextMenu {
    pub fn build(
        window: &mut Window,
        cx: &mut App,
        build: impl FnOnce(Self, &mut Window, &mut Context<UiContextMenu>) -> Self,
    ) -> Entity<UiContextMenu> {
        UiContextMenu::build(window, cx, |menu, window, cx| {
            build(
                Self {
                    inner: Some(menu),
                    popup_items: Vec::new(),
                    has_item_in_group: false,
                    metrics: PopupMetrics::default(),
                    popup_width: POPUP_CONTENT_WIDTH,
                },
                window,
                cx,
            )
            .inner
            .expect("in-window context menus keep their UI menu")
        })
    }

    /// Entry handlers are routed back to `window`; the anchored popup is only
    /// a host for the stock UI context menu.
    pub fn build_popup(
        window: &mut Window,
        cx: &mut App,
        build: impl FnOnce(Self) -> Self,
    ) -> AnchoredContextMenu {
        let target_window = window.window_handle();
        let built = build(Self {
            inner: None,
            popup_items: Vec::new(),
            has_item_in_group: false,
            metrics: PopupMetrics::default(),
            popup_width: POPUP_CONTENT_WIDTH,
        });
        AnchoredContextMenu {
            items: built.popup_items,
            target_window,
            height: built.metrics.height(cx),
            width: built.popup_width,
        }
    }

    fn before_item(mut self) -> Self {
        if self.has_item_in_group {
            if let Some(inner) = self.inner.take() {
                self.inner = Some(inner.custom_row(|_, _| div().h_1().into_any_element()));
            } else {
                self.popup_items.push(PopupItem::Gap);
            }
            self.metrics.inter_item_gaps += 1;
        }
        self.has_item_in_group = true;
        self.metrics.entries += 1;
        self
    }

    pub fn entry(
        self,
        label: impl Into<SharedString>,
        action: Option<Box<dyn Action>>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let mut this = self.before_item();
        if let Some(inner) = this.inner.take() {
            this.inner = Some(inner.entry(label, action, handler));
        } else {
            this.popup_items.push(PopupItem::Entry(PopupEntry {
                label: label.into(),
                toggle: None,
                action,
                handler: Rc::new(handler),
            }));
        }
        this
    }

    pub fn toggleable_entry(
        self,
        label: impl Into<SharedString>,
        toggled: bool,
        position: IconPosition,
        action: Option<Box<dyn Action>>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let mut this = self.before_item();
        if let Some(inner) = this.inner.take() {
            this.inner = Some(inner.toggleable_entry(label, toggled, position, action, handler));
        } else {
            this.popup_items.push(PopupItem::Entry(PopupEntry {
                label: label.into(),
                toggle: Some((position, toggled)),
                action,
                handler: Rc::new(handler),
            }));
        }
        this
    }

    /// Add a non-selectable row with arbitrary layout to either menu host.
    /// `height` lets the native popup size itself before that row is rendered.
    pub fn custom_row(
        mut self,
        height: Pixels,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        let render: PopupRowRenderer = Rc::new(render);
        if let Some(inner) = self.inner.take() {
            let render = render.clone();
            self.inner = Some(inner.custom_row(move |window, cx| render(window, cx)));
        } else {
            self.popup_items.push(PopupItem::CustomRow(render));
        }
        self.metrics.custom_rows_height += height;
        self
    }

    /// Set the content width of an anchored native popup.
    pub fn popup_width(mut self, width: Pixels) -> Self {
        self.popup_width = width.max(px(1.));
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.fixed_width(self.popup_width.into()));
        }
        self
    }

    pub fn separator(mut self) -> Self {
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.separator());
        } else {
            self.popup_items.push(PopupItem::Separator);
        }
        self.has_item_in_group = false;
        self.metrics.separators += 1;
        self
    }

    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.header(title));
        } else {
            self.popup_items.push(PopupItem::Header(title.into()));
        }
        self.has_item_in_group = false;
        self.metrics.headers += 1;
        self
    }
}

impl FluentBuilder for ContextMenu {}

type PopupBuilder = Rc<dyn Fn(&mut Window, &mut App) -> Option<AnchoredContextMenu>>;

/// A button-triggered menu rendered in a separate native popup window.
#[derive(IntoElement)]
pub struct PopupMenu {
    id: ElementId,
    trigger_builder: Option<Box<dyn FnOnce(bool, &mut Window, &mut App) -> AnyElement>>,
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    anchor: Rc<Cell<Anchor>>,
    builder: Rc<RefCell<Option<PopupBuilder>>>,
}

impl PopupMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger_builder: None,
            trigger_bounds: Rc::default(),
            anchor: Rc::new(Cell::new(Anchor::TopLeft)),
            builder: Rc::default(),
        }
    }

    pub fn trigger<T: ui::PopoverTrigger>(mut self, trigger: T) -> Self {
        self.trigger_builder =
            Some(Box::new(move |_, _, _| trigger.toggle_state(false).into_any_element()));
        self
    }

    pub fn trigger_with_tooltip<T: ui::PopoverTrigger + ui::ButtonCommon>(
        mut self,
        trigger: T,
        tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        self.trigger_builder = Some(Box::new(move |window_active, _, _| {
            let trigger = trigger.toggle_state(false);
            if window_active {
                trigger.tooltip(tooltip).into_any_element()
            } else {
                trigger.into_any_element()
            }
        }));
        self
    }

    pub fn anchor(self, anchor: Anchor) -> Self {
        self.anchor.set(anchor);
        self
    }

    pub fn menu(
        self,
        builder: impl Fn(&mut Window, &mut App) -> Option<AnchoredContextMenu> + 'static,
    ) -> Self {
        *self.builder.borrow_mut() = Some(Rc::new(builder));
        self
    }
}

impl RenderOnce for PopupMenu {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let trigger = self.trigger_builder.take().expect("popup menus require a trigger")(
            window.is_window_active(),
            window,
            cx,
        );
        let bounds = self.trigger_bounds;
        let measured_bounds = bounds.clone();
        let anchor = self.anchor;
        let builder = self.builder;
        div()
            .id(self.id)
            .relative()
            .child(trigger)
            .child(
                canvas(move |measured, _, _| measured_bounds.set(Some(measured)), |_, _, _, _| {})
                    .absolute()
                    .inset_0(),
            )
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                cx.stop_propagation();
                let Some(bounds) = bounds.get() else {
                    return;
                };
                let Some(builder) = builder.borrow().clone() else {
                    return;
                };
                let Some(menu) = builder(window, cx) else {
                    return;
                };
                open_popup(menu, bounds, anchor.get(), window, cx);
            })
    }
}

/// A secondary-click menu rendered in a separate native popup window.
#[derive(IntoElement)]
pub struct PopupRightClickMenu {
    id: ElementId,
    child_builder: Option<Box<dyn FnOnce(bool, &mut Window, &mut App) -> AnyElement>>,
    menu_builder: Rc<RefCell<Option<PopupBuilder>>>,
}

pub fn popup_right_click_menu(id: impl Into<ElementId>) -> PopupRightClickMenu {
    PopupRightClickMenu { id: id.into(), child_builder: None, menu_builder: Rc::default() }
}

impl PopupRightClickMenu {
    pub fn trigger<F, E>(mut self, trigger: F) -> Self
    where
        F: FnOnce(bool, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        self.child_builder = Some(Box::new(move |active, window, cx| {
            trigger(active, window, cx).into_any_element()
        }));
        self
    }

    pub fn menu(
        self,
        builder: impl Fn(&mut Window, &mut App) -> AnchoredContextMenu + 'static,
    ) -> Self {
        *self.menu_builder.borrow_mut() =
            Some(Rc::new(move |window, cx| Some(builder(window, cx))));
        self
    }
}

impl RenderOnce for PopupRightClickMenu {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let child = self.child_builder.take().expect("right-click menus require a trigger")(
            false, window, cx,
        );
        let builder = self.menu_builder;
        div().id(self.id).child(child).on_mouse_down(
            MouseButton::Right,
            move |event, window, cx| {
                cx.stop_propagation();
                window.prevent_default();
                let Some(builder) = builder.borrow().clone() else {
                    return;
                };
                let Some(menu) = builder(window, cx) else {
                    return;
                };
                open_popup(
                    menu,
                    Bounds::new(event.position, size(px(1.), px(1.))),
                    Anchor::TopLeft,
                    window,
                    cx,
                );
            },
        )
    }
}

struct AnchoredMenuWindow {
    menu: Entity<UiContextMenu>,
}

impl AnchoredMenuWindow {
    fn new(menu: AnchoredContextMenu, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let target_window = menu.target_window;
        let menu_height = menu.height;
        let menu_width = menu.width;
        let context_menu = UiContextMenu::build(window, cx, move |mut context_menu, _, _| {
            context_menu =
                context_menu.max_height(menu_height.into()).fixed_width(menu_width.into());
            for item in menu.items {
                context_menu = match item {
                    PopupItem::Entry(entry) => {
                        let target = target_window;
                        let handler = entry.handler;
                        if let Some((position, toggled)) = entry.toggle {
                            context_menu.toggleable_entry(
                                entry.label,
                                toggled,
                                position,
                                entry.action,
                                move |_, cx| {
                                    let _ = target.update(cx, |_, window, cx| handler(window, cx));
                                },
                            )
                        } else {
                            context_menu.entry(entry.label, entry.action, move |_, cx| {
                                let _ = target.update(cx, |_, window, cx| handler(window, cx));
                            })
                        }
                    }
                    PopupItem::CustomRow(render) => {
                        context_menu.custom_row(move |window, cx| render(window, cx))
                    }
                    PopupItem::Gap => {
                        context_menu.custom_row(|_, _| div().h_1().into_any_element())
                    }
                    PopupItem::Separator => context_menu.separator(),
                    PopupItem::Header(label) => context_menu.header(label),
                };
            }
            context_menu
        });

        window
            .subscribe(&context_menu, cx, move |_, _: &DismissEvent, window, _| {
                window.remove_window();
            })
            .detach();

        let focus = context_menu.focus_handle(cx);
        window.on_next_frame(move |window, _| {
            window.on_next_frame(move |window, cx| window.focus(&focus, cx));
        });
        Self { menu: context_menu }
    }
}

impl Render for AnchoredMenuWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p(POPUP_OUTSET).child(self.menu.clone())
    }
}

fn open_popup(
    mut menu: AnchoredContextMenu,
    trigger_bounds: Bounds<Pixels>,
    anchor: Anchor,
    parent_window: &mut Window,
    cx: &mut App,
) {
    let display = parent_window.display(cx);
    let display_id = display.as_ref().map(|display| display.id());
    let maximum_height = display
        .as_ref()
        .map(|display| display.visible_bounds().size.height)
        .unwrap_or_else(|| parent_window.bounds().size.height);
    let maximum_width = display
        .as_ref()
        .map(|display| display.visible_bounds().size.width)
        .unwrap_or_else(|| parent_window.bounds().size.width);
    let popup_height = clamp_pixels(menu.height + POPUP_OUTSET * 2., px(1.), maximum_height);
    let popup_width = clamp_pixels(menu.width + POPUP_OUTSET * 2., px(1.), maximum_width);
    menu.height = (popup_height - POPUP_OUTSET * 2.).max(px(1.));
    menu.width = (popup_width - POPUP_OUTSET * 2.).max(px(1.));
    let popup_size = size(popup_width, popup_height);
    let kind = anchored_popup_window_kind(parent_window, trigger_bounds, anchor);
    let opened = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Default::default(),
                popup_size,
            ))),
            titlebar: None,
            focus: true,
            show: true,
            kind,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            display_id,
            window_background: WindowBackgroundAppearance::Transparent,
            window_min_size: Some(popup_size),
            ..Default::default()
        },
        move |window, cx| cx.new(|cx| AnchoredMenuWindow::new(menu, window, cx)),
    );

    match opened {
        Ok(_) => parent_window.refresh(),
        Err(error) => eprintln!("Chartr could not open a menu: {error}"),
    }
}

fn anchored_popup_window_kind(
    parent: &Window,
    trigger: Bounds<Pixels>,
    anchor: Anchor,
) -> WindowKind {
    use gpui::popup::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupOptions};

    let (anchor, gravity, offset) = match anchor {
        Anchor::TopLeft => (
            PopupAnchor::BottomLeft,
            PopupGravity::BottomRight,
            point(-POPUP_OUTSET, POPUP_GAP - POPUP_OUTSET),
        ),
        Anchor::TopCenter => {
            (PopupAnchor::Bottom, PopupGravity::Bottom, point(px(0.), POPUP_GAP - POPUP_OUTSET))
        }
        Anchor::TopRight => (
            PopupAnchor::BottomRight,
            PopupGravity::BottomLeft,
            point(POPUP_OUTSET, POPUP_GAP - POPUP_OUTSET),
        ),
        Anchor::BottomLeft => (
            PopupAnchor::TopLeft,
            PopupGravity::TopRight,
            point(-POPUP_OUTSET, POPUP_OUTSET - POPUP_GAP),
        ),
        Anchor::BottomCenter => {
            (PopupAnchor::Top, PopupGravity::Top, point(px(0.), POPUP_OUTSET - POPUP_GAP))
        }
        Anchor::BottomRight => (
            PopupAnchor::TopRight,
            PopupGravity::TopLeft,
            point(POPUP_OUTSET, POPUP_OUTSET - POPUP_GAP),
        ),
        Anchor::LeftCenter => {
            (PopupAnchor::Left, PopupGravity::Left, point(POPUP_OUTSET - POPUP_GAP, px(0.)))
        }
        Anchor::RightCenter => {
            (PopupAnchor::Right, PopupGravity::Right, point(POPUP_GAP - POPUP_OUTSET, px(0.)))
        }
    };
    WindowKind::AnchoredPopup(PopupOptions {
        parent: parent.window_handle(),
        anchor_rect: trigger,
        anchor,
        gravity,
        constraint_adjustment: PopupConstraintAdjustment::SLIDE_X
            | PopupConstraintAdjustment::SLIDE_Y
            | PopupConstraintAdjustment::FLIP_X
            | PopupConstraintAdjustment::FLIP_Y,
        offset,
        grab: true,
    })
}

fn clamp_pixels(value: Pixels, minimum: Pixels, maximum: Pixels) -> Pixels {
    px(value.as_f32().clamp(minimum.as_f32(), maximum.max(minimum).as_f32()))
}

#[cfg(test)]
mod popup_menu_tests {
    use super::*;
    use gpui::{Modifiers, TestAppContext};
    use std::time::Duration;

    struct MenuHarness {
        invoked: Rc<Cell<bool>>,
    }

    impl Render for MenuHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let invoked = self.invoked.clone();
            PopupMenu::new("popup-test")
                .trigger(ui::Button::new("popup-test-trigger", "Open"))
                .menu(move |window, cx| {
                    let invoked = invoked.clone();
                    Some(ContextMenu::build_popup(window, cx, move |menu| {
                        menu.entry("Run", None, move |_, _| invoked.set(true))
                    }))
                })
        }
    }

    struct TooltipMenuHarness {
        tooltip_built: Rc<Cell<bool>>,
    }

    struct TestTooltip;

    impl Render for TestTooltip {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child("Open menu")
        }
    }

    impl Render for TooltipMenuHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let tooltip_built = self.tooltip_built.clone();
            PopupMenu::new("tooltip-popup-test")
                .trigger_with_tooltip(
                    ui::Button::new("tooltip-popup-test-trigger", "Open"),
                    move |_, cx| {
                        tooltip_built.set(true);
                        cx.new(|_| TestTooltip).into()
                    },
                )
                .menu(|window, cx| {
                    Some(ContextMenu::build_popup(window, cx, |menu| {
                        menu.entry("Run", None, |_, _| {})
                    }))
                })
        }
    }

    #[gpui::test]
    fn anchored_menu_entries_receive_clicks_and_close(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });

        let invoked = Rc::new(Cell::new(false));
        let invoked_for_view = invoked.clone();
        let (_, cx) = cx.add_window_view(|_, _| MenuHarness { invoked: invoked_for_view });
        let parent = cx.window_handle();

        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        let popup = cx
            .windows()
            .into_iter()
            .find(|window| *window != parent)
            .expect("clicking the trigger should open an anchored menu window");

        let mut popup = gpui::VisualTestContext::from_window(popup, cx);
        popup.run_until_parked();
        let entry = popup
            .debug_bounds("MENU_ITEM-Run")
            .expect("the stock context-menu entry should render in the popup");
        popup.simulate_click(entry.center(), Modifiers::none());

        assert!(invoked.get(), "clicking a popup entry should invoke its parent-window handler");
        assert_eq!(popup.windows(), vec![parent], "confirming an entry should close the popup");
    }

    #[gpui::test]
    fn opening_a_popup_cancels_its_pending_trigger_tooltip(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });

        let tooltip_built = Rc::new(Cell::new(false));
        let tooltip_built_for_view = tooltip_built.clone();
        let (_, cx) =
            cx.add_window_view(|_, _| TooltipMenuHarness { tooltip_built: tooltip_built_for_view });

        cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_move(point(px(11.), px(10.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        cx.executor().advance_clock(Duration::from_millis(600));
        cx.run_until_parked();

        assert!(
            !tooltip_built.get(),
            "the trigger tooltip must stay hidden while its menu is open"
        );
    }
}

/// A vertical collection of selectable rows. The inter-row gap is part of the
/// collection rather than any individual row, so adjacent state backgrounds
/// are always separated consistently.
pub fn selection_list() -> Div {
    v_flex().gap_px()
}

/// Chartr's common selectable-row treatment. This mirrors Zed's sparse
/// `ListItem`, with one pixel removed from each vertical side. Zed only exposes
/// dense and sparse presets, so keeping the intermediate density here ensures
/// every Chartr list uses the same geometry and full-row hit target.
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

/// A compact radio-like control whose options share one outline and are split
/// by dividers. Selection uses the theme's neutral element surface instead of
/// its semantic accent tint so it remains balanced across light and dark
/// themes.
#[derive(IntoElement)]
pub struct SegmentedControl {
    label: SharedString,
    options: Vec<SegmentedControlOption>,
    disabled: bool,
}

impl SegmentedControl {
    pub fn new(
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = SegmentedControlOption>,
    ) -> Self {
        Self { label: label.into(), options: options.into_iter().collect(), disabled: false }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let option_count = self.options.len();
        let colors = cx.theme().colors();
        let border = colors.border.opacity(0.8);

        h_flex()
            .id(self.label.clone())
            .role(Role::RadioGroup)
            .aria_label(self.label)
            .rounded_md()
            .overflow_hidden()
            .border_1()
            .border_color(border)
            .when(self.disabled, |control| control.opacity(0.5))
            .children(self.options.into_iter().enumerate().map(|(index, option)| {
                let selected = option.selected;
                h_flex()
                    .id(option.id)
                    .role(Role::RadioButton)
                    .aria_selected(selected)
                    .h(ButtonSize::Default.rems())
                    .px_3()
                    .when(index + 1 < option_count, |item| item.border_r_1().border_color(border))
                    .when(selected, |item| item.bg(colors.ghost_element_selected))
                    .when(!selected && !self.disabled, |item| {
                        item.hover(|style| style.bg(colors.ghost_element_hover))
                            .active(|style| style.bg(colors.ghost_element_active))
                    })
                    .when_else(
                        self.disabled,
                        |item| item.cursor_not_allowed(),
                        |item| item.cursor_pointer().on_click(option.on_click),
                    )
                    .child(Label::new(option.label).size(LabelSize::Small).when(
                        !selected,
                        |label| {
                            label.color(if self.disabled { Color::Disabled } else { Color::Muted })
                        },
                    ))
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

impl RenderOnce for SelectionRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sparse_padding = window.rem_size() * 0.25;
        let vertical_padding =
            if sparse_padding > px(1.) { sparse_padding - px(1.) } else { px(0.) };
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
