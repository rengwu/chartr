use crate::{Conversation, Observation, ProviderPaths, Status, transcripts::Reader};
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::{collections::HashMap, path::Path, time::Duration};

/// Application-owned history. Provider stores are only ever read by adapters.
pub struct Store {
    db: Connection,
    rows: HashMap<String, Conversation>,
    runtimes: HashMap<String, String>,
    aliases: HashMap<String, String>,
    paths: ProviderPaths,
    reader: Reader,
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
            paths,
            reader: Reader::default(),
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
        for row in self.rows.values_mut() {
            row.runtime = None;
            row.terminal = None;
            row.status = Status::Ended;
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
                problem: None,
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
            if let Some(title) = observation.title.as_ref().filter(|title| !title.trim().is_empty())
            {
                row.title = title.clone();
            }
            if let Some(native) = &observation.native {
                match self.reader.read(provider, native, &self.paths) {
                    Ok(transcript) => {
                        if let Some(updated) = transcript.updated {
                            row.updated = updated;
                        }
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
            } else {
                row.problem = Some("Enable the agent integration, then start or resume a conversation in its terminal.".to_owned());
            }
            if let Some(old_id) = promoted {
                let transaction = self.db.transaction()?;
                transaction.execute("INSERT INTO conversations (id, value_json) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET value_json = excluded.value_json", params![row.id, serde_json::to_string(&row)?])?;
                transaction.execute("DELETE FROM conversations WHERE id = ?1", [&old_id])?;
                transaction.commit()?;
                self.rows.remove(&old_id);
                self.aliases.insert(old_id.clone(), id.clone());
            } else {
                self.save_if_changed(&row)?;
            }
            self.runtimes.insert(observation.runtime.clone(), id.clone());
            self.rows.insert(id, row);
        }
        // Only the completed snapshot establishes which sessions have ended.
        // In particular, opening saved history must not archive live sessions
        // before their runtimes have been rediscovered.
        let ended: Vec<_> = self
            .rows
            .values()
            .filter(|row| row.status == Status::Ended && !row.archived)
            .map(|row| row.id.clone())
            .collect();
        for id in ended {
            self.archive(&id, true)?;
        }
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

    pub fn rename(&mut self, id: &str, title: String) -> Result<()> {
        let title = title.trim().chars().take(150).collect::<String>();
        self.edit(id, |row| row.custom_title = (!title.is_empty()).then_some(title))
    }

    pub fn archive(&mut self, id: &str, archived: bool) -> Result<()> {
        self.edit(id, |row| row.archived = archived)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NativeSession, Provider};

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
            pi: root.join("pi"),
            kimi: root.join("kimi"),
        }
    }

    #[test]
    fn ended_sessions_are_archived_without_losing_history_or_archiving_idle_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        let live = observation("live", Some("live"));
        store.reconcile(vec![live.clone(), observation("ending", Some("ending"))], 10).unwrap();
        let live_id = store.for_runtime("live").unwrap().to_owned();
        let ended_id = store.for_runtime("ending").unwrap().to_owned();
        store.rename(&ended_id, "Saved conversation".into()).unwrap();
        store.edit(&ended_id, |row| row.draft = "Retained draft".into()).unwrap();

        store.reconcile(vec![live.clone()], 20).unwrap();
        assert!(!store.get(&live_id).unwrap().archived, "idle is still a live session");
        let ended = store.get(&ended_id).unwrap();
        assert!(ended.archived);
        assert_eq!(ended.status, Status::Ended);
        assert!(ended.runtime.is_none());
        assert_eq!(ended.display_title(), "Saved conversation");
        assert_eq!(ended.draft, "Retained draft");
        assert_eq!(ended.updated, 10, "archiving must not change conversation recency");

