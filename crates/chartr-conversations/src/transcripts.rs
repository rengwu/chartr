use crate::{Message, NativeSession, Provider, Role};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

mod kimi;
mod titles;

const MAX_TRANSCRIPT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_MESSAGES: usize = 300;

#[derive(Clone, Debug)]
pub struct ProviderPaths {
    pub codex: PathBuf,
    pub claude: PathBuf,
    pub opencode: PathBuf,
    pub pi: PathBuf,
    pub kimi: PathBuf,
    pub grok: PathBuf,
    pub omp: PathBuf,
    pub cursor: PathBuf,
    pub antigravity: PathBuf,
    pub antigravity_cli: PathBuf,
}

impl ProviderPaths {
    /// Locate the original log for an external editor, without parsing or truncating it.
    pub fn session_log(&self, provider: Provider, native: &NativeSession) -> Result<PathBuf> {
        validate_native_id(&native.id)?;
        let path = match provider {
            Provider::Codex => find_transcript_in_roots(
                &[self.codex.join("sessions"), self.codex.join("archived_sessions")],
                &native.id,
                provider,
            )?,
            Provider::Claude => {
                find_exact_transcript(&self.claude.join("projects"), &native.id, provider)?
            }
            Provider::Pi | Provider::Omp => match &native.path {
                Some(path) => {
                    ensure!(
                        path.is_absolute(),
                        "The provider must report an absolute session path"
                    );
                    ensure!(
                        NativeSession::from_identity(provider, "path", &path.to_string_lossy())
                            .is_some_and(|session| session.id == native.id),
                        "The reported log path does not match the session ID"
                    );
                    path.clone()
                }
                None => {
                    let root = if provider == Provider::Pi { &self.pi } else { &self.omp };
                    find_exact_transcript(&root.join("sessions"), &native.id, provider)?
                }
            },
            Provider::Kimi => kimi::session_log(self, native)?,
            Provider::OpenCode => anyhow::bail!(
                "OpenCode stores chats in a database; no individual session log file is available."
            ),
            Provider::Grok => {
                find_exact_transcript(&self.grok.join("sessions"), &native.id, provider)?
            }
            Provider::Cursor => {
                find_exact_transcript(&self.cursor.join("projects"), &native.id, provider)?
            }
            Provider::Antigravity => find_transcript_in_roots(
                &[self.antigravity.join("brain"), self.antigravity_cli.join("brain")],
                &native.id,
                provider,
            )?,
        };
        ensure!(path.is_file(), "The session log file is no longer available.");
        Ok(path)
    }

    pub fn from_environment() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        Self {
            kimi: std::env::var_os("KIMI_CODE_HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".kimi-code")),
            codex: std::env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".codex")),
            claude: std::env::var_os("CLAUDE_CONFIG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".claude")),
            opencode: data.join("opencode"),
            grok: environment_path("GROK_HOME", home.join(".grok"), &home),
            omp: environment_path(
                "PI_CODING_AGENT_DIR",
                environment_path("OMP_CONFIG_DIR", home.join(".omp"), &home).join("agent"),
                &home,
            ),
            cursor: environment_path("CURSOR_CONFIG_DIR", home.join(".cursor"), &home),
            antigravity: home.join(".gemini/antigravity"),
            antigravity_cli: home.join(".gemini/antigravity-cli"),
            pi: std::env::var_os("PI_CODING_AGENT_DIR")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .map(|path| {
                    path.strip_prefix("~").map(|rest| home.join(rest)).unwrap_or(path.clone())
                })
                .unwrap_or_else(|| home.join(".pi/agent")),
        }
    }

    pub(crate) fn namespace(&self, provider: Provider) -> String {
        match provider {
            Provider::Codex => &self.codex,
            Provider::Claude => &self.claude,
            Provider::OpenCode => &self.opencode,
            Provider::Grok => return "local:grok".to_owned(),
            Provider::Kimi => return "local:kimi".to_owned(),
            Provider::Pi => &self.pi,
            Provider::Omp => &self.omp,
            Provider::Cursor => &self.cursor,
            Provider::Antigravity => &self.antigravity_cli,
        }
        .to_string_lossy()
        .into_owned()
    }
}

