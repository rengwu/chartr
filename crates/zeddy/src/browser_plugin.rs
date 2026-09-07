//! Chartr's host-owned browser surface, activated by a separately installed plugin package.

#![forbid(unsafe_code)]

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    process::Command,
    rc::{Rc, Weak},
};

use futures::{StreamExt as _, channel::mpsc};
use gpui::{
    App, AppContext as _, Bounds, Context, Element, ElementId, FocusHandle, Focusable as _,
    GlobalElementId, InspectorElementId, IntoElement, KeyBinding, LayoutId, MouseButton,
    ParentElement as _, Pixels, Render, Size, Style, Styled as _, Window, actions, div, px,
};
use serde::Serialize;
use theme::ActiveTheme as _;
use ui::{ButtonSize, IconButtonShape, Tooltip, prelude::*};
use url::Url;
use wry::{
    PageLoadEvent, PermissionResponse, Rect, WebContext, WebViewBuilder,
    dpi::{LogicalPosition, LogicalSize, Position, Size as WrySize},
};
use zeddy_plugin::InstanceContext;

use crate::{
    item::PluginView,
    text_input::TextInput,
    web_plugin::{FocusHandler, NativeViewLease, NativeViewLeaseOwner, NativeWebViewHandle},
};

pub type TitleHandler = Rc<dyn Fn(String, &mut App)>;

actions!(
    chartr_browser,
    [SubmitAddress, FocusAddressBar, SelectPageContent, ReloadPage, GoBack, GoForward, StopLoading]
);

const TOOLBAR_HEIGHT: f32 = 40.;

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("enter", SubmitAddress, Some("ChartrBrowserAddress"))]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-l", FocusAddressBar, Some("ChartrBrowser")),
        KeyBinding::new("cmd-a", SelectPageContent, Some("ChartrBrowserContent")),
        KeyBinding::new("cmd-r", ReloadPage, Some("ChartrBrowser")),
        KeyBinding::new("cmd-[", GoBack, Some("ChartrBrowser")),
        KeyBinding::new("cmd-]", GoForward, Some("ChartrBrowser")),
    ]);
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-l", FocusAddressBar, Some("ChartrBrowser")),
        KeyBinding::new("ctrl-r", ReloadPage, Some("ChartrBrowser")),
        KeyBinding::new("alt-left", GoBack, Some("ChartrBrowser")),
        KeyBinding::new("alt-right", GoForward, Some("ChartrBrowser")),
    ]);
    cx.bind_keys([KeyBinding::new("escape", StopLoading, Some("ChartrBrowser"))]);
}

