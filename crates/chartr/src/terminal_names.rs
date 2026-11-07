//! Host-owned terminal names. Backend pane numbers and UI positions are not identities.

use std::collections::{BTreeMap, HashMap};

use chartr_conversations::{NativeSession, Provider};
use chartr_herdr::control::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualNameLifetime {
    #[default]
    Keep,
    Conversation,
}

impl ManualNameLifetime {
    pub const ALL: [Self; 2] = [Self::Keep, Self::Conversation];

    pub fn label(self) -> &'static str {
        match self {
            Self::Keep => "Keep until renamed or reset",
            Self::Conversation => "Reset for a new conversation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct ConversationKey {
    provider: Provider,
    id: String,
}

impl ConversationKey {
    fn from_session(session: &Session) -> Option<Self> {
        let provider = Provider::detect(session.agent.as_deref()?)?;
        let identity = session.agent_session.as_ref()?;
        if Provider::detect(&identity.agent) != Some(provider) {
            return None;
        }
        let native = NativeSession::from_identity(provider, &identity.kind, &identity.value)?;
        let id = native.id.trim();
        (!id.is_empty()).then(|| Self { provider, id: id.to_owned() })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct ManualName {
    name: String,
    conversation: Option<ConversationKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SavedNames {
    version: u32,
    terminals: BTreeMap<String, ManualName>,
}

impl Default for SavedNames {
    fn default() -> Self {
        Self { version: 1, terminals: BTreeMap::new() }
    }
}

impl SavedNames {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.version == 1, "unsupported terminal names version {}", self.version);
        for name in self.terminals.values() {
            anyhow::ensure!(clean_name(&name.name)?.is_some(), "empty saved terminal name");
        }
        Ok(())
    }
}

#[derive(Default)]
struct AutomaticName {
    provider: Option<String>,
    conversation: Option<ConversationKey>,
    title: Option<String>,
    // After a known change, don't display a title carried over from the old task.
    stale_title: Option<String>,
}

#[derive(Default)]
pub struct TerminalNames {
    saved: SavedNames,
    automatic: HashMap<String, AutomaticName>,
}

impl TerminalNames {
    pub fn restore(saved: SavedNames) -> Self {
        Self { saved, automatic: HashMap::new() }
    }

    pub fn saved(&self) -> SavedNames {
        self.saved.clone()
    }

    pub fn manual(&self, terminal: &str) -> Option<&str> {
        self.saved.terminals.get(terminal).map(|name| name.name.as_str())
    }

    pub fn rename(&mut self, session: &Session, name: &str) -> anyhow::Result<()> {
        if let Some(name) = clean_name(name)? {
            let conversation = self
                .automatic
                .get(&session.terminal.0)
                .and_then(|auto| auto.conversation.clone())
                .or_else(|| ConversationKey::from_session(session))
                .or_else(|| {
                    self.saved
                        .terminals
                        .get(&session.terminal.0)
                        .and_then(|name| name.conversation.clone())
                });
            self.saved
                .terminals
                .insert(session.terminal.0.clone(), ManualName { name, conversation });
        } else {
            self.saved.terminals.remove(&session.terminal.0);
        }
        Ok(())
    }

    pub fn retire(&mut self, terminal: &str) {
        self.saved.terminals.remove(terminal);
        self.automatic.remove(terminal);
    }

