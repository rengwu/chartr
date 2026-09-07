//! chartr's client for herdr, the backend that owns every PTY.
//!
//! herdr is infrastructure chartr hides rather than a feature chartr exposes.
//! Nothing above this crate knows the name, and the only thing this crate
//! promises upward is: lifecycle and metadata over the socket API, plus a
//! namespace-safe process specification for Herdr's native interactive
//! `terminal attach` client.
//!
//! # Whose herdr
//!
//! chartr runs a **private** daemon: its own socket, its own XDG directories,
//! its own session name. A [`Namespace`] is those locations plus the
//! environment every herdr process chartr launches is placed in, and the
//! [`control::Client`] carries one. An inherited `HERDR_SOCKET_PATH` from the
//! user's own shell therefore cannot split chartr across two backends.
//!
//! The user's own herdr is never discovered, attached to, stopped, upgraded, or
//! written.

#![forbid(unsafe_code)]
#![cfg(unix)]

pub mod control;
pub mod namespace;
pub mod protocol;
pub mod sidecar;

pub use namespace::Namespace;
pub use sidecar::Sidecar;

use std::fmt;

/// The exact Herdr build this client speaks to.
///
/// Not a floor and not a range. Direct attachment rides Herdr's command line,
/// which carries no compatibility promise, so the client and the vendored
/// executable move together or not at all.
pub const SUPPORTED_HERDR_VERSION: &str = "0.8.2-chartr.5a2dee700eee";

/// The upstream package version used to produce [`SUPPORTED_HERDR_VERSION`].
pub const SUPPORTED_HERDR_UPSTREAM_VERSION: &str = "0.8.2";

/// The immutable upstream source revision used to build the sidecar.
///
/// This post-0.8.2 revision carries Herdr's semantic direct-attach mouse
/// forwarding. Tagged 0.8.2 consumes non-wheel reports before they reach the
/// child terminal, which makes mouse-aware TUIs such as Claude Code unclickable.
pub const SUPPORTED_HERDR_REVISION: &str = "5a2dee700eeeea68267a4d16777307632f77172f";

/// The socket API protocol version [`SUPPORTED_HERDR_VERSION`] speaks.
pub const SUPPORTED_PROTOCOL: u32 = 22;

/// One terminal — a herdr *pane*.
///
/// herdr's own "session" is a whole named server instance holding many of
/// these. This crate speaks herdr's vocabulary at the wire and chartr's above
/// it, and this newtype is where the two meet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub String);

/// The persistent PTY Herdr exposes through `terminal attach`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TerminalId(pub String);

/// A group of panes — a herdr *workspace*, one per project directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub String);

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for TerminalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Everything that can go wrong between chartr and its backend.
#[derive(Debug)]
pub enum Error {
    /// The vendored executable is missing or is the wrong build.
    Sidecar(String),
    /// The socket refused, closed, or was never there.
    Transport(std::io::Error),
    /// A private daemon answered but is not the exact build this client ships.
    IncompatibleDaemon { version: String, protocol: u32 },
    /// herdr answered, and the answer was a failure.
    Backend { method: &'static str, code: String, message: String },
    /// herdr answered with something this client cannot read.
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sidecar(why) => write!(f, "herdr sidecar unusable: {why}"),
            Self::Transport(err) => write!(f, "herdr transport failed: {err}"),
            Self::IncompatibleDaemon { version, protocol } => write!(
                f,
                "daemon is herdr {version} (protocol {protocol}); chartr ships \
                 {SUPPORTED_HERDR_VERSION} (protocol {SUPPORTED_PROTOCOL})"
            ),
            Self::Backend { method, code, message } if !code.is_empty() => {
                write!(f, "herdr rejected {method}: {message} ({code})")
            }
            Self::Backend { method, message, .. } => {
                write!(f, "herdr rejected {method}: {message}")
            }
            Self::Protocol(why) => write!(f, "herdr sent something unreadable: {why}"),
        }
    }
}

impl Error {
    /// Whether Herdr rejected an API request with this machine-readable code.
    ///
    /// Keep callers off the human-readable message: Herdr may improve that
    /// text without changing the condition clients need to handle.
    pub fn is_backend_code(&self, expected: &str) -> bool {
        matches!(self, Self::Backend { code, .. } if code == expected)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Transport(err)
    }
}
