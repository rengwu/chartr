//! Lazy native surface hosting. This module has no plugin-specific behavior.
use crate::{
    item::PluginView,
    text_input::TextInput,
    web_plugin::{FocusHandler, NativeViewLease, NativeViewLeaseOwner},
};
use anyhow::{Context as _, Result, bail};
use chartr_native_plugin::{self as abi, Event, Input};
use futures::{StreamExt as _, channel::mpsc};
use gpui::{
    App, AppContext as _, Bounds, Context, Element, ElementId, Entity, FocusHandle, Focusable as _,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, MouseButton, ParentElement as _,
    Pixels, Render, Size, Style, Styled as _, Window, div, px,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    ffi::{CString, c_void},
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};
use theme::ActiveTheme as _;
use ui::{Tooltip, prelude::*};
use wry::raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub type TitleHandler = Rc<dyn Fn(String, &mut App)>;
type Sender = mpsc::UnboundedSender<Event>;
struct Library {
    api: abi::Api,
    _code: libloading::Library,
}
#[derive(Default)]
struct Runtime {
    libraries: BTreeMap<PathBuf, Rc<Library>>,
    surfaces: Vec<Weak<Surface>>,
}
thread_local! { static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::default()); }

fn load(path: &Path) -> Result<Rc<Library>> {
    let path = path.canonicalize()?;
    if let Some(library) = RUNTIME.with(|r| r.borrow().libraries.get(&path).cloned()) {
        return Ok(library);
    }
    // SAFETY: explicit installation consent authorizes this native code. Symbol
    // and ABI header are validated before reading the versioned function table.
    let library = unsafe {
        let code = libloading::Library::new(&path)?;
        let entry: libloading::Symbol<unsafe extern "C" fn() -> *const abi::Api> =
            code.get(abi::ENTRY_POINT)?;
        let table = entry();
        if table.is_null() {
            bail!("The plugin returned an empty native interface");
        }
        let header = table.cast::<u32>();
        if *header != abi::ABI_VERSION || *header.add(1) < std::mem::size_of::<abi::Api>() as u32 {
            bail!("This plugin uses an unsupported native interface version");
        }
        Rc::new(Library { api: *table, _code: code })
    };
    RUNTIME.with(|r| {
        r.borrow_mut().libraries.insert(path, library.clone());
    });
    Ok(library)
}

struct Surface {
    library: Rc<Library>,
    handle: Cell<*mut c_void>,
    // Keep the callback target alive until destroy has synchronized callbacks.
    _sender: Box<Sender>,
    parent: usize,
}
impl Surface {
    fn close(&self) {
        let handle = self.handle.replace(std::ptr::null_mut());
        if !handle.is_null() {
            unsafe {
                (self.library.api.destroy)(handle);
            }
        }
    }
    fn dispatch(&self, input: Input) {
        if self.handle.get().is_null() {
            return;
        }
        if let Ok(bytes) = serde_json::to_vec(&input) {
            unsafe {
                (self.library.api.dispatch)(self.handle.get(), abi::Bytes::new(&bytes));
            }
        }
    }
    fn focus(&self, content: bool) {
        if !self.handle.get().is_null() {
            unsafe {
                (self.library.api.focus)(self.handle.get(), content);
            }
        }
    }
    fn visible(&self, visible: bool) {
        if !self.handle.get().is_null() {
            unsafe {
                (self.library.api.visible)(self.handle.get(), visible);
            }
        }
    }
    fn resize(&self, bounds: Bounds<Pixels>, scale: f32) {
        if !self.handle.get().is_null() {
            unsafe {
                (self.library.api.resize)(
                    self.handle.get(),
                    bounds.left().as_f32() as f64,
                    bounds.top().as_f32() as f64,
                    bounds.size.width.as_f32() as f64,
                    bounds.size.height.as_f32() as f64,
                    scale as f64,
                );
            }
        }
    }
}
impl Drop for Surface {
    fn drop(&mut self) {
        self.close();
    }
}

unsafe extern "C" fn emit(context: *mut c_void, bytes: abi::Bytes) {
    // A native plugin is trusted code, but reject malformed messages instead of
    // allowing unbounded allocations. The callback never re-enters plugin code.
    if context.is_null() || bytes.data.is_null() || bytes.len > abi::MAX_MESSAGE_BYTES {
        return;
    }
    let _ = std::panic::catch_unwind(|| {
        let sender = unsafe { &*context.cast::<Sender>() };
        let bytes = unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) };
        if let Ok(event) = serde_json::from_slice(bytes) {
            let _ = sender.unbounded_send(event);
        }
    });
}

