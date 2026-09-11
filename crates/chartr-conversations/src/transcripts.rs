use crate::{Message, NativeSession, Provider, Role, opencode};
use anyhow::{Context, Result, bail, ensure};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

const MAX_TRANSCRIPT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_MESSAGES: usize = 300;

#[derive(Clone, Debug)]
pub struct ProviderPaths {
    pub codex: PathBuf,
    pub claude: PathBuf,
    pub opencode: PathBuf,
}

impl ProviderPaths {
    pub fn from_environment() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        Self {
            codex: std::env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".codex")),
            claude: std::env::var_os("CLAUDE_CONFIG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".claude")),
            opencode: data.join("opencode"),
        }
    }

    pub(crate) fn namespace(&self, provider: Provider) -> String {
        match provider {
            Provider::Codex => &self.codex,
            Provider::Claude => &self.claude,
            Provider::OpenCode => &self.opencode,
            Provider::Grok => return "local:grok".to_owned(),
        }
        .to_string_lossy()
        .into_owned()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Transcript {
    pub title: Option<String>,
    pub messages: Vec<Message>,
}

#[derive(Default)]
pub(crate) struct Reader {
    paths: HashMap<(Provider, String), PathBuf>,
    cache: HashMap<PathBuf, (SystemTime, u64, Transcript)>,
}

impl Reader {
    pub fn read(
        &mut self,
        provider: Provider,
        native: &NativeSession,
        paths: &ProviderPaths,
    ) -> Result<Transcript> {
        ensure!(
            !native.id.is_empty()
                && native.id.len() <= 256
                && native.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
            "Unrecognized native conversation ID"
        );
        if provider == Provider::OpenCode {
            return read_opencode(&paths.opencode.join("opencode.db"), &native.id);
        }
        if provider == Provider::Grok {
            bail!(
                "This Grok version has no verified transcript adapter yet. Continue in terminal."
            );
        }
        let root = match provider {
            Provider::Claude => paths.claude.join("projects"),
            Provider::Codex => paths.codex.join("sessions"),
            _ => unreachable!(),
        };
        let key = (provider, native.id.clone());
        let path = if let Some(path) = self.paths.get(&key).filter(|p| p.is_file()) {
            path.clone()
        } else {
            let path = find_exact_transcript(&root, &native.id, provider)?;
            self.paths.insert(key, path.clone());
            path
        };
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified()?;
        if let Some((time, len, transcript)) = self.cache.get(&path)
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
        self.cache.insert(path, (modified, metadata.len(), transcript.clone()));
        Ok(transcript)
    }
}

fn find_exact_transcript(root: &Path, id: &str, provider: Provider) -> Result<PathBuf> {
    let mut directories = vec![(root.to_owned(), 0)];
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
                _ => false,
            };
            if matches_id {
                matches.push(entry.path());
            }
        }
    }
    ensure!(
        matches.len() == 1,
        "No unique transcript was found for this native conversation. Continue in terminal."
    );
    Ok(matches.remove(0))
}

fn parse_jsonl(reader: impl BufRead, provider: Provider, native_id: &str) -> Result<Transcript> {
    let mut result = Transcript::default();
    let mut seen = HashSet::new();
    let mut native_verified = false;
    let mut codex_user_events = false;
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
        if value["type"] == "custom-title" {
            result.title = value["customTitle"].as_str().map(str::to_owned);
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
    if result.title.is_none() {
        result.title = crate::prompt_title(&result.messages);
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
    let mut messages = opencode::parse_messages(&json!(data))?;
    bound_messages(&mut messages);
    Ok(Transcript { title: Some(title), messages })
}

#[cfg(test)]
mod tests {
    use super::*;

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
