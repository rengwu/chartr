//! The herdr executable zeddy ships, resolved by path and never through `PATH`.
//!
//! Both halves of the integration — control requests and interactive attach —
//! execute or handshake Herdr, and they must be the same build. Resolving once and
//! carrying the result makes that true by construction instead of by
//! convention.

use std::path::{Path, PathBuf};

use crate::{Error, Result, SUPPORTED_HERDR_VERSION};

/// A resolved herdr executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidecar {
    path: PathBuf,
}

impl Sidecar {
    /// The herdr beside zeddy's own executable, as the build script placed it.
    ///
    /// `PATH` is deliberately not consulted. A herdr the user installed for
    /// themselves is theirs; picking it up would make zeddy's backend version
    /// depend on the machine.
    pub fn beside_current_exe() -> Result<Self> {
        let exe = std::env::current_exe().map_err(|err| {
            Error::Sidecar(format!("cannot locate zeddy's own executable: {err}"))
        })?;
        let dir = exe
            .parent()
            .ok_or_else(|| Error::Sidecar(format!("{} has no directory", exe.display())))?;
        Self::at(dir.join("herdr"))
    }

    /// A sidecar at an exact path, checked for existence only.
    ///
    /// The version is *not* probed here. Probing costs a process launch on a
    /// path the window is waiting on, and a wrong version surfaces at the
    /// handshake anyway, with a better message.
    pub fn at(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if !path.is_file() {
            return Err(Error::Sidecar(format!(
                "no herdr {SUPPORTED_HERDR_VERSION} at {}; run `sh vendor/herdr/fetch.sh`",
                path.display()
            )));
        }
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_sidecar_says_how_to_get_one() {
        let err = Sidecar::at("/nowhere/herdr").expect_err("must not resolve");
        let message = err.to_string();
        assert!(message.contains("fetch.sh"), "{message}");
        assert!(message.contains(SUPPORTED_HERDR_VERSION), "{message}");
    }

    #[test]
    fn a_directory_is_not_an_executable() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(Sidecar::at(tmp.path()).is_err());
    }
}
