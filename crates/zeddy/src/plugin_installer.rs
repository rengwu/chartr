//! Staged plugin installation from a Git repository or local directory.
//!
//! Discovery remains in `zeddy-plugin-host`: this module performs the
//! deliberately separate, user-initiated mutation. Packages are copied into a
//! same-filesystem staging directory, validated, and only then renamed into
//! pending plugin root. Startup activates them before any catalog is loaded.
//! Installation never compiles or executes package
//! code. Separately installed packages are web content or explicit
//! Chartr-hosted surfaces; Rust/GPUI dylibs are rejected at this boundary.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use anyhow::{Context as _, Result, anyhow, bail};
use tempfile::TempDir;
use zeddy_plugin::{Kind, Manifest};
use zeddy_plugin_host::{Installation, Paths, validate_package};

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
    pub commit: Option<String>,
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
                format!(
                    "Source: {}. Declared host access: {}.{}",
                    self.source.label(),
                    self.manifest.permissions.summary(),
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
    prepare_cancellable(source, paths, &AtomicBool::new(false))
}

pub fn prepare_cancellable(source: Source, paths: &Paths, cancel: &AtomicBool) -> Result<Prepared> {
    check_cancelled(cancel)?;
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
    let mut commit = None;
    match &source {
        Source::Git(url) => {
            let checkout = temp.path().join("checkout");
            commit = Some(clone_git(url, &checkout, cancel)?);
            copy_tree(&checkout, &package, cancel)?;
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
            copy_tree(&path, &package, cancel)?;
        }
    }

    check_cancelled(cancel)?;
    let manifest = Manifest::read(&package).map_err(|error| anyhow!(error))?;
    validate_package(&package, &manifest)?;
    let replacing = paths.installed.join(&manifest.id).exists()
        || pending_root(paths).join(&manifest.id).exists();
    Ok(Prepared { temp, source, commit, manifest, replacing })
}

/// Validate and atomically queue a prepared plugin for the next startup without
/// changing any package used by the running catalog.
pub fn install(prepared: Prepared, paths: &Paths) -> Result<Installed> {
    let Prepared { temp, source, commit, manifest, replacing } = prepared;
    let package = temp.path().join("package");
    let packaged_manifest = Manifest::read(&package).map_err(|error| anyhow!(error))?;
    if packaged_manifest != manifest {
        bail!("the plugin manifest changed after the install confirmation");
    }

    validate_package(&package, &manifest)?;
    // Always overwrite package-supplied metadata. Activation copies this record
    // unchanged, rather than recording the pending directory as a new source.
    let installation = Installation { source: source.label(), commit };
    fs::write(package.join(Installation::FILE), serde_json::to_vec_pretty(&installation)?)
        .context("recording the plugin installation source")?;

    // The live catalog and all of its assets remain untouched until startup.
    let pending = pending_root(paths);
    fs::create_dir_all(&pending)?;
    replace_directory(&package, &pending.join(&manifest.id))?;
    Ok(Installed { id: manifest.id, name: manifest.name, replaced: replacing })
}

fn pending_root(paths: &Paths) -> PathBuf {
    paths.installed.with_file_name("plugin-pending")
}

pub fn has_pending(paths: &Paths) -> bool {
    fs::read_dir(pending_root(paths)).is_ok_and(|mut entries| entries.next().is_some())
}

/// The caller closes and disables the plugin before removing its managed copies.
/// Remove pending updates first so a later startup cannot resurrect the plugin.
/// Source repositories, plugin data, and preferences are preserved.
pub fn uninstall(id: &str, paths: &Paths) -> Result<()> {
    let mut components = Path::new(id).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        bail!("invalid plugin directory name");
    }
    for directory in [pending_root(paths).join(id), paths.installed.join(id)] {
        match fs::remove_dir_all(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("removing {}", directory.display()));
            }
        }
    }
    Ok(())
}

/// Activate only at process startup, before constructing any catalog or pane.
/// Pending packages remain intact until success, so interruption is retryable.
/// Reapplying the same package after a crash is harmless.
pub fn activate_pending(paths: &Paths) -> Vec<anyhow::Error> {
    let entries = match fs::read_dir(pending_root(paths)) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => return vec![error.into()],
    };
    let mut errors = Vec::new();
    for entry in entries {
        let result = (|| -> Result<()> {
            let source = entry?.path();
            let prepared = prepare(Source::Local(source.clone()), paths)?;
            if source.file_name() != Some(OsStr::new(&prepared.manifest.id)) {
                bail!("pending package directory does not match its plugin id");
            }
            let destination = paths.installed.join(&prepared.manifest.id);
            replace_directory(&prepared.temp.path().join("package"), &destination)?;
            fs::remove_dir_all(source).context("removing the activated pending package")?;
            Ok(())
        })();
        if let Err(error) = result {
            errors.push(error);
        }
    }
    errors
}