fn environment_path(name: &str, fallback: PathBuf, home: &Path) -> PathBuf {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| {
            path.strip_prefix("~")
                .map(|rest| home.join(rest))
                .unwrap_or_else(|_| if path.is_absolute() { path } else { home.join(path) })
        })
        .unwrap_or(fallback)
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Transcript {
    pub title: Option<String>,
    /// Prompt excerpts must never replace a provider's own title.
    pub title_is_fallback: bool,
    pub messages: Vec<Message>,
    /// Provider-recorded time of the last submitted prompt, not output activity.
    pub updated: Option<u64>,
}

#[derive(Default)]
pub(crate) struct Reader {
    paths: HashMap<(Provider, String), PathBuf>,
    cache: HashMap<(Provider, String, PathBuf), (SystemTime, u64, Transcript)>,
    title_indexes: HashMap<PathBuf, (SystemTime, u64, HashMap<String, String>)>,
}

impl Reader {
    pub fn read(
        &mut self,
        provider: Provider,
        native: &NativeSession,
        paths: &ProviderPaths,
    ) -> Result<Transcript> {
        validate_native_id(&native.id)?;
        if provider == Provider::OpenCode {
            return read_opencode(&paths.opencode.join("opencode.db"), &native.id);
        }
        if provider == Provider::Kimi {
            return self.read_kimi(native, paths);
        }
        if matches!(provider, Provider::Grok | Provider::Cursor | Provider::Antigravity) {
            // These providers supply titles through the observed terminal. A local
            // transcript reader is not required for Inbox detection or terminal use.
            return Ok(Transcript::default());
        }
        let root = match provider {
            Provider::Claude => paths.claude.join("projects"),
            Provider::Codex => paths.codex.join("sessions"),
            Provider::Pi => paths.pi.join("sessions"),
            Provider::Omp => paths.omp.join("sessions"),
            _ => unreachable!(),
        };
        let key = (provider, native.id.clone());
        let path = if let Some(path) =
            native.path.as_ref().filter(|_| matches!(provider, Provider::Pi | Provider::Omp))
        {
            ensure!(path.is_absolute(), "The provider must report an absolute session path");
            path.clone()
        } else if let Some(path) = self.paths.get(&key).filter(|p| p.is_file()) {
            path.clone()
        } else {
            let path = find_exact_transcript(&root, &native.id, provider)?;
            self.paths.insert(key, path.clone());
            path
        };
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let cache_key = (provider, native.id.clone(), path.clone());
        if let Some((time, len, transcript)) = self.cache.get(&cache_key)
            && *time == modified
            && *len == metadata.len()
        {
            return Ok(transcript.clone());
        }
        ensure!(
            metadata.len() <= MAX_TRANSCRIPT_BYTES,
            "This transcript is too large for the current history reader. Continue in terminal."
        );
        let file = fs::File::open(&path)?;
        let transcript =
            parse_jsonl(BufReader::new(file.take(MAX_TRANSCRIPT_BYTES)), provider, &native.id)?;
        self.cache.insert(cache_key, (modified, metadata.len(), transcript.clone()));
        Ok(transcript)
    }
}

fn find_exact_transcript(root: &Path, id: &str, provider: Provider) -> Result<PathBuf> {
    find_transcript_in_roots(&[root.to_owned()], id, provider)
}

pub(crate) fn validate_native_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty()
            && !id.starts_with('-')
            && id.len() <= 256
            && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
        "Unrecognized native conversation ID"
    );
    Ok(())
}

fn find_transcript_in_roots(roots: &[PathBuf], id: &str, provider: Provider) -> Result<PathBuf> {
    let mut directories: Vec<_> =
        roots.iter().filter(|root| root.is_dir()).map(|root| (root.clone(), 0)).collect();
    let mut matches = Vec::new();
    let mut visited = 0;
    while let Some((directory, depth)) = directories.pop() {
        for entry in fs::read_dir(directory).context(
            "Conversation history is unavailable; enable the provider integration before launching",
        )? {
            let entry = entry?;
            visited += 1;
            ensure!(visited <= 50_000, "Conversation history index is too large to scan safely");
            let kind = entry.file_type()?;
            if kind.is_dir() && depth < 4 {
                directories.push((entry.path(), depth + 1));
            }
            if !kind.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let matches_id = match provider {
                Provider::Claude => name == format!("{id}.jsonl"),
                Provider::Codex => {
                    name.starts_with("rollout-") && name.ends_with(&format!("-{id}.jsonl"))
                }
                Provider::Pi | Provider::Omp => name.ends_with(&format!("_{id}.jsonl")),
                Provider::Grok => {
                    depth == 2
                        && name == "updates.jsonl"
                        && entry
                            .path()
                            .parent()
                            .and_then(Path::file_name)
                            .is_some_and(|name| name == id)
                }
                Provider::Cursor => cursor_log_matches(&entry.path(), id),
                Provider::Antigravity => entry
                    .path()
                    .ends_with(Path::new(id).join(".system_generated/logs/transcript.jsonl")),
                _ => false,
            };
            if matches_id {
                matches.push(entry.path());
            }
        }
    }
    ensure!(matches.len() == 1, "No unique session log file was found for this conversation.");
    Ok(matches.remove(0))
}

