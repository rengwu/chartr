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
    /// Herdr's own directory (`<config home>/herdr`).
    root: PathBuf,
}

impl Namespace {
    /// The namespace Chartr-zeddy uses in production:
    /// `<config>/chartr-zeddy/herdr`.
    ///
    /// Herdr itself resolves all private runtime paths relative to its config
    /// home, so this follows the proven Chartr-rs namespace shape exactly.
    pub fn private() -> Self {
        Self::rooted(config_home().join("chartr-zeddy").join("herdr"))
    }

    /// A namespace under an arbitrary root. Tests use this to get a whole
    /// private backend in a scratch directory; nothing else should need it.
    pub fn rooted(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The Unix socket the control plane connects to.
    pub fn socket(&self) -> PathBuf {
        self.root.join("herdr.sock")
    }

    /// The daemon's log, which is the only thing here a human reads.
    pub fn log(&self) -> PathBuf {
        self.root.join("daemon.log")
    }

    /// Herdr's persisted workspace/tab/pane shape.
    ///
    /// A replacement after a crash must start without this file. The PTYs that
    /// were represented by the saved shape died with the daemon; letting herdr
    /// recreate it would present fresh shells as if they were the old work.
    pub fn saved_shape(&self) -> PathBuf {
        self.root.join("session.json")
    }

    /// Create every directory herdr will expect to write into.
    pub fn prepare(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
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
            set("XDG_CONFIG_HOME", self.root.parent().unwrap_or(&self.root).as_os_str().to_owned()),
            set("HERDR_SOCKET_PATH", self.socket().into()),
            clear("HERDR_SESSION"),
            clear("HERDR_CLIENT_SOCKET_PATH"),
            clear("HERDR_CONFIG_PATH"),
            clear("HERDR_ENV"),
            clear("HERDR_WORKSPACE_ID"),
            clear("HERDR_TAB_ID"),
            clear("HERDR_PANE_ID"),
        ]
    }
}

/// `$XDG_CONFIG_HOME`, or the platform default when it is unset or relative.
fn config_home() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        let dir = PathBuf::from(dir);
        if dir.is_absolute() {
            return dir;
        }
    }
    home().join(".config")
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
        for path in [ns.socket(), ns.log(), ns.saved_shape()] {
            assert!(path.starts_with("/scratch/root"), "{path:?} escaped the private root");
        }
    }

    #[test]
    fn inherited_herdr_context_is_cleared_not_merely_overridden() {
        let ns = Namespace::rooted("/scratch/root");
        let env = ns.env();
        for key in [
            "HERDR_SESSION",
            "HERDR_PANE_ID",
            "HERDR_TAB_ID",
            "HERDR_WORKSPACE_ID",
            "HERDR_CONFIG_PATH",
        ] {
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
        let config = ns
            .env()
            .into_iter()
            .find(|(key, _)| key == "XDG_CONFIG_HOME")
            .and_then(|(_, value)| value)
            .expect("config home");
        assert_eq!(PathBuf::from(config), ns.root.parent().expect("config home"));
    }
}