pub fn view(
    data_dir: PathBuf,
    instance: &InstanceContext,
    on_focus: Option<FocusHandler>,
    on_title_change: Option<TitleHandler>,
    window: &mut Window,
    cx: &mut App,
) -> PluginView {
    let space = instance.space.clone();
    let instance_id = instance.instance_id;
    let theme = active_pane_theme(cx);
    let content = NativeWebViewHandle::default();
    let view = cx.new(|cx| {
        BrowserView::new(
            data_dir,
            space,
            instance_id,
            theme,
            on_focus,
            on_title_change,
            content.clone(),
            window,
            cx,
        )
    });
    let close = content.clone();
    PluginView::with_close(view.into(), move || close.shutdown())
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PaneTheme {
    toolbar: gpui::Hsla,
    page: gpui::Hsla,
    field: gpui::Hsla,
    border: gpui::Hsla,
    focus: gpui::Hsla,
    text: gpui::Hsla,
    muted: gpui::Hsla,
}

#[derive(Debug)]
enum BrowserEvent {
    Navigate(String),
    Back,
    Forward,
    Reload,
    Stop,
    FocusAddress,
    FocusPane,
    TitleChanged(String),
    LoadStarted(String),
    LoadFinished(String),
}

struct BrowserView {
    content: NativeWebViewHandle,
    content_visibility: NativeViewLeaseOwner,
    content_focus: FocusHandle,
    address_input: gpui::Entity<TextInput>,
    runtime: BrowserRuntime,
    on_focus: Option<FocusHandler>,
    on_title_change: Option<TitleHandler>,
    pane_theme: PaneTheme,
    theme: BrowserTheme,
    _event_task: gpui::Task<()>,
    #[cfg(target_os = "linux")]
    _gtk_pump: gpui::Task<()>,
    error: Option<String>,
    element_key: String,
}

struct BrowserRuntime {
    content: Weak<wry::WebView>,
    current_url: Option<String>,
    last_attempt: Option<String>,
    loading: bool,
    showing_local: bool,
    state_path: PathBuf,
    theme: BrowserTheme,
}

impl BrowserView {
    fn new(
        data_dir: PathBuf,
        space: String,
        instance_id: u64,
        pane_theme: PaneTheme,
        on_focus: Option<FocusHandler>,
        on_title_change: Option<TitleHandler>,
        content: NativeWebViewHandle,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let content_visibility = NativeViewLeaseOwner::default();
        let content_focus = cx.focus_handle();
        let theme = BrowserTheme::from_pane(pane_theme, theme::theme_settings(cx).ui_font_size(cx));
        let state_path = state_path(&data_dir, &space, instance_id);
        let restored = read_saved_url(&state_path);
        let address_input = cx.new(|cx| {
            let mut input = TextInput::new("Search or enter address", cx);
            if let Some(restored) = restored.as_ref() {
                input.set_text(restored.clone(), false, cx);
            }
            input
        });
        let mut web_context = WebContext::new(None);
        let mut runtime = BrowserRuntime {
            content: Weak::new(),
            current_url: restored.clone(),
            last_attempt: restored.clone(),
            loading: restored.is_some(),
            showing_local: restored.is_none(),
            state_path,
            theme: theme.clone(),
        };
        let (event_tx, mut event_rx) = mpsc::unbounded();
        let event_task = cx.spawn_in(window, async move |this, cx| {
            while let Some(event) = event_rx.next().await {
                let _ = this.update_in(cx, |this, window, cx| this.handle_event(event, window, cx));
            }
        });

        #[cfg(target_os = "linux")]
        let gtk_pump = Self::pump_gtk(cx);
        #[cfg(target_os = "linux")]
        if let Err(error) = gtk::init() {
            return Self {
                content,
                content_visibility,
                content_focus,
                address_input,
                runtime,
                on_focus,
                on_title_change,
                pane_theme,
                theme,
                _event_task: event_task,
                _gtk_pump: gtk_pump,
                error: Some(format!("Could not initialize WebKitGTK: {error}")),
                element_key: format!("browser-{instance_id}"),
            };
        }

        let content_events = event_tx.clone();
        let window_events = event_tx.clone();
        let title_events = event_tx.clone();
        let load_events = event_tx;
        let builder = WebViewBuilder::new_with_web_context(&mut web_context);
        let builder = builder
            .with_incognito(true)
            .with_initialization_script(CONTENT_BRIDGE)
            .with_ipc_handler(move |request| {
                if let Some(event) = decode_command(request.body()) {
                    let _ = content_events.unbounded_send(event);
                }
            })
            .with_navigation_handler(|url| is_allowed_navigation(&url))
            .with_new_window_req_handler(move |url, _| {
                let _ = window_events.unbounded_send(BrowserEvent::Navigate(url));
                wry::NewWindowResponse::Deny
            })
            .with_permission_handler(|_| PermissionResponse::Deny)
            .with_download_started_handler(move |url, _| {
                open_external(&url);
                false
            })
            .with_document_title_changed_handler(move |title| {
                let _ = title_events.unbounded_send(BrowserEvent::TitleChanged(title));
            })
            .with_on_page_load_handler(move |event, url| {
                let event = match event {
                    PageLoadEvent::Started => BrowserEvent::LoadStarted(url),
                    PageLoadEvent::Finished => BrowserEvent::LoadFinished(url),
                };
                let _ = load_events.unbounded_send(event);
            })
            .with_bounds(unit_rect())
            .with_visible(false)
            .with_focused(false);
        let builder = if let Some(url) = &restored {
            builder.with_url(url)
        } else {
            builder.with_html(local_page(&runtime.theme, LocalPage::Start))
        };
        let error = match builder.build_as_child(window) {
            Ok(webview) => {
                let webview = Rc::new(webview);
                runtime.content = Rc::downgrade(&webview);
                content.install(webview);
                None
            }
            Err(error) => Some(format!("Could not create the website view: {error}")),
        };

        Self {
            content,
            content_visibility,
            content_focus,
            address_input,
            runtime,
            on_focus,
            on_title_change,
            pane_theme,
            theme,
            _event_task: event_task,
            #[cfg(target_os = "linux")]
            _gtk_pump: gtk_pump,
            error,
            element_key: format!("browser-{instance_id}"),
        }
    }

    fn handle_event(&mut self, event: BrowserEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            BrowserEvent::FocusAddress => self.focus_address(window, cx),
            BrowserEvent::FocusPane => {
                // The webview already owns native keyboard focus. Move GPUI's
                // logical focus off the address input as well so its keymap
                // cannot consume webpage shortcuts such as Cmd+A.
                window.focus(&self.content_focus, cx);
                if let Some(on_focus) = &self.on_focus {
                    on_focus(cx.entity_id(), cx);
                }
            }
            BrowserEvent::TitleChanged(title) => {
                if let Some(on_title_change) = &self.on_title_change {
                    on_title_change(display_title(&title), cx);
                }
            }
            event => {
                self.runtime.handle(event);
                self.sync_address_input(window, false, cx);
                cx.notify();
            }
        }
    }

    fn sync_address_input(&self, window: &Window, force: bool, cx: &mut Context<Self>) {
        let focus = self.address_input.focus_handle(cx);
        if force || !focus.is_focused(window) {
            let address = self.runtime.address().to_owned();
            if self.address_input.read(cx).text() != address {
                self.address_input.update(cx, |input, cx| input.set_text(address, false, cx));
            }
        }
    }

    fn focus_address(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.relinquish_content_focus();
        let address = self.runtime.address().to_owned();
        self.address_input.update(cx, |input, cx| input.set_text(address, true, cx));
        self.address_input.focus_handle(cx).focus(window, cx);
    }

    fn focus_address_from_click(&self, window: &mut Window, cx: &mut Context<Self>) {
        // GPUI focus alone does not replace WKWebView as AppKit's first
        // responder, so transfer both focus systems for an address-bar click.
        self.relinquish_content_focus();
        self.address_input.focus_handle(cx).focus(window, cx);
    }

    fn relinquish_content_focus(&self) {
        if let Some(content) = self.content.get() {
            let _ = content.focus_parent();
        }
    }

    fn submit_address(&mut self, _: &SubmitAddress, window: &mut Window, cx: &mut Context<Self>) {
        let address = self.address_input.read(cx).text().to_owned();
        self.runtime.handle(BrowserEvent::Navigate(address));
        self.sync_address_input(window, true, cx);
        cx.notify();
    }

    fn focus_address_action(
        &mut self,
        _: &FocusAddressBar,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_address(window, cx);
    }

    fn select_page_content(
        &mut self,
        _: &SelectPageContent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        if let Some(content) = self.content.get() {
            let _ = content.evaluate_script(SELECT_PAGE_CONTENT);
        }
    }

    fn reload(&mut self, _: &ReloadPage, _: &mut Window, cx: &mut Context<Self>) {
        self.runtime.handle(BrowserEvent::Reload);
        cx.notify();
    }

    fn go_back(&mut self, _: &GoBack, _: &mut Window, cx: &mut Context<Self>) {
        self.runtime.handle(BrowserEvent::Back);
        cx.notify();
    }

    fn go_forward(&mut self, _: &GoForward, _: &mut Window, cx: &mut Context<Self>) {
        self.runtime.handle(BrowserEvent::Forward);
        cx.notify();
    }

    fn stop_loading(&mut self, _: &StopLoading, _: &mut Window, cx: &mut Context<Self>) {
        self.runtime.handle(BrowserEvent::Stop);
        cx.notify();
    }
}

