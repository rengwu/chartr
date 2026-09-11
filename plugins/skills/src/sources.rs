//! Ordered, plugin-owned skill sources. Filesystem and Git work run off the UI thread.
use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::Read as _,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Kind {
    #[default]
    #[serde(rename = "dir")]
    Local,
    #[serde(rename = "git")]
    Git,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Folder on this machine",
            Self::Git => "Git repository",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Source {
    pub name: String,
    pub kind: Kind,
    pub path: PathBuf,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub git_ref: String,
    #[serde(default)]
    pub commit: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Skill {
    pub name: String,
    pub dir: PathBuf,
    pub shadowed: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct State {
    pub skills: Vec<Skill>,
    pub unavailable: bool,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
pub(super) struct Store {
    pub root: PathBuf,
    pub sources: Vec<Source>,
}

pub(super) enum Operation {
    Scan,
    Save { source: Source, editing: Option<String> },
    Remove(String),
    Enable(String, bool),
    Move { name: String, before: String },
    Refresh(String),
}

impl Store {
    pub fn catalog(&self) -> Result<chartr_plugin::services::SkillCatalog> {
        use chartr_plugin::services::{Skill as ResolvedSkill, SkillCatalog};
        let mut catalog = SkillCatalog::default();
        for (source, state) in self.sources.iter().zip(self.states()) {
            if !source.enabled {
                continue;
            }
            if state.unavailable {
                catalog.warnings.push(format!("{} is unavailable.", source.name));
            }
            catalog
                .warnings
                .extend(state.warnings.iter().map(|warning| format!("{}: {warning}", source.name)));
            for skill in state.skills {
                let path = skill.dir.join("SKILL.md");
                let read = (|| -> std::io::Result<String> {
                    let file = fs::File::open(&path)?;
                    if !file.metadata()?.is_file() {
                        return Err(std::io::Error::other("Not a regular skill file"));
                    }
                    let mut body = String::new();
                    file.take(256 * 1024 + 1).read_to_string(&mut body)?;
                    Ok(body)
                })();
                match read {
                    Ok(body) if body.len() <= 256 * 1024 => catalog.skills.push(ResolvedSkill {
                        source: source.name.clone(),
                        name: skill.name,
                        directory: skill.dir,
                        commit: source.commit.clone(),
                        body,
                        shadowed: skill.shadowed,
                    }),
                    Ok(_) => catalog
                        .warnings
                        .push(format!("{} is too large to include in a prompt.", path.display())),
                    Err(error) => catalog.warnings.push(format!("{}: {error}", path.display())),
                }
            }
        }
        Ok(catalog)
    }

    pub fn load(root: PathBuf) -> Result<Self> {
        let sources: Vec<Source> = match fs::read(root.join("sources.json")) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).context("reading registered skill sources")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error).context("reading registered skill sources"),
        };
        let mut names = HashSet::new();
        for source in &sources {
            validate_name(&source.name)?;
            if !names.insert(source.name.to_lowercase()) {
                bail!("Duplicate skill source name: {}", source.name);
            }
            if !source.path.is_absolute() {
                bail!("Source {} has a non-absolute path", source.name);
            }
        }
        Ok(Self { root, sources })
    }

    pub fn apply(&mut self, operation: Operation, cancel: &AtomicBool) -> Result<()> {
        if cancel.load(Ordering::Relaxed) {
            bail!("Skill source operation cancelled.");
        }
        if matches!(operation, Operation::Scan) {
            return Ok(());
        }
        let mut next = self.sources.clone();
        let mut checkout = None;
        match operation {
            Operation::Scan => unreachable!(),
            Operation::Save { mut source, editing } => {
                source.name = source.name.trim().to_owned();
                validate_name(&source.name)?;
                let index = editing.as_ref().map(|name| self.index(name)).transpose()?;
                if next.iter().enumerate().any(|(i, other)| {
                    Some(i) != index && other.name.to_lowercase() == source.name.to_lowercase()
                }) {
                    bail!("A source named “{}” is already registered.", source.name);
                }
                let previous = index.map(|index| &self.sources[index]);
                source.enabled = previous.is_none_or(|source| source.enabled);
                match source.kind {
                    Kind::Local => {
                        source.path = local_path(&source.path.to_string_lossy())?;
                        source.url.clear();
                        source.git_ref.clear();
                        source.commit.clear();
                    }
                    Kind::Git => {
                        source.url = source.url.trim().to_owned();
                        source.git_ref = source.git_ref.trim().to_owned();
                        if source.url.is_empty() {
                            bail!("Repository URL is required.");
                        }
                        if let Some(old) = previous.filter(|old| {
                            old.kind == Kind::Git
                                && old.url == source.url
                                && old.git_ref == source.git_ref
                        }) {
                            source.path = old.path.clone();
                            source.commit = old.commit.clone();
                        } else {
                            checkout = Some(self.checkout(&mut source, cancel)?);
                        }
                    }
                }
                if let Some(index) = index {
                    next[index] = source;
                } else {
                    next.push(source);
                }
            }
            Operation::Remove(name) => {
                next.remove(self.index(&name)?);
            }
            Operation::Enable(name, enabled) => {
                next[self.index(&name)?].enabled = enabled;
            }
            Operation::Move { name, before } => {
                let from = self.index(&name)?;
                let to = self.index(&before)?;
                if from == to {
                    return Ok(());
                }
                let source = next.remove(from);
                // Drop onto a row to take its position, in either direction.
                next.insert(to, source);
            }
            Operation::Refresh(name) => {
                let source = &mut next[self.index(&name)?];
                if source.kind != Kind::Git {
                    bail!("Only remote sources need a Git refresh.");
                }
                checkout = Some(self.checkout(source, cancel)?);
            }
        }
        if cancel.load(Ordering::Relaxed) {
            bail!("Skill source operation cancelled.");
        }
        self.persist(&next)?;
        if let Some(checkout) = checkout {
            let _ = checkout.keep();
        }
        let previous = std::mem::replace(&mut self.sources, next);
        // Only remove plugin-owned checkouts, never registered local folders.
        for source in previous {
            if source.kind == Kind::Git
                && !self.sources.iter().any(|other| other.path == source.path)
                && source.path.parent() == Some(self.root.join("sources").as_path())
                && source
                    .path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("source-"))
            {
                // A cleanup failure leaves an unused checkout, not a lost registry.
                if let Err(error) = fs::remove_dir_all(&source.path) {
                    eprintln!(
                        "Could not remove unused skill checkout {}: {error}",
                        source.path.display()
                    );
                }
            }
        }
        Ok(())
    }

    fn index(&self, name: &str) -> Result<usize> {
        self.sources
            .iter()
            .position(|source| source.name == name)
            .context("This source is no longer registered.")
    }

    fn persist(&self, sources: &[Source]) -> Result<()> {
        let mut encoded = serde_json::to_vec_pretty(sources)?;
        encoded.push(b'\n');
        chartr_storage::write_atomic(&self.root.join("sources.json"), &encoded)
            .context("saving skill sources")?;
        Ok(())
    }

    fn checkout(&self, source: &mut Source, cancel: &AtomicBool) -> Result<tempfile::TempDir> {
        let root = self.root.join("sources");
        fs::create_dir_all(&root)?;
        let checkout = tempfile::Builder::new().prefix("source-").tempdir_in(&root)?;
        let mut command = Command::new("git");
        command.args(["clone", "--depth", "1", "--single-branch"]);
        if !source.git_ref.is_empty() {
            command.arg("--branch").arg(&source.git_ref);
        }
        command.arg("--").arg(&source.url).arg(checkout.path());
        git(&mut command, cancel).context("cloning the skill source")?;
        source.commit = git(
            Command::new("git").arg("-C").arg(checkout.path()).args(["rev-parse", "HEAD"]),
            cancel,
        )?;
        if source.git_ref.is_empty() {
            let branch = git(
                Command::new("git").arg("-C").arg(checkout.path()).args([
                    "rev-parse",
                    "--abbrev-ref",
                    "HEAD",
                ]),
                cancel,
            )?;
            if branch != "HEAD" {
                source.git_ref = branch;
            }
        }
        source.path = checkout.path().to_owned();
        Ok(checkout)
    }

    pub fn states(&self) -> Vec<State> {
        let mut claimed = HashSet::new();
        self.sources
            .iter()
            .map(|source| {
                let mut state = State {
                    unavailable: !source.path.is_dir() || fs::read_dir(&source.path).is_err(),
                    ..State::default()
                };
                let mut found = Vec::new();
                discover(&source.path, 1, &mut found);
                let mut seen = HashSet::new();
                for dir in found {
                    let name = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
                    let key = name.to_lowercase();
                    if !seen.insert(key.clone()) {
                        state.warnings.push(format!(
                            "Duplicate skill “{name}”: the first path in sorted order wins."
                        ));
                        continue;
                    }
                    let shadowed = claimed.contains(&key);
                    if source.enabled {
                        claimed.insert(key);
                    }
                    state.skills.push(Skill { name, dir, shadowed });
                }
                state
            })
            .collect()
    }
}

