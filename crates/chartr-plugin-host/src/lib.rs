//! chartr's plugin runtime catalog and isolation boundary.
//!
//! Plugin packages are discovered the same way — a directory with a
//! `chartr-plugin.toml` in it — and all arrive at the app as the same thing: a
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
//! with chartr are linked into the application instead.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use chartr_plugin::{
    Host, PaneKey, PaneSpec, PluginObject, Registrar,
    manifest::{Capabilities, Invalid, Kind, Manifest, Permissions, ProjectAccess},
};

/// One plugin, loaded and activated.
pub struct Loaded {
    pub manifest: Manifest,
    pub dir: PathBuf,
    pub installation: Option<Installation>,
    pub panes: Vec<PaneSpec>,
    pub has_settings: bool,
    tier: Tier,
    source: LoadSource,
}

/// Recorded by the installer, for display only; this is not an authenticity check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Installation {
    pub source: String,
    pub commit: Option<String>,
}

impl Installation {
    pub const FILE: &str = ".chartr-install.json";

    fn read(dir: &Path) -> Option<Self> {
        serde_json::from_slice(&std::fs::read(dir.join(Self::FILE)).ok()?).ok()
    }
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
    Web { entry: PathBuf },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedSurface {
    Browser,
}

impl HostedSurface {
    /// Resolve the small, explicit allowlist of surfaces implemented by chartr.
    pub fn named(name: &str) -> Result<Self, LoadError> {
        match name {
            "browser" => Ok(Self::Browser),
            name => Err(LoadError::UnsupportedSurface(name.to_owned())),
        }
    }
}

pub enum SettingsSource {
    Native(gpui::AnyView),
    Declarative(chartr_plugin::settings::SettingsSchema),
}

/// A native module linked into chartr and the object it produced.
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

    /// The package-owned Hugeicons SVG used by chartr's tab chrome.
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
            Tier::Hosted(_) | Tier::Web { .. } => {
                self.manifest.settings.clone().map(SettingsSource::Declarative)
            }
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
    pub installation: Option<Installation>,
    source: LoadSource,
}

