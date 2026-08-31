//! The only code in zeddy that loads foreign code.
//!
//! Both plugin tiers are discovered the same way — a directory with a
//! `zeddy-plugin.toml` in it — and both arrive at the app as the same thing: a
//! list of panes with a way to build each one. Everything above this crate sees
//! [`Loaded`] and never asks which tier a pane came from.
//!
//! # Discovery, not installation
//!
//! This crate reads what is already on disk. Fetching a plugin from Git,
//! building one from source, and deciding whether the user trusts it are
//! separate concerns and separate code; putting them here would mean the window
//! could not enumerate plugins without also being able to install them.
//!
//! # Native libraries are never unloaded
//!
//! A native plugin's GPUI views hold vtables that live in its library. Dropping
//! the library while a view is alive is a use-after-free, and there is no
//! reliable moment at which zeddy knows the last one is gone. So [`Native`]
//! leaks its [`libloading::Library`] deliberately: a reload brings a *new*
//! generation up and swaps it in, and the old code stays mapped until the
//! process exits. Memory is the cost, and it is the cheap side of that trade.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use zeddy_plugin::{
    Entry, Host, PaneKey, PaneSpec, PluginObject, Registrar,
    manifest::{Invalid, Kind, Manifest},
};

/// One plugin, loaded and activated.
pub struct Loaded {
    pub manifest: Manifest,
    pub dir: PathBuf,
    pub panes: Vec<PaneSpec>,
    tier: Tier,
}

/// The tier-specific half of a loaded plugin — the only place the difference
/// between "native" and "web" is still visible.
enum Tier {
    Native(Native),
    Web { entry: PathBuf },
}

/// A loaded native library and the object it produced.
struct Native {
    plugin: Box<dyn PluginObject>,
    /// Kept for the life of the process. See the module comment.
    _library: &'static libloading::Library,
}

/// How a pane should be built, once something above decides to show it.
pub enum PaneSource<'a> {
    /// Call into the plugin for a GPUI view, mounted directly in the tree.
    Native(&'a mut dyn PluginObject),
    /// Point a webview at this document.
    Web(&'a Path),
}

impl Loaded {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn kind(&self) -> Kind {
        self.manifest.kind
    }

    /// How to build one of this plugin's panes.
    ///
    /// `None` for a pane this plugin did not declare — which is what a stale
    /// saved layout looks like after a plugin drops a pane.
    pub fn pane(&mut self, key: &PaneKey) -> Option<PaneSource<'_>> {
        if !self.panes.iter().any(|pane| &pane.key == key) {
            return None;
        }
        Some(match &mut self.tier {
            Tier::Native(native) => PaneSource::Native(native.plugin.as_mut()),
            Tier::Web { entry } => PaneSource::Web(entry.as_path()),
        })
    }
}

/// A plugin directory that could not be loaded, kept so Settings can say why
/// rather than silently showing one fewer plugin.
#[derive(Debug, Clone)]
pub struct Rejected {
    pub dir: PathBuf,
    pub why: String,
}

/// Everything found in one scan.
#[derive(Default)]
pub struct Catalog {
    /// Loaded plugins, by id. A `BTreeMap` so the sidebar's order is the same
    /// on every launch rather than the order the filesystem happened to answer.
    pub loaded: BTreeMap<String, Loaded>,
    pub rejected: Vec<Rejected>,
}

impl Catalog {
    /// Every pane every loaded plugin contributes, in a stable order.
    pub fn panes(&self) -> Vec<&PaneSpec> {
        self.loaded.values().flat_map(|plugin| plugin.panes.iter()).collect()
    }

    pub fn get_mut(&mut self, plugin: &str) -> Option<&mut Loaded> {
        self.loaded.get_mut(plugin)
    }
}

/// Where plugins and their data live.
#[derive(Debug, Clone)]
pub struct Paths {
    /// One directory per plugin id, each with a `zeddy-plugin.toml`.
    pub installed: PathBuf,
    /// One directory per plugin id, owned by the plugin and never by zeddy.
    pub data: PathBuf,
}

impl Paths {
    pub fn under(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self { installed: root.join("plugins"), data: root.join("plugin-data") }
    }
}

