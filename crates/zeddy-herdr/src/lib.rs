//! zeddy's client for herdr, the backend that owns every PTY.
//!
//! herdr is infrastructure zeddy hides rather than a feature zeddy exposes.
//! Nothing above this crate knows the name, and the only thing this crate
//! promises upward is: a list of live panes, a stream of bytes per pane, and a
//! way to push bytes and geometry back down.
//!
//! # Two surfaces, two transports
//!
//! - [`control`] — the socket API. NDJSON over a Unix socket, one request per
//!   connection. Creating, listing, and closing panes happens here.
//! - [`stream`] — the terminal byte stream. herdr exposes this as a CLI stream,
//!   not a socket method, so attaching means spawning a child process. That is
//!   a real coupling to herdr's command line and it is why
//!   [`SUPPORTED_HERDR_VERSION`] is pinned rather than probed.
//!
//! # Whose herdr
//!
//! zeddy runs a **private** daemon: its own socket, its own XDG directories,
//! its own session name. A [`Namespace`] is those locations plus the
//! environment every herdr process zeddy launches is placed in, and both a
//! [`control::Client`] and a [`stream::Attachment`] carry one. An inherited
//! `HERDR_SOCKET_PATH` from the user's own shell therefore cannot split zeddy
//! across two backends.
//!
//! The user's own herdr is never discovered, attached to, stopped, upgraded, or
//! written.

#![forbid(unsafe_code)]
#![cfg(unix)]

pub mod control;
pub mod namespace;
pub mod protocol;
pub mod sidecar;
pub mod stream;

pub use namespace::Namespace;
pub use sidecar::Sidecar;

use std::fmt;

/// The exact herdr release this client speaks to.
///
/// Not a floor and not a range. The frame stream rides herdr's command line,
/// which carries no compatibility promise, so the client and the vendored
/// executable move together or not at all.
pub const SUPPORTED_HERDR_VERSION: &str = "0.8.0";

/// The socket API protocol version [`SUPPORTED_HERDR_VERSION`] speaks.
pub const SUPPORTED_PROTOCOL: u32 = 19;

/// One terminal — a herdr *pane*.
///
/// herdr's own "session" is a whole named server instance holding many of
/// these. This crate speaks herdr's vocabulary at the wire and zeddy's above
/// it, and this newtype is where the two meet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub String);

/// A group of panes — a herdr *workspace*, one per project directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub String);

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The grid a pane's PTY is running at.
///
/// zeddy owns this, not herdr: the window decides how many cells fit and tells
/// the backend, never the other way round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Geometry {
    pub cols: u16,
    pub rows: u16,
}

impl Geometry {
    /// A geometry clamped to what a PTY will accept. A zero-sized grid is a
    /// real thing to compute during a resize and not a real thing to send.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols: cols.max(1), rows: rows.max(1) }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

/// Everything that can go wrong between zeddy and its backend.
#[derive(Debug)]
pub enum Error {
    /// The vendored executable is missing or is the wrong build.
    Sidecar(String),
    /// The socket refused, closed, or was never there.
    Transport(std::io::Error),
    /// herdr answered, and the answer was a failure.
    Backend { method: &'static str, message: String },
    /// herdr answered with something this client cannot read.
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sidecar(why) => write!(f, "herdr sidecar unusable: {why}"),
            Self::Transport(err) => write!(f, "herdr transport failed: {err}"),
            Self::Backend { method, message } => write!(f, "herdr rejected {method}: {message}"),
            Self::Protocol(why) => write!(f, "herdr sent something unreadable: {why}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Transport(err)
    }
}