fn git(command: &mut Command, cancel: &AtomicBool) -> Result<String> {
    command.env("GIT_TERMINAL_PROMPT", "0");
    let output = crate::process::output(command, Duration::from_secs(120), 1024 * 1024, cancel)
        .context("running Git (two-minute limit; interactive terminal prompts disabled)")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.chars().count() > 64
        || name != name.trim()
        || !name.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
    {
        bail!("Name must be 1–64 letters, numbers, spaces, hyphens, or underscores.");
    }
    Ok(())
}

fn local_path(value: &str) -> Result<PathBuf> {
    let value = value.trim();
    let path = if value == "~" || value.starts_with("~/") {
        PathBuf::from(std::env::var_os("HOME").context("Cannot resolve your home directory.")?)
            .join(value.strip_prefix("~/").unwrap_or(""))
    } else {
        PathBuf::from(value)
    };
    if !path.is_absolute() {
        bail!("Use an absolute folder path or ~/ for your home directory.");
    }
    Ok(path)
}

// Match the original chartr walk: depths 1–3, sorted, skip dot directories and
// node_modules, and stop at each skill so its supporting files are not skills.
fn discover(root: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if depth > 3 {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with('.') || name == "node_modules" || !path.is_dir() {
            continue;
        }
        if path.join("SKILL.md").is_file() {
            found.push(path);
        } else {
            discover(&path, depth + 1, found);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(name: &str, path: &Path) -> Source {
        Source {
            name: name.into(),
            path: path.to_owned(),
            kind: Kind::Local,
            url: String::new(),
            git_ref: String::new(),
            commit: String::new(),
            enabled: true,
        }
    }
    fn apply(store: &mut Store, operation: Operation) {
        store.apply(operation, &AtomicBool::new(false)).unwrap();
    }
    fn save(store: &mut Store, source: Source) {
        apply(store, Operation::Save { source, editing: None });
    }
    fn skill(root: &Path, path: &str) {
        let dir = root.join(path);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), "# Skill").unwrap();
    }
    fn command(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git").current_dir(root).args(args).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap().trim().into()
    }
    fn commit(root: &Path) {
        command(root, &["add", "."]);
        command(
            root,
            &[
                "-c",
                "user.name=chartr Test",
                "-c",
                "user.email=chartr@example.invalid",
                "commit",
                "-qm",
                "skills",
            ],
        );
    }
    fn repository(root: &Path) {
        fs::create_dir_all(root).unwrap();
        command(root, &["init", "-q", "-b", "main"]);
        skill(root, "first");
        commit(root);
    }
    fn remote(name: &str, repo: &Path, git_ref: &str) -> Source {
        Source {
            kind: Kind::Git,
            url: repo.display().to_string(),
            git_ref: git_ref.into(),
            ..source(name, Path::new(""))
        }
    }

    #[test]
    fn local_sources_round_trip_edit_reorder_toggle_and_remove_without_touching_folders() {
        let temp = tempfile::tempdir().unwrap();
        let local = temp.path().join("local");
        skill(&local, "shared");
        let mut store = Store::load(temp.path().join("data")).unwrap();
        save(&mut store, source("First", &local));
        save(&mut store, source("Second", &local));
        assert!(!store.states()[0].skills[0].shadowed);
        assert!(store.states()[1].skills[0].shadowed);
        apply(&mut store, Operation::Move { name: "Second".into(), before: "First".into() });
        apply(&mut store, Operation::Enable("Second".into(), false));
        assert!(!store.states()[1].skills[0].shadowed);
        apply(
            &mut store,
            Operation::Save { source: source("Renamed", &local), editing: Some("Second".into()) },
        );
        assert_eq!(store.sources[0].name, "Renamed");
        assert!(!store.sources[0].enabled);
        let loaded = Store::load(store.root.clone()).unwrap();
        assert_eq!(loaded.sources, store.sources);
        apply(&mut store, Operation::Remove("Renamed".into()));
        assert!(local.join("shared/SKILL.md").is_file());
        assert_eq!(store.sources.len(), 1);
    }

    #[test]
    fn discovery_matches_depth_and_duplicate_rules_and_rescans_local_changes() {
        let temp = tempfile::tempdir().unwrap();
        for path in [
            "a/skill",
            "b/skill",
            "level/two/three",
            "a/skill/support/nested",
            ".hidden/ignored",
            "node_modules/ignored",
            "too/deep/to/see",
        ] {
            skill(temp.path(), path);
        }
        let store =
            Store { root: temp.path().join("data"), sources: vec![source("Local", temp.path())] };
        let state = &store.states()[0];
        assert_eq!(
            state.skills.iter().map(|skill| skill.name.as_str()).collect::<Vec<_>>(),
            ["skill", "three"]
        );
        assert_eq!(state.warnings.len(), 1);
        skill(temp.path(), "new-skill");
        assert_eq!(store.states()[0].skills.len(), 3);
        fs::remove_dir_all(temp.path()).unwrap();
        assert!(store.states()[0].unavailable);
    }

    #[test]
    fn invalid_and_duplicate_names_and_relative_paths_leave_registry_unchanged() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::load(temp.path().join("data")).unwrap();
        save(&mut store, source("Existing", temp.path()));
        let original = fs::read(store.root.join("sources.json")).unwrap();
        for invalid in [
            source("existing", temp.path()),
            source("a/b", temp.path()),
            source("", temp.path()),
            source("valid", Path::new("relative")),
        ] {
            assert!(
                store
                    .apply(
                        Operation::Save { source: invalid, editing: None },
                        &AtomicBool::new(false)
                    )
                    .is_err()
            );
        }
        assert_eq!(fs::read(store.root.join("sources.json")).unwrap(), original);
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        assert_eq!(local_path("~/skills").unwrap(), home.join("skills"));
        assert_eq!(local_path("~").unwrap(), home);
    }

    #[test]
    fn persistence_failure_and_cancellation_preserve_memory_and_disk() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::load(temp.path().join("data")).unwrap();
        save(&mut store, source("Local", temp.path()));
        let original = store.sources.clone();
        assert!(store.apply(Operation::Remove("Local".into()), &AtomicBool::new(true)).is_err());
        assert_eq!(Store::load(store.root.clone()).unwrap().sources, original);
        fs::remove_file(store.root.join("sources.json")).unwrap();
        fs::create_dir(store.root.join("sources.json")).unwrap();
        assert!(
            store.apply(Operation::Enable("Local".into(), false), &AtomicBool::new(false)).is_err()
        );
        assert_eq!(store.sources, original);
    }

    #[test]
    fn remote_refs_are_independent_and_refresh_is_explicit_and_failure_preserves_current_checkout()
    {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        repository(&repo);
        command(&repo, &["tag", "v1"]);
        let mut store = Store::load(temp.path().join("data")).unwrap();
        save(&mut store, remote("Main", &repo, ""));
        save(&mut store, remote("Tag", &repo, "v1"));
        assert_eq!(store.sources[0].git_ref, "main");
        assert_ne!(store.sources[0].path, store.sources[1].path);
        let tag_path = store.sources[1].path.clone();
        let tag_commit = store.sources[1].commit.clone();
        skill(&repo, "second");
        commit(&repo);
        assert_eq!(store.states()[0].skills.len(), 1);
        let old = store.sources[0].path.clone();
        apply(&mut store, Operation::Refresh("Main".into()));
        assert!(!old.exists());
        assert_eq!(store.states()[0].skills.len(), 2);
        assert_eq!(store.sources[1].commit, tag_commit);
        assert!(tag_path.is_dir());
        let mut renamed = store.sources[0].clone();
        renamed.name = "Renamed".into();
        let current = renamed.path.clone();
        apply(&mut store, Operation::Save { source: renamed, editing: Some("Main".into()) });
        assert_eq!(store.sources[0].path, current);
        let original = store.sources.clone();
        fs::rename(&repo, temp.path().join("offline")).unwrap();
        assert!(
            store.apply(Operation::Refresh("Renamed".into()), &AtomicBool::new(false)).is_err()
        );
        assert_eq!(store.sources, original);
        assert!(current.is_dir());
        assert_eq!(Store::load(store.root.clone()).unwrap().sources, original);
        apply(&mut store, Operation::Remove("Renamed".into()));
        assert!(!current.exists());
        assert!(tag_path.exists());
    }

    #[test]
    fn changing_remote_ref_or_kind_is_transactional_and_failed_registration_leaves_no_checkout() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        repository(&repo);
        let mut store = Store::load(temp.path().join("data")).unwrap();
        assert!(
            store
                .apply(
                    Operation::Save { source: remote("Bad", &repo, "missing"), editing: None },
                    &AtomicBool::new(false)
                )
                .is_err()
        );
        assert!(store.sources.is_empty());
        assert_eq!(fs::read_dir(store.root.join("sources")).unwrap().count(), 0);
        save(&mut store, remote("Good", &repo, "main"));
        let original = store.sources[0].clone();
        assert!(
            store
                .apply(
                    Operation::Save {
                        source: remote("Good", &repo, "missing"),
                        editing: Some("Good".into())
                    },
                    &AtomicBool::new(false)
                )
                .is_err()
        );
        assert_eq!(store.sources[0], original);
        apply(
            &mut store,
            Operation::Save { source: source("Good", &repo), editing: Some("Good".into()) },
        );
        assert!(!original.path.exists());
        apply(&mut store, Operation::Remove("Good".into()));
        assert!(repo.join("first/SKILL.md").exists());
    }

    #[test]
    fn malformed_registry_is_reported_without_overwriting_it() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("sources.json");
        fs::write(&file, "not JSON").unwrap();
        assert!(Store::load(temp.path().to_owned()).is_err());
        assert_eq!(fs::read_to_string(file).unwrap(), "not JSON");
    }
}