fn close_parent(parent: usize) {
    let surfaces: Vec<_> = RUNTIME.with(|r| {
        r.borrow()
            .surfaces
            .iter()
            .filter_map(Weak::upgrade)
            .filter(|s| s.parent == parent)
            .collect()
    });
    for surface in surfaces {
        surface.close();
    }
}
pub fn prepare_to_quit() {
    let surfaces: Vec<_> =
        RUNTIME.with(|r| r.borrow().surfaces.iter().filter_map(Weak::upgrade).collect());
    for surface in surfaces {
        surface.close();
    }
}
pub fn shutdown() {
    prepare_to_quit();
    let libraries: Vec<_> = RUNTIME.with(|r| r.borrow().libraries.values().cloned().collect());
    for library in libraries {
        unsafe {
            (library.api.shutdown)();
        }
    }
    // Keep library code mapped: platform callbacks/static destructors may still
    // refer to it. The OS releases mappings at process exit, without dlclose.
    RUNTIME.with(|r| {
        for (_, library) in std::mem::take(&mut r.borrow_mut().libraries) {
            std::mem::forget(library);
        }
    });
}

pub fn pane(
    package: PathBuf,
    library: PathBuf,
    data: PathBuf,
    instance: &chartr_plugin::InstanceContext,
    on_focus: Option<FocusHandler>,
    on_title: Option<TitleHandler>,
    window: &mut Window,
    cx: &mut App,
) -> PluginView {
    let (tx, rx) = mpsc::unbounded();
    let theme = current_theme(cx);
    let config = abi::Instance {
        space: instance.space.clone(),
        instance_id: instance.instance_id,
        theme: theme.clone(),
    };
    let result = (|| -> Result<Rc<Surface>> {
        let library = load(&library).context("Loading the native plugin")?;
        let (parent_kind, parent) = match window.window_handle()?.as_raw() {
            RawWindowHandle::Xcb(h) => (abi::PARENT_X11, h.window.get() as usize),
            RawWindowHandle::Xlib(h) => (abi::PARENT_X11, h.window as usize),
            RawWindowHandle::AppKit(h) => (abi::PARENT_NS_VIEW, h.ns_view.as_ptr() as usize),
            _ => bail!("This window backend does not support embedded native plugins"),
        };
        let mut sender = Box::new(tx.clone());
        let config = serde_json::to_vec(&config)?;
        let package = CString::new(package.to_string_lossy().as_bytes())?;
        let data = CString::new(data.to_string_lossy().as_bytes())?;
        let options = abi::Create {
            abi: abi::ABI_VERSION,
            size: std::mem::size_of::<abi::Create>() as u32,
            parent_kind,
            parent,
            package_dir: package.as_ptr(),
            data_dir: data.as_ptr(),
            instance: abi::Bytes::new(&config),
            host: abi::Host { context: (&mut *sender as *mut Sender).cast(), emit },
        };
        let handle = unsafe { (library.api.create)(&options) };
        if handle.is_null() {
            bail!("The native plugin could not create a view");
        }
        let surface =
            Rc::new(Surface { library, handle: Cell::new(handle), _sender: sender, parent });
        RUNTIME.with(|r| {
            let mut r = r.borrow_mut();
            r.surfaces.retain(|s| s.strong_count() > 0);
            r.surfaces.push(Rc::downgrade(&surface));
        });
        window.on_window_should_close(cx, move |_, _| {
            close_parent(parent);
            true
        });
        Ok(surface)
    })();
    let (surface, error) = match result {
        Ok(s) => (Some(s), None),
        Err(e) => (None, Some(format!("{e:#}"))),
    };
    let close = surface.clone();
    let view = cx.new(|cx| {
        NativePane::new(package, surface, error, rx, theme, on_focus, on_title, window, cx)
    });
    PluginView::with_close(view.into(), move || {
        if let Some(surface) = close {
            surface.close();
        }
    })
}

