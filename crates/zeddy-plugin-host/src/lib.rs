//! Chartr's plugin runtime catalog and isolation boundary.
//!
//! Plugin packages are discovered the same way — a directory with a
//! `zeddy-plugin.toml` in it — and all arrive at the app as the same thing: a
//! list of panes with a way to build each one. Everything above this crate sees
//! [`Loaded`] and never asks which tier a pane came from.
//!
//! # Discovery, not installation
//!
//! This crate reads what is already on disk. Fetching a package and deciding
//! whether the user trusts it are separate concerns and separate code;
//! putting them here would mean the window could not enumerate plugins without
//! also being able to install them.
//!
//! Separately compiled GPUI libraries are rejected. Rust GUI objects and crate
//! globals do not have a stable dynamic-library ABI; native examples that ship
//! with Chartr are linked into the application instead.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use zeddy_plugin::{
    Host, PaneKey, PaneSpec, PluginObject, Registrar,
    manifest::{Capabilities, Invalid, Kind, Manifest, Permissions, ProjectAccess},
};

/// One plugin, loaded and activated.
pub struct Loaded {
    pub manifest: Manifest,
    pub dir: PathBuf,
    pub panes: Vec<PaneSpec>,
    pub has_settings: bool,
    tier: Tier,
    source: LoadSource,
}

/// A statically bundled native example still exercises the native plugin
/// contract, while avoiding a second copy of GPUI in the application bundle.
pub type NativeFactory = fn(Host, &mut gpui::App) -> Box<dyn PluginObject>;

#[derive(Debug, Clone, Copy)]
enum LoadSource {
    Directory,
    BundledNative(NativeFactory),
}

