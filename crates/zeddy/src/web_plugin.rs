//! The web-plugin pane host.
//!
//! A web contribution is an operating-system webview parented to the GPUI
//! window. The element below follows the same visibility lease used by
//! Chartr-rs's browser pane: GPUI owns layout while Wry owns the native pixels.

#[cfg(target_os = "linux")]
use std::time::Duration;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};

use futures::{StreamExt as _, channel::mpsc};
use gpui::{
    AnyView, App, AppContext as _, Bounds, Context, Element, ElementId, EntityId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, ParentElement as _, Pixels, Render, Size, Style,
    Styled as _, Window, div,
};
use serde::{Deserialize, Serialize};
use zeddy_plugin::manifest::Permissions;
use zeddy_plugin_host::FileBroker;

use crate::session::SessionAccess;

pub type FocusHandler = Rc<dyn Fn(EntityId, &mut App)>;

/// Arbitrates ownership when one native child view moves between GPUI element paths.
///
/// GPUI drops element state that was not used by the newest frame after that frame
/// has already been painted. Without a generation, the stale state's destructor
/// can therefore hide a webview that its new location just made visible.
#[derive(Clone, Default)]
pub(crate) struct NativeViewLeaseOwner(Rc<Cell<u64>>);

impl NativeViewLeaseOwner {
    pub(crate) fn acquire(&self) -> NativeViewLease {
        let mut generation = self.0.get().wrapping_add(1);
        if generation == 0 {
            generation = 1;
        }
        self.0.set(generation);
        NativeViewLease { generation, current: self.clone() }
    }
}

pub(crate) struct NativeViewLease {
    generation: u64,
    current: NativeViewLeaseOwner,
}

