use std::borrow::Borrow;
use std::rc::Rc;

use crate::prelude::*;
use crate::{Color, KeyBinding, Label, LabelSize, StyledExt, h_flex, v_flex};
use gpui::{
    Action, AnyElement, AnyView, AnyWindowHandle, AppContext, Bounds, DisplayId, FocusHandle,
    Global, IntoElement, Pixels, Point, Render, Size, WeakEntity, WindowBackgroundAppearance,
    WindowBounds, WindowKind, WindowOptions, canvas, point, px, size,
};

const NATIVE_TOOLTIP_OUTSET: Pixels = px(8.);

#[derive(RegisterComponent)]
pub struct Tooltip {
    title: Title,
    meta: Option<SharedString>,
    key_binding: Option<KeyBinding>,
    native_popup_enabled: bool,
    native_popup: Option<AnyWindowHandle>,
    native_popup_requested: bool,
    native_popup_failed: bool,
    release_registered: bool,
}

// A child webview is composited above the parent GPUI scene by the operating system, so no GPUI
// elevation can place an in-window tooltip over it. Keep the active tooltip in a passive native
// child window, just as Chartr does for menus.
#[derive(Default)]
struct NativeTooltipRegistry {
    current: Option<NativeTooltipRegistration>,
}

impl Global for NativeTooltipRegistry {}

struct NativeTooltipRegistration {
    window: AnyWindowHandle,
    owner: WeakEntity<Tooltip>,
}

pub(crate) fn dismiss_native_tooltip(cx: &mut App) {
    let Some(registration) = cx.default_global::<NativeTooltipRegistry>().current.take() else {
        return;
    };
    let _ = registration.window.update(cx, |_, window, _| window.remove_window());
    if let Some(owner) = registration.owner.upgrade() {
        owner.update(cx, |tooltip, cx| {
            tooltip.native_popup = None;
            tooltip.native_popup_requested = false;
            cx.notify();
        });
    }
}

pub(crate) fn enable_native_tooltip(view: &AnyView, cx: &mut App) {
    if let Ok(tooltip) = view.clone().downcast::<Tooltip>() {
        tooltip.update(cx, |tooltip, _| tooltip.native_popup_enabled = true);
    }
}

#[derive(Clone, IntoElement)]
enum Title {
    Str(SharedString),
    Callback(Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>),
}

impl From<SharedString> for Title {
    fn from(value: SharedString) -> Self {
        Title::Str(value)
    }
}

impl RenderOnce for Title {
    fn render(self, window: &mut Window, cx: &mut App) -> impl gpui::IntoElement {
        match self {
            Title::Str(title) => title.into_any_element(),
            Title::Callback(element) => element(window, cx),
        }
    }
}

impl Tooltip {
    fn from_parts(
        title: Title,
        meta: Option<SharedString>,
        key_binding: Option<KeyBinding>,
    ) -> Self {
        Self {
            title,
            meta,
            key_binding,
            native_popup_enabled: false,
            native_popup: None,
            native_popup_requested: false,
            native_popup_failed: false,
            release_registered: false,
        }
    }

    pub fn simple(title: impl Into<SharedString>, cx: &mut App) -> AnyView {
        cx.new(|_| Self::from_parts(Title::Str(title.into()), None, None))
        .into()
    }

    pub fn text(title: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
        let title = title.into();
        move |_, cx| {
            cx.new(|_| Self::from_parts(title.clone().into(), None, None))
            .into()
        }
    }

    pub fn for_action_title<T: Into<SharedString>>(
        title: T,
        action: &dyn Action,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + use<T> {
        let title = title.into();
        let action = action.boxed_clone();
        move |_, cx| {
            cx.new(|cx| {
                Self::from_parts(
                    Title::Str(title.clone()),
                    None,
                    Some(KeyBinding::for_action(action.as_ref(), cx)),
                )
            })
            .into()
        }
    }

    pub fn for_action_title_in<Str: Into<SharedString>>(
        title: Str,
        action: &dyn Action,
        focus_handle: &FocusHandle,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + use<Str> {
        let title = title.into();
        let action = action.boxed_clone();
        let focus_handle = focus_handle.clone();
        move |_, cx| {
            cx.new(|cx| {
                Self::from_parts(
                    Title::Str(title.clone()),
                    None,
                    Some(KeyBinding::for_action_in(action.as_ref(), &focus_handle, cx)),
                )
            })
            .into()
        }
    }

    pub fn for_action(
        title: impl Into<SharedString>,
        action: &dyn Action,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| {
            Self::from_parts(
                Title::Str(title.into()),
                None,
                Some(KeyBinding::for_action(action, cx)),
            )
        })
        .into()
    }

    pub fn for_action_in(
        title: impl Into<SharedString>,
        action: &dyn Action,
        focus_handle: &FocusHandle,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| {
            Self::from_parts(
                title.into().into(),
                None,
                Some(KeyBinding::for_action_in(action, focus_handle, cx)),
            )
        })
        .into()
    }