    /// Called only on successful observations. Missing metadata is not a conversation change.
    pub fn observe(&mut self, session: &Session, lifetime: ManualNameLifetime) {
        let key = ConversationKey::from_session(session);
        let terminal = &session.terminal.0;
        if let Some(manual) = self.saved.terminals.get_mut(terminal)
            && let Some(key) = &key
        {
            if lifetime == ManualNameLifetime::Conversation
                && manual.conversation.as_ref().is_some_and(|previous| previous != key)
            {
                self.saved.terminals.remove(terminal);
            } else {
                manual.conversation = Some(key.clone());
            }
        }
        let auto = self.automatic.entry(terminal.clone()).or_default();
        let provider = session.agent.as_ref().map(|agent| {
            Provider::detect(agent).map_or_else(|| agent.to_lowercase(), |p| p.name().to_owned())
        });
        let changed = auto.provider != provider
            || key.as_ref().is_some_and(|key| {
                auto.conversation.as_ref().is_some_and(|previous| previous != key)
            });
        if changed && auto.title.is_some() {
            auto.stale_title = auto.title.take();
        }
        auto.provider = provider;
        if key.is_some() {
            auto.conversation = key;
        }
        let title = conversation_title(session);
        // Absence/startup placeholders don't establish that a later identical title is fresh.
        if title.is_some() && title != auto.stale_title {
            auto.stale_title = None;
        }
        auto.title = title.filter(|title| Some(title) != auto.stale_title.as_ref());
    }

    pub fn title(&self, session: &Session) -> String {
        if let Some(manual) = self.manual(&session.terminal.0) {
            return manual.to_owned();
        }
        let automatic = match self.automatic.get(&session.terminal.0) {
            Some(auto) => auto.title.clone(),
            None => conversation_title(session),
        };
        automatic.unwrap_or_else(|| session.title().to_owned())
    }