/// Load every plugin under `paths.installed`.
///
/// One bad plugin is recorded and skipped, never fatal: a plugin that fails to
/// load must not be able to stop zeddy from opening.
pub fn load_all(paths: &Paths, cx: &mut gpui::App) -> Catalog {
    let mut catalog = Catalog::default();
    let Ok(entries) = std::fs::read_dir(&paths.installed) else {
        return catalog;
    };

    let mut dirs: Vec<PathBuf> =
        entries.flatten().map(|entry| entry.path()).filter(|path| path.is_dir()).collect();
    dirs.sort();

    for dir in dirs {
        match load_one(&dir, paths, cx) {
            Ok(plugin) => {
                catalog.loaded.insert(plugin.manifest.id.clone(), plugin);
            }
            Err(why) => catalog.rejected.push(Rejected { dir, why: why.to_string() }),
        }
    }
    catalog
}

/// Why one plugin directory was refused.
#[derive(Debug)]
pub enum LoadError {
    Manifest(Invalid),
    /// The directory's name is not the manifest's id. They must agree, because
    /// the directory name is how a saved layout finds a plugin without parsing
    /// every manifest.
    IdMismatch {
        dir: String,
        manifest: String,
    },
    MissingFile(PathBuf),
    /// `dlopen` failed, or the library had no entry point.
    Library(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(invalid) => write!(f, "{invalid}"),
            Self::IdMismatch { dir, manifest } => {
                write!(f, "directory `{dir}` holds a plugin with id `{manifest}`")
            }
            Self::MissingFile(path) => write!(f, "{} is missing", path.display()),
            Self::Library(why) => write!(f, "cannot load the plugin library: {why}"),
        }
    }
}

impl std::error::Error for LoadError {}

fn load_one(dir: &Path, paths: &Paths, cx: &mut gpui::App) -> Result<Loaded, LoadError> {
    let manifest = Manifest::read(dir).map_err(LoadError::Manifest)?;

    let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
    if dir_name != manifest.id {
        return Err(LoadError::IdMismatch {
            dir: dir_name.into_owned(),
            manifest: manifest.id.clone(),
        });
    }

    let data_dir = paths.data.join(&manifest.id);
    std::fs::create_dir_all(&data_dir).ok();
    let host = Host { data_dir, plugin_dir: dir.to_owned() };

    let (tier, panes) = match manifest.kind {
        Kind::Native => {
            let filename = manifest
                .library_filename()
                .ok_or_else(|| LoadError::MissingFile(dir.join("<library>")))?;
            let library_path = dir.join(&filename);
            if !library_path.is_file() {
                return Err(LoadError::MissingFile(library_path));
            }
            let (native, panes) = open_native(&library_path, &manifest.id, host, cx)?;
            (Tier::Native(native), panes)
        }
        Kind::Web => {
            let entry = dir.join(manifest.entry.as_deref().unwrap_or("index.html"));
            if !entry.is_file() {
                return Err(LoadError::MissingFile(entry));
            }
            // A web plugin's panes come from its manifest rather than from
            // running its code: zeddy must be able to list them without
            // starting a webview.
            let panes = vec![PaneSpec {
                key: PaneKey::new(manifest.id.clone(), "main"),
                title: manifest.name.clone(),
            }];
            (Tier::Web { entry }, panes)
        }
    };

    Ok(Loaded { manifest, dir: dir.to_owned(), panes, tier })
}

fn open_native(
    path: &Path,
    id: &str,
    host: Host,
    cx: &mut gpui::App,
) -> Result<(Native, Vec<PaneSpec>), LoadError> {
    // SAFETY: loading a library runs its initialisers, which is arbitrary
    // native code. That is the documented trust model of the native tier — the
    // manifest's `native_abi` has already been checked to match this build, and
    // nothing beyond that is verifiable in-process.
    let library = unsafe { libloading::Library::new(path) }
        .map_err(|err| LoadError::Library(format!("{}: {err}", path.display())))?;
    // Leaked on purpose: see the module comment.
    let library: &'static libloading::Library = Box::leak(Box::new(library));

    // SAFETY: the symbol's type is the contract in `zeddy-plugin`, and the ABI
    // check above is what makes that contract the same one this build compiled.
    let entry: libloading::Symbol<'static, Entry> =
        unsafe { library.get(zeddy_plugin::ENTRY_SYMBOL) }
            .map_err(|err| LoadError::Library(format!("no zeddy_plugin_entry: {err}")))?;

    // SAFETY: as above.
    let mut plugin = unsafe { entry(host, cx) };

    if plugin.id() != id {
        return Err(LoadError::IdMismatch { dir: id.to_owned(), manifest: plugin.id().to_owned() });
    }

    let mut registrar = Registrar::new(id);
    plugin.activate(&mut registrar, cx);
    let panes = registrar.panes().to_vec();

    Ok((Native { plugin, _library: library }, panes))
}