impl BrowserRuntime {
    fn handle(&mut self, event: BrowserEvent) {
        match event {
            BrowserEvent::Navigate(input) => self.navigate(&input),
            BrowserEvent::Back => self.control(|webview| webview.go_back()),
            BrowserEvent::Forward => self.control(|webview| webview.go_forward()),
            BrowserEvent::Reload => self.control(|webview| webview.reload()),
            BrowserEvent::Stop => {
                self.control(|webview| webview.evaluate_script("window.stop()"));
                self.loading = false;
            }
            BrowserEvent::FocusAddress
            | BrowserEvent::FocusPane
            | BrowserEvent::TitleChanged(_) => {}
            BrowserEvent::LoadStarted(url) if is_http_url(&url) => {
                self.current_url = Some(url.clone());
                self.last_attempt = Some(url);
                self.loading = true;
                self.showing_local = false;
            }
            BrowserEvent::LoadFinished(url) if is_http_url(&url) => {
                self.current_url = Some(url.clone());
                self.last_attempt = Some(url.clone());
                self.loading = false;
                self.showing_local = false;
                let _ = save_url(&self.state_path, &url);
            }
            BrowserEvent::LoadStarted(_) | BrowserEvent::LoadFinished(_) => {}
        }
    }

    fn navigate(&mut self, input: &str) {
        self.last_attempt = Some(input.to_owned());
        match resolve_input(input) {
            Ok(url) => {
                if let Some(content) = self.content.upgrade() {
                    if let Err(error) = content.load_url(url.as_str()) {
                        self.show_error(format!("Could not open this page: {error}"));
                    } else {
                        self.current_url = Some(url.to_string());
                        self.loading = true;
                        self.showing_local = false;
                        let _ = content.focus();
                    }
                }
            }
            Err(error) => self.show_error(error),
        }
    }