    pub fn detail(&self, session: &Session) -> String {
        let mut parts = vec![self.title(session)];
        for part in [session.agent.as_deref(), session.running.as_deref()] {
            if let Some(part) = part.filter(|part| !part.is_empty())
                && !parts.iter().any(|existing| existing.eq_ignore_ascii_case(part))
            {
                parts.push(part.to_owned());
            }
        }
        if let Some(cwd) = &session.cwd {
            parts.push(cwd.display().to_string());
        }
        parts.join(" · ")
    }
}

pub fn clean_name(value: &str) -> anyhow::Result<Option<String>> {
    anyhow::ensure!(
        !value.chars().any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}')),
        "Use a single line without control characters."
    );
    let value = value.trim();
    Ok((!value.is_empty()).then(|| value.to_owned()))
}

fn conversation_title(session: &Session) -> Option<String> {
    let agent = session.agent.as_deref()?;
    let title = clean_name(session.conversation_title.as_deref()?).ok()??;
    // Agent terminal titles can append the project directory, already shown by
    // the owning space. Strip only that exact suffix; task text can contain pipes.
    let title = session
        .cwd
        .as_deref()
        .and_then(|cwd| cwd.file_name())
        .and_then(|directory| directory.to_str())
        .and_then(|directory| title.strip_suffix(&format!(" | {directory}")))
        .unwrap_or(&title)
        .trim();
    let startup = title.eq_ignore_ascii_case(agent)
        || Provider::detect(agent).is_some_and(|p| title.eq_ignore_ascii_case(p.name()));
    (!title.is_empty() && !startup).then(|| title.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_herdr::{
        PaneId, TerminalId, WorkspaceId, control::SessionStatus, protocol::AgentSession,
    };

    fn session(identity: Option<&str>, title: Option<&str>) -> Session {
        Session {
            id: PaneId("pane-a".into()),
            terminal: TerminalId("terminal-a".into()),
            workspace: WorkspaceId("workspace".into()),
            label: "zsh".into(),
            running: Some("Codex".into()),
            status: SessionStatus::Idle,
            agent: Some("codex".into()),
            agent_session: identity.map(|id| AgentSession {
                source: "hook".into(),
                agent: "codex".into(),
                kind: "id".into(),
                value: id.into(),
            }),
            conversation_title: title.map(str::to_owned),
            foreground_pid: None,
            cwd: Some("/project".into()),
        }
    }

    #[test]
    fn precedence_reset_and_agent_exit_keep_backend_metadata_independent() {
        let mut names = TerminalNames::default();
        let mut info = session(Some("A"), Some("Fix login"));
        names.observe(&info, ManualNameLifetime::Keep);
        assert_eq!(names.title(&info), "Fix login");
        names.rename(&info, "  修复 🔐  ").unwrap();
        assert_eq!(names.title(&info), "修复 🔐");
        assert_eq!(
            info.title(),
            "Codex",
            "backend identity and notification fallback are unchanged"
        );
        assert_eq!(names.detail(&info), "修复 🔐 · codex · /project");
        info.agent = None;
        info.running = None;
        names.observe(&info, ManualNameLifetime::Keep);
        assert_eq!(names.title(&info), "修复 🔐");
        names.rename(&info, "  ").unwrap();
        assert_eq!(names.title(&info), "zsh");
    }

    #[test]
    fn names_accept_numbers_and_unicode_but_reject_multiline_or_control_input_atomically() {
        let info = session(None, None);
        let mut names = TerminalNames::default();
        for value in ["123", "任务 🦀", "Same name"] {
            names.rename(&info, value).unwrap();
            assert_eq!(names.title(&info), value);
        }
        for value in ["a\nb", "\ta", "a\r", "a\0b", "a\u{1b}b", "a\u{2028}b", "a\u{2029}b"] {
            assert!(names.rename(&info, value).is_err());
            assert_eq!(names.title(&info), "Same name");
        }
    }

    #[test]
    fn startup_blank_and_invalid_automatic_titles_fall_back() {
        for title in [
            None,
            Some(" "),
            Some("Codex"),
            Some("Codex | project"),
            Some("a\nb"),
            Some("\u{1b}task"),
        ] {
            let mut names = TerminalNames::default();
            let info = session(None, title);
            names.observe(&info, ManualNameLifetime::Keep);
            assert_eq!(names.title(&info), "Codex");
        }
    }

    #[test]
    fn automatic_titles_omit_the_project_suffix_without_changing_task_text_or_manual_names() {
        let mut names = TerminalNames::default();
        let mut info = session(None, Some("Say hello | imparo"));
        info.cwd = Some("/projects/imparo".into());
        assert_eq!(names.title(&info), "Say hello");
        names.observe(&info, ManualNameLifetime::Keep);
        assert_eq!(names.title(&info), "Say hello");
        names.rename(&info, "My name | imparo").unwrap();
        assert_eq!(names.title(&info), "My name | imparo");
        names.rename(&info, "").unwrap();
        for (title, expected) in [
            ("Read | write | imparo", "Read | write"),
            ("Compare imparo", "Compare imparo"),
            ("Compare | other-project", "Compare | other-project"),
        ] {
            info.conversation_title = Some(title.into());
            names.observe(&info, ManualNameLifetime::Keep);
            assert_eq!(names.title(&info), expected);
        }
    }

    #[test]
    fn only_known_to_known_identity_changes_reset_a_manual_name() {
        let mut names = TerminalNames::default();
        let mut info = session(None, None);
        names.rename(&info, "Review").unwrap();
        for id in [None, Some(""), Some("A"), None, Some("A")] {
            info.agent_session = session(id, None).agent_session;
            names.observe(&info, ManualNameLifetime::Conversation);
            assert_eq!(names.title(&info), "Review");
        }
        info.agent_session = session(Some("B"), None).agent_session;
        names.observe(&info, ManualNameLifetime::Conversation);
        assert_eq!(names.title(&info), "Codex");
    }

    #[test]
    fn keep_mode_updates_identity_and_switching_policy_is_not_retroactive() {
        let mut names = TerminalNames::default();
        let mut info = session(Some("A"), None);
        names.rename(&info, "Work slot").unwrap();
        info.agent_session = session(Some("B"), None).agent_session;
        names.observe(&info, ManualNameLifetime::Keep);
        names.observe(&info, ManualNameLifetime::Conversation);
        assert_eq!(names.title(&info), "Work slot");
        info.agent_session = session(Some("C"), None).agent_session;
        names.observe(&info, ManualNameLifetime::Conversation);
        assert_eq!(names.title(&info), "Codex");
    }

    #[test]
    fn provider_mismatch_is_missing_identity_and_provider_changes_are_new_conversations() {
        let mut names = TerminalNames::default();
        let mut info = session(Some("A"), None);
        names.rename(&info, "Task").unwrap();
        info.agent_session.as_mut().unwrap().agent = "pi".into();
        names.observe(&info, ManualNameLifetime::Conversation);
        assert_eq!(names.title(&info), "Task");
        info.agent = Some("pi".into());
        names.observe(&info, ManualNameLifetime::Conversation);
        assert!(names.manual(&info.terminal.0).is_none());
    }

    #[test]
    fn normalized_identity_ignores_source_and_log_path_changes() {
        let mut names = TerminalNames::default();
        let mut info = session(None, None);
        info.agent = Some("pi".into());
        info.agent_session = Some(AgentSession {
            source: "hook".into(),
            agent: "pi".into(),
            kind: "path".into(),
            value: "/logs/project/2026-10-30_abc-123.jsonl".into(),
        });
        names.rename(&info, "Task").unwrap();
        let identity = info.agent_session.as_mut().unwrap();
        identity.source = "environment".into();
        identity.value = "/moved/logs/another-date_abc-123.jsonl".into();
        names.observe(&info, ManualNameLifetime::Conversation);
        info.agent_session.as_mut().unwrap().kind = "id".into();
        info.agent_session.as_mut().unwrap().value = "abc-123".into();
        names.observe(&info, ManualNameLifetime::Conversation);
        assert_eq!(names.title(&info), "Task");
    }

    #[test]
    fn carried_over_titles_stay_hidden_until_a_different_title_arrives() {
        let mut names = TerminalNames::default();
        let mut info = session(Some("A"), Some("Old task"));
        names.observe(&info, ManualNameLifetime::Keep);
        assert_eq!(names.title(&info), "Old task");
        info.agent_session.as_mut().unwrap().value = "B".into();
        names.observe(&info, ManualNameLifetime::Keep);
        assert_eq!(names.title(&info), "Codex");
        for title in [None, Some("Old task"), Some("Codex"), Some("Old task")] {
            info.conversation_title = title.map(str::to_owned);
            names.observe(&info, ManualNameLifetime::Keep);
            assert_eq!(names.title(&info), "Codex");
        }
        info.conversation_title = Some("New task".into());
        names.observe(&info, ManualNameLifetime::Keep);
        assert_eq!(names.title(&info), "New task");
    }

    #[test]
    fn relaunch_and_pane_remap_keep_names_and_identity_but_new_terminals_do_not_inherit() {
        let mut names = TerminalNames::default();
        let mut info = session(Some("A"), Some("Automatic"));
        names.observe(&info, ManualNameLifetime::Keep);
        names.rename(&info, "Manual").unwrap();
        let json = serde_json::to_string(&names.saved()).unwrap();
        assert!(!json.contains("Automatic"), "automatic titles are transient");
        let mut restored = TerminalNames::restore(serde_json::from_str(&json).unwrap());
        info.id = PaneId("remapped".into());
        restored.observe(&info, ManualNameLifetime::Conversation);
        assert_eq!(restored.title(&info), "Manual");
        let mut replacement = info.clone();
        replacement.terminal = TerminalId("replacement".into());
        assert_eq!(restored.title(&replacement), "Automatic");
        info.agent_session.as_mut().unwrap().value = "B".into();
        restored.observe(&info, ManualNameLifetime::Conversation);
        assert!(restored.manual(&info.terminal.0).is_none());
        names.retire(&info.terminal.0);
        assert!(names.saved().terminals.is_empty());
    }

    #[test]
    fn unsupported_saved_versions_and_invalid_names_are_rejected() {
        let mut saved = SavedNames::default();
        saved.version = 2;
        assert!(saved.validate().is_err());
        saved.version = 1;
        saved
            .terminals
            .insert("terminal".into(), ManualName { name: "bad\nname".into(), conversation: None });
        assert!(saved.validate().is_err());
    }
}