/// On the supported macOS/Linux targets a directory exchange is one atomic
/// operation. Failure leaves both directories intact; no rollback is needed.
fn replace_directory(package: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        use rustix::fs::{CWD, RenameFlags, renameat_with};
        renameat_with(CWD, package, CWD, destination, RenameFlags::EXCHANGE)
            .with_context(|| format!("atomically replacing {}", destination.display()))?;
    } else {
        fs::rename(package, destination)
            .with_context(|| format!("installing {}", destination.display()))?;
    }
    Ok(())
}

fn check_cancelled(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        bail!("Plugin installation cancelled");
    }
    Ok(())
}

fn clone_git(url: &str, destination: &Path, cancel: &AtomicBool) -> Result<String> {
    let url = url.trim();
    if url.is_empty() {
        bail!("enter a Git repository URL");
    }
    let output = crate::process::output(
        Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .args([OsStr::new("clone"), OsStr::new("--depth"), OsStr::new("1"), OsStr::new("--")])
            .arg(url)
            .arg(destination),
        Duration::from_secs(120),
        1024 * 1024,
        cancel,
    )
    .context("running Git (2 minute limit, interactive terminal prompts disabled)")?;
    if !output.status.success() {
        bail!(
            "Git clone failed: {}. Check the repository URL and your Git credentials.",
            command_detail(&output)
        );
    }
    let revision = crate::process::output(
        Command::new("git").arg("-C").arg(destination).args(["rev-parse", "HEAD"]),
        Duration::from_secs(10),
        4096,
        cancel,
    )
    .context("reading the installed Git commit")?;
    if !revision.status.success() {
        bail!("reading the installed Git commit: {}", command_detail(&revision));
    }
    Ok(String::from_utf8(revision.stdout)?.trim().to_owned())
}