fn cursor_log_matches(path: &Path, id: &str) -> bool {
    let Some(parent) = path.parent() else { return false };
    let file_matches = path.file_name().is_some_and(|name| {
        name == format!("{id}.jsonl").as_str() || name == format!("{id}.txt").as_str()
    });
    file_matches
        && (parent.file_name().is_some_and(|name| name == "agent-transcripts")
            || (parent.file_name().is_some_and(|name| name == id)
                && parent
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|name| name == "agent-transcripts")))
}

fn parse_jsonl(reader: impl BufRead, provider: Provider, native_id: &str) -> Result<Transcript> {
    let mut result = Transcript::default();
    let mut seen = HashSet::new();
    let mut native_verified = false;
    let mut codex_user_events = false;
    let mut custom_title = None;
    let mut omp_header_title = None;
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        // A provider may be in the middle of appending the final JSON line.
        let Ok(value) = serde_json::from_str::<Value>(&line) else { continue };
        if let Some(id) = value["sessionId"].as_str() {
            ensure!(id == native_id, "Transcript belongs to another conversation");
            native_verified = true;
        }
        if value["type"] == "session_meta" {
            ensure!(
                value["payload"]["id"] == native_id,
                "Transcript belongs to another conversation"
            );
            native_verified = true;
        }
        if provider == Provider::Claude {
            if value["type"] == "custom-title" {
                custom_title = titles::non_blank(&value["customTitle"]);
            }
            if value["type"] == "ai-title" {
                result.title = titles::non_blank(&value["aiTitle"]);
            }
        }
        if matches!(provider, Provider::Pi | Provider::Omp) {
            if value["type"] == "session" {
                ensure!(value["id"] == native_id, "Transcript belongs to another conversation");
                native_verified = true;
                if provider == Provider::Omp {
                    result.title = titles::non_blank(&value["title"]);
                }
            }
            if provider == Provider::Omp {
                if value["type"] == "title" {
                    // OMP rewrites this leading record with the current title.
                    omp_header_title = titles::non_blank(&value["title"]);
                }
                if value["type"] == "title_change" {
                    result.title = titles::non_blank(&value["title"]);
                }
            }
            if value["type"] == "session_info" {
                // Pi uses the latest name, including explicit clears, across branches.
                result.title = value["name"]
                    .as_str()
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned);
            }
        }
        // Codex's model-visible response_items also contain injected AGENTS.md,
        // environment and permission context with role=user. Its user_message
        // events identify what the human actually submitted.
        if provider == Provider::Codex
            && value["type"] == "event_msg"
            && value["payload"]["type"] == "user_message"
        {
            if !codex_user_events {
                result.messages.retain(|m| m.role != Role::User);
                codex_user_events = true;
            }
            if let Some(text) = value["payload"]["message"].as_str().filter(|s| !s.is_empty()) {
                let id = value["payload"]["id"]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("user-event-{index}"));
                if seen.insert(id.clone()) {
                    result.messages.push(Message {
                        id,
                        role: Role::User,
                        text: text.into(),
                        complete: true,
                    });
                }
            }
            continue;
        }
        let (item, role, id) = match provider {
            Provider::Claude => {
                if value["isSidechain"] == true {
                    continue;
                }
                let role = match value["type"].as_str() {
                    Some("user") => Role::User,
                    Some("assistant") => Role::Assistant,
                    _ => continue,
                };
                (
                    &value["message"],
                    role,
                    value["uuid"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("line-{index}")),
                )
            }
            Provider::Codex => {
                if value["type"] != "response_item" {
                    continue;
                }
                let item = &value["payload"];
                let role = match item["role"].as_str() {
                    Some("user") if !codex_user_events => Role::User,
                    Some("assistant") => Role::Assistant,
                    _ => continue,
                };
                (
                    item,
                    role,
                    item["id"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("line-{index}")),
                )
            }
            Provider::Pi | Provider::Omp => {
                if value["type"] != "message" {
                    continue;
                }
                let item = &value["message"];
                let role = match item["role"].as_str() {
                    Some("user") => Role::User,
                    Some("assistant") => Role::Assistant,
                    _ => continue,
                };
                (
                    item,
                    role,
                    value["id"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("line-{index}")),
                )
            }
            _ => continue,
        };
        let content = &item["content"];
        let text = if let Some(text) = content.as_str() {
            text.to_owned()
        } else {
            content
                .as_array()
                .into_iter()
                .flatten()
                .filter(|part| {
                    matches!(part["type"].as_str(), Some("text" | "input_text" | "output_text"))
                })
                .filter_map(|part| part["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n\n")
        };
        if text.is_empty() {
            continue;
        }
        if provider == Provider::Codex
            && role == Role::User
            && (text.trim_start().starts_with("<environment_context>")
                || text.trim_start().starts_with("# AGENTS.md instructions for "))
        {
            continue;
        }
        if seen.insert(id.clone()) {
            result.messages.push(Message { id, role, text, complete: true });
        }
    }
    ensure!(native_verified, "The transcript did not verify its native conversation identity");
    result.title = custom_title.or(omp_header_title).or(result.title);
    if result.title.is_none() {
        result.title = crate::prompt_title(&result.messages);
        result.title_is_fallback = true;
    }
    bound_messages(&mut result.messages);
    Ok(result)
}

