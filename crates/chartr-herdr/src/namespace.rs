//! Where chartr's private herdr lives, and the environment it lives in.
//!
//! Every path here is under a single chartr-owned root, so "which herdr" is one
//! decision made once rather than a rule each call site has to remember. The
//! environment in [`Namespace::env`] is applied to every herdr process chartr
//! launches — the daemon and each interactive attach client alike — which keeps a
//! `HERDR_SOCKET_PATH` inherited from the user's shell from reaching herdr at
//! all.

use std::{ffi::OsString, path::PathBuf};

/// The private locations and environment of chartr's own herdr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    /// Herdr's own directory (`<config home>/herdr`).
    root: PathBuf,
}

impl Namespace {
    /// The namespace chartr uses in production:
    /// `<config>/chartr/herdr`.
    ///
    /// Herdr itself resolves all private runtime paths relative to its config
    /// home, so this follows the proven chartr-rs namespace shape exactly.
    pub fn private() -> Self {
        Self::rooted(config_home().join("chartr").join("herdr"))
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
        self.migrate_session_shell()?;
        Ok(())
    }

    /// Earlier Chartr builds restored the *launching* terminal's Herdr variables
    /// in this generated wrapper, overwriting the new pane's actual identity.
    /// Keep the user's shell/config environment, but let Herdr supply routing.
    fn migrate_session_shell(&self) -> std::io::Result<()> {
        let config = self.root.join("config.toml");
        let shell = self.root.join("chartr-session-shell");
        let Ok(config) = std::fs::read_to_string(config) else { return Ok(()) };
        if !config.starts_with("# chartr's private herdr backend.") {
            return Ok(());
        }
        let Ok(script) = std::fs::read_to_string(&shell) else { return Ok(()) };
        if !script.starts_with("#!/bin/sh\nunset HERDR_SOCKET_PATH HERDR_SESSION ")
            || !script.ends_with("exec \"$user_shell\" \"$@\"\n")
        {
            return Ok(());
        }
        let migrated = script
            .lines()
            .filter(|line| !line.starts_with("unset HERDR_") && !line.starts_with("export HERDR_"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        // Keep a one-time backup. Atomic replacement preserves the executable
        // permissions and avoids a new terminal reading a half-written script.
        let backup = self.root.join("chartr-session-shell.before-conversations");
        if !backup.exists() {
            std::fs::copy(&shell, backup)?;
        }
        let temporary = self.root.join("chartr-session-shell.migrating");
        std::fs::write(&temporary, migrated)?;
        std::fs::set_permissions(&temporary, std::fs::metadata(&shell)?.permissions())?;
        std::fs::rename(temporary, shell)
    }

    /// The older managed shell restores the operator's XDG config location.
    /// Fresh backends inherit the daemon's private config home instead.
    pub(crate) fn session_config_home(&self) -> PathBuf {
        if let Ok(script) = std::fs::read_to_string(self.root.join("chartr-session-shell")) {
            if script.starts_with("#!/bin/sh\n")
                && script.contains("user_shell=${CHARTR_USER_SHELL:-/bin/sh}")
            {
                if let Some(value) = script.lines().find_map(|line| {
                    line.strip_prefix("export XDG_CONFIG_HOME='")
                        .and_then(|value| value.strip_suffix('\''))
                }) {
                    let path = PathBuf::from(value.replace("'\"'\"'", "'"));
                    if path.is_absolute() {
                        return path;
                    }
                }
            }
        }
        self.root.parent().unwrap_or(&self.root).to_owned()
    }

    /// The environment every herdr process chartr launches runs in.
    ///
    /// The XDG redirections keep herdr's config, state, data, and cache inside
    /// the private root. The `HERDR_*` entries pin which daemon it talks to.
    /// `HERDR_WORKSPACE_ID`, `HERDR_TAB_ID`, and `HERDR_PANE_ID` are cleared
    /// rather than set: they identify the pane a process was *launched from*,
    /// and chartr is not launched from one.
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
    use std::{os::unix::fs::PermissionsExt, process::Command};

    #[test]
    fn legacy_shell_keeps_the_new_panes_routing_and_the_users_shell_environment() {
        let tmp = tempfile::tempdir().unwrap();
        let ns = Namespace::rooted(tmp.path());
        std::fs::write(
            tmp.path().join("config.toml"),
            "# chartr's private herdr backend. chartr manages this file\n",
        )
        .unwrap();
        let shell = tmp.path().join("chartr-session-shell");
        let legacy = "#!/bin/sh\nunset HERDR_SOCKET_PATH HERDR_SESSION HERDR_ENV HERDR_PANE_ID\nexport XDG_CONFIG_HOME='/operator/config'\nexport HERDR_SOCKET_PATH='/old/herdr.sock'\nexport HERDR_ENV='1'\nexport HERDR_PANE_ID='old:p1'\nexport CHARTR_USER_SHELL='/bin/sh'\nuser_shell=${CHARTR_USER_SHELL:-/bin/sh}\nunset CHARTR_USER_SHELL\nexport SHELL=\"$user_shell\"\nexec \"$user_shell\" \"$@\"\n";
        std::fs::write(&shell, legacy).unwrap();
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
        ns.prepare().unwrap();
        ns.prepare().unwrap();
        assert_eq!(ns.session_config_home(), PathBuf::from("/operator/config"));
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("chartr-session-shell.before-conversations"))
                .unwrap(),
            legacy
        );
        for pane in ["new:p1", "new:p2"] {
            let result = Command::new(&shell).args(["-c", "printf '%s\\n' \"$HERDR_SOCKET_PATH\" \"$HERDR_PANE_ID\" \"$HERDR_ENV\" \"$XDG_CONFIG_HOME\" \"$SHELL\""])
                .env("HERDR_SOCKET_PATH", "/new/herdr.sock").env("HERDR_PANE_ID", pane).env("HERDR_ENV", "1")
                .output().unwrap();
            assert!(result.status.success());
            assert_eq!(
                String::from_utf8(result.stdout).unwrap(),
                format!("/new/herdr.sock\n{pane}\n1\n/operator/config\n/bin/sh\n")
            );
        }
    }

    #[test]
    fn migration_does_not_rewrite_a_custom_shell() {
        let tmp = tempfile::tempdir().unwrap();
        let ns = Namespace::rooted(tmp.path());
        let shell = tmp.path().join("chartr-session-shell");
        let custom = "#!/bin/sh\nexport HERDR_CUSTOM=1\nexec /bin/zsh\n";
        std::fs::write(&shell, custom).unwrap();
        ns.prepare().unwrap();
        assert_eq!(std::fs::read_to_string(shell).unwrap(), custom);
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
