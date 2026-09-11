//! Plugin-lifetime synchronization of the last applied composition, independent
//! of panes and unapplied drafts. All writes share the manual Apply lock.
use super::document::{self, Bodies, Document, Part};
use chartr_plugin::{
    BackgroundState, BackgroundStatus,
    services::{PromptTemplates, Services},
};
use gpui::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Applied {
    version: u32,
    project: PathBuf,
    document: Document,
}
fn active_path(path: &Path) -> PathBuf {
    path.with_extension("applied.json")
}
fn read(path: &Path) -> Result<Option<Applied>, String> {
    match std::fs::read(active_path(path)) {
        Ok(bytes) => {
            let active: Applied = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if active.version != 1 || active.document.version != 1 {
                return Err("Unsupported applied-composition version.".into());
            }
            Ok(Some(active))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
fn save(path: &Path, active: &Applied) -> Result<(), String> {
    let encoded = serde_json::to_vec_pretty(active).map_err(|e| e.to_string())?;
    chartr_storage::write_atomic(&active_path(path), &encoded).map_err(|e| e.to_string())
}
/// Called with the per-composition lock held, after successful explicit Apply.
pub fn activate(path: &Path, root: &Path, doc: &Document) -> Result<(), String> {
    save(
        path,
        &Applied {
            version: 1,
            project: root.canonicalize().map_err(|e| e.to_string())?,
            document: doc.clone(),
        },
    )
}
pub fn receipts(path: &Path, fallback: &Document) -> Result<HashMap<String, String>, String> {
    let mut receipts = fallback.managed_files.clone();
    if let Some(active) = read(path)? {
        receipts.extend(active.document.managed_files);
    }
    Ok(receipts)
}
/// Saving a draft may pause/resume syncing, but never replaces applied parts.
pub fn set_enabled(path: &Path, enabled: bool) -> Result<(), String> {
    if let Some(mut active) = read(path)? {
        if active.document.enabled != enabled {
            active.document.enabled = enabled;
            save(path, &active)?;
        }
    }
    Ok(())
}
fn discover(data: &Path) -> Result<Vec<(PathBuf, Applied)>, String> {
    let entries = match std::fs::read_dir(data) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let Some(base) = name.to_str().and_then(|s| s.strip_suffix(".applied.json")) else {
            continue;
        };
        let path = data.join(format!("{base}.json"));
        if let Some(active) = read(&path)? {
            result.push((path, active));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
fn update(
    path: &Path,
    expected: &Applied,
    body: &str,
    alive: &AtomicBool,
    revision: &AtomicU64,
    expected_revision: u64,
) -> Result<(), String> {
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path.with_extension("lock"))
        .map_err(|e| e.to_string())?;
    lock.lock().map_err(|e| e.to_string())?;
    if !alive.load(Ordering::SeqCst) || revision.load(Ordering::SeqCst) != expected_revision {
        return Ok(());
    }
    let Some(mut active) = read(path)? else { return Ok(()) };
    // An explicit Apply or pause during expansion wins over this older scan.
    if &active != expected || !active.document.enabled {
        return Ok(());
    }
    document::apply(&active.project, &active.document, body)?;
    if !active.document.append {
        active.document.managed_files.insert(active.document.filename.clone(), body.into());
    }
    if &active != expected {
        save(path, &active)?;
    }
    Ok(())
}

pub struct Manager {
    data: PathBuf,
    services: Services,
    busy: bool,
    pending: bool,
    alive: Arc<AtomicBool>,
    revision: Arc<AtomicU64>,
    problems: HashMap<PathBuf, String>,
    count: usize,
    subscription: Option<gpui::Subscription>,
    timer: Option<gpui::Task<()>>,
    work: Option<gpui::Task<()>>,
}
impl Drop for Manager {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }
}
impl Manager {
    pub fn new(data: PathBuf) -> Self {
        Self {
            data,
            services: Services::default(),
            busy: false,
            pending: false,
            alive: Arc::new(AtomicBool::new(true)),
            revision: Arc::new(AtomicU64::new(0)),
            problems: HashMap::new(),
            count: 0,
            subscription: None,
            timer: None,
            work: None,
        }
    }
    pub fn connect(&mut self, services: Services, cx: &mut Context<Self>) {
        self.services = services;
        if self.subscription.is_none() {
            let changes = PromptTemplates::changes(cx);
            self.subscription = Some(cx.observe(&changes, |this, _, cx| this.request(cx)));
            self.timer = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(Duration::from_secs(2)).await;
                    if this
                        .update(cx, |this, cx| {
                            if !this.busy {
                                this.request(cx);
                            }
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        }
        self.request(cx);
    }
    pub fn problem(&self, path: &Path) -> Option<String> {
        self.problems.get(path).cloned()
    }
    pub fn status(&self) -> BackgroundStatus {
        if !self.problems.is_empty() {
            BackgroundStatus {
                label: "Markdown Prompt: sync needs attention".into(),
                detail: self
                    .problems
                    .iter()
                    .map(|(p, e)| format!("{}: {e}", p.display()))
                    .collect::<Vec<_>>()
                    .join("\n"),
                state: BackgroundState::Error,
            }
        } else {
            BackgroundStatus {
                label: format!("Markdown Prompt: {} active file(s)", self.count),
                detail: "Applied compositions follow current template content automatically."
                    .into(),
                state: if self.busy { BackgroundState::Running } else { BackgroundState::Idle },
            }
        }
    }
    pub fn request(&mut self, cx: &mut Context<Self>) {
        self.revision.fetch_add(1, Ordering::SeqCst);
        if self.busy {
            self.pending = true;
            return;
        }
        self.busy = true;
        self.pending = false;
        let data = self.data.clone();
        let services = self.services.clone();
        let alive = self.alive.clone();
        let revision = self.revision.clone();
        let expected_revision = revision.load(Ordering::SeqCst);
        self.work = Some(cx.spawn(async move |this, cx| {
            let records = cx.background_executor().spawn(async move { discover(&data) }).await;
            let mut problems = HashMap::new(); let mut count = 0;
            match records {
                Err(e) => { problems.insert(PathBuf::from("configuration"), e); },
                Ok(records) => for (path, active) in records {
                    if !active.document.enabled { continue; }
                    count += 1;
                    let mut providers: Vec<_> = active.document.parts.iter().filter_map(|p| if let Part::Template { provider, .. } = p { Some(provider.clone()) } else { None }).collect();
                    providers.sort(); providers.dedup();
                    let mut bodies = Bodies::new(); let mut error = None;
                    for id in providers {
                        let task = cx.update(|cx| services.get::<PromptTemplates>(&id).map(|service| service.list(Some(active.project.clone()), cx)));
                        let result = match task { Some(task) => task.await, _ => Err(format!("Template provider {id} is unavailable.")) };
                        match result {
                            Err(e) => { error = Some(e); break; },
                            Ok(items) => {
                                let mut ids = std::collections::HashSet::new();
                                for item in items {
                                    if item.id.is_empty() || !ids.insert(item.id.clone()) || item.title.trim().is_empty() || item.prompt.len() > 1024 * 1024 { error = Some(format!("{id} returned invalid templates.")); break; }
                                    bodies.insert((id.clone(), item.id), item.prompt);
                                }
                            }
                        }
                    }
                    // Do not write after a provider was disabled during expansion.
                    if active.document.parts.iter().any(|part| matches!(part, Part::Template { provider, .. } if services.get::<PromptTemplates>(provider).is_none())) { error = Some("A referenced template provider is disabled.".into()); }
                    let body = error.map_or_else(|| document::compose(&active.document.parts, &bodies), Err);
                    let result = match body {
                        Err(e) => Err(e),
                        Ok(body) => {
                            let path = path.clone(); let alive = alive.clone(); let revision = revision.clone();
                            cx.background_executor().spawn(async move { update(&path, &active, &body, &alive, &revision, expected_revision) }).await
                        }
                    };
                    if let Err(e) = result { problems.insert(path, e); }
                }
            }
            let _ = this.update(cx, |this, cx| {
                this.busy = false; this.count = count;
                if this.revision.load(Ordering::SeqCst) == expected_revision { this.problems = problems; }
                cx.notify();
                if this.pending { this.request(cx); }
            });
        }));
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_plugin::services::{SavedPrompt, ServiceExport};
    use gpui::AppContext;

    fn apply_fixture(data: &Path, project: &Path, name: &str, append: bool) -> PathBuf {
        let path = data.join(format!("{name}.json"));
        let mut doc = Document {
            filename: format!("{name}.md"),
            append,
            parts: vec![
                Part::Text { text: "Intro\n".into() },
                Part::Template {
                    provider: "fixture".into(),
                    id: "sources".into(),
                    title: "Sources".into(),
                },
            ],
            ..Document::default()
        };
        if append {
            std::fs::write(project.join(&doc.filename), "User instructions\n").unwrap();
        }
        document::apply(project, &doc, "Intro\none").unwrap();
        if !append {
            doc.managed_files.insert(doc.filename.clone(), "Intro\none".into());
        }
        document::save(&path, &doc).unwrap();
        activate(&path, project, &doc).unwrap();
        path
    }

    #[gpui::test]
    fn applied_files_follow_events_without_panes_and_restore_on_restart(
        cx: &mut gpui::TestAppContext,
    ) {
        let data = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let owned = apply_fixture(data.path(), project.path(), "owned", false);
        let appended = apply_fixture(data.path(), project.path(), "append", true);
        let config_before = std::fs::read(&owned).unwrap();
        let source = cx.new(|_| "one".to_owned());
        let services = Services::default();
        let content = source.clone();
        services.publish(
            "fixture",
            vec![ServiceExport::new(PromptTemplates::new(move |_, cx| {
                gpui::Task::ready(Ok(vec![SavedPrompt {
                    id: "sources".into(),
                    title: "Sources".into(),
                    prompt: content.read(cx).clone(),
                }]))
            }))],
        );
        let manager = cx.new(|_| Manager::new(data.path().into()));
        manager.update(cx, |manager, cx| manager.connect(services.clone(), cx));
        cx.run_until_parked();
        let initial_mtime =
            std::fs::metadata(project.path().join("owned.md")).unwrap().modified().unwrap();
        manager.update(cx, |manager, cx| manager.request(cx));
        cx.run_until_parked();
        assert_eq!(
            std::fs::metadata(project.path().join("owned.md")).unwrap().modified().unwrap(),
            initial_mtime
        );
        cx.update(|cx| {
            source.update(cx, |value, _| *value = "two".into());
            PromptTemplates::changed(cx);
        });
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(project.path().join("owned.md")).unwrap(), "Intro\ntwo");
        let appended_text = std::fs::read_to_string(project.path().join("append.md")).unwrap();
        assert!(appended_text.starts_with("User instructions\n\n"));
        assert!(appended_text.contains("Intro\ntwo"));
        assert_eq!(appended_text.matches("<!--chartr-markdown-prompt-begin-->").count(), 1);
        assert_eq!(
            std::fs::read(&owned).unwrap(),
            config_before,
            "background receipts must not stale the open draft"
        );

        let mut draft = document::load(&owned).unwrap();
        draft.parts = vec![Part::Text { text: "Unapplied draft".into() }];
        document::save(&owned, &draft).unwrap();
        set_enabled(&appended, false).unwrap();
        drop(manager);
        cx.update(|cx| source.update(cx, |value, _| *value = "three".into()));
        let restarted = cx.new(|_| Manager::new(data.path().into()));
        restarted.update(cx, |manager, cx| manager.connect(services.clone(), cx));
        cx.run_until_parked();
        assert_eq!(
            std::fs::read_to_string(project.path().join("owned.md")).unwrap(),
            "Intro\nthree"
        );
        assert_eq!(
            std::fs::read_to_string(project.path().join("append.md")).unwrap(),
            appended_text
        );
        assert_eq!(document::load(&owned).unwrap().parts, draft.parts);
        assert_eq!(receipts(&owned, &draft).unwrap()["owned.md"], "Intro\nthree");

        services.remove("fixture");
        restarted.update(cx, |manager, cx| manager.request(cx));
        cx.run_until_parked();
        assert!(restarted.read_with(cx, |manager, _| manager.problem(&owned)).is_some());
        assert_eq!(
            std::fs::read_to_string(project.path().join("owned.md")).unwrap(),
            "Intro\nthree"
        );
    }

    #[test]
    fn superseded_scans_and_external_edits_are_not_written() {
        let data = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let path = apply_fixture(data.path(), project.path(), "owned", false);
        let active = read(&path).unwrap().unwrap();
        let alive = AtomicBool::new(true);
        let revision = AtomicU64::new(2);
        update(&path, &active, "stale", &alive, &revision, 1).unwrap();
        assert_eq!(std::fs::read_to_string(project.path().join("owned.md")).unwrap(), "Intro\none");
        std::fs::write(project.path().join("owned.md"), "External edits").unwrap();
        assert!(update(&path, &active, "fresh", &alive, &revision, 2).is_err());
        assert_eq!(
            std::fs::read_to_string(project.path().join("owned.md")).unwrap(),
            "External edits"
        );
    }
}