pub(crate) fn bound_messages(messages: &mut Vec<Message>) {
    if messages.len() > MAX_MESSAGES {
        messages.drain(..messages.len() - MAX_MESSAGES);
    }
    for message in messages {
        if message.text.len() > 64 * 1024 {
            message.text = message.text.chars().take(16_000).collect::<String>()
                + "\n… Open the terminal for the full output.";
        }
    }
}

fn read_opencode(path: &Path, id: &str) -> Result<Transcript> {
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    db.busy_timeout(std::time::Duration::from_millis(200))?;
    let title =
        db.query_row("SELECT title FROM session WHERE id = ?1", [id], |r| r.get::<_, String>(0))?;
    let mut query = db.prepare("SELECT id, data FROM message WHERE session_id = ?1 ORDER BY time_created DESC, id DESC LIMIT 300")?;
    let rows = query
        .query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut data = Vec::new();
    for (message_id, raw) in rows.into_iter().rev() {
        let mut info: Value = serde_json::from_str(&raw)?;
        info["id"] = json!(message_id);
        let mut parts =
            db.prepare("SELECT id, data FROM part WHERE message_id = ?1 ORDER BY id")?;
        let parts = parts
            .query_map([&message_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let parts = parts
            .into_iter()
            .map(|(id, raw)| {
                let mut part: Value = serde_json::from_str(&raw)?;
                part["id"] = json!(id);
                Ok(part)
            })
            .collect::<Result<Vec<_>>>()?;
        data.push(json!({"info": info, "parts": parts}));
    }
    let mut messages = parse_opencode_messages(&json!(data))?;
    bound_messages(&mut messages);
    let title_is_fallback = title.trim().is_empty()
        || title.starts_with("New session - ")
        || title.starts_with("Child session - ");
    let title = if title_is_fallback { crate::prompt_title(&messages) } else { Some(title) };
    Ok(Transcript { title, title_is_fallback, messages, updated: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn log_paths(root: &Path) -> ProviderPaths {
        ProviderPaths {
            codex: root.join("codex"),
            claude: root.join("claude"),
            opencode: root.join("opencode"),
            pi: root.join("pi"),
            kimi: root.join("kimi"),
            grok: root.join("grok"),
            omp: root.join("omp"),
            cursor: root.join("cursor"),
            antigravity: root.join("antigravity"),
            antigravity_cli: root.join("antigravity-cli"),
        }
    }

    #[test]
    #[ignore = "Requires existing local Grok sessions; locates files without reading their contents"]
    fn installed_grok_log_matches_its_directory_identity() {
        let paths = ProviderPaths::from_environment();
        for group in fs::read_dir(paths.grok.join("sessions")).unwrap() {
            let group = group.unwrap();
            if !group.file_type().unwrap().is_dir() {
                continue;
            }
            for session in fs::read_dir(group.path()).unwrap() {
                let session = session.unwrap();
                let log = session.path().join("updates.jsonl");
                if !log.is_file() {
                    continue;
                }
                let native =
                    NativeSession { id: session.file_name().to_str().unwrap().into(), path: None };
                assert_eq!(paths.session_log(Provider::Grok, &native).unwrap(), log);
                return;
            }
        }
        panic!("A local Grok session is required");
    }

    #[test]
    fn additional_provider_logs_match_exact_sessions_and_survive_large_files() {
        let dir = tempfile::tempdir().unwrap();
        let paths = log_paths(dir.path());
        let native = NativeSession { id: "session-a".into(), path: None };
        for (provider, relative) in [
            (Provider::Omp, "omp/sessions/project/2026-09-14_session-a.jsonl"),
            (Provider::Grok, "grok/sessions/project/session-a/updates.jsonl"),
            (
                Provider::Cursor,
                "cursor/projects/project/agent-transcripts/session-a/session-a.jsonl",
            ),
            (
                Provider::Antigravity,
                "antigravity-cli/brain/session-a/.system_generated/logs/transcript.jsonl",
            ),
        ] {
            let file = dir.path().join(relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::File::create(&file).unwrap().set_len(MAX_TRANSCRIPT_BYTES + 1).unwrap();
            assert_eq!(paths.session_log(provider, &native).unwrap(), file);
            let other = NativeSession { id: "session-b".into(), path: None };
            assert!(paths.session_log(provider, &other).is_err());
        }
        // OMP's reported path also supports profiles, symlinked projects and custom session roots.
        let custom = dir.path().join("custom profile/2026-09-14_session-a.jsonl");
        fs::create_dir_all(custom.parent().unwrap()).unwrap();
        fs::write(&custom, "").unwrap();
        let reported =
            NativeSession::from_identity(Provider::Omp, "path", custom.to_str().unwrap()).unwrap();
        assert_eq!(paths.session_log(Provider::Omp, &reported).unwrap(), custom);
        let mismatched = NativeSession { id: "session-b".into(), path: Some(custom) };
        assert!(paths.session_log(Provider::Omp, &mismatched).is_err());
    }

    #[test]
    fn cursor_legacy_logs_are_supported_but_subagents_and_duplicates_are_not_selected() {
        let dir = tempfile::tempdir().unwrap();
        let paths = log_paths(dir.path());
        let native = NativeSession { id: "session-a".into(), path: None };
        let root = paths.cursor.join("projects/project/agent-transcripts");
        let subagent = root.join("parent/subagents/session-a.jsonl");
        fs::create_dir_all(subagent.parent().unwrap()).unwrap();
        fs::write(subagent, "").unwrap();
        assert!(paths.session_log(Provider::Cursor, &native).is_err());
        let legacy = root.join("session-a.txt");
        fs::write(&legacy, "").unwrap();
        assert_eq!(paths.session_log(Provider::Cursor, &native).unwrap(), legacy);
        fs::write(root.join("session-a.jsonl"), "").unwrap();
        assert!(paths.session_log(Provider::Cursor, &native).is_err());
    }

    #[test]
    fn antigravity_checks_both_roots_and_rejects_ambiguous_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let paths = log_paths(dir.path());
        let native = NativeSession { id: "session-a".into(), path: None };
        for root in [&paths.antigravity, &paths.antigravity_cli] {
            let file = root.join("brain/session-a/.system_generated/logs/transcript.jsonl");
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, "").unwrap();
        }
        assert!(paths.session_log(Provider::Antigravity, &native).is_err());
        for id in ["../session-a", "--help", "", "session/a"] {
            let invalid = NativeSession { id: id.into(), path: None };
            assert!(paths.session_log(Provider::Antigravity, &invalid).is_err());
        }
    }

    #[test]
    fn session_log_finds_archived_codex_logs_and_rejects_ambiguous_matches() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let paths = ProviderPaths {
            codex: root.to_owned(),
            claude: root.to_owned(),
            opencode: root.to_owned(),
            pi: root.to_owned(),
            kimi: root.to_owned(),
            grok: root.to_owned(),
            omp: root.to_owned(),
            cursor: root.to_owned(),
            antigravity: root.join("antigravity"),
            antigravity_cli: root.join("antigravity-cli"),
        };
        let native = NativeSession { id: "session-a".into(), path: None };
        assert!(paths.session_log(Provider::Codex, &native).is_err());
        fs::create_dir(root.join("archived_sessions")).unwrap();
        let archived = root.join("archived_sessions/rollout-2026-09-14-session-a.jsonl");
        // Opening the original file must not depend on the history reader's size limit.
        fs::File::create(&archived).unwrap().set_len(MAX_TRANSCRIPT_BYTES + 1).unwrap();
        assert_eq!(paths.session_log(Provider::Codex, &native).unwrap(), archived);
        fs::create_dir_all(root.join("sessions/2026/09/14")).unwrap();
        fs::write(root.join("sessions/2026/09/14/rollout-2026-09-14-session-a.jsonl"), "").unwrap();
        assert!(paths.session_log(Provider::Codex, &native).is_err());
        assert!(paths.session_log(Provider::OpenCode, &native).is_err());
        assert!(paths.session_log(Provider::Grok, &native).is_err());
        let invalid = NativeSession { id: "../session-a".into(), path: None };
        assert!(paths.session_log(Provider::Kimi, &invalid).is_err());
    }

    #[test]
    fn pi_uses_first_prompt_or_latest_session_name_and_verifies_identity() {
        let data = concat!(
            "{\"type\":\"session\",\"version\":3,\"id\":\"pi-a\",\"cwd\":\"/same/project\"}\n",
            "{\"type\":\"message\",\"id\":\"m1\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"image\"},{\"type\":\"text\",\"text\":\"hi! tell me some stuff about axolotls\"}]}}\n",
            "{\"type\":\"message\",\"id\":\"m2\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"private\"},{\"type\":\"text\",\"text\":\"Axolotls are salamanders.\"}]}}\n",
            "{\"type\":\"message\",\"id\":\"m3\",\"message\":{\"role\":\"user\",\"content\":\"A follow-up\"}}\n",
        );
        let transcript = parse_jsonl(data.as_bytes(), Provider::Pi, "pi-a").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("hi! tell me some stuff about axolotls"));
        assert_eq!(transcript.messages.len(), 3);
        assert_eq!(transcript.messages[1].text, "Axolotls are salamanders.");
        assert!(parse_jsonl(data.as_bytes(), Provider::Pi, "pi-b").is_err());
        assert!(
            parse_jsonl(
                b"{\"type\":\"session_info\",\"name\":\"Wrong\"}".as_slice(),
                Provider::Pi,
                "pi-a"
            )
            .is_err()
        );

        let named = format!(
            "{data}{{\"type\":\"session_info\",\"name\":\"Old name\"}}\n{{\"type\":\"session_info\",\"name\":\" Axolotls \"}}\n{{\"type\":"
        );
        assert_eq!(
            parse_jsonl(named.as_bytes(), Provider::Pi, "pi-a").unwrap().title.as_deref(),
            Some("Axolotls")
        );
        let cleared = format!("{named}\n{{\"type\":\"session_info\",\"name\":\"\"}}\n");
        assert_eq!(
            parse_jsonl(cleared.as_bytes(), Provider::Pi, "pi-a").unwrap().title,
            transcript.title
        );
    }

    #[test]
    fn pi_reads_only_the_reported_session_and_rechecks_identity_before_cache_hits() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ProviderPaths {
            codex: dir.path().join("codex"),
            claude: dir.path().join("claude"),
            opencode: dir.path().join("opencode"),
            pi: dir.path().join("pi"),
            kimi: dir.path().join("kimi"),
            grok: dir.path().join("grok"),
            omp: dir.path().join("omp"),
            cursor: dir.path().join("cursor"),
            antigravity: dir.path().join("antigravity"),
            antigravity_cli: dir.path().join("antigravity-cli"),
        };
        let sessions = paths.pi.join("sessions/--same-project--");
        fs::create_dir_all(&sessions).unwrap();
        let file = sessions.join("2026-09-11T14-00-00-000Z_pi-a.jsonl");
        let other = sessions.join("2026-09-11T15-00-00-000Z_pi-b.jsonl");
        fs::write(&file, "{\"type\":\"session\",\"id\":\"pi-a\"}\n{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"Axolotls\"}}\n").unwrap();
        fs::write(&other, "{\"type\":\"session\",\"id\":\"pi-b\"}\n{\"type\":\"session_info\",\"name\":\"Dinner\"}\n").unwrap();
        let mut reader = Reader::default();
        let mut native =
            NativeSession::from_identity(Provider::Pi, "path", file.to_str().unwrap()).unwrap();
        assert_eq!(
            reader.read(Provider::Pi, &native, &paths).unwrap().title.as_deref(),
            Some("Axolotls")
        );
        native.id = "pi-b".into();
        assert!(reader.read(Provider::Pi, &native, &paths).is_err());
        native.path = None;
        assert_eq!(
            reader.read(Provider::Pi, &native, &paths).unwrap().title.as_deref(),
            Some("Dinner")
        );
        native.id = "missing".into();
        assert!(reader.read(Provider::Pi, &native, &paths).is_err());
        // A reported path outside the default root is authoritative, but its header must match.
        let custom = dir.path().join("custom_pi-a.jsonl");
        fs::rename(&file, &custom).unwrap();
        native =
            NativeSession::from_identity(Provider::Pi, "path", custom.to_str().unwrap()).unwrap();
        assert_eq!(
            reader.read(Provider::Pi, &native, &paths).unwrap().title.as_deref(),
            Some("Axolotls")
        );
    }

    #[test]
    fn claude_deduplicates_and_ignores_incomplete_tail_and_subagents() {
        let data = "{\"sessionId\":\"a\",\"type\":\"user\",\"uuid\":\"u\",\"message\":{\"content\":\"Fix login\"}}\n{\"sessionId\":\"a\",\"type\":\"user\",\"uuid\":\"u\",\"message\":{\"content\":\"Fix login\"}}\n{\"sessionId\":\"a\",\"type\":\"assistant\",\"isSidechain\":true,\"message\":{\"content\":\"child\"}}\n{\"partial\":";
        let transcript = parse_jsonl(data.as_bytes(), Provider::Claude, "a").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("Fix login"));
        assert_eq!(transcript.messages.len(), 1);
        assert!(parse_jsonl(data.as_bytes(), Provider::Claude, "b").is_err());
    }

    #[test]
    fn codex_uses_response_items_not_duplicate_event_messages() {
        let data = "{\"type\":\"session_meta\",\"payload\":{\"id\":\"c\"}}\n{\"type\":\"response_item\",\"payload\":{\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Done\"}]}}\n{\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_message\",\"message\":\"Done\"}}\n";
        assert_eq!(parse_jsonl(data.as_bytes(), Provider::Codex, "c").unwrap().messages.len(), 1);
    }

    #[test]
    fn codex_user_events_exclude_injected_context_but_preserve_user_quoted_xml() {
        let data = [
            json!({"type":"session_meta","payload":{"id":"c"}}),
            json!({"type":"response_item","payload":{"role":"user","content":[{"type":"input_text","text":"<environment_context>injected</environment_context>"}]}}),
            json!({"type":"event_msg","payload":{"type":"user_message","message":"hi"}}),
            json!({"type":"response_item","payload":{"role":"user","content":[{"type":"input_text","text":"hi"}]}}),
            json!({"type":"event_msg","payload":{"type":"user_message","message":"<environment_context>user quoted</environment_context>"}}),
        ].into_iter().map(|v|v.to_string()).collect::<Vec<_>>().join("\n");
        let transcript = parse_jsonl(data.as_bytes(), Provider::Codex, "c").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("hi"));
        assert_eq!(transcript.messages.len(), 2);
        assert!(transcript.messages[1].text.contains("user quoted"));
    }

    #[test]
    fn claude_native_names_prefer_custom_over_ai_and_ignore_blank_names() {
        let data = [
            json!({"sessionId":"a","type":"user","uuid":"u","message":{"content":"Raw prompt"}}),
            json!({"sessionId":"a","type":"ai-title","aiTitle":"Generated title"}),
            json!({"sessionId":"a","type":"custom-title","customTitle":"Named in Claude"}),
            json!({"sessionId":"a","type":"ai-title","aiTitle":"Later generated title"}),
        ]
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let transcript = parse_jsonl(data.as_bytes(), Provider::Claude, "a").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("Named in Claude"));
        assert!(!transcript.title_is_fallback);
        let cleared = format!(
            "{data}\n{}",
            json!({"sessionId":"a","type":"custom-title","customTitle":"  "})
        );
        let transcript = parse_jsonl(cleared.as_bytes(), Provider::Claude, "a").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("Later generated title"));
        let unnamed =
            format!("{cleared}\n{}", json!({"sessionId":"a","type":"ai-title","aiTitle":""}));
        let transcript = parse_jsonl(unnamed.as_bytes(), Provider::Claude, "a").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("Raw prompt"));
        assert!(transcript.title_is_fallback);
        assert!(parse_jsonl(data.as_bytes(), Provider::Claude, "other").is_err());
    }

    #[test]
    fn omp_current_header_title_wins_over_historical_title_entries() {
        let data = [
            json!({"type":"title","title":"Current generated title","source":"auto"}),
            json!({"type":"session","id":"a","title":"Original title"}),
            json!({"type":"message","message":{"role":"user","content":"Raw prompt"}}),
            json!({"type":"title_change","title":"Older branch title"}),
        ]
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let transcript = parse_jsonl(data.as_bytes(), Provider::Omp, "a").unwrap();
        assert_eq!(transcript.title.as_deref(), Some("Current generated title"));
        assert!(!transcript.title_is_fallback);
        assert!(parse_jsonl(data.as_bytes(), Provider::Omp, "other").is_err());
        let legacy = data.lines().skip(1).collect::<Vec<_>>().join("\n");
        assert_eq!(
            parse_jsonl(legacy.as_bytes(), Provider::Omp, "a").unwrap().title.as_deref(),
            Some("Older branch title")
        );
    }

    #[test]
    fn opencode_placeholder_titles_do_not_replace_observed_titles() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("opencode.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE session(id TEXT, title TEXT); CREATE TABLE message(id TEXT, session_id TEXT, time_created INTEGER, data TEXT); CREATE TABLE part(id TEXT, message_id TEXT, data TEXT);
            INSERT INTO session VALUES ('a', 'New session - 2026-09-11');").unwrap();
        assert!(read_opencode(&path, "a").unwrap().title.is_none());
        db.execute("INSERT INTO message VALUES ('u', 'a', 1, ?1)", [r#"{"role":"user"}"#]).unwrap();
        db.execute(
            "INSERT INTO part VALUES ('p', 'u', ?1)",
            [r#"{"type":"text","text":"Fix the sidebar"}"#],
        )
        .unwrap();
        assert_eq!(read_opencode(&path, "a").unwrap().title.as_deref(), Some("Fix the sidebar"));
        db.execute("UPDATE session SET title = 'Provider-generated title'", []).unwrap();
        assert_eq!(
            read_opencode(&path, "a").unwrap().title.as_deref(),
            Some("Provider-generated title")
        );
    }

    #[test]
    fn lookup_requires_exact_unique_id() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("a.jsonl"), "").unwrap();
        fs::write(root.path().join("newest-b.jsonl"), "").unwrap();
        assert_eq!(
            find_exact_transcript(root.path(), "a", Provider::Claude).unwrap(),
            root.path().join("a.jsonl")
        );
        assert!(find_exact_transcript(root.path(), "b", Provider::Claude).is_err());
    }
}