/// The runtime-specific half of a loaded plugin.
enum Tier {
    Native(Native),
    Hosted(HostedSurface),
    Web { entry: PathBuf, settings_entry: Option<PathBuf> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedSurface {
    Browser,
}

impl HostedSurface {
    /// Resolve the small, explicit allowlist of surfaces implemented by Chartr.
    pub fn named(name: &str) -> Result<Self, LoadError> {
        match name {
            "browser" => Ok(Self::Browser),
            name => Err(LoadError::UnsupportedSurface(name.to_owned())),
        }
    }
}

pub enum SettingsSource {
    Native(gpui::AnyView),
    Web(PathBuf),
}

/// A native module linked into Chartr and the object it produced.
struct Native {
    plugin: Box<dyn PluginObject>,
}

/// How a pane should be built, once something above decides to show it.
pub enum PaneSource<'a> {
    /// Call into the plugin for a GPUI view, mounted directly in the tree.
    Native(&'a mut dyn PluginObject),
    Hosted(HostedSurface),
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

    /// The package-owned Hugeicons SVG used by Chartr's tab chrome.
    pub fn icon_path(&self) -> PathBuf {
        self.manifest.icon_path(&self.dir)
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
            Tier::Hosted(surface) => PaneSource::Hosted(*surface),
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
            Tier::Hosted(_) => None,
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
    source: LoadSource,
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

    pub fn contains(&self, plugin: &str) -> bool {
        self.loaded.contains_key(plugin) || self.disabled.contains_key(plugin)
    }

    pub fn disable(&mut self, plugin: &str) -> bool {
        let Some(loaded) = self.loaded.remove(plugin) else {
            return false;
        };
        self.disabled.insert(
            plugin.to_owned(),
            Disabled { manifest: loaded.manifest, dir: loaded.dir, source: loaded.source },
        );
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
        let result = match disabled.source {
            LoadSource::Directory => load_one(&disabled.dir, paths, cx),
            LoadSource::BundledNative(factory) => load_builtin_native(
                disabled.manifest.clone(),
                disabled.dir.clone(),
                paths,
                factory,
                cx,
            ),
        };
        match result {
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

    /// Add one plugin from a directory outside the user installation root.
    /// Bundled web examples use the exact same loader and sandbox as installed
    /// web plugins; only their source directory differs.
    pub fn add_directory(&mut self, dir: &Path, paths: &Paths, enabled: bool, cx: &mut gpui::App) {
        let manifest = match Manifest::read(dir) {
            Ok(manifest) => manifest,
            Err(why) => {
                self.rejected.push(Rejected { dir: dir.to_owned(), why: why.to_string() });
                return;
            }
        };
        if self.loaded.contains_key(&manifest.id) || self.disabled.contains_key(&manifest.id) {
            return;
        }
        if !enabled {
            self.disabled.insert(
                manifest.id.clone(),
                Disabled { manifest, dir: dir.to_owned(), source: LoadSource::Directory },
            );
            return;
        }
        match load_one(dir, paths, cx) {
            Ok(plugin) => {
                self.loaded.insert(plugin.manifest.id.clone(), plugin);
            }
            Err(why) => {
                self.rejected.push(Rejected { dir: dir.to_owned(), why: why.to_string() });
            }
        }
    }

    /// Add a trusted native plugin compiled into Chartr itself.
    pub fn add_bundled_native(
        &mut self,
        manifest: Manifest,
        dir: PathBuf,
        paths: &Paths,
        enabled: bool,
        factory: NativeFactory,
        cx: &mut gpui::App,
    ) {
        if self.loaded.contains_key(&manifest.id) || self.disabled.contains_key(&manifest.id) {
            return;
        }
        if !enabled {
            self.disabled.insert(
                manifest.id.clone(),
                Disabled { manifest, dir, source: LoadSource::BundledNative(factory) },
            );
            return;
        }
        match load_builtin_native(manifest, dir.clone(), paths, factory, cx) {
            Ok(plugin) => {
                self.loaded.insert(plugin.manifest.id.clone(), plugin);
            }
            Err(why) => self.rejected.push(Rejected { dir, why: why.to_string() }),
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
    /// Application-owned copies of examples shipped with this build.
    pub bundled: PathBuf,
}

impl Paths {
    pub fn under(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            installed: root.join("plugins"),
            data: root.join("plugin-data"),
            bundled: root.join("bundled-plugins"),
        }
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
            catalog.disabled.insert(
                manifest.id.clone(),
                Disabled { manifest, dir, source: LoadSource::Directory },
            );
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
    ExternalNative,
    UnsupportedSurface(String),
    BundledKind(Kind),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(invalid) => write!(f, "{invalid}"),
            Self::IdMismatch { dir, manifest } => {
                write!(f, "directory `{dir}` holds a plugin with id `{manifest}`")
            }
            Self::MissingFile(path) => write!(f, "{} is missing", path.display()),
            Self::ExternalNative => write!(
                f,
                "separately compiled native GPUI plugins are unsupported; use a web package or a Chartr-hosted surface"
            ),
            Self::UnsupportedSurface(surface) => {
                write!(f, "Chartr does not support the hosted surface `{surface}`")
            }
            Self::BundledKind(kind) => {
                write!(f, "a bundled native factory cannot use a {kind:?} manifest")
            }
        }
    }
}

impl std::error::Error for LoadError {}

fn load_one(dir: &Path, paths: &Paths, _cx: &mut gpui::App) -> Result<Loaded, LoadError> {
    let manifest = Manifest::read(dir).map_err(LoadError::Manifest)?;

    let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
    if dir_name != manifest.id {
        return Err(LoadError::IdMismatch {
            dir: dir_name.into_owned(),
            manifest: manifest.id.clone(),
        });
    }

    let icon = manifest.icon_path(dir);
    if !icon.is_file() {
        return Err(LoadError::MissingFile(icon));
    }

    std::fs::create_dir_all(paths.data.join(&manifest.id)).ok();

    let (tier, panes, has_settings) = match manifest.kind {
        Kind::Native => {
            return Err(LoadError::ExternalNative);
        }
        Kind::Hosted => {
            let surface = HostedSurface::named(manifest.surface.as_deref().unwrap_or_default())?;
            let panes = vec![PaneSpec {
                key: PaneKey::new(manifest.id.clone(), "main"),
                title: manifest.name.clone(),
            }];
            (Tier::Hosted(surface), panes, false)
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

    Ok(Loaded {
        manifest,
        dir: dir.to_owned(),
        panes,
        has_settings,
        tier,
        source: LoadSource::Directory,
    })
}

fn load_builtin_native(
    manifest: Manifest,
    dir: PathBuf,
    paths: &Paths,
    factory: NativeFactory,
    cx: &mut gpui::App,
) -> Result<Loaded, LoadError> {
    if manifest.kind != Kind::Native {
        return Err(LoadError::BundledKind(manifest.kind));
    }
    let icon = manifest.icon_path(&dir);
    if !icon.is_file() {
        return Err(LoadError::MissingFile(icon));
    }
    let host = Host { data_dir: paths.data.join(&manifest.id), plugin_dir: dir.clone() };
    std::fs::create_dir_all(&host.data_dir).ok();
    let mut plugin = factory(host, cx);
    if plugin.id() != manifest.id {
        return Err(LoadError::IdMismatch {
            dir: manifest.id.clone(),
            manifest: plugin.id().to_owned(),
        });
    }
    let mut registrar = Registrar::new(&manifest.id);
    plugin.activate(&mut registrar, cx);
    let panes = registrar.panes().to_vec();
    let has_settings = registrar.has_settings();
    Ok(Loaded {
        manifest,
        dir,
        panes,
        has_settings,
        tier: Tier::Native(Native { plugin }),
        source: LoadSource::BundledNative(factory),
    })
}

#[cfg(test)]
mod tests {
    //! Discovery, validation, portable web packages, hosted surfaces, and
    //! rejection of separately linked native libraries.

    use super::*;
    use gpui::{AppContext as _, ParentElement as _};
    use zeddy_plugin::Plugin as _;

    struct BundledPlugin;

    impl zeddy_plugin::Plugin for BundledPlugin {
        const ID: &'static str = "com.example.bundled";

        fn new(_: Host, _: &mut gpui::App) -> Self {
            Self
        }

        fn activate(&mut self, registrar: &mut Registrar, _: &mut gpui::App) {
            registrar.add_pane("main", "Bundled");
        }

        fn view(
            &mut self,
            _: &PaneKey,
            _: &zeddy_plugin::InstanceContext,
            _: &mut gpui::Window,
            cx: &mut gpui::App,
        ) -> gpui::AnyView {
            cx.new(|_| BundledView).into()
        }
    }

    struct BundledView;

    impl gpui::Render for BundledView {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            gpui::div().child("Bundled")
        }
    }

    fn bundled_factory(host: Host, cx: &mut gpui::App) -> Box<dyn PluginObject> {
        Box::new(<BundledPlugin as zeddy_plugin::Plugin>::new(host, cx))
    }

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
                 version = \"0.1.0\"\nkind = \"web\"\nicon = \"NoteIcon\"\nentry = \"index.html\"\n"
            ),
        )
        .expect("manifest");
        std::fs::create_dir_all(dir.join("icons")).expect("icons dir");
        std::fs::write(dir.join("icons/NoteIcon.svg"), "<svg/>").expect("icon");
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
    fn a_plugin_with_no_declared_hugeicon_svg_is_rejected(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = write_web(&paths, "com.example.notes", "com.example.notes");
        std::fs::remove_file(dir.join("icons/NoteIcon.svg")).expect("remove icon");

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert_eq!(catalog.rejected.len(), 1);
        assert!(catalog.rejected[0].why.contains("icons/NoteIcon.svg"));
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
    fn a_missing_plugin_directory_is_an_empty_catalog_not_an_error(cx: &mut gpui::TestAppContext) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let paths = Paths::under(tmp.path().join("nothing-here"));
        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.loaded.is_empty() && catalog.rejected.is_empty());
    }

    #[gpui::test]
    fn a_bundled_native_plugin_can_be_disabled_and_enabled_again(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = paths.bundled.join(BundledPlugin::ID);
        std::fs::create_dir_all(dir.join("icons")).unwrap();
        std::fs::write(dir.join("icons/BundleIcon.svg"), "<svg/>").unwrap();
        let manifest = Manifest::parse(
            "manifest_version = 2\nid = \"com.example.bundled\"\nname = \"Bundled\"\n\
            version = \"0.1.0\"\nkind = \"native\"\nicon = \"BundleIcon\"\n",
        )
        .unwrap();
        let mut catalog = cx.update(|cx| {
            let mut catalog = Catalog::default();
            catalog.add_bundled_native(manifest, dir, &paths, true, bundled_factory, cx);
            catalog
        });

        assert_eq!(catalog.panes()[0].title, "Bundled");
        assert!(catalog.disable(BundledPlugin::ID));
        cx.update(|cx| catalog.enable(&paths, BundledPlugin::ID, cx)).unwrap();
        assert_eq!(catalog.panes()[0].key.plugin, BundledPlugin::ID);
    }

    #[gpui::test]
    fn a_hosted_browser_surface_is_discovered_without_loading_code(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = paths.installed.join("com.chartr.browser");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.chartr.browser'\nname = 'Browser'\nversion = '1'\nkind = 'hosted'\nicon = 'InternetIcon'\nsurface = 'browser'\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("icons")).unwrap();
        std::fs::write(dir.join("icons/InternetIcon.svg"), "<svg/>").unwrap();

        let mut catalog = cx.update(|cx| load_all(&paths, cx));
        let browser = catalog.get_mut("com.chartr.browser").unwrap();
        assert!(matches!(
            browser.pane(&PaneKey::new("com.chartr.browser", "main")),
            Some(PaneSource::Hosted(HostedSurface::Browser))
        ));
    }

    #[gpui::test]
    fn unknown_hosted_surfaces_and_manually_copied_native_plugins_are_rejected(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_tmp, paths) = paths();
        for (id, kind) in [
            ("com.example.unknown", "kind = 'hosted'\nsurface = 'unknown'"),
            ("com.example.native", "kind = 'native'"),
        ] {
            let dir = paths.installed.join(id);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("zeddy-plugin.toml"),
                format!(
                    "manifest_version = 2\nid = '{id}'\nname = 'Rejected'\nversion = '1'\nicon = 'TestIcon'\n{kind}\n"
                ),
            )
            .unwrap();
            std::fs::create_dir_all(dir.join("icons")).unwrap();
            std::fs::write(dir.join("icons/TestIcon.svg"), "<svg/>").unwrap();
        }

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.loaded.is_empty());
        assert_eq!(catalog.rejected.len(), 2);
        assert!(
            catalog
                .rejected
                .iter()
                .any(|rejected| rejected.why.contains("hosted surface `unknown`"))
        );
        assert!(
            catalog
                .rejected
                .iter()
                .any(|rejected| rejected.why.contains("separately compiled native"))
        );
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