struct NativePane {
    package: PathBuf,
    surface: Option<Rc<Surface>>,
    error: Option<String>,
    controls: Vec<abi::Control>,
    shortcuts: Vec<abi::Shortcut>,
    inputs: BTreeMap<String, Entity<TextInput>>,
    content_focus: FocusHandle,
    visibility: NativeViewLeaseOwner,
    theme: abi::Theme,
    on_focus: Option<FocusHandler>,
    on_title: Option<TitleHandler>,
    _events: gpui::Task<()>,
}
impl NativePane {
    fn new(
        package: PathBuf,
        surface: Option<Rc<Surface>>,
        error: Option<String>,
        mut rx: mpsc::UnboundedReceiver<Event>,
        theme: abi::Theme,
        on_focus: Option<FocusHandler>,
        on_title: Option<TitleHandler>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let events = cx.spawn_in(window, async move |this, cx| {
            while let Some(event) = rx.next().await {
                if this.update_in(cx, |this, window, cx| this.event(event, window, cx)).is_err() {
                    break;
                }
            }
        });
        Self {
            package,
            surface,
            error,
            controls: Vec::new(),
            shortcuts: Vec::new(),
            inputs: BTreeMap::new(),
            content_focus: cx.focus_handle(),
            visibility: NativeViewLeaseOwner::default(),
            theme,
            on_focus,
            on_title,
            _events: events,
        }
    }
    fn event(&mut self, event: Event, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            Event::Wake => {
                if let Some(surface) = &self.surface {
                    surface.dispatch(Input::Poll);
                }
            }
            Event::Error { message } => self.error = Some(message),
            Event::State { title, controls, shortcuts } => {
                // Keep the UI contract deliberately small; no arbitrary layout tree.
                if controls.len() > 32 || shortcuts.len() > 64 {
                    return;
                }
                self.inputs
                    .retain(|id, _| controls.iter().any(|c| &c.id == id && c.value.is_some()));
                for control in &controls {
                    if let Some(value) = &control.value {
                        let input = self.inputs.entry(control.id.clone()).or_insert_with(|| {
                            cx.new(|cx| TextInput::new(control.label.clone(), cx))
                        });
                        if !input.focus_handle(cx).is_focused(window)
                            && input.read(cx).text() != value
                        {
                            input.update(cx, |input, cx| input.set_text(value.clone(), false, cx));
                        }
                    }
                }
                self.controls = controls;
                self.shortcuts = shortcuts;
                if let Some(on_title) = &self.on_title {
                    on_title(title, cx);
                }
            }
            Event::Focus { control: Some(id), select_all } => {
                self.focus_input(&id, select_all, window, cx)
            }
            Event::Focus { control: None, .. } => {
                self.content_focus.focus(window, cx);
                if let Some(on_focus) = &self.on_focus {
                    on_focus(cx.entity_id(), cx);
                }
            }
        }
        cx.notify();
    }
    fn focus_input(&self, id: &str, select_all: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(input) = self.inputs.get(id) {
            if let Some(surface) = &self.surface {
                surface.focus(false);
            }
            if select_all {
                let value = self
                    .controls
                    .iter()
                    .find(|c| c.id == id)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_default();
                input.update(cx, |input, cx| input.set_text(value, true, cx));
            }
            input.focus_handle(cx).focus(window, cx);
            if let Some(on_focus) = &self.on_focus {
                on_focus(cx.entity_id(), cx);
            }
        }
    }
    fn action(&self, id: String, value: Option<String>) {
        if let Some(surface) = &self.surface {
            surface.dispatch(Input::Action { id, value });
        }
    }
    fn icon(&self, control: &abi::Control) -> Option<Icon> {
        let icon = Path::new(control.icon.as_deref()?);
        if icon.is_absolute()
            || icon.components().any(|p| matches!(p, std::path::Component::ParentDir))
        {
            return None;
        }
        Some(
            Icon::from_external_svg(self.package.join(icon).to_string_lossy().into_owned().into())
                .size(IconSize::Small)
                .color(Color::Muted),
        )
    }
}
impl Render for NativePane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(cx);
        if theme != self.theme {
            if let Some(surface) = &self.surface {
                surface.dispatch(Input::Theme { theme: theme.clone() });
            }
            self.theme = theme;
        }
        let colors = cx.theme().colors();
        let mut toolbar = h_flex()
            .w_full()
            .h(px(40.))
            .flex_none()
            .gap_1()
            .px_2()
            .border_b_1()
            .border_color(colors.border_variant)
            .bg(colors.surface_background);
        for control in self.controls.clone() {
            let id = control.id.clone();
            let icon = self.icon(&control);
            if let Some(input) = self.inputs.get(&id).cloned() {
                let focus = input.focus_handle(cx);
                let field_id = id.clone();
                toolbar = toolbar.child(
                    h_flex()
                        .id(format!("native-field-{id}"))
                        .h(crate::components::FORM_CONTROL_SIZE.rems())
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .border_1()
                        .border_color(colors.border_variant)
                        .bg(colors.element_background)
                        .track_focus(&focus)
                        .in_focus(|field| field.border_color(colors.border_focused))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                this.focus_input(&field_id, false, window, cx)
                            }),
                        )
                        .children(icon)
                        .child(input),
                );
            } else {
                let enabled = control.enabled;
                let mut button = h_flex()
                    .id(format!("native-button-{id}"))
                    .h(px(28.))
                    .min_w(px(24.))
                    .px_1()
                    .justify_center()
                    .rounded_sm()
                    .opacity(if enabled { 1. } else { 0.4 })
                    .tooltip(Tooltip::text(control.label.clone()));
                if let Some(icon) = icon {
                    button = button.child(icon);
                } else {
                    button = button.child(control.label);
                }
                if enabled {
                    button = button
                        .cursor_pointer()
                        .hover(|b| b.bg(colors.element_hover))
                        .on_click(cx.listener(move |this, _, _, _| this.action(id.clone(), None)));
                }
                toolbar = toolbar.child(button);
            }
        }
        let mut root =
            div().size_full().flex().flex_col().track_focus(&self.content_focus).on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                    let key = event.keystroke.unparse();
                    let focused =
                        this.inputs.iter().find(|(_, i)| i.focus_handle(cx).is_focused(window));
                    if key == "enter"
                        && let Some((id, input)) = focused
                    {
                        let id = id.clone();
                        let value = input.read(cx).text().to_owned();
                        this.content_focus.focus(window, cx);
                        this.action(id, Some(value));
                        cx.stop_propagation();
                        return;
                    }
                    if let Some(shortcut) = this
                        .shortcuts
                        .iter()
                        .find(|s| s.key == key && (!s.content_only || focused.is_none()))
                    {
                        this.action(shortcut.action.clone(), None);
                        cx.stop_propagation();
                    }
                }),
            );
        if !self.controls.is_empty() {
            root = root.child(toolbar);
        }
        if let Some(surface) = &self.surface {
            root = root.child(div().w_full().flex_1().min_h_0().child(SurfaceElement {
                surface: surface.clone(),
                visibility: self.visibility.clone(),
                id: "native-surface".into(),
            }));
        }
        if let Some(error) = &self.error {
            root = root.child(div().p_3().text_color(colors.text_muted).child(error.clone()));
        }
        root
    }
}
fn current_theme(cx: &App) -> abi::Theme {
    let c = cx.theme().colors();
    fn css(c: gpui::Hsla) -> String {
        format!("hsla({:.1}, {:.1}%, {:.1}%, {:.3})", c.h * 360., c.s * 100., c.l * 100., c.a)
    }
    abi::Theme {
        page: css(c.editor_background),
        field: css(c.element_background),
        border: css(c.border_variant),
        focus: css(c.border_focused),
        text: css(c.text),
        muted: css(c.text_muted),
        ui_font_size: format!("{}px", theme::theme_settings(cx).ui_font_size(cx).as_f32()),
    }
}