#[cfg(test)]
mod tests {
    //! Loading a real native library needs one to have been built, so these
    //! cover discovery, validation, and the web tier. The native path is
    //! exercised end to end by `plugins/hello` in the app's own tests.

    use super::*;

    fn paths() -> (tempfile::TempDir, Paths) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let paths = Paths::under(tmp.path());
        std::fs::create_dir_all(&paths.installed).expect("installed dir");
        (tmp, paths)
    }

    fn write_web(paths: &Paths, id: &str, dir_name: &str) -> PathBuf {
        let dir = paths.installed.join(dir_name);
        std::fs::create_dir_all(&dir).expect("plugin dir");
        std::fs::write(
            dir.join("zeddy-plugin.toml"),
            format!(
                "manifest_version = 1\nid = \"{id}\"\nname = \"Notes\"\n\
                 version = \"0.1.0\"\nkind = \"web\"\nentry = \"index.html\"\n"
            ),
        )
        .expect("manifest");
        std::fs::write(dir.join("index.html"), "<p>hi</p>").expect("entry");
        dir
    }

    #[gpui::test]
    fn a_web_plugin_is_discovered_and_contributes_a_pane(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        write_web(&paths, "com.example.notes", "com.example.notes");

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert_eq!(catalog.loaded.len(), 1);
        assert_eq!(catalog.panes().len(), 1);
        assert_eq!(catalog.panes()[0].title, "Notes");
    }

    #[gpui::test]
    fn a_directory_whose_name_disagrees_with_the_manifest_is_rejected(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_tmp, paths) = paths();
        write_web(&paths, "com.example.notes", "notes");

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.loaded.is_empty());
        assert_eq!(catalog.rejected.len(), 1);
        assert!(catalog.rejected[0].why.contains("com.example.notes"), "{:?}", catalog.rejected[0]);
    }

    #[gpui::test]
    fn a_web_plugin_with_no_entry_document_is_rejected(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = write_web(&paths, "com.example.notes", "com.example.notes");
        std::fs::remove_file(dir.join("index.html")).expect("remove entry");

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert_eq!(catalog.rejected.len(), 1);
        assert!(catalog.rejected[0].why.contains("index.html"));
    }

    #[gpui::test]
    fn one_bad_plugin_does_not_stop_the_others_loading(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        write_web(&paths, "com.example.notes", "com.example.notes");
        let broken = paths.installed.join("com.example.broken");
        std::fs::create_dir_all(&broken).expect("dir");
        std::fs::write(broken.join("zeddy-plugin.toml"), "not toml {{{").expect("manifest");

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert_eq!(catalog.loaded.len(), 1);
        assert_eq!(catalog.rejected.len(), 1);
    }

    #[gpui::test]
    fn a_pane_a_plugin_never_declared_has_no_source(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        write_web(&paths, "com.example.notes", "com.example.notes");

        let mut catalog = cx.update(|cx| load_all(&paths, cx));
        let plugin = catalog.get_mut("com.example.notes").expect("loaded");
        assert!(plugin.pane(&PaneKey::new("com.example.notes", "main")).is_some());
        assert!(plugin.pane(&PaneKey::new("com.example.notes", "gone")).is_none());
    }

    #[gpui::test]
    fn a_missing_plugin_directory_is_an_empty_catalog_not_an_error(cx: &mut gpui::TestAppContext) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let paths = Paths::under(tmp.path().join("nothing-here"));
        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.loaded.is_empty() && catalog.rejected.is_empty());
    }
}