fn copy_tree(source: &Path, destination: &Path, cancel: &AtomicBool) -> Result<()> {
    check_cancelled(cancel)?;
    fs::create_dir_all(destination)
        .with_context(|| format!("creating {}", destination.display()))?;
    for entry in fs::read_dir(source).with_context(|| format!("reading {}", source.display()))? {
        check_cancelled(cancel)?;
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
            copy_tree(&from, &to, cancel)?;
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
            "manifest_version = 2\nid = 'com.example.notes'\nname = 'Notes'\nversion = '1'\nkind = 'web'\nicon = 'NoteIcon'\nentry = 'index.html'\n",
        );
        write(source.join("icons/NoteIcon.svg"), "<svg/>");
        write(source.join("index.html"), "<h1>Notes</h1>");
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        assert!(!prepared.replacing);
        let installed = install(prepared, &paths).unwrap();
        assert!(activate_pending(&paths).is_empty());

        assert_eq!(installed.id, "com.example.notes");
        assert!(paths.installed.join("com.example.notes/index.html").is_file());
        assert!(paths.installed.join("com.example.notes/icons/NoteIcon.svg").is_file());
    }

    #[test]
    fn confirmation_applies_to_the_staged_bytes_not_later_source_changes() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("notes-source");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.notes'\nname = 'Notes'\nversion = '1'\nkind = 'web'\nicon = 'NoteIcon'\nentry = 'index.html'\n",
        );
        write(source.join("icons/NoteIcon.svg"), "<svg/>");
        write(source.join("index.html"), "confirmed");
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source.clone()), &paths).unwrap();
        write(source.join("index.html"), "changed after confirmation");
        install(prepared, &paths).unwrap();
        assert!(activate_pending(&paths).is_empty());

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
            "manifest_version = 2\nid = 'com.example.notes'\nname = 'Notes'\nversion = '2'\nkind = 'web'\nicon = 'NoteIcon'\nentry = 'index.html'\n",
        );
        write(source.join("icons/NoteIcon.svg"), "<svg/>");
        write(source.join("index.html"), "new");

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        assert!(prepared.replacing);
        assert!(install(prepared, &paths).unwrap().replaced);
        assert!(old.join("old.txt").exists(), "upgrade must wait for restart");
        assert!(activate_pending(&paths).is_empty());

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
            "manifest_version = 2\nid = 'com.example.bad'\nname = 'Bad'\nversion = '1'\nkind = 'web'\nicon = 'TestIcon'\nentry = '../outside.html'\n",
        );
        write(source.join("icons/TestIcon.svg"), "<svg/>");
        write(temp.path().join("outside.html"), "outside");
        let error = prepare(Source::Local(source), &paths(&temp)).unwrap_err();
        assert!(error.to_string().contains("escapes"));
    }

    #[test]
    fn a_manifest_selected_hugeicon_must_be_packaged() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("missing-icon");
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.icon'\nname = 'Icon'\nversion = '1'\nkind = 'web'\nicon = 'MissingIcon'\nentry = 'index.html'\n",
        );
        write(source.join("index.html"), "ok");

        let error = prepare(Source::Local(source), &paths(&temp)).unwrap_err();
        assert!(error.to_string().contains("icons/MissingIcon.svg"));
    }

    #[test]
    fn a_default_branch_git_repository_installs() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("repository");
        fs::create_dir_all(&source).unwrap();
        write(
            source.join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.git'\nname = 'Git plugin'\nversion = '1'\nkind = 'web'\nicon = 'TestIcon'\nentry = 'index.html'\n",
        );
        write(source.join("icons/TestIcon.svg"), "<svg/>");
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
        let head =
            Command::new("git").args(["rev-parse", "HEAD"]).current_dir(&source).output().unwrap();
        let commit = String::from_utf8(head.stdout).unwrap().trim().to_owned();
        assert_eq!(prepared.commit.as_deref(), Some(commit.as_str()));
        install(prepared, &paths).unwrap();
        assert!(activate_pending(&paths).is_empty());

        let record: Installation = serde_json::from_slice(
            &fs::read(paths.installed.join("com.example.git").join(Installation::FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(record.source, source.to_string_lossy());
        assert_eq!(record.commit, Some(commit));

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
            "manifest_version = 2\nid = 'com.example.native'\nname = 'Native'\nversion = '1'\nkind = 'native'\nicon = 'TestIcon'\n",
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
        assert!(activate_pending(&paths).is_empty());
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
            "manifest_version = 2\nid = 'com.example.script'\nname = 'Script'\nversion = '1'\nkind = 'web'\nicon = 'TestIcon'\nentry = 'index.html'\n",
        );
        write(source.join("icons/TestIcon.svg"), "<svg/>");
        write(source.join("index.html"), "ok");
        write(source.join("install.sh"), &format!("touch '{}'", marker.display()));
        let paths = paths(&temp);

        let prepared = prepare(Source::Local(source), &paths).unwrap();
        install(prepared, &paths).unwrap();
        assert!(activate_pending(&paths).is_empty());

        assert!(!marker.exists());
        assert!(paths.installed.join("com.example.script/install.sh").is_file());
    }

    #[test]
    fn a_local_source_cannot_contain_the_managed_staging_directory() {
        let temp = tempfile::tempdir().unwrap();
        write(
            temp.path().join("zeddy-plugin.toml"),
            "manifest_version = 2\nid = 'com.example.recursive'\nname = 'Recursive'\nversion = '1'\nkind = 'web'\nicon = 'TestIcon'\nentry = 'index.html'\n",
        );
        write(temp.path().join("icons/TestIcon.svg"), "<svg/>");
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
            "manifest_version = 2\nid = 'com.example.unknown'\nname = 'Unknown'\nversion = '1'\nkind = 'hosted'\nicon = 'TestIcon'\nsurface = 'unknown'\n",
        );

        let error = prepare(Source::Local(source), &paths(&temp)).unwrap_err();
        assert!(error.to_string().contains("does not support"));
    }
    fn package(root: &Path, version: &str) {
        write(
            root.join("zeddy-plugin.toml"),
            &format!(
                "manifest_version = 2\nid = 'com.example.upgrade'\nname = 'Upgrade'\nversion = '{version}'\nkind = 'web'\nicon = 'TestIcon'\nentry = 'index.html'\n"
            ),
        );
        write(root.join("icons/TestIcon.svg"), "<svg/>");
        write(root.join("index.html"), version);
    }

    #[gpui::test]
    fn recorded_source_survives_activation_and_enable_cycles(cx: &mut gpui::TestAppContext) {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let source = temp.path().join("source");
        package(&source, "1");
        write(
            source.join(Installation::FILE),
            r#"{"source":"not the real source","commit":"fake"}"#,
        );
        install(prepare(Source::Local(source.clone()), &paths).unwrap(), &paths).unwrap();
        assert!(has_pending(&paths));
        assert!(activate_pending(&paths).is_empty());
        assert!(!has_pending(&paths));
        let expected = Some(Installation { source: source.display().to_string(), commit: None });
        let mut catalog = cx.update(|cx| zeddy_plugin_host::load_all_where(&paths, |_| false, cx));
        let id = "com.example.upgrade";
        assert_eq!(catalog.disabled[id].installation, expected);
        cx.update(|cx| catalog.enable(&paths, id, cx)).unwrap();
        assert_eq!(catalog.get(id).unwrap().installation, expected);
        catalog.disable(id);
        assert_eq!(catalog.disabled[id].installation, expected);
    }

    #[test]
    fn uninstall_removes_installed_and_pending_packages_but_keeps_data_and_sources() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let id = "com.example.upgrade";
        let source = temp.path().join("source");
        package(&source, "old");
        install(prepare(Source::Local(source.clone()), &paths).unwrap(), &paths).unwrap();
        assert!(activate_pending(&paths).is_empty());
        package(&source, "new");
        install(prepare(Source::Local(source.clone()), &paths).unwrap(), &paths).unwrap();
        let data = paths.data.join(id).join("state.json");
        write(&data, "saved state");
        package(&paths.bundled.join(id), "bundled");

        uninstall(id, &paths).unwrap();
        assert!(!paths.installed.join(id).exists());
        assert!(!pending_root(&paths).join(id).exists());
        assert!(activate_pending(&paths).is_empty());
        assert_eq!(fs::read_to_string(data).unwrap(), "saved state");
        assert_eq!(fs::read_to_string(source.join("index.html")).unwrap(), "new");
        assert!(paths.bundled.join(id).exists());
        uninstall(id, &paths).unwrap();
    }

    #[test]
    fn failed_pending_removal_keeps_the_installed_package_and_invalid_ids_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let id = "com.example.upgrade";
        let live = paths.installed.join(id);
        package(&live, "old");
        write(pending_root(&paths).join(id), "not a package directory");
        assert!(uninstall(id, &paths).is_err());
        for id in ["", ".", "..", "../plugin-data", "/tmp", "a/b"] {
            assert!(uninstall(id, &paths).is_err(), "accepted {id:?}");
        }
        assert_eq!(fs::read_to_string(live.join("index.html")).unwrap(), "old");
    }

    #[gpui::test]
    fn upgrades_and_reenable_keep_live_code_until_the_next_startup(cx: &mut gpui::TestAppContext) {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let id = "com.example.upgrade";
        let live = paths.installed.join(id);
        package(&live, "old");
        let mut catalog = cx.update(|cx| zeddy_plugin_host::load_all(&paths, cx));
        let source = temp.path().join("source");
        for version in ["new", "newest"] {
            package(&source, version);
            install(prepare(Source::Local(source.clone()), &paths).unwrap(), &paths).unwrap();
            assert_eq!(fs::read_to_string(live.join("index.html")).unwrap(), "old");
            assert!(catalog.disable(id));
            cx.update(|cx| catalog.enable(&paths, id, cx)).unwrap();
            assert_eq!(catalog.get(id).unwrap().manifest.version, "old");
        }
        assert!(activate_pending(&paths).is_empty());
        assert_eq!(fs::read_to_string(live.join("index.html")).unwrap(), "newest");
        assert!(!pending_root(&paths).join(id).exists());
        assert!(activate_pending(&paths).is_empty());
    }

    #[test]
    fn interrupted_activation_can_be_retried_without_reverting_the_upgrade() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let live = paths.installed.join("com.example.upgrade");
        package(&live, "old");
        let pending = pending_root(&paths).join("com.example.upgrade");
        package(&pending, "new");
        // Model interruption immediately after exchange, before pending cleanup.
        let prepared = prepare(Source::Local(pending.clone()), &paths).unwrap();
        replace_directory(&prepared.temp.path().join("package"), &live).unwrap();
        assert_eq!(fs::read_to_string(live.join("index.html")).unwrap(), "new");
        assert!(pending.exists());
        assert!(activate_pending(&paths).is_empty());
        assert_eq!(fs::read_to_string(live.join("index.html")).unwrap(), "new");
    }

    #[test]
    fn failed_exchange_and_invalid_pending_packages_preserve_the_installed_copy() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let live = paths.installed.join("com.example.upgrade");
        package(&live, "old");
        assert!(replace_directory(&temp.path().join("missing"), &live).is_err());
        let pending = pending_root(&paths).join("com.example.upgrade");
        package(&pending, "new");
        fs::remove_file(pending.join("index.html")).unwrap();
        assert_eq!(activate_pending(&paths).len(), 1);
        assert!(pending.exists(), "failed updates must remain available for recovery");
        assert_eq!(fs::read_to_string(live.join("index.html")).unwrap(), "old");
    }

    #[test]
    fn cancelled_preparation_never_creates_an_install() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(&temp);
        let source = temp.path().join("source");
        package(&source, "new");
        let error =
            prepare_cancellable(Source::Local(source), &paths, &AtomicBool::new(true)).unwrap_err();
        assert!(error.to_string().contains("cancelled"));
        assert!(!paths.installed.exists());
        assert!(!pending_root(&paths).exists());
    }
}
