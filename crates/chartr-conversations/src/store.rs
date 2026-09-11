use crate::{
    Conversation, Observation, OpenCode, Provider, ProviderPaths, Status,
    opencode::endpoints_for_process, transcripts::Reader,
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, params};
use std::{collections::HashMap, path::Path, time::Duration};

/// Application-owned history. Provider stores are only ever read by adapters.
pub struct Store {
    db: Connection,
    rows: HashMap<String, Conversation>,
    runtimes: HashMap<String, String>,
    aliases: HashMap<String, String>,
    observations: HashMap<String, Observation>,
    paths: ProviderPaths,
    reader: Reader,
    draft_versions: HashMap<String, u64>,
    verified_at: Option<std::time::Instant>,
}

impl Store {
    pub fn open(path: &Path, paths: ProviderPaths) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        db.busy_timeout(Duration::from_secs(2))?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS conversations (id TEXT PRIMARY KEY, value_json TEXT NOT NULL);")?;
        let mut rows = HashMap::new();
        {
            let mut query = db.prepare("SELECT value_json FROM conversations")?;
            for raw in query.query_map([], |r| r.get::<_, String>(0))? {
                let mut row: Conversation =
                    serde_json::from_str(&raw?).context("Reading saved conversation history")?;
                row.status = Status::Ended;
                rows.insert(row.id.clone(), row);
            }
        }
        Ok(Self {
            db,
            rows,
            runtimes: HashMap::new(),
            aliases: HashMap::new(),
            observations: HashMap::new(),
            paths,
            reader: Reader::default(),
            draft_versions: HashMap::new(),
            verified_at: None,
        })
    }

    pub fn list(&self) -> Vec<Conversation> {
        let mut rows: Vec<_> = self.rows.values().cloned().collect();
        rows.sort_by(|a, b| b.updated.cmp(&a.updated).then_with(|| a.id.cmp(&b.id)));
        rows
    }

    pub fn resolve_id(&self, id: &str) -> String {
        self.aliases.get(id).cloned().unwrap_or_else(|| id.to_owned())
    }

    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.rows.get(&self.resolve_id(id))
    }

    pub fn for_runtime(&self, runtime: &str) -> Option<&str> {
        self.runtimes.get(runtime).map(String::as_str)
    }

    /// A successful, complete runtime snapshot is the only deletion evidence.
    /// Failed backend reads must not call this with an empty list.
    pub fn reconcile(&mut self, observations: Vec<Observation>, now: u64) -> Result<()> {
        let previous = self.runtimes.clone();
        self.runtimes.clear();
        self.observations.clear();
        for row in self.rows.values_mut() {
            row.runtime = None;
            row.terminal = None;
            row.endpoint = None;
            row.status = Status::Ended;
            row.requests.clear();
        }
        for observation in observations {
            let provider = observation.provider;
            let id = if let Some(native) = &observation.native {
                serde_json::to_string(&(self.paths.namespace(provider), provider, &native.id))?
            } else {
                format!("detected:{}:{}", observation.terminal, provider.slug())
            };
            let mut row = self.rows.get(&id).cloned().unwrap_or_else(|| Conversation {
                id: id.clone(),
                provider,
                native: observation.native.clone(),
                title: observation
                    .title
                    .clone()
                    .unwrap_or_else(|| format!("{} conversation", provider.name())),
                custom_title: None,
                cwd: observation.cwd.clone(),
                space: observation.space.clone(),
                updated: now,
                draft: String::new(),
                archived: false,
                messages: Vec::new(),
                delivery: None,
                runtime: None,
                terminal: None,
                status: Status::Unknown,
                endpoint: None,
                problem: None,
                requests: Vec::new(),
            });
            let promoted = previous
                .get(&observation.runtime)
                .filter(|old| *old != &id)
                .filter(|old_id| {
                    self.rows
                        .get(*old_id)
                        .is_some_and(|old| old.native.is_none() && old.provider == provider)
                })
                .cloned();
            if let Some(old_id) = &promoted {
                let old = self.rows.get(old_id).unwrap();
                if row.space.is_none() {
                    row.space = old.space.clone();
                }
                if row.draft.is_empty() {
                    row.draft = old.draft.clone();
                }
                if row.custom_title.is_none() {
                    row.custom_title = old.custom_title.clone();
                }
                row.archived |= old.archived;
            }
            row.runtime = Some(observation.runtime.clone());
            row.terminal = Some(observation.terminal.clone());
            row.status = observation.status;
            row.cwd = observation.cwd.clone();
            if observation.space.is_some() {
                row.space = observation.space.clone();
            }
            row.native = observation.native.clone();
            row.problem = None;
            if let Some(native) = &observation.native {
                match self.reader.read(provider, native, &self.paths) {
                    Ok(transcript) => {
                        // Stream deltas must not keep moving rows under the pointer.
                        if transcript
                            .messages
                            .iter()
                            .filter(|m| m.role == crate::Role::User)
                            .count()
                            > row.messages.iter().filter(|m| m.role == crate::Role::User).count()
                        {
                            row.updated = now;
                        }
                        if let Some(title) = transcript.title.filter(|s| !s.trim().is_empty()) {
                            row.title = title;
                        }
                        row.messages = transcript.messages;
                    }
                    Err(error) => row.problem = Some(error.to_string()),
                }
                if provider == Provider::OpenCode
                    && let (Some(pid), Some(cwd)) = (observation.pid, observation.cwd.as_deref())
                {
                    if let Ok(endpoints) = endpoints_for_process(pid) {
                        for endpoint in endpoints {
                            let Ok(client) = OpenCode::new(&endpoint, cwd) else { continue };
                            if client.health().is_err() {
                                continue;
                            }
                            if let Ok((title, status, mut messages)) = client.read(&native.id) {
                                let Ok(requests) = client.pending(&native.id) else { continue };
                                crate::transcripts::bound_messages(&mut messages);
                                if messages.iter().filter(|m| m.role == crate::Role::User).count()
                                    > row
                                        .messages
                                        .iter()
                                        .filter(|m| m.role == crate::Role::User)
                                        .count()
                                {
                                    row.updated = now;
                                }
                                row.title = if title.starts_with("New session - ") {
                                    crate::prompt_title(&messages)
                                        .unwrap_or_else(|| "New OpenCode conversation".into())
                                } else {
                                    title
                                };
                                row.messages = messages;
                                row.status =
                                    if !requests.is_empty() { Status::Waiting } else { status };
                                row.endpoint = Some(endpoint);
                                row.requests = requests;
                                row.problem = None;
                                break;
                            }
                        }
                    }
                }
            } else {
                row.problem = Some("Enable the agent integration, then start or resume a conversation in its terminal.".to_owned());
            }
            if let Some(delivery) = &row.delivery {
                if row.messages.iter().any(|message| delivery.matches(message)) {
                    if row.draft == delivery.text {
                        row.draft.clear();
                    }
                    row.delivery = None;
                }
            }
            if let Some(old_id) = promoted {
                let transaction = self.db.transaction()?;
                transaction.execute("INSERT INTO conversations (id, value_json) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET value_json = excluded.value_json", params![row.id, serde_json::to_string(&row)?])?;
                transaction.execute("DELETE FROM conversations WHERE id = ?1", [&old_id])?;
                transaction.commit()?;
                self.rows.remove(&old_id);
                self.aliases.insert(old_id.clone(), id.clone());
                if let Some(version) = self.draft_versions.remove(&old_id) {
                    self.draft_versions.insert(id.clone(), version);
                }
            } else {
                self.save_if_changed(&row)?;
            }
            self.runtimes.insert(observation.runtime.clone(), id.clone());
            self.observations.insert(observation.runtime.clone(), observation);
            self.rows.insert(id, row);
        }
        self.verified_at = Some(std::time::Instant::now());
        Ok(())
    }

    fn save_if_changed(&self, row: &Conversation) -> Result<()> {
        let serialized = serde_json::to_string(row)?;
        if self.rows.get(&row.id).map(serde_json::to_string).transpose()?.as_deref()
            == Some(serialized.as_str())
        {
            return Ok(());
        }
        self.db.execute("INSERT INTO conversations (id, value_json) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET value_json = excluded.value_json", params![row.id, serialized])?;
        Ok(())
    }

    fn edit(&mut self, id: &str, change: impl FnOnce(&mut Conversation)) -> Result<()> {
        let id = self.resolve_id(id);
        let mut row = self.rows.get(&id).cloned().context("Conversation no longer exists")?;
        change(&mut row);
        self.save_if_changed(&row)?;
        self.rows.insert(id, row);
        Ok(())
    }

    pub fn set_draft(&mut self, id: &str, draft: String) -> Result<()> {
        ensure!(draft.len() <= 1024 * 1024, "Draft is too large");
        self.edit(id, |row| row.draft = draft)
    }

    pub fn set_draft_version(&mut self, id: &str, draft: String, version: u64) -> Result<()> {
        let id = self.resolve_id(id);
        if self.draft_versions.get(&id).is_some_and(|saved| *saved > version) {
            return Ok(());
        }
        self.set_draft(&id, draft)?;
        self.draft_versions.insert(id, version);
        Ok(())
    }

    pub fn rename(&mut self, id: &str, title: String) -> Result<()> {
        let title = title.trim().chars().take(150).collect::<String>();
        self.edit(id, |row| row.custom_title = (!title.is_empty()).then_some(title))
    }

    pub fn archive(&mut self, id: &str, archived: bool) -> Result<()> {
        self.edit(id, |row| row.archived = archived)
    }

    pub fn begin_delivery(&mut self, id: &str, message_id: String, text: String) -> Result<()> {
        ensure!(
            self.get(id).is_some_and(|row| row.delivery.is_none()),
            "Resolve the previous delivery before sending again"
        );
        self.edit(id, |row| {
            row.delivery = Some(crate::Delivery { message_id, text, prior_user_messages: None })
        })
    }

    pub fn begin_terminal_delivery(
        &mut self,
        id: &str,
        message_id: String,
        text: String,
    ) -> Result<()> {
        ensure!(
            self.get(id).is_some_and(|row| row.delivery.is_none()),
            "Resolve the previous delivery before sending again"
        );
        self.edit(id, |row| {
            let prior_user_messages = Some(
                row.messages
                    .iter()
                    .filter(|m| m.role == crate::Role::User)
                    .map(|m| m.id.clone())
                    .collect(),
            );
            row.delivery = Some(crate::Delivery { message_id, text, prior_user_messages });
        })
    }

    pub fn terminal_target(&self, id: &str) -> Result<Observation> {
        ensure!(
            self.verified_at.is_some_and(|at| at.elapsed() < Duration::from_secs(10)),
            "Waiting for a fresh terminal observation"
        );
        let row = self.get(id).context("Conversation no longer exists")?;
        ensure!(
            row.provider.transport() == chartr_agent::MessageTransport::TerminalPrompt
                && row.native.is_some(),
            "This conversation has no terminal input binding"
        );
        let runtime = row.runtime.as_ref().context("This conversation is no longer running")?;
        self.observations.get(runtime).cloned().context("The runtime is unavailable")
    }

    pub fn confirm_delivery(&mut self, id: &str) -> Result<()> {
        self.edit(id, |row| row.delivery = None)
    }

    pub fn live_client(&self, id: &str) -> Result<(OpenCode, String)> {
        ensure!(
            self.verified_at.is_some_and(|at| at.elapsed() < Duration::from_secs(10)),
            "Waiting for a fresh terminal observation"
        );
        let row = self.get(id).context("Conversation no longer exists")?;
        let runtime = row.runtime.as_ref().context("This conversation is no longer running")?;
        let observation = self.observations.get(runtime).context("The runtime is unavailable")?;
        let pid = observation.pid.context("The running agent could not be verified")?;
        let endpoint =
            row.endpoint.as_ref().context("Continue this conversation in terminal mode")?;
        ensure!(
            endpoints_for_process(pid)?.contains(endpoint),
            "The agent's local connection changed; wait for it to reconnect"
        );
        let native = row.native.as_ref().context("Conversation identity is unavailable")?;
        let cwd = row.cwd.as_deref().context("Conversation directory is unavailable")?;
        Ok((OpenCode::new(endpoint, cwd)?, native.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeSession;

    fn observation(runtime: &str, native: Option<&str>) -> Observation {
        Observation {
            space: None,
            runtime: runtime.into(),
            terminal: format!("terminal-{runtime}"),
            provider: Provider::Claude,
            native: native.map(|id| NativeSession { id: id.into(), path: None }),
            cwd: Some("/same/project".into()),
            title: None,
            status: Status::Idle,
            pid: None,
        }
    }

    fn paths(root: &Path) -> ProviderPaths {
        ProviderPaths {
            codex: root.join("codex"),
            claude: root.join("claude"),
            opencode: root.join("opencode"),
        }
    }

    #[test]
    fn space_owner_survives_identity_promotion_exit_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        let owner = crate::SpaceIdentity {
            key: "folder:/original".into(),
            name: "Original project".into(),
        };
        let mut seen = observation("one", None);
        seen.space = Some(owner.clone());
        store.reconcile(vec![seen], 1).unwrap();
        let mut promoted = observation("one", Some("native"));
        promoted.cwd = Some("/another/cwd".into());
        store.reconcile(vec![promoted], 2).unwrap();
        let id = store.for_runtime("one").unwrap().to_owned();
        assert_eq!(store.get(&id).unwrap().space, Some(owner.clone()));
        store.reconcile(vec![], 3).unwrap();
        drop(store);
        let store = Store::open(&file, paths(dir.path())).unwrap();
        assert_eq!(store.get(&id).unwrap().space, Some(owner));
        assert!(store.get(&id).unwrap().runtime.is_none());
    }

    #[test]
    fn terminal_delivery_requires_a_new_matching_user_item_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(&dir.path().join("db"), paths(dir.path())).unwrap();
        store.reconcile(vec![observation("one", Some("a"))], 1).unwrap();
        let id = store.for_runtime("one").unwrap().to_owned();
        let old = crate::Message {
            id: "old".into(),
            role: crate::Role::User,
            text: "repeat".into(),
            complete: true,
        };
        store.edit(&id, |row| row.messages.push(old.clone())).unwrap();
        store.begin_terminal_delivery(&id, "delivery".into(), "repeat".into()).unwrap();
        drop(store);
        let store = Store::open(&dir.path().join("db"), paths(dir.path())).unwrap();
        let delivery = store.get(&id).unwrap().delivery.as_ref().unwrap();
        assert!(!delivery.matches(&old));
        assert!(!delivery.matches(&crate::Message {
            id: "new".into(),
            role: crate::Role::Assistant,
            ..old.clone()
        }));
        assert!(delivery.matches(&crate::Message { id: "new".into(), ..old }));
    }

    #[test]
    fn delayed_draft_writes_cannot_overwrite_shutdown_and_uncertain_sends_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        store.reconcile(vec![observation("one", Some("a"))], 1).unwrap();
        let id = store.for_runtime("one").unwrap().to_owned();
        store.set_draft_version(&id, "latest".into(), 2).unwrap();
        store.set_draft_version(&id, "old background write".into(), 1).unwrap();
        store.begin_delivery(&id, "msg_123".into(), "Unconfirmed message".into()).unwrap();
        drop(store);
        let store = Store::open(&file, paths(dir.path())).unwrap();
        let row = store.get(&id).unwrap();
        assert_eq!(row.draft, "latest");
        assert_eq!(row.delivery.as_ref().unwrap().text, "Unconfirmed message");
        assert!(!row.can_send());
    }

    #[test]
    fn same_directory_sessions_stay_distinct_and_resume_keeps_identity() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("conversations.sqlite");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        store
            .reconcile(vec![observation("one", Some("a")), observation("two", Some("b"))], 1)
            .unwrap();
        let a = store.for_runtime("one").unwrap().to_owned();
        let b = store.for_runtime("two").unwrap().to_owned();
        assert_ne!(a, b);
        store.set_draft(&a, "keep draft".into()).unwrap();
        store.rename(&a, "Human title".into()).unwrap();
        store
            .reconcile(vec![observation("one", Some("new")), observation("three", Some("a"))], 2)
            .unwrap();
        assert_eq!(store.for_runtime("three"), Some(a.as_str()));
        assert_eq!(store.get(&a).unwrap().draft, "keep draft");
        assert_eq!(store.get(&a).unwrap().display_title(), "Human title");
        assert_eq!(store.get(&b).unwrap().status, Status::Ended);
        assert_eq!(store.list().len(), 3);
        drop(store);
        let reopened = Store::open(&file, paths(dir.path())).unwrap();
        assert_eq!(reopened.get(&a).unwrap().draft, "keep draft");
        assert!(!reopened.get(&a).unwrap().can_send());
        assert!(reopened.get(&a).unwrap().runtime.is_none());
    }

    #[test]
    fn verified_identity_adopts_provisional_draft_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(&dir.path().join("db"), paths(dir.path())).unwrap();
        store.reconcile(vec![observation("one", None)], 1).unwrap();
        let old = store.for_runtime("one").unwrap().to_owned();
        store.set_draft(&old, "draft".into()).unwrap();
        store.reconcile(vec![observation("one", Some("a"))], 2).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_ne!(store.resolve_id(&old), old);
        assert_eq!(store.get(&old).unwrap().draft, "draft");
        store.archive(&old, true).unwrap();
        assert!(store.get(&old).unwrap().runtime.is_some(), "Archiving must not stop a runtime");
    }
}
