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
    manifest::{Capabilities, Invalid, Kind, Manifest, Permissions, ProjectAccess},
};

/// One plugin, loaded and activated.
pub struct Loaded {
    pub manifest: Manifest,
    pub dir: PathBuf,
    pub panes: Vec<PaneSpec>,
    pub has_settings: bool,
    tier: Tier,
}

/// The tier-specific half of a loaded plugin — the only place the difference
/// between "native" and "web" is still visible.
enum Tier {
    Native(Native),
    Web { entry: PathBuf, settings_entry: Option<PathBuf> },
}

pub enum SettingsSource {
    Native(gpui::AnyView),
    Web(PathBuf),
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

    pub fn capabilities(&self) -> &Capabilities {
        &self.manifest.capabilities
    }

    pub fn permissions(&self) -> &Permissions {
        &self.manifest.permissions
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
            Tier::Web { entry, .. } => PaneSource::Web(entry.as_path()),
        })
    }

    pub fn settings(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Option<SettingsSource> {
        if !self.has_settings {
            return None;
        }
        match &mut self.tier {
            Tier::Native(native) => native.plugin.settings(window, cx).map(SettingsSource::Native),
            Tier::Web { settings_entry, .. } => settings_entry.clone().map(SettingsSource::Web),
        }
    }
}

/// A plugin directory that could not be loaded, kept so Settings can say why
/// rather than silently showing one fewer plugin.
#[derive(Debug, Clone)]
pub struct Rejected {
    pub dir: PathBuf,
    pub why: String,
}

#[derive(Debug, Clone)]
pub struct Disabled {
    pub manifest: Manifest,
    pub dir: PathBuf,
}

/// Everything found in one scan.
#[derive(Default)]
pub struct Catalog {
    /// Loaded plugins, by id. A `BTreeMap` so the sidebar's order is the same
    /// on every launch rather than the order the filesystem happened to answer.
    pub loaded: BTreeMap<String, Loaded>,
    pub disabled: BTreeMap<String, Disabled>,
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

    pub fn get(&self, plugin: &str) -> Option<&Loaded> {
        self.loaded.get(plugin)
    }

    pub fn disable(&mut self, plugin: &str) -> bool {
        let Some(loaded) = self.loaded.remove(plugin) else {
            return false;
        };
        self.disabled
            .insert(plugin.to_owned(), Disabled { manifest: loaded.manifest, dir: loaded.dir });
        true
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        self.loaded.retain(|id, _| keep(id));
    }

    pub fn enable(
        &mut self,
        paths: &Paths,
        plugin: &str,
        cx: &mut gpui::App,
    ) -> Result<(), LoadError> {
        let Some(disabled) = self.disabled.remove(plugin) else {
            return Ok(());
        };
        match load_one(&disabled.dir, paths, cx) {
            Ok(loaded) => {
                self.loaded.insert(plugin.to_owned(), loaded);
                Ok(())
            }
            Err(error) => {
                self.disabled.insert(plugin.to_owned(), disabled);
                Err(error)
            }
        }
    }
}

/// Filesystem authority for one web-plugin instance.
#[derive(Debug, Clone)]
pub struct FileBroker {
    project: Option<PathBuf>,
    data: PathBuf,
    access: ProjectAccess,
    unsafe_filesystem: bool,
}

impl FileBroker {
    pub fn new(
        project: Option<PathBuf>,
        data: PathBuf,
        access: ProjectAccess,
        unsafe_filesystem: bool,
    ) -> Self {
        Self { project, data, access, unsafe_filesystem }
    }

    pub fn project_path(&self, requested: &Path, write: bool) -> Result<PathBuf, BrokerError> {
        if self.unsafe_filesystem {
            return Ok(if requested.is_absolute() {
                requested.to_owned()
            } else if let Some(project) = &self.project {
                project.join(requested)
            } else {
                self.data.join(requested)
            });
        }
        match (self.access, write) {
            (ProjectAccess::None, _) | (ProjectAccess::Read, true) => {
                return Err(BrokerError::Denied);
            }
            _ => {}
        }
        let root = self.project.as_ref().ok_or(BrokerError::Folderless)?;
        contained(root, requested, write)
    }

    pub fn data_path(&self, requested: &Path, write: bool) -> Result<PathBuf, BrokerError> {
        contained(&self.data, requested, write)
    }
}

fn contained(root: &Path, requested: &Path, write: bool) -> Result<PathBuf, BrokerError> {
    if requested.is_absolute()
        || requested.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(BrokerError::Escape);
    }
    let root = root.canonicalize().map_err(BrokerError::Io)?;
    let candidate = root.join(requested);
    let resolved = if write && !candidate.exists() {
        let parent = candidate.parent().ok_or(BrokerError::Escape)?;
        let parent = parent.canonicalize().map_err(BrokerError::Io)?;
        parent.join(candidate.file_name().ok_or(BrokerError::Escape)?)
    } else {
        candidate.canonicalize().map_err(BrokerError::Io)?
    };
    if !resolved.starts_with(&root) {
        return Err(BrokerError::Escape);
    }
    Ok(resolved)
}