    pub fn with_meta(
        title: impl Into<SharedString>,
        action: Option<&dyn Action>,
        meta: impl Into<SharedString>,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| {
            Self::from_parts(
                title.into().into(),
                Some(meta.into()),
                action.map(|action| KeyBinding::for_action(action, cx)),
            )
        })
        .into()
    }

    pub fn with_meta_in(
        title: impl Into<SharedString>,
        action: Option<&dyn Action>,
        meta: impl Into<SharedString>,
        focus_handle: &FocusHandle,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| {
            Self::from_parts(
                title.into().into(),
                Some(meta.into()),
                action.map(|action| KeyBinding::for_action_in(action, focus_handle, cx)),
            )
        })
        .into()
    }

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self::from_parts(title.into().into(), None, None)
    }

    pub fn new_element(title: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        Self::from_parts(Title::Callback(Rc::new(title)), None, None)
    }

    pub fn element(
        title: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView {
        let title = Title::Callback(Rc::new(title));
        move |_, cx| {
            let title = title.clone();
            cx.new(|_| Self::from_parts(title, None, None))
            .into()
        }
    }

    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn key_binding(mut self, key_binding: impl Into<Option<KeyBinding>>) -> Self {
        self.key_binding = key_binding.into();
        self
    }
}

impl Render for Tooltip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.native_popup_enabled {
            return tooltip_content(
                self.title.clone(),
                self.meta.clone(),
                self.key_binding.clone(),
                cx,
            )
            .into_any_element();
        }

        if !self.release_registered {
            cx.on_release(|this, cx| {
                if let Some(popup) = this.native_popup.take() {
                    let _ = popup.update(cx, |_, window, _| window.remove_window());
                    let registry = cx.default_global::<NativeTooltipRegistry>();
                    if registry.current.as_ref().is_some_and(|current| current.window == popup) {
                        registry.current = None;
                    }
                }
            })
            .detach();
            self.release_registered = true;
        }

        if self.native_popup.is_some() {
            return div().into_any_element();
        }

        let content = tooltip_content(
            self.title.clone(),
            self.meta.clone(),
            self.key_binding.clone(),
            cx,
        );

        if self.native_popup_failed || self.native_popup_requested {
            return content.into_any_element();
        }

        self.native_popup_requested = true;
        let tooltip = cx.weak_entity();
        let parent = window.window_handle();
        let mouse_position = window.mouse_position();
        let rem_size = window.rem_size();
        let display_id = window.display(cx).map(|display| display.id());
        let title = self.title.clone();
        let meta = self.meta.clone();
        let key_binding = self.key_binding.clone();

        // GPUI has already applied its normal tooltip layout here. Measure that exact content and
        // use the resulting size for the native window so existing tooltip styling and wrapping do
        // not change.
        div()
            .relative()
            .child(content)
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let tooltip = tooltip.clone();
                        let title = title.clone();
                        let meta = meta.clone();
                        let key_binding = key_binding.clone();
                        cx.defer(move |cx| {
                            let Some(tooltip) = tooltip.upgrade() else {
                                return;
                            };
                            if tooltip.read(cx).native_popup.is_some() {
                                return;
                            }

                            match open_native_tooltip(
                                parent,
                                mouse_position,
                                bounds.size,
                                rem_size,
                                display_id,
                                title,
                                meta,
                                key_binding,
                                cx,
                            ) {
                                Ok(popup) => {
                                    let popup = popup.into();
                                    dismiss_native_tooltip(cx);
                                    tooltip.update(cx, |this, cx| {
                                        this.native_popup = Some(popup);
                                        cx.notify();
                                    });
                                    cx.default_global::<NativeTooltipRegistry>().current =
                                        Some(NativeTooltipRegistration {
                                            window: popup,
                                            owner: tooltip.downgrade(),
                                        });
                                }
                                Err(error) => {
                                    log::error!("Could not open native tooltip: {error}");
                                    let _ = tooltip.update(cx, |this, cx| {
                                        this.native_popup_failed = true;
                                        cx.notify();
                                    });
                                }
                            }
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .into_any_element()
    }
}

struct NativeTooltipWindow {
    title: Title,
    meta: Option<SharedString>,
    key_binding: Option<KeyBinding>,
}