    fn control(&mut self, operation: impl FnOnce(&wry::WebView) -> wry::Result<()>) {
        if let Some(content) = self.content.upgrade()
            && let Err(error) = operation(content.as_ref())
        {
            self.show_error(format!("Browser control failed: {error}"));
        }
    }

    fn show_error(&mut self, message: String) {
        self.loading = false;
        self.showing_local = true;
        self.current_url = None;
        if let Some(content) = self.content.upgrade() {
            let retry = self.last_attempt.as_deref().unwrap_or_default();
            let _ = content
                .load_html(&local_page(&self.theme, LocalPage::Error { message: &message, retry }));
        }
    }

    fn address(&self) -> &str {
        self.current_url.as_deref().or(self.last_attempt.as_deref()).unwrap_or_default()
    }

    fn can_go_back(&self) -> bool {
        self.content.upgrade().and_then(|view| view.can_go_back().ok()).unwrap_or(false)
    }

    fn can_go_forward(&self) -> bool {
        self.content.upgrade().and_then(|view| view.can_go_forward().ok()).unwrap_or(false)
    }
}

impl BrowserView {
    #[cfg(target_os = "linux")]
    fn pump_gtk(cx: &mut Context<Self>) -> gpui::Task<()> {
        use std::time::Duration;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(16)).await;
                if this
                    .update(cx, |_, _| {
                        while gtk::events_pending() {
                            gtk::main_iteration_do(false);
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
    }

    fn apply_theme(&self, theme: &BrowserTheme) {
        let encoded = serde_json::to_string(theme).unwrap_or_else(|_| "{}".into());
        if self.runtime.showing_local
            && let Some(content) = self.content.get()
        {
            let _ = content.evaluate_script(&format!("window.setChartrTheme({encoded})"));
        }
    }
}

impl Render for BrowserView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_theme = active_pane_theme(cx);
        let theme = BrowserTheme::from_pane(pane_theme, theme::theme_settings(cx).ui_font_size(cx));
        if self.theme != theme {
            self.apply_theme(&theme);
            self.runtime.theme = theme.clone();
            self.theme = theme;
            self.pane_theme = pane_theme;
        }
        let can_go_back = self.runtime.can_go_back();
        let can_go_forward = self.runtime.can_go_forward();
        let loading = self.runtime.loading;
        let secure = self.runtime.address().starts_with("https://");
        let address_focus = self.address_input.focus_handle(cx);

        let back = IconButton::new(format!("{}-back", self.element_key), IconName::ArrowLeft)
            .shape(IconButtonShape::Square)
            .size(ButtonSize::None)
            .icon_size(IconSize::Medium)
            .disabled(!can_go_back)
            .aria_label("Back")
            .tooltip(Tooltip::text("Back"))
            .on_click(cx.listener(|this, _, _, cx| {
                this.runtime.handle(BrowserEvent::Back);
                cx.notify();
            }));
        let forward =
            IconButton::new(format!("{}-forward", self.element_key), IconName::ArrowRight)
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .icon_size(IconSize::Medium)
                .disabled(!can_go_forward)
                .aria_label("Forward")
                .tooltip(Tooltip::text("Forward"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.runtime.handle(BrowserEvent::Forward);
                    cx.notify();
                }));
        let reload = IconButton::new(
            format!("{}-reload", self.element_key),
            if loading { IconName::Stop } else { IconName::RotateCw },
        )
        .shape(IconButtonShape::Wide)
        .size(ButtonSize::Default)
        .icon_size(IconSize::Medium)
        .aria_label(if loading { "Stop" } else { "Reload" })
        .tooltip(Tooltip::text(if loading { "Stop" } else { "Reload" }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.runtime.handle(if loading { BrowserEvent::Stop } else { BrowserEvent::Reload });
            cx.notify();
        }));
        let address = h_flex()
            .id(format!("{}-address", self.element_key))
            .key_context("ChartrBrowserAddress")
            .h(crate::components::FORM_CONTROL_SIZE.rems())
            .flex_1()
            .min_w_0()
            .gap_1()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(self.pane_theme.border)
            .bg(self.pane_theme.field)
            .track_focus(&address_focus)
            .in_focus(|field| field.border_color(self.pane_theme.focus))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.focus_address_from_click(window, cx);
                }),
            )
            .child(
                Icon::new(if secure { IconName::Lock } else { IconName::Public })
                    .size(IconSize::XSmall)
                    .color(Color::Muted),
            )
            .child(self.address_input.clone())
            .on_action(cx.listener(Self::submit_address));
        let toolbar = h_flex()
            .w_full()
            .h(px(TOOLBAR_HEIGHT))
            .flex_none()
            .gap_1()
            .px_2()
            .border_b_1()
            .border_color(self.pane_theme.border)
            .bg(self.pane_theme.toolbar)
            .child(back)
            .child(forward)
            .child(reload)
            .child(address);

        let mut root = div()
            .key_context("ChartrBrowser")
            .size_full()
            .flex()
            .flex_col()
            .on_action(cx.listener(Self::focus_address_action))
            .on_action(cx.listener(Self::reload))
            .on_action(cx.listener(Self::go_back))
            .on_action(cx.listener(Self::go_forward))
            .on_action(cx.listener(Self::stop_loading))
            .child(toolbar);
        if let Some(content) = self.content.get() {
            root = root.child(
                div()
                    .key_context("ChartrBrowserContent")
                    .track_focus(&self.content_focus)
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .on_action(cx.listener(Self::select_page_content))
                    .child(NativeWebViewElement::new(
                        content,
                        format!("{}-content", self.element_key),
                        self.content_visibility.clone(),
                    )),
            );
        }
        if let Some(error) = &self.error {
            root = root.child(
                div()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(self.pane_theme.page)
                    .text_color(self.pane_theme.muted)
                    .child(error.clone()),
            );
        }
        root
    }
}