        // Repeated snapshots are harmless, and the archive flag survives restart.
        store.reconcile(vec![live], 30).unwrap();
        drop(store);
        let reopened = Store::open(&file, paths(dir.path())).unwrap();
        assert!(reopened.get(&ended_id).unwrap().archived);
        assert!(!reopened.get(&live_id).unwrap().archived);
        assert_eq!(reopened.list().len(), 2);
    }

    #[test]
    fn startup_waits_for_rediscovery_before_archiving_missing_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        let live = observation("live", None);
        let manual = observation("manual", None);
        store.reconcile(vec![live.clone(), manual.clone(), observation("gone", None)], 1).unwrap();
        let live_id = store.for_runtime("live").unwrap().to_owned();
        let manual_id = store.for_runtime("manual").unwrap().to_owned();
        let gone_id = store.for_runtime("gone").unwrap().to_owned();
        store.archive(&manual_id, true).unwrap();
        drop(store);

        let mut reopened = Store::open(&file, paths(dir.path())).unwrap();
        assert!(!reopened.get(&live_id).unwrap().archived);
        assert!(!reopened.get(&gone_id).unwrap().archived);
        reopened.reconcile(vec![live, manual], 2).unwrap();
        assert!(!reopened.get(&live_id).unwrap().archived);
        assert!(reopened.get(&manual_id).unwrap().archived, "preserve manual archives");
        assert!(reopened.get(&gone_id).unwrap().archived, "also archive older ended history");
    }

    #[test]
    fn pi_prompt_titles_promote_the_right_row_and_preserve_manual_names_on_resume() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        let mut a = observation("a", None);
        a.provider = Provider::Pi;
        let mut b = observation("b", None);
        b.provider = Provider::Pi;
        store.reconcile(vec![a.clone(), b.clone()], 1).unwrap();
        let provisional = store.for_runtime("a").unwrap().to_owned();
        for (observed, prompt) in [(&mut a, "Axolotls"), (&mut b, "Dinner")] {
            let path = dir.path().join(format!("timestamp_{}.jsonl", observed.runtime));
            std::fs::write(&path, format!("{{\"type\":\"session\",\"id\":\"{}\"}}\n{{\"type\":\"message\",\"message\":{{\"role\":\"user\",\"content\":\"{prompt}\"}}}}\n", observed.runtime)).unwrap();
            observed.native =
                NativeSession::from_identity(Provider::Pi, "path", path.to_str().unwrap());
        }
        store.reconcile(vec![a.clone(), b.clone()], 2).unwrap();
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.get(&provisional).unwrap().display_title(), "Axolotls");
        assert_eq!(store.get(store.for_runtime("b").unwrap()).unwrap().display_title(), "Dinner");
        store.rename(&provisional, "My axolotl notes".into()).unwrap();
        let native_id = store.resolve_id(&provisional);
        store.reconcile(vec![], 3).unwrap();
        drop(store);
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        a.runtime = "resumed".into();
        a.terminal = "resumed-terminal".into();
        store.reconcile(vec![a, b], 4).unwrap();
        assert_eq!(store.for_runtime("resumed"), Some(native_id.as_str()));
        assert_eq!(store.get(&native_id).unwrap().display_title(), "My axolotl notes");
        assert_eq!(store.list().len(), 2);
    }

    #[test]
    fn detected_kimi_and_pi_sessions_are_indexed_and_live_titles_replace_fallbacks() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        let mut observations = Vec::new();
        for name in ["kimi", "pi", "opencode", "grok"] {
            let mut observed = observation(name, None);
            observed.provider =
                Provider::detect(name).expect("Every supported launcher must reach Inbox");
            observations.push(observed);
        }
        store.reconcile(observations.clone(), 1).unwrap();
        assert_eq!(store.list().len(), 4);
        for observed in &mut observations {
            observed.title = Some(format!("Task from {}", observed.provider.name()));
        }
        store.reconcile(observations.clone(), 2).unwrap();
        for observed in &observations {
            let id = store.for_runtime(&observed.runtime).unwrap();
            assert_eq!(store.get(id).unwrap().display_title(), observed.title.as_ref().unwrap());
        }
        let kimi = store.for_runtime("kimi").unwrap().to_owned();
        store.rename(&kimi, "My Kimi task".into()).unwrap();
        // Provider hooks can attach later; promotion keeps the entry and its manual name.
        for observed in &mut observations {
            observed.native = Some(NativeSession { id: "native".into(), path: None });
        }
        store.reconcile(observations, 3).unwrap();
        assert_eq!(store.list().len(), 4);
        assert_eq!(store.get(&kimi).unwrap().display_title(), "My Kimi task");
        store.reconcile(vec![], 4).unwrap();
        let reopened = Store::open(&file, paths(dir.path())).unwrap();
        assert_eq!(reopened.list().len(), 4);
        assert!(reopened.list().iter().all(|row| row.status == Status::Ended));
    }

    #[test]
    fn kimi_recency_tracks_exact_session_prompts_across_polling_and_restart() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let provider_paths = paths(dir.path());
        let session = provider_paths.kimi.join("sessions/workspace/session-a");
        let other = provider_paths.kimi.join("sessions/workspace/session-b");
        for (path, id, time) in [(&session, "session-a", 10), (&other, "session-b", 20)] {
            std::fs::create_dir_all(path.join("agents/main")).unwrap();
            std::fs::write(path.join("state.json"), serde_json::json!({"id":id}).to_string())
                .unwrap();
            std::fs::write(path.join("agents/main/wire.jsonl"), format!("{}\n", serde_json::json!({"type":"prompt.accepted","agentId":"main","promptId":"first","time":time}))).unwrap();
        }
        let mut store = Store::open(&file, provider_paths.clone()).unwrap();
        let mut a = observation("a", Some("session-a"));
        a.provider = Provider::Kimi;
        a.title = Some("hi kimi".into());
        let mut b = a.clone();
        b.runtime = "b".into();
        b.terminal = "terminal-b".into();
        b.native.as_mut().unwrap().id = "session-b".into();
        let observations = vec![a, b];
        store.reconcile(observations.clone(), 100).unwrap();
        let id = store.for_runtime("a").unwrap().to_owned();
        assert_eq!(store.get(&id).unwrap().updated, 10);
        assert_ne!(store.list()[0].id, id);
        store.rename(&id, "My Kimi chat".into()).unwrap();
        let mut log = std::fs::OpenOptions::new()
            .append(true)
            .open(session.join("agents/main/wire.jsonl"))
            .unwrap();
        // A complete prompt/response between polls must still refresh the row.
        writeln!(log, "{}", serde_json::json!({"type":"prompt.accepted","agentId":"main","promptId":"second","time":200})).unwrap();
        writeln!(log, "{}", serde_json::json!({"type":"turn.ended","agentId":"main","time":210}))
            .unwrap();
        store.reconcile(observations.clone(), 300).unwrap();
        assert_eq!(store.list()[0].id, id);
        assert_eq!(store.get(&id).unwrap().updated, 200);
        assert_eq!(store.get(&id).unwrap().display_title(), "My Kimi chat");
        assert!(store.get(&id).unwrap().messages.is_empty());
        // Output, subagent prompts and partial appends cannot move the timestamp.
        writeln!(log, "{}", serde_json::json!({"type":"context.append_message","time":310}))
            .unwrap();
        writeln!(log, "{}", serde_json::json!({"type":"prompt.accepted","agentId":"agent-0","promptId":"child","time":320})).unwrap();
        write!(
            log,
            "{}",
            r#"{"type":"prompt.accepted","agentId":"main","promptId":"third","time":"#
        )
        .unwrap();
        store.reconcile(observations.clone(), 400).unwrap();
        assert_eq!(store.get(&id).unwrap().updated, 200);
        writeln!(log, "450}}").unwrap();
        store.reconcile(observations.clone(), 500).unwrap();
        assert_eq!(store.get(&id).unwrap().updated, 450);
        store.reconcile(observations.clone(), 600).unwrap();
        assert_eq!(store.get(&id).unwrap().updated, 450);
        drop(store);
        let mut store = Store::open(&file, provider_paths).unwrap();
        store.reconcile(observations.clone(), 700).unwrap();
        assert_eq!(store.get(&id).unwrap().updated, 450);
        // Verify identity even when the unchanged wire log would hit the cache.
        std::fs::write(session.join("state.json"), r#"{"id":"session-b"}"#).unwrap();
        store.reconcile(observations.clone(), 800).unwrap();
        assert!(store.get(&id).unwrap().problem.is_some());
        assert_eq!(store.get(&id).unwrap().updated, 450);
        std::fs::write(session.join("state.json"), r#"{"id":"session-a"}"#).unwrap();
        let duplicate = paths(dir.path()).kimi.join("sessions/another-workspace/session-a");
        std::fs::create_dir_all(duplicate).unwrap();
        store.reconcile(observations, 900).unwrap();
        assert!(store.get(&id).unwrap().problem.is_some());
        assert_eq!(store.get(&id).unwrap().updated, 450);
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
    fn legacy_chat_data_is_retained_without_replaying_it() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("db");
        let mut store = Store::open(&file, paths(dir.path())).unwrap();
        store.reconcile(vec![observation("one", Some("a"))], 1).unwrap();
        let id = store.for_runtime("one").unwrap().to_owned();
        store
            .edit(&id, |row| {
                row.draft = "unsent draft".into();
                row.delivery = Some(serde_json::json!({"message_id":"old", "text":"old send"}));
            })
            .unwrap();
        store.reconcile(vec![observation("one", Some("a"))], 2).unwrap();
        store.archive(&id, true).unwrap();
        drop(store);
        let store = Store::open(&file, paths(dir.path())).unwrap();
        let row = store.get(&id).unwrap();
        assert_eq!(row.draft, "unsent draft");
        assert_eq!(row.delivery.as_ref().unwrap()["text"], "old send");
        assert!(row.archived);
        assert_eq!(row.status, Status::Ended);
        assert!(row.runtime.is_none());
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
        store.edit(&a, |row| row.draft = "keep draft".into()).unwrap();
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
        assert!(reopened.get(&a).unwrap().runtime.is_none());
    }

    #[test]
    fn verified_identity_adopts_provisional_draft_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(&dir.path().join("db"), paths(dir.path())).unwrap();
        store.reconcile(vec![observation("one", None)], 1).unwrap();
        let old = store.for_runtime("one").unwrap().to_owned();
        store.edit(&old, |row| row.draft = "draft".into()).unwrap();
        store.reconcile(vec![observation("one", Some("a"))], 2).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_ne!(store.resolve_id(&old), old);
        assert_eq!(store.get(&old).unwrap().draft, "draft");
        store.archive(&old, true).unwrap();
        assert!(store.get(&old).unwrap().runtime.is_some(), "Archiving must not stop a runtime");
    }
}