/// Everything found in one scan.
#[derive(Default)]
pub struct Catalog {
    pub services: chartr_plugin::services::Services,
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
        self.loaded.contains_key(plugin)
            || self.disabled.contains_key(plugin)
            || self
                .rejected
                .iter()
                .any(|rejected| rejected.dir.file_name().is_some_and(|name| name == plugin))
    }

    pub fn manifest(&self, plugin: &str) -> Option<&Manifest> {
        self.loaded
            .get(plugin)
            .map(|loaded| &loaded.manifest)
            .or_else(|| self.disabled.get(plugin).map(|disabled| &disabled.manifest))
    }

    /// The complete dependent closure, including disabled dependents, in stable order.
    pub fn dependents(&self, plugin: &str) -> Vec<String> {
        let mut affected = std::collections::BTreeSet::from([plugin.to_owned()]);
        loop {
            let before = affected.len();
            for manifest in self
                .loaded
                .values()
                .map(|p| &p.manifest)
                .chain(self.disabled.values().map(|p| &p.manifest))
            {
                if manifest.dependencies.iter().any(|d| affected.contains(&d.plugin)) {
                    affected.insert(manifest.id.clone());
                }
            }
            if affected.len() == before {
                break;
            }
        }
        affected.remove(plugin);
        affected.into_iter().collect()
    }

    pub fn prerequisite_error(&self, plugin: &str) -> Option<String> {
        let manifest = self.manifest(plugin)?;
        let missing: Vec<_> = manifest
            .dependencies
            .iter()
            .filter(|dependency| !self.loaded.contains_key(&dependency.plugin))
            .map(|dependency| {
                self.manifest(&dependency.plugin)
                    .map(|provider| provider.name.clone())
                    .unwrap_or_else(|| dependency.plugin.clone())
            })
            .collect();
        (!missing.is_empty())
            .then(|| format!("Requires {} to be installed and enabled.", missing.join(", ")))
    }

    /// Discover first, then activate providers before their consumers. Missing,
    /// disabled, failed and cyclic prerequisites leave consumers disabled.
    pub fn enable_requested(
        &mut self,
        paths: &Paths,
        mut requested: impl FnMut(&str) -> bool,
        cx: &mut gpui::App,
    ) {
        let mut pending: std::collections::BTreeSet<_> =
            self.disabled.keys().filter(|id| requested(id)).cloned().collect();
        loop {
            let ready: Vec<_> = pending
                .iter()
                .filter(|id| self.prerequisite_error(id).is_none())
                .cloned()
                .collect();
            if ready.is_empty() {
                break;
            }
            for id in ready {
                pending.remove(&id);
                if let Err(error) = self.enable(paths, &id, cx) {
                    // Preserve the entry so the user can inspect/retry the package.
                    eprintln!("Cannot enable plugin {id}: {error}");
                }
            }
        }
    }

    pub fn disable(&mut self, plugin: &str) -> bool {
        for dependent in self.dependents(plugin) {
            self.disable_one(&dependent);
        }
        self.disable_one(plugin)
    }

    fn disable_one(&mut self, plugin: &str) -> bool {
        let Some(loaded) = self.loaded.remove(plugin) else {
            return false;
        };
        self.services.remove(plugin);
        self.disabled.insert(
            plugin.to_owned(),
            Disabled {
                manifest: loaded.manifest,
                dir: loaded.dir,
                installation: loaded.installation,
                source: loaded.source,
            },
        );
        true
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        let removed: Vec<_> = self.loaded.keys().filter(|id| !keep(id)).cloned().collect();
        for id in removed {
            self.disable(&id);
            self.disabled.remove(&id);
        }
    }

    pub fn enable(
        &mut self,
        paths: &Paths,
        plugin: &str,
        cx: &mut gpui::App,
    ) -> Result<(), LoadError> {
        if let Some(error) = self.prerequisite_error(plugin) {
            return Err(LoadError::Prerequisites(error));
        }
        let Some(disabled) = self.disabled.remove(plugin) else {
            return if self.loaded.contains_key(plugin) {
                Ok(())
            } else {
                Err(LoadError::NotInstalled(plugin.to_owned()))
            };
        };
        let result = match disabled.source {
            LoadSource::Directory => read_directory_manifest(&disabled.dir).and_then(|manifest| {
                let missing: Vec<_> = manifest
                    .dependencies
                    .iter()
                    .filter(|d| !self.loaded.contains_key(&d.plugin))
                    .map(|d| d.plugin.clone())
                    .collect();
                if !missing.is_empty() {
                    return Err(LoadError::Prerequisites(format!(
                        "Requires {} to be installed and enabled.",
                        missing.join(", ")
                    )));
                }
                load_validated(&disabled.dir, paths, manifest)
            }),
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
                self.publish_services(&loaded);
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
    pub fn add_directory(&mut self, dir: &Path, paths: &Paths, enabled: bool, _cx: &mut gpui::App) {
        let manifest = match read_directory_manifest(dir) {
            Ok(manifest) => manifest,
            Err(why) => {
                self.rejected.push(Rejected { dir: dir.to_owned(), why: why.to_string() });
                return;
            }
        };
        if self.loaded.contains_key(&manifest.id) || self.disabled.contains_key(&manifest.id) {
            return;
        }
        if !enabled || manifest.dependencies.iter().any(|d| !self.loaded.contains_key(&d.plugin)) {
            self.disabled.insert(
                manifest.id.clone(),
                Disabled {
                    manifest,
                    dir: dir.to_owned(),
                    installation: Installation::read(dir),
                    source: LoadSource::Directory,
                },
            );
            return;
        }
        match load_validated(dir, paths, manifest) {
            Ok(plugin) => {
                self.loaded.insert(plugin.manifest.id.clone(), plugin);
            }
            Err(why) => {
                self.rejected.push(Rejected { dir: dir.to_owned(), why: why.to_string() });
            }
        }
    }

    /// Add a trusted native plugin compiled into chartr itself.
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
        if !enabled || manifest.dependencies.iter().any(|d| !self.loaded.contains_key(&d.plugin)) {
            self.disabled.insert(
                manifest.id.clone(),
                Disabled {
                    manifest,
                    dir,
                    installation: None,
                    source: LoadSource::BundledNative(factory),
                },
            );
            return;
        }
        match load_builtin_native(manifest, dir.clone(), paths, factory, cx) {
            Ok(plugin) => {
                self.publish_services(&plugin);
                self.loaded.insert(plugin.manifest.id.clone(), plugin);
            }
            Err(why) => self.rejected.push(Rejected { dir, why: why.to_string() }),
        }
    }

    fn publish_services(&self, loaded: &Loaded) {
        if let Tier::Native(native) = &loaded.tier {
            self.services.publish(loaded.id(), native.plugin.services());
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

    pub fn process_directory(&self, requested: Option<&Path>) -> Result<PathBuf, BrokerError> {
        // Process permission already grants user-level execution. Relative
        // working directories are rooted in the owning project (or plugin data).
        let base = self.project.as_ref().unwrap_or(&self.data);
        let path = requested.map_or_else(|| base.clone(), |path| base.join(path));
        let path = path.canonicalize().map_err(BrokerError::Io)?;
        if !path.is_dir() {
            return Err(BrokerError::Denied);
        }
        Ok(path)
    }

    pub fn open_project(&self, requested: &Path) -> Result<std::fs::File, BrokerError> {
        let path = self.project_path(requested, false)?;
        if self.unsafe_filesystem {
            return std::fs::File::open(path).map_err(BrokerError::Io);
        }
        open_contained(self.project.as_ref().ok_or(BrokerError::Folderless)?, &path)
    }

    pub fn open_data(&self, requested: &Path) -> Result<std::fs::File, BrokerError> {
        open_contained(&self.data, &self.data_path(requested, false)?)
    }

    pub fn write_project(&self, requested: &Path, bytes: &[u8]) -> Result<(), BrokerError> {
        let path = self.project_path(requested, true)?;
        if self.unsafe_filesystem {
            let path = match path.canonicalize() {
                Ok(path) => path,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => path
                    .parent()
                    .ok_or(BrokerError::Escape)?
                    .canonicalize()
                    .map_err(BrokerError::Io)?
                    .join(path.file_name().ok_or(BrokerError::Escape)?),
                Err(error) => return Err(BrokerError::Io(error)),
            };
            return write_contained(path.parent().ok_or(BrokerError::Escape)?, &path, bytes);
        }
        write_contained(self.project.as_ref().ok_or(BrokerError::Folderless)?, &path, bytes)
    }

    pub fn write_data(&self, requested: &Path, bytes: &[u8]) -> Result<(), BrokerError> {
        write_contained(&self.data, &self.data_path(requested, true)?, bytes)
    }
}

/// Walk directories through descriptors so a path cannot be redirected by a
/// symlink introduced between validation and the actual operation.
fn open_parent(
    root: &Path,
    resolved: &Path,
) -> Result<(std::os::fd::OwnedFd, std::ffi::OsString), BrokerError> {
    use rustix::fs::{Mode, OFlags, open, openat};
    let root = root.canonicalize().map_err(BrokerError::Io)?;
    let relative = resolved.strip_prefix(&root).map_err(|_| BrokerError::Escape)?;
    let mut parts = relative.components().peekable();
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = open(&root, flags, Mode::empty()).map_err(|e| BrokerError::Io(e.into()))?;
    while let Some(part) = parts.next() {
        if !matches!(part, std::path::Component::Normal(_)) {
            return Err(BrokerError::Escape);
        }
        if parts.peek().is_none() {
            return Ok((directory, part.as_os_str().to_owned()));
        }
        directory = openat(&directory, part.as_os_str(), flags, Mode::empty())
            .map_err(|e| BrokerError::Io(e.into()))?;
    }
    Err(BrokerError::Denied)
}

fn open_contained(root: &Path, resolved: &Path) -> Result<std::fs::File, BrokerError> {
    use rustix::fs::{Mode, OFlags, openat};
    let (directory, name) = open_parent(root, resolved)?;
    let fd = openat(
        &directory,
        &name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map_err(|e| BrokerError::Io(e.into()))?;
    let file = std::fs::File::from(fd);
    if !file.metadata().map_err(BrokerError::Io)?.is_file() {
        return Err(BrokerError::Denied);
    }
    Ok(file)
}

fn write_contained(root: &Path, resolved: &Path, bytes: &[u8]) -> Result<(), BrokerError> {
    use rustix::fs::{AtFlags, Mode, OFlags, openat, renameat, unlinkat};
    use std::{
        io::Write as _,
        sync::atomic::{AtomicU64, Ordering},
    };
    let (directory, name) = open_parent(root, resolved)?;
    // Validate existing destinations without truncating them and retain their
    // permissions. Never replace directories, devices, or newly introduced links.
    let permissions = match openat(
        &directory,
        &name,
        OFlags::WRONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
    ) {
        Ok(fd) => {
            let metadata = std::fs::File::from(fd).metadata().map_err(BrokerError::Io)?;
            if !metadata.is_file() {
                return Err(BrokerError::Denied);
            }
            Some(metadata.permissions())
        }
        Err(rustix::io::Errno::NOENT) => None,
        Err(e) => return Err(BrokerError::Io(e.into())),
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temporary =
        format!(".chartr-write-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
    let fd = openat(
        &directory,
        &temporary,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC,
        Mode::from_raw_mode(0o600),
    )
    .map_err(|e| BrokerError::Io(e.into()))?;
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::from(fd);
        file.write_all(bytes)?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        file.sync_all()?;
        renameat(&directory, &temporary, &directory, &name)?;
        Ok(())
    })();
    // Both success and failure leave no partial file under the public name.
    let _ = unlinkat(&directory, &temporary, AtFlags::empty());
    result.map_err(BrokerError::Io)
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
    // `exists` follows symlinks: a dangling link is not a new, safe filename.
    let missing = match candidate.symlink_metadata() {
        Ok(_) => false,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => return Err(BrokerError::Io(error)),
    };
    let resolved = if write && missing {
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
    /// One directory per plugin id, each with a `chartr-plugin.toml`.
    pub installed: PathBuf,
    /// One directory per plugin id, owned by the plugin and never by chartr.
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
/// load must not be able to stop chartr from opening.
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
        catalog.add_directory(&dir, paths, false, cx);
    }
    catalog.enable_requested(paths, |id| enabled(id), cx);
    catalog
}

/// Why one plugin directory was refused.
#[derive(Debug)]
pub enum LoadError {
    Manifest(Invalid),
    Prerequisites(String),
    NotInstalled(String),
    /// The directory's name is not the manifest's id. They must agree, because
    /// the directory name is how a saved layout finds a plugin without parsing
    /// every manifest.
    IdMismatch {
        dir: String,
        manifest: String,
    },
    MissingFile(PathBuf),
    EscapingFile(PathBuf),
    ExternalNative,
    UnsupportedSurface(String),
    BundledKind(Kind),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled(plugin) => write!(f, "Plugin {plugin} is not installed."),
            Self::Prerequisites(detail) => write!(f, "{detail}"),
            Self::Manifest(invalid) => write!(f, "{invalid}"),
            Self::IdMismatch { dir, manifest } => {
                write!(f, "directory `{dir}` holds a plugin with id `{manifest}`")
            }
            Self::MissingFile(path) => write!(f, "{} is missing", path.display()),
            Self::EscapingFile(path) => {
                write!(f, "{} escapes the plugin directory", path.display())
            }
            Self::ExternalNative => write!(
                f,
                "separately compiled native GPUI libraries are not supported because Rust GUI objects are not safe across a dynamic-library boundary. Use a web package or a chartr-hosted surface; installation never compiles plugin source"
            ),
            Self::UnsupportedSurface(surface) => {
                write!(f, "chartr does not support the hosted surface `{surface}`")
            }
            Self::BundledKind(kind) => {
                write!(f, "a bundled native factory cannot use a {kind:?} manifest")
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// Validate an external package identically during installation and discovery.
/// Staging directories need not have the plugin's id as their name.
pub fn validate_package(dir: &Path, manifest: &Manifest) -> Result<(), LoadError> {
    match manifest.kind {
        Kind::Native => return Err(LoadError::ExternalNative),
        Kind::Hosted => {
            HostedSurface::named(manifest.surface.as_deref().unwrap_or_default())?;
        }
        Kind::Web => {}
    }
    require_package_file(dir, &manifest.icon_relative_path())?;
    if manifest.kind == Kind::Web {
        let entry = manifest
            .entry
            .as_deref()
            .ok_or(LoadError::Manifest(Invalid::Missing { field: "entry", kind: Kind::Web }))?;
        require_package_file(dir, Path::new(entry))?;
    }
    Ok(())
}

fn require_package_file(dir: &Path, relative: &Path) -> Result<(), LoadError> {
    if relative.is_absolute()
        || relative.components().any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(LoadError::EscapingFile(relative.to_owned()));
    }
    let file = dir.join(relative);
    let root = dir.canonicalize().map_err(|_| LoadError::MissingFile(dir.to_owned()))?;
    let resolved = file.canonicalize().map_err(|_| LoadError::MissingFile(file.clone()))?;
    if !resolved.starts_with(root) {
        return Err(LoadError::EscapingFile(relative.to_owned()));
    }
    if !resolved.is_file() {
        return Err(LoadError::MissingFile(file));
    }
    Ok(())
}

fn read_directory_manifest(dir: &Path) -> Result<Manifest, LoadError> {
    let manifest = Manifest::read(dir).map_err(LoadError::Manifest)?;

    let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
    if dir_name != manifest.id {
        return Err(LoadError::IdMismatch {
            dir: dir_name.into_owned(),
            manifest: manifest.id.clone(),
        });
    }

    validate_package(dir, &manifest)?;
    Ok(manifest)
}

fn load_validated(dir: &Path, paths: &Paths, manifest: Manifest) -> Result<Loaded, LoadError> {
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
            (Tier::Hosted(surface), panes, manifest.settings.is_some())
        }
        Kind::Web => {
            let entry = dir.join(manifest.entry.as_deref().unwrap_or("index.html"));
            // A web plugin's panes come from its manifest rather than from
            // running its code: chartr must be able to list them without
            // starting a webview.
            let panes = vec![PaneSpec {
                key: PaneKey::new(manifest.id.clone(), "main"),
                title: manifest.name.clone(),
            }];
            let has_settings = manifest.settings.is_some();
            (Tier::Web { entry }, panes, has_settings)
        }
    };

    Ok(Loaded {
        manifest,
        dir: dir.to_owned(),
        installation: Installation::read(dir),
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
    require_package_file(&dir, &manifest.icon_relative_path())?;
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
        installation: None,
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
    use chartr_plugin::Plugin as _;
    use gpui::{AppContext as _, ParentElement as _};

    struct BundledPlugin;

    impl chartr_plugin::Plugin for BundledPlugin {
        const ID: &'static str = "com.example.bundled";

        fn new(_: Host, _: &mut gpui::App) -> Self {
            Self
        }

        fn activate(&mut self, registrar: &mut Registrar, _: &mut gpui::App) {
            registrar.add_pane("main", "Bundled");
        }

        fn services(&self) -> Vec<chartr_plugin::services::ServiceExport> {
            vec![chartr_plugin::services::ServiceExport::new(42u32)]
        }

        fn view(
            &mut self,
            _: &PaneKey,
            _: &chartr_plugin::InstanceContext,
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
        Box::new(<BundledPlugin as chartr_plugin::Plugin>::new(host, cx))
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
            dir.join("chartr-plugin.toml"),
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

    fn write_dependent(paths: &Paths, id: &str, providers: &[&str]) {
        let dir = write_web(paths, id, id);
        let mut manifest = std::fs::read_to_string(dir.join("chartr-plugin.toml")).unwrap();
        for provider in providers {
            manifest.push_str(&format!(
                "\n[[dependencies]]\nplugin = '{provider}'\nfeature = 'Required feature'\n"
            ));
        }
        std::fs::write(dir.join("chartr-plugin.toml"), manifest).unwrap();
    }

    #[gpui::test]
    fn prerequisites_load_in_order_and_disabling_cascades_through_a_diamond(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_temp, paths) = paths();
        // Consumers sort before providers, exercising discovery order independence.
        write_dependent(&paths, "com.test.a", &["com.test.b", "com.test.c"]);
        write_dependent(&paths, "com.test.b", &["com.test.z"]);
        write_dependent(&paths, "com.test.c", &["com.test.z"]);
        write_dependent(&paths, "com.test.z", &[]);
        write_dependent(&paths, "com.test.unrelated", &[]);
        let mut catalog = cx.update(|cx| load_all(&paths, cx));
        assert_eq!(catalog.loaded.len(), 5);
        assert_eq!(catalog.dependents("com.test.z"), ["com.test.a", "com.test.b", "com.test.c"]);
        assert!(catalog.disable("com.test.z"));
        assert_eq!(catalog.loaded.len(), 1);
        assert!(catalog.get("com.test.unrelated").is_some());
        assert!(cx.update(|cx| catalog.enable(&paths, "com.test.a", cx)).is_err());
        assert!(catalog.disabled.contains_key("com.test.a"));
        cx.update(|cx| catalog.enable(&paths, "com.test.z", cx)).unwrap();
        assert_eq!(catalog.loaded.len(), 2, "consumers must not be enabled implicitly");
        cx.update(|cx| catalog.enable(&paths, "com.test.b", cx)).unwrap();
        assert!(cx.update(|cx| catalog.enable(&paths, "com.test.a", cx)).is_err());
        cx.update(|cx| catalog.enable(&paths, "com.test.c", cx)).unwrap();
        cx.update(|cx| catalog.enable(&paths, "com.test.a", cx)).unwrap();
        assert_eq!(catalog.loaded.len(), 5);
    }

    #[gpui::test]
    fn missing_disabled_and_cyclic_prerequisites_block_startup_and_enable(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_temp, paths) = paths();
        write_dependent(&paths, "com.test.missing", &["com.test.absent"]);
        write_dependent(&paths, "com.test.consumer", &["com.test.provider"]);
        write_dependent(&paths, "com.test.provider", &[]);
        write_dependent(&paths, "com.test.cycle_a", &["com.test.cycle_b"]);
        write_dependent(&paths, "com.test.cycle_b", &["com.test.cycle_a"]);
        let mut catalog =
            cx.update(|cx| load_all_where(&paths, |id| id != "com.test.provider", cx));
        assert!(catalog.loaded.is_empty());
        assert_eq!(catalog.disabled.len(), 5);
        assert!(catalog.rejected.is_empty(), "blocked plugins retain their single list entry");
        for id in ["com.test.missing", "com.test.consumer", "com.test.cycle_a", "com.test.cycle_b"]
        {
            assert!(catalog.prerequisite_error(id).is_some());
            assert!(cx.update(|cx| catalog.enable(&paths, id, cx)).is_err());
        }
        catalog.disabled.remove("com.test.provider");
        assert!(cx.update(|cx| catalog.enable(&paths, "com.test.provider", cx)).is_err());
        assert!(
            catalog.prerequisite_error("com.test.consumer").unwrap().contains("com.test.provider")
        );
        assert!(cx.update(|cx| catalog.enable(&paths, "com.test.consumer", cx)).is_err());
    }

    #[gpui::test]
    fn a_web_plugin_is_discovered_and_contributes_a_pane(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        write_web(&paths, "com.example.notes", "com.example.notes");

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert_eq!(catalog.loaded.len(), 1);
        assert_eq!(catalog.panes().len(), 1);
        assert_eq!(catalog.panes()[0].title, "Notes");
        assert!(catalog.get("com.example.notes").unwrap().installation.is_none());
    }

    #[gpui::test]
    fn disabled_packages_receive_the_same_validation_as_enabled_packages(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_temp, paths) = paths();
        write_web(&paths, "com.example.mismatch", "wrong-directory");
        let missing = write_web(&paths, "com.example.missing", "com.example.missing");
        std::fs::remove_file(missing.join("index.html")).unwrap();
        for enabled in [false, true] {
            let catalog = cx.update(|cx| load_all_where(&paths, |_| enabled, cx));
            assert!(catalog.loaded.is_empty() && catalog.disabled.is_empty());
            assert_eq!(catalog.rejected.len(), 2);
        }
    }

    #[gpui::test]
    fn package_entries_and_icons_must_stay_inside_the_package(cx: &mut gpui::TestAppContext) {
        let (temp, paths) = paths();
        let id = "com.example.notes";
        let dir = write_web(&paths, id, id);
        let original = std::fs::read_to_string(dir.join("chartr-plugin.toml")).unwrap();
        let outside = temp.path().join("outside.html");
        std::fs::write(&outside, "outside").unwrap();
        for entry in ["../outside.html".to_owned(), outside.display().to_string()] {
            std::fs::write(dir.join("chartr-plugin.toml"), original.replace("index.html", &entry))
                .unwrap();
            let manifest = Manifest::read(&dir).unwrap();
            assert!(matches!(validate_package(&dir, &manifest), Err(LoadError::EscapingFile(_))));
            let catalog = cx.update(|cx| load_all_where(&paths, |_| false, cx));
            assert_eq!(catalog.rejected.len(), 1);
        }
        std::fs::write(dir.join("chartr-plugin.toml"), &original).unwrap();
        for asset in ["index.html", "icons/NoteIcon.svg"] {
            let path = dir.join(asset);
            std::fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(&outside, &path).unwrap();
            let catalog = cx.update(|cx| load_all(&paths, cx));
            assert!(catalog.rejected[0].why.contains("escapes"));
            std::fs::remove_file(&path).unwrap();
            std::fs::write(&path, "restored").unwrap();
        }
        std::fs::write(
            dir.join("chartr-plugin.toml"),
            format!("{original}settings_entry = '../outside.html'\n"),
        )
        .unwrap();
        assert_eq!(cx.update(|cx| load_all(&paths, cx)).rejected.len(), 1);
    }

    #[gpui::test]
    fn portable_settings_are_discovered_without_an_html_document(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = write_web(&paths, "com.example.notes", "com.example.notes");
        let manifest = std::fs::read_to_string(dir.join("chartr-plugin.toml")).unwrap();
        std::fs::write(
            dir.join("chartr-plugin.toml"),
            format!(
                r#"{manifest}
[settings]
file = "settings.json"
[[settings.fields]]
key = "enabled"
label = "Enabled"
type = "toggle"
default = true
"#
            ),
        )
        .unwrap();

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.get("com.example.notes").unwrap().has_settings);
    }

    #[gpui::test]
    fn legacy_html_settings_are_rejected_with_a_migration_message(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = write_web(&paths, "com.example.notes", "com.example.notes");
        let manifest = std::fs::read_to_string(dir.join("chartr-plugin.toml")).unwrap();
        std::fs::write(
            dir.join("chartr-plugin.toml"),
            format!("{manifest}settings_entry = \"missing.html\"\n"),
        )
        .unwrap();

        let catalog = cx.update(|cx| load_all(&paths, cx));
        assert!(catalog.loaded.is_empty());
        assert!(catalog.rejected[0].why.contains("declare native fields"));
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
        std::fs::write(broken.join("chartr-plugin.toml"), "not toml {{{").expect("manifest");

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
        let consumer = catalog.services.clone();
        assert_eq!(*consumer.get::<u32>(BundledPlugin::ID).unwrap(), 42);
        assert!(catalog.disable(BundledPlugin::ID));
        assert!(consumer.get::<u32>(BundledPlugin::ID).is_none());
        cx.update(|cx| catalog.enable(&paths, BundledPlugin::ID, cx)).unwrap();
        assert_eq!(*consumer.get::<u32>(BundledPlugin::ID).unwrap(), 42);
        assert_eq!(catalog.panes()[0].key.plugin, BundledPlugin::ID);
    }

    #[gpui::test]
    fn a_hosted_browser_surface_is_discovered_without_loading_code(cx: &mut gpui::TestAppContext) {
        let (_tmp, paths) = paths();
        let dir = paths.installed.join("com.chartr.browser");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("chartr-plugin.toml"),
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
                dir.join("chartr-plugin.toml"),
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

    #[cfg(unix)]
    #[test]
    fn dangling_symlinks_cannot_create_files_outside_either_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let outside = temp.path().join("outside.txt");
        std::fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("escape")).unwrap();
        let broker = FileBroker::new(Some(root.clone()), root, ProjectAccess::ReadWrite, false);
        assert!(broker.project_path(Path::new("escape"), true).is_err());
        assert!(broker.data_path(Path::new("escape"), true).is_err());
        assert!(broker.write_project(Path::new("escape"), b"bad").is_err());
        assert!(broker.write_data(Path::new("escape"), b"bad").is_err());
        assert!(!outside.exists());
    }

    #[cfg(unix)]
    #[test]
    fn opening_rejects_symlinks_swapped_in_after_validation() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::create_dir(&outside).unwrap();
        let target = outside.join("note");
        std::fs::write(&target, "untouched").unwrap();
        let validated = contained(&root, Path::new("nested/note"), true).unwrap();
        std::os::unix::fs::symlink(&target, &validated).unwrap();
        assert!(write_contained(&root, &validated, b"bad").is_err());
        std::fs::rename(root.join("nested"), root.join("old")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("nested")).unwrap();
        assert!(write_contained(&root, &validated, b"bad").is_err());
        assert_eq!(std::fs::read_to_string(target).unwrap(), "untouched");
    }

    #[cfg(unix)]
    #[test]
    fn contained_links_and_normal_file_operations_still_work() {
        use std::io::Read as _;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        std::fs::create_dir_all(root.join("nested")).unwrap();
        let broker =
            FileBroker::new(Some(root.clone()), root.clone(), ProjectAccess::ReadWrite, false);
        broker.write_project(Path::new("nested/note"), b"long original").unwrap();
        std::os::unix::fs::symlink(root.join("nested/note"), root.join("link")).unwrap();
        broker.write_data(Path::new("link"), b"new").unwrap();
        let mut text = String::new();
        broker.open_project(Path::new("link")).unwrap().read_to_string(&mut text).unwrap();
        assert_eq!(text, "new");
        assert!(broker.write_project(Path::new("nested"), b"bad").is_err());
        let read_only = FileBroker::new(Some(root.clone()), root, ProjectAccess::Read, false);
        assert!(read_only.write_project(Path::new("link"), b"bad").is_err());
    }
    #[test]
    fn whole_file_writes_are_atomic_for_readers_and_concurrent_instances() {
        use std::sync::{Arc, Barrier};
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path().to_owned();
        let broker =
            FileBroker::new(Some(root.clone()), root.clone(), ProjectAccess::ReadWrite, false);
        let first = vec![b'a'; 64 * 1024];
        let second = vec![b'b'; 17 * 1024];
        broker.write_data(Path::new("state"), &first).unwrap();
        // An already-open reader must retain the old complete file after replacement.
        let mut reader = broker.open_data(Path::new("state")).unwrap();
        broker.write_data(Path::new("state"), &second).unwrap();
        let mut old = Vec::new();
        std::io::Read::read_to_end(&mut reader, &mut old).unwrap();
        assert_eq!(old, first);
        let barrier = Arc::new(Barrier::new(3));
        std::thread::scope(|scope| {
            for value in [&first, &second] {
                let broker = broker.clone();
                let barrier = barrier.clone();
                scope.spawn(move || {
                    barrier.wait();
                    for _ in 0..20 {
                        broker.write_data(Path::new("state"), value).unwrap();
                    }
                });
            }
            barrier.wait();
            for _ in 0..100 {
                let bytes = std::fs::read(root.join("state")).unwrap();
                assert!(
                    bytes == first || bytes == second,
                    "reader observed a partial/interleaved write"
                );
            }
        });
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    }

    #[test]
    fn failed_atomic_writes_leave_existing_destinations_untouched() {
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path();
        std::fs::create_dir(root.join("directory")).unwrap();
        std::fs::write(root.join("directory/keep"), "keep").unwrap();
        let broker = FileBroker::new(
            Some(root.to_owned()),
            root.to_owned(),
            ProjectAccess::ReadWrite,
            false,
        );
        assert!(broker.write_project(Path::new("directory"), b"replacement").is_err());
        assert_eq!(std::fs::read_to_string(root.join("directory/keep")).unwrap(), "keep");
        let read_only =
            FileBroker::new(Some(root.to_owned()), root.to_owned(), ProjectAccess::Read, false);
        assert!(read_only.write_project(Path::new("directory/keep"), b"replacement").is_err());
        assert_eq!(std::fs::read_to_string(root.join("directory/keep")).unwrap(), "keep");
    }
}