fn active_pane_theme(cx: &App) -> PaneTheme {
    let colors = cx.theme().colors();
    PaneTheme {
        toolbar: colors.surface_background,
        page: colors.editor_background,
        field: colors.element_background,
        border: colors.border_variant,
        focus: colors.border_focused,
        text: colors.text,
        muted: colors.text_muted,
    }
}

fn unit_rect() -> Rect {
    Rect {
        position: Position::Logical(LogicalPosition::new(0., 0.)),
        size: WrySize::Logical(LogicalSize::new(1., 1.)),
    }
}

fn open_external(url: &str) {
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "windows")]
    let command = "explorer";
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return;

    let _ = Command::new(command).arg(url).spawn();
}

fn decode_command(body: &str) -> Option<BrowserEvent> {
    let request: serde_json::Value = serde_json::from_str(body).ok()?;
    match request.get("action")?.as_str()? {
        "navigate" => Some(BrowserEvent::Navigate(request.get("value")?.as_str()?.to_owned())),
        "back" => Some(BrowserEvent::Back),
        "forward" => Some(BrowserEvent::Forward),
        "reload" => Some(BrowserEvent::Reload),
        "stop" => Some(BrowserEvent::Stop),
        "focus-address" => Some(BrowserEvent::FocusAddress),
        "focus" => Some(BrowserEvent::FocusPane),
        _ => None,
    }
}