impl NativeViewLease {
    pub(crate) fn is_current(&self) -> bool {
        self.current.0.get() == self.generation
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
use wry::{
    Rect, WebViewBuilder,
    dpi::{LogicalPosition, LogicalSize, Position, Size as WrySize},
    http::{Response, header},
};

pub fn view(
    entry: PathBuf,
    broker: FileBroker,
    permissions: Permissions,
    session: Option<SessionAccess>,
    on_focus: Option<FocusHandler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyView {
    cx.new(|cx| WebPluginView::new(entry, broker, permissions, session, on_focus, window, cx))
        .into()
}

struct WebPluginView {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    webview: Option<Rc<wry::WebView>>,
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    visibility: NativeViewLeaseOwner,
    #[cfg(target_os = "linux")]
    _gtk_pump: gpui::Task<()>,
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    _focus_task: gpui::Task<()>,
    error: Option<String>,
}

impl WebPluginView {
    fn new(
        entry: PathBuf,
        broker: FileBroker,
        permissions: Permissions,
        session: Option<SessionAccess>,
        on_focus: Option<FocusHandler>,
        window: &Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let _ = (entry, broker, permissions, session, on_focus, window, _cx);
            return Self { error: Some("Web plugins are supported on macOS and Linux.".into()) };
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            let visibility = NativeViewLeaseOwner::default();
            let (focus_tx, mut focus_rx) = mpsc::unbounded();
            let entity_id = _cx.entity_id();
            let focus_task = _cx.spawn(async move |_, cx| {
                while focus_rx.next().await.is_some() {
                    if let Some(on_focus) = &on_focus {
                        let _ = cx.update(|cx| on_focus(entity_id, cx));
                    }
                }
            });
            #[cfg(target_os = "linux")]
            let gtk_pump = Self::pump_gtk(_cx);
            #[cfg(target_os = "linux")]
            if let Err(error) = gtk::init() {
                return Self {
                    webview: None,
                    visibility,
                    _gtk_pump: gtk_pump,
                    _focus_task: focus_task,
                    error: Some(format!("Could not initialize GTK: {error}")),
                };
            }

            let Some(root) = entry.parent().and_then(|path| path.canonicalize().ok()) else {
                return Self {
                    webview: None,
                    visibility,
                    #[cfg(target_os = "linux")]
                    _gtk_pump: gtk_pump,
                    _focus_task: focus_task,
                    error: Some(format!("Plugin entry is unavailable: {}", entry.display())),
                };
            };
            let entry_name =
                entry.file_name().and_then(|name| name.to_str()).unwrap_or("index.html");
            let root_for_protocol = root.clone();
            let webview_slot = Rc::new(RefCell::new(None::<Weak<wry::WebView>>));
            let responder = webview_slot.clone();
            let builder = WebViewBuilder::new()
                .with_custom_protocol("chartr-plugin".into(), move |_, request| {
                    asset_response(&root_for_protocol, request.uri().path())
                })
                .with_initialization_script(BRIDGE)
                .with_ipc_handler(move |request| {
                    if is_focus_request(request.body()) {
                        let _ = focus_tx.unbounded_send(());
                        return;
                    }
                    let response =
                        handle_request(&broker, &permissions, session.as_ref(), request.body());
                    if let Some(webview) = responder.borrow().as_ref().and_then(Weak::upgrade)
                        && let Ok(response) = serde_json::to_string(&response)
                    {
                        let _ =
                            webview.evaluate_script(&format!("window.__chartrReply({response})"));
                    }
                })
                .with_navigation_handler(|url| url.starts_with("chartr-plugin://plugin/"))
                .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
                .with_bounds(Rect {
                    position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                    size: WrySize::Logical(LogicalSize::new(1.0, 1.0)),
                })
                .with_visible(false)
                .with_focused(false)
                .with_url(format!("chartr-plugin://plugin/{entry_name}"));

            let webview = match builder.build_as_child(window) {
                Ok(webview) => Rc::new(webview),
                Err(error) => {
                    return Self {
                        webview: None,
                        visibility,
                        #[cfg(target_os = "linux")]
                        _gtk_pump: gtk_pump,
                        _focus_task: focus_task,
                        error: Some(format!("Could not create the plugin webview: {error}")),
                    };
                }
            };
            *webview_slot.borrow_mut() = Some(Rc::downgrade(&webview));
            Self {
                webview: Some(webview),
                visibility,
                #[cfg(target_os = "linux")]
                _gtk_pump: gtk_pump,
                _focus_task: focus_task,
                error: None,
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn pump_gtk(cx: &mut Context<Self>) -> gpui::Task<()> {
        // Wry's documented non-GTK-parent integration requires advancing GTK
        // alongside the host event loop. The task owns no WebView and ends as
        // soon as this pane entity is dropped.
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
}

impl Render for WebPluginView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().size_full();
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if let Some(webview) = self.webview.clone() {
            root = root.child(NativeWebViewElement::new(
                webview,
                "chartr-web-plugin",
                self.visibility.clone(),
            ));
        }
        if let Some(error) = &self.error {
            root = root.flex().items_center().justify_center().child(error.clone());
        }
        root
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn asset_response(root: &Path, uri_path: &str) -> Response<Cow<'static, [u8]>> {
    let relative = uri_path.trim_start_matches('/');
    let relative = relative.strip_prefix("plugin/").unwrap_or(relative);
    let result = (|| {
        let path = root.join(relative).canonicalize().map_err(|error| error.to_string())?;
        if !path.starts_with(root) || !path.is_file() {
            return Err("asset escapes the plugin directory".to_owned());
        }
        std::fs::read(&path).map(|body| (path, body)).map_err(|error| error.to_string())
    })();
    match result {
        Ok((path, body)) => Response::builder()
            .header(header::CONTENT_TYPE, content_type(&path))
            .header(
                "Content-Security-Policy",
                "default-src 'self' data: blob:; connect-src 'none'; frame-src 'none'; object-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'",
            )
            .body(Cow::Owned(body))
            .expect("valid asset response"),
        Err(error) => Response::builder()
            .status(404)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Cow::Owned(error.into_bytes()))
            .expect("valid error response"),
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()).unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

// The API exists from the first script tick. Host actions are intentionally
// denied until an instance receives an explicit broker; web content cannot
// silently fall back to direct network access because the CSP blocks it.
const BRIDGE: &str = r#"
(() => {
  let next = 1;
  const pending = new Map();
  window.__chartrReply = response => {
    const pair = pending.get(response.id);
    if (!pair) return;
    pending.delete(response.id);
    response.ok ? pair[0](response.value) : pair[1](new Error(response.error));
  };
  const invoke = (action, options = {}) => new Promise((resolve, reject) => {
    const id = next++;
    pending.set(id, [resolve, reject]);
    window.ipc.postMessage(JSON.stringify({ id, action, ...options }));
  });
  window.addEventListener("pointerdown", () => {
    window.ipc.postMessage(JSON.stringify({ id: 0, action: "chartr.focus" }));
  }, true);
  Object.defineProperty(window, "chartr", { value: Object.freeze({ invoke }) });
})();
"#;

fn is_focus_request(encoded: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(encoded)
        .ok()
        .and_then(|request| request.get("action")?.as_str().map(str::to_owned))
        .is_some_and(|action| action == "chartr.focus")
}

#[derive(Deserialize)]
struct HostRequest {
    id: u64,
    action: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    data: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Serialize)]
struct HostResponse {
    id: u64,
    ok: bool,
    value: serde_json::Value,
    error: String,
}

fn handle_request(
    broker: &FileBroker,
    permissions: &Permissions,
    session: Option<&SessionAccess>,
    encoded: &str,
) -> HostResponse {
    let request: HostRequest = match serde_json::from_str(encoded) {
        Ok(request) => request,
        Err(error) => {
            return HostResponse {
                id: 0,
                ok: false,
                value: serde_json::Value::Null,
                error: format!("Invalid host request: {error}"),
            };
        }
    };
    let result = match request.action.as_str() {
        "project.read" => broker
            .project_path(Path::new(&request.path), false)
            .and_then(|path| {
                std::fs::read_to_string(path).map_err(zeddy_plugin_host::BrokerError::Io)
            })
            .map(serde_json::Value::String)
            .map_err(|error| error.to_string()),
        "project.write" => broker
            .project_path(Path::new(&request.path), true)
            .and_then(|path| {
                std::fs::write(path, request.data.as_bytes())
                    .map_err(zeddy_plugin_host::BrokerError::Io)
            })
            .map(|_| serde_json::Value::Bool(true))
            .map_err(|error| error.to_string()),
        "data.read" => broker
            .data_path(Path::new(&request.path), false)
            .and_then(|path| {
                std::fs::read_to_string(path).map_err(zeddy_plugin_host::BrokerError::Io)
            })
            .map(serde_json::Value::String)
            .map_err(|error| error.to_string()),
        "data.write" => broker
            .data_path(Path::new(&request.path), true)
            .and_then(|path| {
                std::fs::write(path, request.data.as_bytes())
                    .map_err(zeddy_plugin_host::BrokerError::Io)
            })
            .map(|_| serde_json::Value::Bool(true))
            .map_err(|error| error.to_string()),
        "network.fetch" => fetch(&request.url, &permissions.network),
        "process.run" if permissions.process => std::process::Command::new(&request.command)
            .args(&request.args)
            .output()
            .map(|output| {
                serde_json::json!({
                    "status": output.status.code(),
                    "stdout": String::from_utf8_lossy(&output.stdout),
                    "stderr": String::from_utf8_lossy(&output.stderr),
                })
            })
            .map_err(|error| error.to_string()),
        "process.run" => Err("the plugin did not declare process access".to_owned()),
        "session.metadata" if permissions.session => session
            .map(|session| {
                serde_json::json!({
                    "id": session.info.id.0,
                    "workspace": session.info.workspace.0,
                    "title": session.info.title(),
                    "agent": session.info.agent,
                    "cwd": session.info.cwd,
                })
            })
            .ok_or_else(|| "this plugin instance is not bound to a session".to_owned()),
        "session.send" if permissions.session => session
            .ok_or_else(|| "this plugin instance is not bound to a session".to_owned())
            .and_then(|session| {
                session
                    .send(request.data.as_bytes())
                    .map(|_| serde_json::Value::Bool(true))
                    .map_err(|error| error.to_string())
            }),
        "session.metadata" | "session.send" => {
            Err("the plugin did not declare session access".to_owned())
        }
        _ => Err(format!("unknown host action `{}`", request.action)),
    };
    match result {
        Ok(value) => HostResponse { id: request.id, ok: true, value, error: String::new() },
        Err(error) => {
            HostResponse { id: request.id, ok: false, value: serde_json::Value::Null, error }
        }
    }
}

fn fetch(requested: &str, allowed: &[String]) -> Result<serde_json::Value, String> {
    let url = url::Url::parse(requested).map_err(|error| error.to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only HTTP and HTTPS network actions are allowed".to_owned());
    }
    let host = url.host_str().ok_or_else(|| "the URL has no host".to_owned())?;
    if !allowed.iter().any(|entry| {
        let allowed_host = url::Url::parse(entry)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
            .unwrap_or_else(|| entry.trim_start_matches("*.").to_owned());
        host == allowed_host
            || (entry.starts_with("*.") && host.ends_with(&format!(".{allowed_host}")))
    }) {
        return Err(format!("network access to `{host}` is not declared"));
    }
    let mut response = ureq::get(requested).call().map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    let body = response.body_mut().read_to_string().map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "status": status, "body": body }))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
struct NativeWebViewElement {
    webview: Rc<wry::WebView>,
    id: ElementId,
    visibility: NativeViewLeaseOwner,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl NativeWebViewElement {
    fn new(
        webview: Rc<wry::WebView>,
        id: impl Into<ElementId>,
        visibility: NativeViewLeaseOwner,
    ) -> Self {
        Self { webview, id: id.into(), visibility }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl IntoElement for NativeWebViewElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
struct VisibleWebView {
    webview: Weak<wry::WebView>,
    lease: NativeViewLease,
    frame: Option<NativeFrame>,
    visible: bool,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
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

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Copy, PartialEq, Eq)]
struct NativeFrame {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
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

#[cfg(any(target_os = "macos", target_os = "linux"))]
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
        let id = id.expect("native webview elements always have an id");
        let frame = NativeFrame::snapped(bounds);
        window.with_element_state(id, |lease: Option<VisibleWebView>, _| {
            let is_new = lease.is_none();
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
            if is_new && visible {
                let _ = self.webview.focus_parent();
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
    use zeddy_plugin::manifest::ProjectAccess;

    fn request(id: u64, action: &str, fields: serde_json::Value) -> String {
        let mut value = serde_json::json!({ "id": id, "action": action });
        value.as_object_mut().unwrap().extend(fields.as_object().unwrap().clone());
        value.to_string()
    }

    #[test]
    fn a_repaned_native_view_supersedes_its_stale_visibility_lease() {
        let owner = NativeViewLeaseOwner::default();
        let old_location = owner.acquire();
        assert!(old_location.is_current());

        let new_location = owner.acquire();
        assert!(new_location.is_current());
        assert!(!old_location.is_current());
    }

    #[test]
    fn internal_focus_messages_do_not_enter_the_plugin_host_action_api() {
        assert!(is_focus_request(r#"{"id":0,"action":"chartr.focus"}"#));
        assert!(!is_focus_request(r#"{"id":1,"action":"project.read"}"#));
        assert!(!is_focus_request("not json"));
    }

    #[test]
    fn host_filesystem_actions_use_the_instance_broker() {
        let scratch = tempfile::tempdir().unwrap();
        let project = scratch.path().join("project");
        let data = scratch.path().join("data");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        let broker = FileBroker::new(Some(project.clone()), data, ProjectAccess::ReadWrite, false);
        let permissions =
            Permissions { project_files: ProjectAccess::ReadWrite, ..Permissions::default() };
        let write = handle_request(
            &broker,
            &permissions,
            None,
            &request(1, "project.write", serde_json::json!({ "path": "note.txt", "data": "safe" })),
        );
        assert!(write.ok, "{}", write.error);
        assert_eq!(std::fs::read_to_string(project.join("note.txt")).unwrap(), "safe");
        let escape = handle_request(
            &broker,
            &permissions,
            None,
            &request(2, "project.read", serde_json::json!({ "path": "../outside" })),
        );
        assert!(!escape.ok);
    }

    #[test]
    fn process_actions_are_manifest_gated() {
        let scratch = tempfile::tempdir().unwrap();
        let data = scratch.path().join("data");
        std::fs::create_dir_all(&data).unwrap();
        let broker = FileBroker::new(None, data, ProjectAccess::None, false);
        let encoded = request(
            1,
            "process.run",
            serde_json::json!({ "command": "printf", "args": ["hello"] }),
        );
        assert!(!handle_request(&broker, &Permissions::default(), None, &encoded).ok);
        let allowed = Permissions { process: true, ..Permissions::default() };
        let response = handle_request(&broker, &allowed, None, &encoded);
        assert!(response.ok, "{}", response.error);
        assert_eq!(response.value["stdout"], "hello");
    }
}