fn parse_opencode_messages(value: &Value) -> Result<Vec<Message>> {
    let records = value.as_array().context("Unsupported OpenCode message format")?;
    let mut messages = Vec::new();
    for record in records {
        let info = &record["info"];
        let Some(id) = info["id"].as_str() else { continue };
        let role = match info["role"].as_str() {
            Some("user") => Role::User,
            Some("assistant") => Role::Assistant,
            _ => continue,
        };
        let Some(parts) = record["parts"].as_array() else { continue };
        let text = parts
            .iter()
            .filter(|p| p["type"] == "text" && p["synthetic"] != true)
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        if !text.is_empty() {
            messages.push(Message {
                id: id.to_owned(),
                role,
                text,
                complete: role == Role::User || info["time"]["completed"].is_number(),
            });
        }
        for part in parts.iter().filter(|p| p["type"] == "tool") {
            let state = &part["state"];
            let name = state["title"].as_str().or(part["tool"].as_str()).unwrap_or("Tool");
            let output = state["output"].as_str().or(state["error"].as_str()).unwrap_or("");
            messages.push(Message {
                id: part["id"].as_str().unwrap_or(id).to_owned(),
                role: Role::Tool,
                text: format!("{name}\n{output}"),
                complete: matches!(state["status"].as_str(), Some("completed" | "error")),
            });
        }
    }
    Ok(messages)
}
