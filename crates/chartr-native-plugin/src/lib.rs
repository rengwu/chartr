//! Native surface ABI v1. No Rust-owned objects cross this boundary.
//!
//! All API calls occur on the host UI thread. `emit` may run on any thread;
//! it copies the bytes before returning and never calls the plugin reentrantly.
//! Input strings/bytes are borrowed for the duration of the call only.
//! `create` returns an opaque plugin-owned handle; only `destroy` frees it.
//! After `destroy` returns, the plugin MUST stop emitting for that handle and
//! finish using its parent. The host keeps libraries loaded until process exit.
//! `shutdown` runs once, after all handles are destroyed, on the UI thread.
//! No panics or exceptions may unwind across any function in this interface.

use serde::{Deserialize, Serialize};
use std::ffi::{c_char, c_void};

pub const ABI_VERSION: u32 = 1;
pub const ENTRY_POINT: &[u8] = b"chartr_native_plugin_v1\0";
pub const PARENT_X11: u32 = 1;
pub const PARENT_NS_VIEW: u32 = 2;
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bytes {
    pub data: *const u8,
    pub len: usize,
}
impl Bytes {
    pub fn new(bytes: &[u8]) -> Self {
        Self { data: bytes.as_ptr(), len: bytes.len() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Host {
    pub context: *mut c_void,
    pub emit: unsafe extern "C" fn(*mut c_void, Bytes),
}

#[repr(C)]
pub struct Create {
    pub abi: u32,
    pub size: u32,
    pub parent_kind: u32,
    /// X11 XID or an NSView pointer. Borrowed until destroy returns.
    pub parent: usize,
    pub package_dir: *const c_char,
    pub data_dir: *const c_char,
    /// UTF-8 JSON Instance.
    pub instance: Bytes,
    pub host: Host,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Api {
    pub abi: u32,
    pub size: u32,
    /// Emit Error and return null on failure.
    pub create: unsafe extern "C" fn(*const Create) -> *mut c_void,
    pub destroy: unsafe extern "C" fn(*mut c_void),
    /// Logical top-left coordinates within the parent; scale is physical/logical.
    pub resize: unsafe extern "C" fn(*mut c_void, f64, f64, f64, f64, f64),
    pub visible: unsafe extern "C" fn(*mut c_void, bool),
    /// true focuses the surface; false returns native focus to the host controls.
    pub focus: unsafe extern "C" fn(*mut c_void, bool),
    /// UTF-8 JSON Input; call Poll only in response to Wake (no idle polling).
    pub dispatch: unsafe extern "C" fn(*mut c_void, Bytes),
    pub shutdown: unsafe extern "C" fn(),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub space: String,
    pub instance_id: u64,
    pub theme: Theme,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub page: String,
    pub field: String,
    pub border: String,
    pub focus: String,
    pub text: String,
    pub muted: String,
    pub ui_font_size: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Input {
    Poll,
    Action { id: String, value: Option<String> },
    Theme { theme: Theme },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Wake,
    State {
        title: String,
        controls: Vec<Control>,
        shortcuts: Vec<Shortcut>,
    },
    /// None means native content already gained focus; update host logical focus.
    Focus {
        control: Option<String>,
        select_all: bool,
    },
    Error {
        message: String,
    },
}

/// A single optional toolbar row. Buttons and text inputs reuse host widgets.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Control {
    pub id: String,
    pub label: String,
    /// Package-relative SVG, never a host icon identifier.
    pub icon: Option<String>,
    pub enabled: bool,
    /// Some means a text input; None means a button. Enter submits its value.
    pub value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shortcut {
    /// GPUI keystroke spelling, e.g. ctrl-l or cmd-l.
    pub key: String,
    pub action: String,
    pub content_only: bool,
}