impl Render for NativeTooltipWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().p(NATIVE_TOOLTIP_OUTSET).child(
            tooltip_content(
                self.title.clone(),
                self.meta.clone(),
                self.key_binding.clone(),
                cx,
            )
            .id("native-tooltip-body")
            .debug_selector(|| "NATIVE_TOOLTIP_BODY".into()),
        )
    }
}

fn tooltip_content<C>(
    title: Title,
    meta: Option<SharedString>,
    key_binding: Option<KeyBinding>,
    cx: &mut C,
) -> Div
where
    C: AppContext + Borrow<App>,
{
    tooltip_container(cx, |el, _| {
        el.child(
            h_flex()
                .gap_4()
                .child(div().max_w_72().child(title))
                .when_some(key_binding, |this, key_binding| {
                    this.justify_between().child(key_binding)
                }),
        )
        .when_some(meta, |this, meta| {
            this.child(
                div()
                    .max_w_72()
                    .child(Label::new(meta).size(LabelSize::Small).color(Color::Muted)),
            )
        })
    })
}

fn open_native_tooltip(
    parent: AnyWindowHandle,
    mouse_position: Point<Pixels>,
    tooltip_size: Size<Pixels>,
    rem_size: Pixels,
    display_id: Option<DisplayId>,
    title: Title,
    meta: Option<SharedString>,
    key_binding: Option<KeyBinding>,
    cx: &mut App,
) -> Result<gpui::WindowHandle<NativeTooltipWindow>, String> {
    use gpui::popup::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupOptions};

    let popup_size = size(
        (tooltip_size.width + NATIVE_TOOLTIP_OUTSET * 2.).max(px(1.)),
        (tooltip_size.height + NATIVE_TOOLTIP_OUTSET * 2.).max(px(1.)),
    );
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Default::default(),
                popup_size,
            ))),
            titlebar: None,
            focus: false,
            show: true,
            kind: WindowKind::AnchoredPopup(PopupOptions {
                parent,
                anchor_rect: Bounds::new(mouse_position, size(px(1.), px(1.))),
                anchor: PopupAnchor::TopLeft,
                gravity: PopupGravity::BottomRight,
                constraint_adjustment: PopupConstraintAdjustment::SLIDE_X
                    | PopupConstraintAdjustment::SLIDE_Y
                    | PopupConstraintAdjustment::FLIP_X
                    | PopupConstraintAdjustment::FLIP_Y,
                offset: point(
                    px(1.) - NATIVE_TOOLTIP_OUTSET,
                    px(1.) - NATIVE_TOOLTIP_OUTSET,
                ),
                grab: false,
            }),
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            display_id,
            window_background: WindowBackgroundAppearance::Transparent,
            window_min_size: Some(popup_size),
            ..Default::default()
        },
        move |window, cx| {
            // The tooltip was measured in its parent window. Native windows start with GPUI's
            // default rem size, so preserve the parent's scale before laying the content out
            // again or the text can grow beyond the measured popup bounds.
            window.set_rem_size(rem_size);
            cx.new(|_| NativeTooltipWindow {
                title,
                meta,
                key_binding,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub fn tooltip_container<C>(cx: &mut C, f: impl FnOnce(Div, &mut C) -> Div) -> Div
where
    C: AppContext + Borrow<App>,
{
    let app = (*cx).borrow();
    let ui_font = theme::theme_settings(app).ui_font(app).clone();

    // padding to avoid tooltip appearing right below the mouse cursor
    div().pl_2().pt_2p5().child(
        v_flex()
            .elevation_2(app)
            .font(ui_font)
            .text_ui(app)
            .text_color(app.theme().colors().text)
            .py_1()
            .px_2()
            .map(|el| f(el, cx)),
    )
}

pub struct LinkPreview {
    link: SharedString,
}

impl LinkPreview {
    pub fn new(url: &str, cx: &mut App) -> AnyView {
        let mut wrapped_url = String::new();
        for (i, ch) in url.chars().enumerate() {
            if i == 500 {
                wrapped_url.push('…');
                break;
            }
            if i % 100 == 0 && i != 0 {
                wrapped_url.push('\n');
            }
            wrapped_url.push(ch);
        }
        cx.new(|_| LinkPreview {
            link: wrapped_url.into(),
        })
        .into()
    }
}

impl Render for LinkPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tooltip_container(cx, |el, _| {
            el.child(
                Label::new(self.link.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
        })
    }
}

impl Component for Tooltip {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> &'static str {
        "A tooltip that appears when hovering over an element, \
        optionally showing a keybinding or additional metadata."
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> AnyElement {
        example_group(vec![single_example(
            "Text only",
            Button::new("delete-example", "Delete")
                .tooltip(Tooltip::text("This is a tooltip!"))
                .into_any_element(),
        )])
        .into_any_element()
    }
}
