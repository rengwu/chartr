//! `zeddy-plugin.toml` — the one file both plugin tiers have in common.
//!
//! A plugin is a directory with this file in it. What the directory *contains*
//! beyond the manifest is what makes it native or web, and the manifest's
//! [`Kind`] is what says which.

use std::path::Path;

use serde::Deserialize;

/// The manifest version this build reads. Bumped when a field changes meaning.
pub const MANIFEST_VERSION: u32 = 2;

/// The native ABI this build links.
///
/// A native plugin passes Rust and GPUI objects across a dynamic-library
/// boundary, so its `native_abi` must match zeddy's *exactly*. There is no
/// compatibility range and there is not going to be one: a mismatch is a
/// vtable from a different compilation, and the failure mode is a crash rather
/// than a wrong answer.
pub const NATIVE_ABI: u32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Multiplicity {
    /// Reopening focuses the existing item in that owning space.
    #[default]
    PerSpace,
    /// Each open request creates an independent instance.
    Multiple,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAccess {
    #[default]
    None,
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub project_files: ProjectAccess,
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub process: bool,
    #[serde(default)]
    pub session: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub multiplicity: Multiplicity,
    #[serde(default)]
    pub cloneable: bool,
    #[serde(default)]
    pub restorable: bool,
    #[serde(default)]
    pub session_binding: bool,
}

/// Which tier a plugin belongs to.
///
/// The two tiers exist because "anyone can author one" and "fast enough to
/// paint a star-map at 120fps" are different requirements, and one runtime
/// cannot honestly be both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A `cdylib` mounted directly in zeddy's element tree. Its view is an
    /// ordinary GPUI view: same frame path, same input, same scrolling as a
    /// built-in. Installing one is installing native code, and the trust model
    /// says so out loud.
    Native,
    /// HTML and JavaScript in an OS webview. Sandboxed, hot-reloadable,
    /// authorable by anyone who has written a web page — and a frame behind
    /// native, because it is composited rather than painted.
    Web,
}

/// A parsed `zeddy-plugin.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    pub manifest_version: u32,
    /// Reverse-DNS, and the identity everything else keys off: the install
    /// directory, the data directory, the pane ids. Two plugins with the same
    /// id are the same plugin at different versions.
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: Kind,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub permissions: Permissions,
    /// Native only: the Cargo library stem. zeddy appends the platform's
    /// extension, so one manifest covers `.dylib`, `.so`, and `.dll`.
    #[serde(default)]
    pub library: Option<String>,
    /// Native only, and required there.
    #[serde(default)]
    pub native_abi: Option<u32>,
    /// Web only: the entry document, relative to the plugin directory.
    #[serde(default)]
    pub entry: Option<String>,
    /// Web only: an optional document constructed lazily inside Settings.
    #[serde(default)]
    pub settings_entry: Option<String>,
}

/// Why a manifest was refused.
#[derive(Debug, Clone, PartialEq)]
pub enum Invalid {
    Unreadable(String),
    Malformed(String),
    /// The manifest is from a different generation of the format.
    ManifestVersion {
        found: u32,
    },
    /// A native plugin compiled against a different zeddy.
    NativeAbi {
        found: Option<u32>,
    },
    /// A field the manifest's own `kind` requires is missing.
    Missing {
        field: &'static str,
        kind: Kind,
    },
    BadId(String),
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable(why) => write!(f, "cannot read zeddy-plugin.toml: {why}"),
            Self::Malformed(why) => write!(f, "zeddy-plugin.toml is not valid: {why}"),
            Self::ManifestVersion { found } => {
                write!(f, "manifest_version is {found}; this zeddy reads {MANIFEST_VERSION}")
            }
            Self::NativeAbi { found } => match found {
                Some(found) => write!(f, "native_abi is {found}; this zeddy links {NATIVE_ABI}"),
                None => write!(f, "a native plugin must declare native_abi = {NATIVE_ABI}"),
            },
            Self::Missing { field, kind } => {
                write!(f, "a {kind:?} plugin must declare `{field}`")
            }
            Self::BadId(id) => write!(f, "`{id}` is not a usable plugin id"),
        }
    }
}

impl std::error::Error for Invalid {}

