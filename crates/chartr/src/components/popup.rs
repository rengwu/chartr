//! Context menus hosted in anchored native windows.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    Action, Anchor, AnyElement, AnyView, AnyWindowHandle, App, AppContext as _, Bounds, Context,
    DismissEvent, ElementId, Entity, Focusable, IntoElement, MouseButton, ParentElement, Pixels,
    Render, RenderOnce, SharedString, Window, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, canvas, div, point, px, size,
};
use ui::{
    ContextMenu as UiContextMenu, ContextMenuEntry as UiContextMenuEntry, DynamicSpacing,
    IconPosition, prelude::*,
};

#[cfg(test)]
use super::open_native_modal;
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
            + header * self.headers;
        px(height.as_f32().ceil().max(1.))
    }
}

/// A context menu prepared for rendering in an anchored child window.
pub struct AnchoredContextMenu {
    items: Vec<PopupItem>,
    target_window: AnyWindowHandle,
    height: Pixels,
    width: Pixels,
    has_custom_rows: bool,
}

type PopupHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type PopupRowRenderer = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

struct PopupEntry {
    label: SharedString,
    toggle: Option<(IconPosition, bool)>,
    icon_path: Option<SharedString>,
    label_color: Option<Color>,
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

/// chartr's shared context-menu builder.
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
    has_custom_rows: bool,
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
                    has_custom_rows: false,
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
            has_custom_rows: false,
        });
        AnchoredContextMenu {
            items: built.popup_items,
            target_window,
            height: built.metrics.height(cx),
            width: built.popup_width,
            has_custom_rows: built.has_custom_rows,
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
                icon_path: None,
                label_color: None,
                action,
                handler: Rc::new(handler),
            }));
        }
        this
    }

    /// Add a destructive action using the theme's danger text color.
    pub fn danger_entry(
        self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let mut this = self.before_item();
        let label = label.into();
        if let Some(inner) = this.inner.take() {
            this.inner = Some(inner.custom_entry(
                move |_, _| {
                    Label::new(label.clone()).color(Color::Error).truncate().into_any_element()
                },
                handler,
            ));
        } else {
            this.popup_items.push(PopupItem::Entry(PopupEntry {
                label,
                toggle: None,
                icon_path: None,
                label_color: Some(Color::Error),
                action: None,
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
                icon_path: None,
                label_color: None,
                action,
                handler: Rc::new(handler),
            }));
        }
        this
    }

    /// Add a selectable row with an embedded icon at the start and its check at the end.
    pub fn toggleable_entry_with_icon_path(
        self,
        label: impl Into<SharedString>,
        icon_path: impl Into<SharedString>,
        toggled: bool,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let mut this = self.before_item();
        let label = label.into();
        let icon_path = icon_path.into();
        if let Some(inner) = this.inner.take() {
            this.inner = Some(
                inner.item(
                    UiContextMenuEntry::new(label)
                        .custom_icon_path(icon_path)
                        .icon_position(IconPosition::Start)
                        .icon_size(IconSize::Small)
                        .toggle(IconPosition::End, toggled)
                        .handler(handler),
                ),
            );
        } else {
            this.popup_items.push(PopupItem::Entry(PopupEntry {
                label,
                toggle: Some((IconPosition::End, toggled)),
                icon_path: Some(icon_path),
                label_color: None,
                action: None,
                handler: Rc::new(handler),
            }));
        }
        this
    }

    /// Add a non-selectable row whose rendered content determines its height.
    pub fn custom_row(
        mut self,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        let render: PopupRowRenderer = Rc::new(render);
        if let Some(inner) = self.inner.take() {
            let render = render.clone();
            self.inner = Some(inner.custom_row(move |window, cx| render(window, cx)));
        } else {
            self.popup_items.push(PopupItem::CustomRow(render));
        }
        self.has_custom_rows = true;
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
        let trigger_id = self.id.clone();
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
                if dismiss_popup_for_trigger(&trigger_id, window.window_handle(), cx) {
                    return;
                }
                let Some(bounds) = bounds.get() else {
                    return;
                };
                let Some(builder) = builder.borrow().clone() else {
                    return;
                };
                let Some(menu) = builder(window, cx) else {
                    return;
                };
                open_popup(menu, bounds, anchor.get(), Some(trigger_id.clone()), window, cx);
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
                    None,
                    window,
                    cx,
                );
            },
        )
    }
}

struct AnchoredMenuWindow {
    menu: Entity<UiContextMenu>,
    target_window: AnyWindowHandle,
    trigger_id: Option<ElementId>,
    popup_width: Pixels,
    maximum_height: Pixels,
    fitted_height: Rc<Cell<Option<Pixels>>>,
}