fn resolve_input(input: &str) -> Result<Url, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter a URL or search query.".into());
    }
    if input.starts_with("http://") || input.starts_with("https://") {
        return Url::parse(input)
            .map_err(|_| "That URL is not valid.".to_owned())
            .and_then(validate_http);
    }
    if is_local_host(input) {
        return Url::parse(&format!("http://{input}"))
            .map_err(|_| "That URL is not valid.".to_owned())
            .and_then(validate_http);
    }
    if looks_like_host(input) && has_no_scheme_like_colon(input) {
        return Url::parse(&format!("https://{input}"))
            .map_err(|_| "That URL is not valid.".to_owned())
            .and_then(validate_http);
    }
    if let Ok(url) = Url::parse(input) {
        return validate_http(url);
    }
    if input.contains("://") {
        return Err("That URL is not valid.".into());
    }
    Url::parse_with_params("https://duckduckgo.com/", &[("q", input)])
        .map_err(|error| error.to_string())
}

fn validate_http(url: Url) -> Result<Url, String> {
    if matches!(url.scheme(), "http" | "https") && url.host().is_some() {
        Ok(url)
    } else {
        Err("Browser supports only HTTP and HTTPS addresses.".into())
    }
}

fn looks_like_host(input: &str) -> bool {
    !input.chars().any(char::is_whitespace) && (input.contains('.') || is_local_host(input))
}

fn has_no_scheme_like_colon(input: &str) -> bool {
    let authority = input.split(['/', '?', '#']).next().unwrap_or(input);
    !authority.contains(':')
        || authority.rsplit_once(':').is_some_and(|(_, port)| port.parse::<u16>().is_ok())
}

fn is_local_host(input: &str) -> bool {
    let host = input.split(['/', '?', '#']).next().unwrap_or(input);
    let lowercase = host.to_ascii_lowercase();
    lowercase
        .strip_prefix("localhost")
        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(':'))
        || is_ip_like(host)
}

fn is_ip_like(input: &str) -> bool {
    if let Some(bracketed) = input.strip_prefix('[').and_then(|value| value.split(']').next()) {
        return bracketed.parse::<std::net::IpAddr>().is_ok();
    }
    if input.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    input.rsplit_once(':').is_some_and(|(host, port)| {
        port.parse::<u16>().is_ok() && host.parse::<std::net::Ipv4Addr>().is_ok()
    })
}

fn is_http_url(url: &str) -> bool {
    Url::parse(url).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
}

fn is_allowed_navigation(url: &str) -> bool {
    is_http_url(url) || url == "about:blank"
}

fn display_title(title: &str) -> String {
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.is_empty() { "Browser".to_owned() } else { title }
}

fn state_path(data: &Path, space: &str, instance: u64) -> PathBuf {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in space.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    data.join("instances").join(format!("{hash:016x}-{instance}.url"))
}

fn read_saved_url(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim();
    is_http_url(value).then(|| value.to_owned())
}

fn save_url(path: &Path, url: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("url.tmp");
    fs::write(&temporary, url)?;
    fs::rename(temporary, path)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserTheme {
    page: String,
    field: String,
    border: String,
    focus: String,
    text: String,
    muted: String,
    ui_font_size: String,
}

impl BrowserTheme {
    fn from_pane(colors: PaneTheme, ui_font_size: Pixels) -> Self {
        Self {
            page: css_color(colors.page),
            field: css_color(colors.field),
            border: css_color(colors.border),
            focus: css_color(colors.focus),
            text: css_color(colors.text),
            muted: css_color(colors.muted),
            ui_font_size: format!("{}px", ui_font_size.as_f32()),
        }
    }
}

fn css_color(color: gpui::Hsla) -> String {
    format!(
        "hsla({:.1}, {:.1}%, {:.1}%, {:.3})",
        color.h * 360.,
        color.s * 100.,
        color.l * 100.,
        color.a
    )
}

enum LocalPage<'a> {
    Start,
    Error { message: &'a str, retry: &'a str },
}

fn local_page(theme: &BrowserTheme, page: LocalPage<'_>) -> String {
    let theme = serde_json::to_string(theme).unwrap_or_else(|_| "{}".into());
    let body = match page {
        LocalPage::Start => {
            "<div class='globe'>🌏</div><h1>Browse the web</h1><p>Enter a URL or search above"
                .to_owned()
        }
        LocalPage::Error { message, retry } => format!(
            "<div class='globe'>!</div><h1>Page unavailable</h1><p>{}</p><button id='retry' data-address='{}'>Retry</button>",
            escape_html(message),
            escape_html(retry),
        ),
    };
    LOCAL_HTML.replace("__THEME__", &theme).replace("__BODY__", &body)
}