#[derive(Debug)]
pub enum BrokerError {
    Denied,
    Folderless,
    Escape,
    Io(std::io::Error),
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Denied => write!(formatter, "the plugin did not declare this project access"),
            Self::Folderless => {
                write!(formatter, "safe mode exposes no project filesystem in the folderless space")
            }
            Self::Escape => write!(formatter, "the requested path escapes the allowed root"),
            Self::Io(error) => write!(formatter, "resolving the requested path: {error}"),
        }
    }
}

impl std::error::Error for BrokerError {}

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
    load_all_where(paths, |_| true, cx)
}

pub fn load_all_where(
    paths: &Paths,
    mut enabled: impl FnMut(&str) -> bool,
    cx: &mut gpui::App,
) -> Catalog {
    let mut catalog = Catalog::default();
    let Ok(entries) = std::fs::read_dir(&paths.installed) else {
        return catalog;
    };

    let mut dirs: Vec<PathBuf> =
        entries.flatten().map(|entry| entry.path()).filter(|path| path.is_dir()).collect();
    dirs.sort();

    for dir in dirs {
        if let Ok(manifest) = Manifest::read(&dir)
            && !enabled(&manifest.id)
        {
            catalog.disabled.insert(manifest.id.clone(), Disabled { manifest, dir });
            continue;
        }
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

    let (tier, panes, has_settings) = match manifest.kind {
        Kind::Native => {
            let filename = manifest
                .library_filename()
                .ok_or_else(|| LoadError::MissingFile(dir.join("<library>")))?;
            let library_path = dir.join(&filename);
            if !library_path.is_file() {
                return Err(LoadError::MissingFile(library_path));
            }
            let (native, panes, has_settings) = open_native(&library_path, &manifest.id, host, cx)?;
            (Tier::Native(native), panes, has_settings)
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
            let settings_entry = manifest.settings_entry.as_ref().map(|entry| dir.join(entry));
            if let Some(settings_entry) = &settings_entry
                && !settings_entry.is_file()
            {
                return Err(LoadError::MissingFile(settings_entry.clone()));
            }
            let has_settings = settings_entry.is_some();
            (Tier::Web { entry, settings_entry }, panes, has_settings)
        }
    };

    Ok(Loaded { manifest, dir: dir.to_owned(), panes, has_settings, tier })
}

fn open_native(
    path: &Path,
    id: &str,
    host: Host,
    cx: &mut gpui::App,
) -> Result<(Native, Vec<PaneSpec>, bool), LoadError> {
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
    let has_settings = registrar.has_settings();

    Ok((Native { plugin, _library: library }, panes, has_settings))
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
                "manifest_version = 2\nid = \"{id}\"\nname = \"Notes\"\n\
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
    fn a_web_settings_document_is_validated_but_not_constructed_during_discovery(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_tmp, paths) = paths();
        let dir = write_web(&paths, "com.example.notes", "com.example.notes");
        let manifest = std::fs::read_to_string(dir.join("zeddy-plugin.toml")).unwrap();
        std::fs::write(
            dir.join("zeddy-plugin.toml"),
            format!("{manifest}settings_entry = \"settings.html\"\n"),
        )
        .unwrap();
        std::fs::write(dir.join("settings.html"), "<p>settings</p>").unwrap();

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.get("com.example.notes").unwrap().has_settings);
    }

    #[gpui::test]
    fn a_declared_missing_web_settings_document_rejects_the_plugin(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = write_web(&paths, "com.example.notes", "com.example.notes");
        let manifest = std::fs::read_to_string(dir.join("zeddy-plugin.toml")).unwrap();
        std::fs::write(
            dir.join("zeddy-plugin.toml"),
            format!("{manifest}settings_entry = \"missing.html\"\n"),
        )
        .unwrap();

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.loaded.is_empty());
        assert!(catalog.rejected[0].why.contains("missing.html"));
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

    #[test]
    fn safe_project_access_stays_beneath_the_canonical_root() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let data = temp.path().join("data");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(project.join("readme.md"), "hello").unwrap();
        let broker = FileBroker::new(Some(project.clone()), data, ProjectAccess::ReadWrite, false);

        assert_eq!(
            broker.project_path(Path::new("readme.md"), false).unwrap(),
            project.canonicalize().unwrap().join("readme.md")
        );
        assert!(matches!(
            broker.project_path(Path::new("../outside"), true),
            Err(BrokerError::Escape)
        ));
    }

    #[test]
    fn folderless_safe_plugins_receive_only_their_data_root() {
        let temp = tempfile::tempdir().unwrap();
        let data = temp.path().join("data");
        std::fs::create_dir_all(&data).unwrap();
        let broker = FileBroker::new(None, data.clone(), ProjectAccess::ReadWrite, false);

        assert!(matches!(
            broker.project_path(Path::new("anything"), false),
            Err(BrokerError::Folderless)
        ));
        assert_eq!(
            broker.data_path(Path::new("state.json"), true).unwrap(),
            data.canonicalize().unwrap().join("state.json")
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_cannot_escape_a_safe_project_root() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let outside = temp.path().join("outside");
        let data = temp.path().join("data");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        symlink(&outside, project.join("escape")).unwrap();
        let broker = FileBroker::new(Some(project), data, ProjectAccess::ReadWrite, false);

        assert!(matches!(
            broker.project_path(Path::new("escape/file.txt"), true),
            Err(BrokerError::Escape)
        ));
    }
}
