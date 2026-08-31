//! Where zeddy's private herdr lives, and the environment it lives in.
//!
//! Every path here is under a single zeddy-owned root, so "which herdr" is one
//! decision made once rather than a rule each call site has to remember. The
//! environment in [`Namespace::env`] is applied to every herdr process zeddy
//! launches — the daemon and each frame stream alike — which is what keeps a
//! `HERDR_SOCKET_PATH` inherited from the user's shell from reaching herdr at
//! all.

use std::{ffi::OsString, path::PathBuf};

/// The private locations and environment of zeddy's own herdr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    root: PathBuf,
    session: String,
}

impl Namespace {
    /// The namespace zeddy uses in production: `<state>/zeddy/herdr`.
    ///
    /// State rather than config or cache, because what lives here is neither
    /// something an operator edits nor something safe to evict mid-session.
    pub fn private() -> Self {
        Self::rooted(state_home().join("zeddy").join("herdr"))
    }

    /// A namespace under an arbitrary root. Tests use this to get a whole
    /// private backend in a scratch directory; nothing else should need it.
    pub fn rooted(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), session: "zeddy".to_owned() }
    }

    /// The Unix socket the control plane connects to.
    pub fn socket(&self) -> PathBuf {
        self.root.join("herdr.sock")
    }

    /// The daemon's log, which is the only thing here a human reads.
    pub fn log(&self) -> PathBuf {
        self.root.join("daemon.log")
    }

    /// herdr's named session inside this namespace.
    pub fn session(&self) -> &str {
        &self.session
    }

    /// Create every directory herdr will expect to write into.
    pub fn prepare(&self) -> std::io::Result<()> {
        for dir in [
            &self.root,
            &self.xdg("config"),
            &self.xdg("state"),
            &self.xdg("data"),
            &self.xdg("cache"),
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// The environment every herdr process zeddy launches runs in.
    ///
    /// The XDG redirections keep herdr's config, state, data, and cache inside
    /// the private root. The `HERDR_*` entries pin which daemon it talks to.
    /// `HERDR_WORKSPACE_ID`, `HERDR_TAB_ID`, and `HERDR_PANE_ID` are cleared
    /// rather than set: they identify the pane a process was *launched from*,
    /// and zeddy is not launched from one.
    pub fn env(&self) -> Vec<(OsString, Option<OsString>)> {
        let set = |k: &str, v: OsString| (OsString::from(k), Some(v));
        let clear = |k: &str| (OsString::from(k), None);
        vec![
            set("XDG_CONFIG_HOME", self.xdg("config").into()),
            set("XDG_STATE_HOME", self.xdg("state").into()),
            set("XDG_DATA_HOME", self.xdg("data").into()),
            set("XDG_CACHE_HOME", self.xdg("cache").into()),
            set("HERDR_SOCKET_PATH", self.socket().into()),
            set("HERDR_SESSION", self.session.clone().into()),
            clear("HERDR_CLIENT_SOCKET_PATH"),
            clear("HERDR_CONFIG_PATH"),
            clear("HERDR_ENV"),
            clear("HERDR_WORKSPACE_ID"),
            clear("HERDR_TAB_ID"),
            clear("HERDR_PANE_ID"),
        ]
    }

    fn xdg(&self, which: &str) -> PathBuf {
        self.root.join("xdg").join(which)
    }
}

/// `$XDG_STATE_HOME`, or the platform default when it is unset or relative.
fn state_home() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_STATE_HOME") {
        let dir = PathBuf::from(dir);
        if dir.is_absolute() {
            return dir;
        }
    }
    home().join(".local").join("state")
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_private_path_stays_under_the_root() {
        let ns = Namespace::rooted("/scratch/root");
        for path in [ns.socket(), ns.log(), ns.xdg("config"), ns.xdg("state")] {
            assert!(path.starts_with("/scratch/root"), "{path:?} escaped the private root");
        }
    }

    #[test]
    fn inherited_herdr_context_is_cleared_not_merely_overridden() {
        let ns = Namespace::rooted("/scratch/root");
        let env = ns.env();
        for key in ["HERDR_PANE_ID", "HERDR_TAB_ID", "HERDR_WORKSPACE_ID", "HERDR_CONFIG_PATH"] {
            let entry = env.iter().find(|(k, _)| k == key).expect("key is in the namespace env");
            assert!(entry.1.is_none(), "{key} must be removed, not set");
        }
    }

    #[test]
    fn the_socket_and_the_env_agree() {
        let ns = Namespace::rooted("/scratch/root");
        let env = ns.env();
        let socket =
            env.iter().find(|(k, _)| k == "HERDR_SOCKET_PATH").and_then(|(_, v)| v.clone());
        assert_eq!(socket, Some(ns.socket().into()));
    }

    #[test]
    fn prepare_creates_what_herdr_will_write_into() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ns = Namespace::rooted(tmp.path().join("ns"));
        ns.prepare().expect("prepare");
        assert!(ns.socket().parent().expect("root").is_dir());
        assert!(ns.xdg("config").is_dir());
    }
}