fn escape_html(value: &str) -> Cow<'_, str> {
    if !value.contains(['&', '<', '>', '"', '\'']) {
        return Cow::Borrowed(value);
    }
    Cow::Owned(
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;"),
    )
}

const CONTENT_BRIDGE: &str = r#"
(() => {
  const send = (action, value) => window.ipc.postMessage(JSON.stringify({ action, value }));
  addEventListener('pointerdown', () => send('focus'), true);
  addEventListener('keydown', event => {
    const command = navigator.platform.includes('Mac') ? event.metaKey : event.ctrlKey;
    let action = null;
    if (command && event.key.toLowerCase() === 'l') action = 'focus-address';
    else if (command && event.key.toLowerCase() === 'r') action = 'reload';
    else if (event.key === 'Escape') action = 'stop';
    else if (event.altKey && event.key === 'ArrowLeft') action = 'back';
    else if (event.altKey && event.key === 'ArrowRight') action = 'forward';
    else if (event.metaKey && event.key === '[') action = 'back';
    else if (event.metaKey && event.key === ']') action = 'forward';
    if (action) { event.preventDefault(); event.stopPropagation(); send(action); }
  }, true);
})();
"#;

// Wry's child WKWebView intentionally declines macOS key equivalents so the
// host can handle menu shortcuts. That also keeps Cmd+A out of webpage
// JavaScript, so the browser-content key context performs the equivalent DOM
// selection explicitly.
const SELECT_PAGE_CONTENT: &str = r#"
(() => {
  let active = document.activeElement;
  while (active?.shadowRoot?.activeElement) active = active.shadowRoot.activeElement;

  if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) {
    try {
      active.select();
      return;
    } catch (_) {}
  }

  if (active instanceof HTMLElement && active.isContentEditable) {
    const range = document.createRange();
    range.selectNodeContents(active);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    return;
  }

  const range = document.createRange();
  range.selectNodeContents(document.body);
  const selection = window.getSelection();
  selection.removeAllRanges();
  selection.addRange(range);
})();
"#;