impl Manifest {
    /// Parse and validate an embedded or otherwise in-memory manifest.
    pub fn parse(text: &str) -> Result<Self, Invalid> {
        let manifest: Self =
            toml::from_str(text).map_err(|err| Invalid::Malformed(err.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Read and validate the manifest in a plugin directory.
    ///
    /// Validation is total: a manifest that comes back `Ok` has everything its
    /// own tier needs, so nothing downstream re-checks a field.
    pub fn read(dir: &Path) -> Result<Self, Invalid> {
        let path = dir.join("zeddy-plugin.toml");
        let text = std::fs::read_to_string(&path)
            .map_err(|err| Invalid::Unreadable(format!("{}: {err}", path.display())))?;
        Self::parse(&text)
    }

    fn validate(&self) -> Result<(), Invalid> {
        if self.manifest_version != MANIFEST_VERSION {
            return Err(Invalid::ManifestVersion { found: self.manifest_version });
        }
        if !is_usable_id(&self.id) {
            return Err(Invalid::BadId(self.id.clone()));
        }
        match self.kind {
            Kind::Native => {
                if self.native_abi != Some(NATIVE_ABI) {
                    return Err(Invalid::NativeAbi { found: self.native_abi });
                }
                if self.library.is_none() {
                    return Err(Invalid::Missing { field: "library", kind: self.kind });
                }
            }
            Kind::Web => {
                if self.entry.is_none() {
                    return Err(Invalid::Missing { field: "entry", kind: self.kind });
                }
            }
        }
        Ok(())
    }

    /// The library filename this platform expects, for a native plugin.
    pub fn library_filename(&self) -> Option<String> {
        let stem = self.library.as_deref()?;
        Some(if cfg!(target_os = "windows") {
            format!("{stem}.dll")
        } else if cfg!(target_os = "macos") {
            format!("lib{stem}.dylib")
        } else {
            format!("lib{stem}.so")
        })
    }
}

/// An id has to be safe to use as a directory name, because it is used as one.
fn is_usable_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        && !id.starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml: &str) -> Result<Manifest, Invalid> {
        Manifest::parse(toml)
    }

    const NATIVE: &str = r#"
        manifest_version = 2
        id = "com.example.starmap"
        name = "Star map"
        version = "0.1.0"
        kind = "native"
        library = "starmap"
        native_abi = 2
    "#;

    const WEB: &str = r#"
        manifest_version = 2
        id = "com.example.notes"
        name = "Notes"
        version = "0.1.0"
        kind = "web"
        entry = "index.html"
    "#;

    #[test]
    fn both_tiers_parse() {
        assert_eq!(parse(NATIVE).expect("native").kind, Kind::Native);
        assert_eq!(parse(WEB).expect("web").kind, Kind::Web);
    }

    #[test]
    fn capabilities_and_permissions_are_explicit_and_default_safe() {
        let defaults = parse(WEB).expect("web defaults");
        assert_eq!(defaults.capabilities.multiplicity, Multiplicity::PerSpace);
        assert!(!defaults.capabilities.cloneable);
        assert_eq!(defaults.permissions.project_files, ProjectAccess::None);
        assert!(defaults.permissions.network.is_empty());

        let declared = parse(&format!(
            "{WEB}\n[capabilities]\nmultiplicity = 'multiple'\ncloneable = true\nrestorable = true\nsession_binding = true\n\
             [permissions]\nproject_files = 'read_write'\nnetwork = ['https://api.example.com']\nprocess = true\nsession = true\n"
        ))
        .expect("declared contract");
        assert_eq!(declared.capabilities.multiplicity, Multiplicity::Multiple);
        assert!(declared.capabilities.cloneable && declared.capabilities.restorable);
        assert_eq!(declared.permissions.project_files, ProjectAccess::ReadWrite);
        assert!(declared.permissions.process && declared.permissions.session);
    }

    #[test]
    fn a_native_plugin_from_another_abi_is_refused() {
        let wrong = NATIVE.replace("native_abi = 2", "native_abi = 99");
        assert_eq!(parse(&wrong), Err(Invalid::NativeAbi { found: Some(99) }));
    }

    #[test]
    fn a_native_plugin_without_an_abi_is_refused_rather_than_assumed() {
        let missing = NATIVE.replace("native_abi = 2", "");
        assert_eq!(parse(&missing), Err(Invalid::NativeAbi { found: None }));
    }

    #[test]
    fn each_tier_requires_only_its_own_fields() {
        // A web plugin needs no ABI, and a native plugin needs no entry point.
        assert!(parse(WEB).is_ok());
        assert!(parse(NATIVE).is_ok());
        let no_entry = WEB.replace("entry = \"index.html\"", "");
        assert_eq!(parse(&no_entry), Err(Invalid::Missing { field: "entry", kind: Kind::Web }));
    }

    #[test]
    fn an_id_that_could_escape_its_directory_is_refused() {
        for bad in ["", ".", "../etc", "a/b", ".hidden"] {
            let toml = NATIVE.replace("com.example.starmap", bad);
            assert!(matches!(parse(&toml), Err(Invalid::BadId(_))), "accepted {bad:?}");
        }
    }

    #[test]
    fn the_library_filename_follows_the_platform() {
        let name = parse(NATIVE).expect("native").library_filename().expect("a name");
        assert!(name.contains("starmap"));
        assert!(name.ends_with(std::env::consts::DLL_SUFFIX));
    }

    #[test]
    fn a_web_plugin_has_no_library_filename() {
        assert_eq!(parse(WEB).expect("web").library_filename(), None);
    }
}
