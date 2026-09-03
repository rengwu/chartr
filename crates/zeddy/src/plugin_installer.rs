//! Staged plugin installation from a Git repository or local directory.
//!
//! Discovery remains in `zeddy-plugin-host`: this module performs the
//! deliberately separate, user-initiated mutation. Packages are copied into a
//! same-filesystem staging directory, validated, and only then renamed into
//! the managed plugin root. Installation never compiles or executes package
//! code. Separately installed packages are web content or explicit
//! Chartr-hosted surfaces; Rust/GPUI dylibs are rejected at this boundary.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context as _, Result, anyhow, bail};
use tempfile::TempDir;
use zeddy_plugin::{Kind, Manifest};
use zeddy_plugin_host::{HostedSurface, Paths};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Git(String),
    Local(PathBuf),
}

impl Source {
    pub fn label(&self) -> String {
        match self {
            Self::Git(url) => url.clone(),
            Self::Local(path) => path.display().to_string(),
        }
    }
}

/// A source whose manifest is safe to describe in a confirmation prompt.
/// No native code has been executed or loaded by Chartr.
#[derive(Debug)]
pub struct Prepared {
    temp: TempDir,
    pub source: Source,
    pub manifest: Manifest,
    pub replacing: bool,
}

impl Prepared {
    pub fn trust_detail(&self) -> String {
        let replacement = if self.replacing {
            " This replaces the installed copy and preserves its plugin data."
        } else {
            ""
        };
        match self.manifest.kind {
            Kind::Native => unreachable!("separately installed native plugins are rejected"),
            Kind::Hosted => format!(
                "Source: {}. This package activates Chartr's built-in `{}` surface and contains no executable plugin code.{}",
                self.source.label(),
                self.manifest.surface.as_deref().unwrap_or("unknown"),
                replacement
            ),
            Kind::Web => {
                let permissions = &self.manifest.permissions;
                let project = match permissions.project_files {
                    zeddy_plugin::ProjectAccess::None => "no project files",
                    zeddy_plugin::ProjectAccess::Read => "read project files",
                    zeddy_plugin::ProjectAccess::ReadWrite => "read and write project files",
                };
                let mut grants = vec![project.to_owned()];
                if !permissions.network.is_empty() {
                    grants.push(format!("network: {}", permissions.network.join(", ")));
                }
                if permissions.process {
                    grants.push("process execution".into());
                }
                if permissions.session {
                    grants.push("bound-session access".into());
                }
                format!(
                    "Source: {}. Declared access: {}.{}",
                    self.source.label(),
                    grants.join("; "),
                    replacement
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    pub id: String,
    pub name: String,
    pub replaced: bool,
}

/// Fetch or copy a package and inspect its manifest before the trust prompt.
/// No source compilation or package script occurs.
pub fn prepare(source: Source, paths: &Paths) -> Result<Prepared> {
    fs::create_dir_all(&paths.installed)
        .with_context(|| format!("creating {}", paths.installed.display()))?;
    let staging_root = paths
        .installed
        .parent()
        .context("the plugin directory has no parent")?
        .join("plugin-staging");
    fs::create_dir_all(&staging_root)
        .with_context(|| format!("creating {}", staging_root.display()))?;
    let temp = tempfile::Builder::new()
        .prefix("install-")
        .tempdir_in(staging_root)
        .context("creating plugin staging directory")?;
    let package = temp.path().join("package");
    match &source {
        Source::Git(url) => {
            let checkout = temp.path().join("checkout");
            clone_git(url, &checkout)?;
            copy_tree(&checkout, &package)?;
        }
        Source::Local(path) => {
            let path =
                path.canonicalize().with_context(|| format!("opening {}", path.display()))?;
            if !path.is_dir() {
                bail!("{} is not a directory", path.display());
            }
            let staging =
                temp.path().canonicalize().context("resolving plugin staging directory")?;
            if staging.starts_with(&path) {
                bail!("the selected folder contains Chartr's managed plugin staging directory");
            }
            copy_tree(&path, &package)?;
        }
    }

    let manifest = Manifest::read(&package).map_err(|error| anyhow!(error))?;
    if manifest.kind == Kind::Native {
        bail!(
            "separately installed native GPUI libraries are not supported because Rust GUI objects are not safe across a dynamic-library boundary. Use a web package or a Chartr-hosted surface; installation never compiles plugin source"
        );
    }
    if manifest.kind == Kind::Hosted {
        HostedSurface::named(manifest.surface.as_deref().unwrap_or_default())
            .map_err(|error| anyhow!(error))?;
    }
    validate_declared_files(&package, &manifest)?;
    let replacing = paths.installed.join(&manifest.id).exists();
    Ok(Prepared { temp, source, manifest, replacing })
}

/// Validate and atomically place a prepared plugin without executing build
/// tools or plugin code.
pub fn install(prepared: Prepared, paths: &Paths) -> Result<Installed> {
    let Prepared { temp, source: _, manifest, replacing } = prepared;
    let package = temp.path().join("package");
    let packaged_manifest = Manifest::read(&package).map_err(|error| anyhow!(error))?;
    if packaged_manifest != manifest {
        bail!("the plugin manifest changed after the install confirmation");
    }

    validate_declared_files(&package, &manifest)?;

    let destination = paths.installed.join(&manifest.id);
    let backup = temp.path().join("previous");
    if destination.exists() {
        fs::rename(&destination, &backup)
            .with_context(|| format!("staging the previous {}", manifest.name))?;
    }
    if let Err(error) = fs::rename(&package, &destination) {
        if backup.exists() {
            let _ = fs::rename(&backup, &destination);
        }
        return Err(error).with_context(|| format!("installing {}", manifest.name));
    }

    Ok(Installed { id: manifest.id, name: manifest.name, replaced: replacing })
}

fn clone_git(url: &str, destination: &Path) -> Result<()> {
    let url = url.trim();
    if url.is_empty() {
        bail!("enter a Git repository URL");
    }
    let output = Command::new("git")
        .args([OsStr::new("clone"), OsStr::new("--depth"), OsStr::new("1"), OsStr::new("--")])
        .arg(url)
        .arg(destination)
        .output()
        .context("starting Git; install Git and make sure it is available in PATH")?;
    if !output.status.success() {
        bail!("Git clone failed: {}", command_detail(&output));
    }
    Ok(())
}

fn validate_declared_files(root: &Path, manifest: &Manifest) -> Result<()> {
    match manifest.kind {
        Kind::Native => {}
        Kind::Hosted => {}
        Kind::Web => {
            let entry = manifest.entry.as_deref().context("missing web entry")?;
            require_relative_file(root, entry, "web entry")?;
            if let Some(settings) = &manifest.settings_entry {
                require_relative_file(root, settings, "settings entry")?;
            }
        }
    }
    Ok(())
}

fn require_relative_file(root: &Path, relative: &str, label: &str) -> Result<()> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        bail!("{label} escapes the plugin directory");
    }
    let file = root.join(relative);
    if !file.is_file() {
        bail!("{label} `{}` is missing", relative.display());
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("creating {}", destination.display()))?;
    for entry in fs::read_dir(source).with_context(|| format!("reading {}", source.display()))? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let kind = entry.file_type()?;
        let from = entry.path();
        let to = destination.join(&name);
        if kind.is_symlink() {
            bail!("plugin source contains unsupported symbolic link {}", from.display());
        } else if kind.is_dir() {
            copy_tree(&from, &to)?;
        } else if kind.is_file() {
            fs::copy(&from, &to).with_context(|| format!("copying {}", from.display()))?;
        }
    }
    Ok(())
}

fn command_detail(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
    const LIMIT: usize = 4_000;
    if detail.len() <= LIMIT {
        detail.to_owned()
    } else {
        let mut tail: String = detail.chars().rev().take(LIMIT).collect();
        tail = tail.chars().rev().collect();
        format!("…{tail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(temp: &TempDir) -> Paths {
        Paths::under(temp.path().join("chartr"))
    }

    fn write(path: impl AsRef<Path>, contents: &str) {
        let path = path.as_ref();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn a_local_web_plugin_is_staged_validated_and_installed() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("notes-source");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.notes'\nname = 'Notes'\nversion = '1'\nkind = 'web'\nentry = 'index.html'\n",
        );
        write(source.join("index.html"), "<h1>Notes</h1>");
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        assert!(!prepared.replacing);
        let installed = install(prepared, &paths).unwrap();

        assert_eq!(installed.id, "com.example.notes");
        assert!(paths.installed.join("com.example.notes/index.html").is_file());
    }

    #[test]
    fn confirmation_applies_to_the_staged_bytes_not_later_source_changes() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("notes-source");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.notes'\nname = 'Notes'\nversion = '1'\nkind = 'web'\nentry = 'index.html'\n",
        );
        write(source.join("index.html"), "confirmed");
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source.clone()), &paths).unwrap();
        write(source.join("index.html"), "changed after confirmation");
        install(prepared, &paths).unwrap();

        assert_eq!(
            fs::read_to_string(paths.installed.join("com.example.notes/index.html")).unwrap(),
            "confirmed"
        );
    }

    #[test]
    fn replacement_is_atomic_and_does_not_touch_plugin_data() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let old = paths.installed.join("com.example.notes");
        write(old.join("old.txt"), "old");
        write(paths.data.join("com.example.notes/cookie"), "session");
        let source = temp.path().join("new-source");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.notes'\nname = 'Notes'\nversion = '2'\nkind = 'web'\nentry = 'index.html'\n",
        );
        write(source.join("index.html"), "new");

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        assert!(prepared.replacing);
        assert!(install(prepared, &paths).unwrap().replaced);

        assert!(!old.join("old.txt").exists());
        assert_eq!(fs::read_to_string(old.join("index.html")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(paths.data.join("com.example.notes/cookie")).unwrap(),
            "session"
        );
    }

    #[test]
    fn web_entries_cannot_escape_the_plugin() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("bad");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.bad'\nname = 'Bad'\nversion = '1'\nkind = 'web'\nentry = '../outside.html'\n",
        );
        write(temp.path().join("outside.html"), "outside");
        let error = prepare(Source::Local(source), &paths(&temp)).unwrap_err();
        assert!(error.to_string().contains("escapes"));
    }

    #[test]
    fn a_default_branch_git_repository_installs() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("repository");
        fs::create_dir_all(&source).unwrap();
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.git'\nname = 'Git plugin'\nversion = '1'\nkind = 'web'\nentry = 'index.html'\n",
        );
        write(source.join("index.html"), "from git");
        let run_git = |args: &[&str]| {
            let output = Command::new("git").args(args).current_dir(&source).output().unwrap();
            assert!(output.status.success(), "{}", command_detail(&output));
        };
        run_git(&["init", "-q", "-b", "main"]);
        run_git(&["add", "."]);
        run_git(&[
            "-c",
            "user.name=Chartr Test",
            "-c",
            "user.email=chartr@example.invalid",
            "commit",
            "-q",
            "-m",
            "plugin",
        ]);
        let paths = paths(&temp);

        let prepared = prepare(Source::Git(source.to_string_lossy().into_owned()), &paths).unwrap();
        install(prepared, &paths).unwrap();

        assert_eq!(
            fs::read_to_string(paths.installed.join("com.example.git/index.html")).unwrap(),
            "from git"
        );
    }

    #[test]
    fn a_separately_compiled_native_gpui_plugin_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("native");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.native'\nname = 'Native'\nversion = '1'\nkind = 'native'\n",
        );
        write(source.join("libexample.dylib"), "precompiled artifact");
        let error = prepare(Source::Local(source), &paths(&temp)).unwrap_err();

        assert!(error.to_string().contains("never compiles"));
        assert!(error.to_string().contains("not safe across a dynamic-library boundary"));
    }

    #[gpui::test]
    fn browser_package_installs_and_loads_through_the_real_installer(
        cx: &mut gpui::TestAppContext,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/browser");
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        assert_eq!(prepared.manifest.kind, Kind::Hosted);
        assert!(prepared.trust_detail().contains("no executable plugin code"));
        let installed = install(prepared, &paths).unwrap();
        let manifest = Manifest::read(&paths.installed.join(&installed.id)).unwrap();

        assert_eq!(installed.id, "com.chartr.browser");
        assert_eq!(manifest.surface.as_deref(), Some("browser"));
        assert!(!paths.installed.join(&installed.id).join("Cargo.toml").exists());

        let catalog = cx.update(|cx| zeddy_plugin_host::load_all(&paths, cx));
        assert!(catalog.rejected.is_empty(), "Browser was rejected: {:?}", catalog.rejected);
        assert_eq!(catalog.panes()[0].key.plugin, "com.chartr.browser");
        assert_eq!(catalog.panes()[0].title, "Browser");
    }

    #[test]
    fn package_scripts_are_copied_but_never_executed() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("with-script");
        let marker = temp.path().join("script-ran");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.script'\nname = 'Script'\nversion = '1'\nkind = 'web'\nentry = 'index.html'\n",
        );
        write(source.join("index.html"), "ok");
        write(source.join("install.sh"), &format!("touch '{}'", marker.display()));
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        install(prepared, &paths).unwrap();

        assert!(!marker.exists());
        assert!(paths.installed.join("com.example.script/install.sh").is_file());
    }

    #[test]
    fn a_local_source_cannot_contain_the_managed_staging_directory() {
        let temp = tempfile::tempdir().unwrap();
        write(
            temp.path().join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.recursive'\nname = 'Recursive'\nversion = '1'\nkind = 'web'\nentry = 'index.html'\n",
        );
        write(temp.path().join("index.html"), "ok");
        let error = prepare(Source::Local(temp.path().to_owned()), &paths(&temp)).unwrap_err();

        assert!(error.to_string().contains("staging directory"));
    }

    #[test]
    fn an_unknown_hosted_surface_is_rejected_before_confirmation() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("unknown");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.unknown'\nname = 'Unknown'\nversion = '1'\nkind = 'hosted'\nsurface = 'unknown'\n",
        );

        let error = prepare(Source::Local(source), &paths(&temp)).unwrap_err();
        assert!(error.to_string().contains("does not support"));
    }
}