const LOCAL_HTML: &str = r#"<!doctype html>
<meta charset="utf-8"><meta name="viewport" content="width=device-width">
<style>
  :root { --page:#181818;--field:#202020;--border:#393939;--text:#ddd;--muted:#888;--focus:#d97757;--uiFontSize:14px; }
  html { font-size:var(--uiFontSize); } html,body { height:100%;margin:0; } body { display:grid;place-items:center;background:var(--page);color:var(--text);font:.857142857rem/1.45 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; }
  main { max-width:560px;padding:32px;text-align:center; } .globe { color:var(--muted);font-size:2rem;line-height:1;margin-bottom:1rem; } h1 { margin:0 0 .5rem;font-size:1rem;font-weight:600; } p { margin:0;color:var(--muted); } button { margin-top:24px;padding:9px 16px;border:1px solid var(--border);border-radius:8px;background:var(--field);color:var(--text);font:inherit;cursor:pointer; }
</style><main>__BODY__</main>
<script>window.setChartrTheme=t=>{for(const [key,value] of Object.entries(t))document.documentElement.style.setProperty('--'+key,value)};window.setChartrTheme(__THEME__);const retry=document.querySelector('#retry');if(retry)retry.onclick=()=>window.ipc.postMessage(JSON.stringify({action:'navigate',value:retry.dataset.address}));</script>"#;

struct NativeWebViewElement {
    webview: Rc<wry::WebView>,
    id: ElementId,
    visibility: NativeViewLeaseOwner,
}

impl NativeWebViewElement {
    fn new(
        webview: Rc<wry::WebView>,
        id: impl Into<ElementId>,
        visibility: NativeViewLeaseOwner,
    ) -> Self {
        Self { webview, id: id.into(), visibility }
    }
}

impl IntoElement for NativeWebViewElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

struct VisibleWebView {
    webview: Weak<wry::WebView>,
    lease: NativeViewLease,
    frame: Option<NativeFrame>,
    visible: bool,
}

impl Drop for VisibleWebView {
    fn drop(&mut self) {
        if !self.lease.is_current() {
            return;
        }
        if let Some(webview) = self.webview.upgrade() {
            let _ = webview.focus_parent();
            let _ = webview.set_visible(false);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct NativeFrame {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl NativeFrame {
    fn snapped(bounds: Bounds<Pixels>) -> Self {
        let left = bounds.left().as_f32().round() as i32;
        let top = bounds.top().as_f32().round() as i32;
        let right = bounds.right().as_f32().round() as i32;
        let bottom = bounds.bottom().as_f32().round() as i32;
        Self { x: left, y: top, width: (right - left).max(0), height: (bottom - top).max(0) }
    }

    fn wry(self) -> Rect {
        Rect {
            position: Position::Logical(LogicalPosition::new(f64::from(self.x), f64::from(self.y))),
            size: WrySize::Logical(LogicalSize::new(f64::from(self.width), f64::from(self.height))),
        }
    }
}

impl Element for NativeWebViewElement {
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
    ) -> (LayoutId, Self::RequestLayoutState) {
        (window.request_layout(Style { size: Size::full(), ..Style::default() }, [], cx), ())
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let id = id.expect("browser webview elements always have an id");
        let frame = NativeFrame::snapped(bounds);
        window.with_element_state(id, |lease: Option<VisibleWebView>, _| {
            let mut lease = lease.unwrap_or_else(|| VisibleWebView {
                webview: Rc::downgrade(&self.webview),
                lease: self.visibility.acquire(),
                frame: None,
                visible: false,
            });
            if lease.frame != Some(frame) {
                let _ = self.webview.set_bounds(frame.wry());
                lease.frame = Some(frame);
            }
            let visible = !cx.has_active_drag();
            if lease.visible != visible {
                let _ = self.webview.set_visible(visible);
                lease.visible = visible;
            }
            ((), lease)
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        _: &mut Window,
        _: &mut App,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_input_distinguishes_urls_local_hosts_and_searches() {
        assert_eq!(resolve_input("example.com/a").unwrap().as_str(), "https://example.com/a");
        assert_eq!(resolve_input("localhost:3000").unwrap().as_str(), "http://localhost:3000/");
        assert_eq!(resolve_input("127.0.0.1:8080").unwrap().as_str(), "http://127.0.0.1:8080/");
        assert_eq!(resolve_input("[::1]:8080").unwrap().as_str(), "http://[::1]:8080/");
        assert_eq!(
            resolve_input("example.com:8443/path").unwrap().as_str(),
            "https://example.com:8443/path"
        );
        assert_eq!(
            resolve_input("localhost.example/path").unwrap().as_str(),
            "https://localhost.example/path"
        );
        let search = resolve_input("small browser").unwrap();
        assert_eq!(search.host_str(), Some("duckduckgo.com"));
        assert_eq!(search.query_pairs().find(|(key, _)| key == "q").unwrap().1, "small browser");
    }

    #[test]
    fn non_web_schemes_are_rejected() {
        for input in ["file:///tmp/secret", "mailto:user@example.com", "tel:123"] {
            assert!(resolve_input(input).is_err(), "{input}");
        }
    }

    #[test]
    fn document_titles_are_normalized_for_tabs() {
        assert_eq!(display_title("  Example\n  Page  "), "Example Page");
        assert_eq!(display_title(" \n\t "), "Browser");
    }

    #[test]
    fn instance_state_is_namespaced_by_space_and_instance() {
        let root = Path::new("/data");
        assert_ne!(state_path(root, "folder:a", 7), state_path(root, "folder:b", 7));
        assert_ne!(state_path(root, "folder:a", 7), state_path(root, "folder:a", 8));
    }

    #[test]
    fn local_pages_follow_the_host_interface_font_size() {
        let color = gpui::black();
        let pane = PaneTheme {
            toolbar: color,
            page: color,
            field: color,
            border: color,
            focus: color,
            text: color,
            muted: color,
        };
        let theme = BrowserTheme::from_pane(pane, px(18.));
        let page = local_page(&theme, LocalPage::Start);

        assert_eq!(theme.ui_font_size, "18px");
        assert!(page.contains(r#""uiFontSize":"18px""#));
        assert!(page.contains("html { font-size:var(--uiFontSize); }"));
    }
}