struct SurfaceElement {
    surface: Rc<Surface>,
    visibility: NativeViewLeaseOwner,
    id: ElementId,
}
struct Visible {
    surface: Weak<Surface>,
    lease: NativeViewLease,
    bounds: Option<(Bounds<Pixels>, f32)>,
    visible: bool,
}
impl Drop for Visible {
    fn drop(&mut self) {
        if self.lease.is_current()
            && let Some(surface) = self.surface.upgrade()
        {
            surface.focus(false);
            surface.visible(false);
        }
    }
}
impl IntoElement for SurfaceElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for SurfaceElement {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (window.request_layout(Style { size: Size::full(), ..Style::default() }, [], cx), ())
    }
    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let frame = (bounds, window.scale_factor());
        window.with_element_state(id.unwrap(), |state: Option<Visible>, _| {
            let mut state = state.unwrap_or_else(|| Visible {
                surface: Rc::downgrade(&self.surface),
                lease: self.visibility.acquire(),
                bounds: None,
                visible: false,
            });
            if state.bounds != Some(frame) {
                self.surface.resize(bounds, frame.1);
                state.bounds = Some(frame);
            }
            let visible =
                !cx.has_active_drag() && bounds.size.width > px(0.) && bounds.size.height > px(0.);
            if state.visible != visible {
                self.surface.visible(visible);
                state.visible = visible;
            }
            ((), state)
        });
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }
}