impl AnchoredMenuWindow {
    fn new(
        menu: AnchoredContextMenu,
        trigger_id: Option<ElementId>,
        maximum_height: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
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
                        if let Some(icon_path) = entry.icon_path {
                            let mut menu_entry = UiContextMenuEntry::new(entry.label)
                                .custom_icon_path(icon_path)
                                .icon_position(IconPosition::Start)
                                .icon_size(IconSize::Small)
                                .handler(move |_, cx| {
                                    let _ = target.update(cx, |_, window, cx| handler(window, cx));
                                });
                            if let Some((position, toggled)) = entry.toggle {
                                menu_entry = menu_entry.toggle(position, toggled);
                            }
                            if let Some(action) = entry.action {
                                menu_entry = menu_entry.action(action);
                            }
                            context_menu.item(menu_entry)
                        } else if let Some(label_color) = entry.label_color {
                            let label = entry.label;
                            context_menu.custom_entry(
                                move |_, _| {
                                    Label::new(label.clone())
                                        .color(label_color)
                                        .truncate()
                                        .into_any_element()
                                },
                                move |_, cx| {
                                    let _ = target.update(cx, |_, window, cx| handler(window, cx));
                                },
                            )
                        } else if let Some((position, toggled)) = entry.toggle {
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
        Self {
            menu: context_menu,
            target_window,
            trigger_id,
            popup_width: menu_width + POPUP_OUTSET * 2.,
            maximum_height,
            fitted_height: Rc::default(),
        }
    }
}

impl Render for AnchoredMenuWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let fitted_height = self.fitted_height.clone();
        let popup_width = self.popup_width;
        let maximum_height = self.maximum_height;
        div().size_full().p(POPUP_OUTSET).child(
            div().id("anchored-menu-content").relative().w_full().child(self.menu.clone()).child(
                canvas(
                    move |bounds, window, _| {
                        if bounds.size.height <= px(1.) {
                            return;
                        }
                        let height = clamp_pixels(
                            bounds.size.height + POPUP_OUTSET * 2.,
                            px(1.),
                            maximum_height,
                        );
                        if fitted_height.replace(Some(height)) != Some(height) {
                            window.resize(size(popup_width, height));
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            ),
        )
    }
}

/// Close any button-triggered menu owned by `parent`. Returning `true` for the same trigger
/// gives native popups toggle behavior and prevents their dismissing click from opening a second
/// popup before the first child window has finished closing.
fn dismiss_popup_for_trigger(
    trigger_id: &ElementId,
    parent: AnyWindowHandle,
    cx: &mut App,
) -> bool {
    let mut matched_trigger = false;
    let open_menus = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<AnchoredMenuWindow>())
        .collect::<Vec<_>>();

    for popup in open_menus {
        let (owned_by_parent, matches_trigger) = popup
            .read(cx)
            .map(|menu| {
                (
                    menu.target_window == parent && menu.trigger_id.is_some(),
                    menu.target_window == parent && menu.trigger_id.as_ref() == Some(trigger_id),
                )
            })
            .unwrap_or_default();
        if owned_by_parent {
            matched_trigger |= matches_trigger;
            let _ = popup.update(cx, |_, window, _| window.remove_window());
        }
    }

    matched_trigger
}

fn open_popup(
    mut menu: AnchoredContextMenu,
    trigger_bounds: Bounds<Pixels>,
    anchor: Anchor,
    trigger_id: Option<ElementId>,
    parent_window: &mut Window,
    cx: &mut App,
) {
    let display = parent_window.display(cx);
    let display_id = display.as_ref().map(|display| display.id());
    let display_height = display
        .as_ref()
        .map(|display| display.visible_bounds().size.height)
        .unwrap_or_else(|| parent_window.bounds().size.height);
    let maximum_height =
        (parent_window.viewport_size().height - POPUP_GAP).max(px(1.)).min(display_height);
    let maximum_width = display
        .as_ref()
        .map(|display| display.visible_bounds().size.width)
        .unwrap_or_else(|| parent_window.bounds().size.width);
    let popup_height = if menu.has_custom_rows {
        maximum_height
    } else {
        clamp_pixels(menu.height + POPUP_OUTSET * 2., px(1.), maximum_height)
    };
    let popup_width = clamp_pixels(menu.width + POPUP_OUTSET * 2., px(1.), maximum_width);
    menu.height = (maximum_height - POPUP_OUTSET * 2.).max(px(1.));
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
            window_min_size: Some(size(popup_width, px(1.))),
            ..Default::default()
        },
        move |window, cx| {
            cx.new(|cx| AnchoredMenuWindow::new(menu, trigger_id, maximum_height, window, cx))
        },
    );

    match opened {
        Ok(_) => parent_window.refresh(),
        Err(error) => eprintln!("chartr could not open a menu: {error}"),
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
#[path = "popup_tests.rs"]
mod tests;
