use crate::{Message, Request, Role, Status};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

/// A second client of an existing local OpenCode TUI server, never a new agent.
#[derive(Clone, Debug)]
pub struct OpenCode {
    endpoint: String,
    directory: String,
}

impl OpenCode {
    /// OpenCode orders turns by its 48-bit timestamp prefix. Arbitrary IDs
    /// satisfy its schema but can make an answered user message look newer
    /// than every assistant message, causing the agent loop to repeat.
    pub fn message_id() -> String {
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed) & 0xffffff;
        let encoded = (crate::now_millis() << 12) & 0xffffffffffff;
        format!("msg_{encoded:012x}{:08x}{seq:06x}", std::process::id())
    }
    pub fn new(endpoint: &str, directory: &Path) -> Result<Self> {
        let url = url::Url::parse(endpoint)?;
        let local = match url.host() {
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            _ => false,
        };
        ensure!(
            url.scheme() == "http"
                && local
                && url.username().is_empty()
                && url.password().is_none(),
            "Only the agent's local loopback endpoint is supported"
        );
        ensure!(
            url.path() == "/" && url.query().is_none() && url.fragment().is_none(),
            "Invalid local agent endpoint"
        );
        Ok(Self {
            endpoint: url.as_str().trim_end_matches('/').to_owned(),
            directory: directory
                .canonicalize()
                .unwrap_or_else(|_| directory.to_owned())
                .to_string_lossy()
                .into_owned(),
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn request(&self, path: &str, body: Option<Value>) -> Result<Value> {
        let mut url = url::Url::parse(&format!("{}{path}", self.endpoint))?;
        url.query_pairs_mut().append_pair("directory", &self.directory);
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .proxy(None)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(if path == "/session" { 15 } else { 3 })))
            .build()
            .into();
        let mut response = if let Some(body) = body {
            agent
                .post(url.as_str())
                .header("content-type", "application/json")
                .send(body.to_string())?
        } else {
            agent.get(url.as_str()).call()?
        };
        ensure!(response.status().is_success(), "The agent returned HTTP {}", response.status());
        let body = response.body_mut().with_config().limit(8 * 1024 * 1024).read_to_string()?;
        if body.is_empty() { Ok(Value::Null) } else { Ok(serde_json::from_str(&body)?) }
    }

    pub fn health(&self) -> Result<()> {
        let health = self.request("/global/health", None)?;
        ensure!(
            health["healthy"] == true && health["version"].is_string(),
            "The endpoint is not an OpenCode server"
        );
        Ok(())
    }

    pub fn create(&self) -> Result<String> {
        let session = self.request("/session", Some(json!({})))?;
        let id = session["id"].as_str().context("Agent did not return a conversation ID")?;
        validate_id(id)?;
        Ok(id.to_owned())
    }

    pub fn select(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        self.request("/tui/select-session", Some(json!({"sessionID": id})))?;
        Ok(())
    }

    pub fn read(&self, id: &str) -> Result<(String, Status, Vec<Message>)> {
        validate_id(id)?;
        let session = self.request(&format!("/session/{id}"), None)?;
        ensure!(session["id"] == id, "The agent returned a different conversation");
        let states = self.request("/session/status", None)?;
        let status = match states[id]["type"].as_str() {
            Some("busy" | "retry") => Status::Working,
            _ => Status::Idle,
        };
        let data = self.request(&format!("/session/{id}/message?limit=300"), None)?;
        Ok((
            session["title"].as_str().unwrap_or("OpenCode conversation").to_owned(),
            status,
            parse_messages(&data)?,
        ))
    }

    pub fn send(&self, id: &str, message_id: &str, text: &str) -> Result<()> {
        self.send_with_options(id, message_id, text, None, None)
    }

    pub fn send_with_options(
        &self,
        id: &str,
        message_id: &str,
        text: &str,
        model: Option<&str>,
        agent: Option<&str>,
    ) -> Result<()> {
        validate_id(id)?;
        validate_id(message_id)?;
        ensure!(
            !text.trim().is_empty() && text.len() <= 1024 * 1024,
            "Message is empty or too large"
        );
        // Address the conversation, not the terminal's current input buffer.
        // The caller supplies an ID so an uncertain outcome can be reconciled.
        let mut body = json!({
            "messageID": message_id, "parts": [{"type": "text", "text": text}]
        });
        // The server otherwise falls back to its default agent on every turn.
        // Preserve the previous native user's model/agent, including TUI changes.
        let history = self.request(&format!("/session/{id}/message?limit=300"), None)?;
        if let Some(info) = history
            .as_array()
            .and_then(|messages| {
                messages.iter().rev().find(|message| message["info"]["role"] == "user")
            })
            .map(|message| &message["info"])
        {
            for field in ["model", "agent", "variant"] {
                if !info[field].is_null() {
                    body[field] = info[field].clone();
                }
            }
        }
        if let Some(model) = model {
            let (provider, model) =
                model.split_once('/').context("OpenCode model must be provider/model")?;
            ensure!(!provider.is_empty() && !model.is_empty(), "Invalid OpenCode model");
            body["model"] = json!({"providerID": provider, "modelID": model});
        }
        if let Some(agent) = agent {
            body["agent"] = json!(agent);
        }
        self.request(&format!("/session/{id}/prompt_async"), Some(body))?;
        Ok(())
    }

    pub fn ready(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        let session = self.request(&format!("/session/{id}"), None)?;
        ensure!(session["id"] == id, "Conversation identity changed");
        let states = self.request("/session/status", None)?;
        ensure!(
            !matches!(states[id]["type"].as_str(), Some("busy" | "retry")),
            "The agent is still working"
        );
        ensure!(self.pending(id)?.is_empty(), "Answer the agent's pending request first");
        Ok(())
    }

    pub fn pending(&self, id: &str) -> Result<Vec<Request>> {
        validate_id(id)?;
        let mut requests = Vec::new();
        let questions = self.request("/question", None)?;
        for item in questions
            .as_array()
            .context("Unsupported question format")?
            .iter()
            .filter(|q| q["sessionID"] == id)
        {
            let request_id = item["id"].as_str().context("Missing question ID")?.to_owned();
            let qs = item["questions"].as_array().context("Missing question contents")?;
            if qs.len() != 1 || qs[0]["multiple"] == true {
                requests.push(Request::TerminalRequired);
                continue;
            }
            let q = &qs[0];
            requests.push(Request::Question {
                id: request_id,
                prompt: q["question"].as_str().unwrap_or("Choose an answer").to_owned(),
                options: q["options"]
                    .as_array()
                    .context("Missing answer options")?
                    .iter()
                    .filter_map(|o| {
                        Some((
                            o["label"].as_str()?.to_owned(),
                            o["description"].as_str().unwrap_or("").to_owned(),
                        ))
                    })
                    .collect(),
            });
        }
        let permissions = self.request("/permission", None)?;
        for item in permissions
            .as_array()
            .context("Unsupported permission format")?
            .iter()
            .filter(|p| p["sessionID"] == id)
        {
            requests.push(Request::Permission {
                id: item["id"].as_str().context("Missing permission ID")?.to_owned(),
                permission: item["permission"].as_str().unwrap_or("Agent permission").to_owned(),
                patterns: item["patterns"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            });
        }
        Ok(requests)
    }

    /// Re-read the request before responding: an obsolete card never becomes
    /// a keystroke or an answer to a newer prompt.
    pub fn answer(&self, session: &str, expected: &Request, answer: &str) -> Result<()> {
        ensure!(
            self.pending(session)?.contains(expected),
            "This request was already answered or changed. Refresh the conversation."
        );
        match expected {
            Request::Question { id, options, .. } => {
                validate_id(id)?;
                ensure!(options.iter().any(|(label, _)| label == answer), "Unknown answer");
                self.request(
                    &format!("/question/{id}/reply"),
                    Some(json!({"answers":[[answer]]})),
                )?;
            }
            Request::Permission { id, .. } => {
                validate_id(id)?;
                ensure!(matches!(answer, "once" | "reject"), "Unsupported permission response");
                self.request(&format!("/permission/{id}/reply"), Some(json!({"reply":answer})))?;
            }
            Request::TerminalRequired => anyhow::bail!("Continue this request in terminal"),
        }
        Ok(())
    }

    pub fn interrupt(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        self.request(&format!("/session/{id}/abort"), Some(json!({})))?;
        Ok(())
    }
}

fn validate_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty()
            && id.len() <= 256
            && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
        "Invalid provider conversation ID"
    );
    Ok(())
}

pub(crate) fn parse_messages(value: &Value) -> Result<Vec<Message>> {
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

/// Discover only listeners belonging to the foreground process supplied by the
/// terminal owner. Never scan arbitrary local ports or guess from a directory.
pub fn endpoints_for_process(pid: u32) -> Result<Vec<String>> {
    ensure!(pid > 1, "Invalid agent process");
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/sbin/lsof")
            .args(["-a", "-nP", "-p", &pid.to_string(), "-iTCP", "-sTCP:LISTEN", "-Fn"])
            .output()
            .context("Inspecting the agent's local listener")?;
        Ok(parse_listeners(&String::from_utf8_lossy(&output.stdout)))
    }
    #[cfg(target_os = "linux")]
    {
        let mut sockets = std::collections::HashSet::new();
        for entry in std::fs::read_dir(format!("/proc/{pid}/fd"))?.flatten() {
            if let Ok(path) = std::fs::read_link(entry.path()) {
                if let Some(inode) = path
                    .to_string_lossy()
                    .strip_prefix("socket:[")
                    .and_then(|s| s.strip_suffix(']'))
                {
                    sockets.insert(inode.to_owned());
                }
            }
        }
        let mut endpoints = Vec::new();
        for line in std::fs::read_to_string(format!("/proc/{pid}/net/tcp"))?.lines().skip(1) {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() < 10 || fields[3] != "0A" || !sockets.contains(fields[9]) {
                continue;
            }
            if let Some((host, port)) = fields[1].split_once(':') {
                if host == "0100007F" {
                    if let Ok(port) = u16::from_str_radix(port, 16) {
                        endpoints.push(format!("http://127.0.0.1:{port}"));
                    }
                }
            }
        }
        Ok(endpoints)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    anyhow::bail!("Local endpoint discovery is unavailable on this platform")
}

fn parse_listeners(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let address = line.strip_prefix('n')?;
            let socket = address.parse::<std::net::SocketAddr>().ok()?;
            socket.ip().is_loopback().then(|| format!("http://{socket}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_ids_sort_before_the_response_and_are_distinct() {
        let before = crate::now_millis();
        let first = OpenCode::message_id();
        let second = OpenCode::message_id();
        let after = crate::now_millis();
        assert_ne!(first, second);
        assert_eq!(first.len(), 30);
        let encoded = u64::from_str_radix(&first[4..16], 16).unwrap();
        assert!((before..=after).any(|ms| ((ms << 12) & 0xffffffffffff) == encoded));
        let response = format!("msg_{:012x}aaaaaaaaaaaaaa", encoded + 1);
        assert!(first < response, "provider turn ordering must not repeat an answered prompt");
        assert!(OpenCode::new("http://[::1]:1234", Path::new("/tmp")).is_ok());
    }

    #[test]
    fn discovery_excludes_other_hosts_and_metadata() {
        assert_eq!(
            parse_listeners("p42\nn127.0.0.1:4301\nn*:8080\nn10.0.0.2:9000\nn[::1]:4302\n"),
            ["http://127.0.0.1:4301", "http://[::1]:4302"]
        );
    }

    #[test]
    fn endpoint_and_native_ids_cannot_redirect_requests() {
        for endpoint in [
            "http://example.com",
            "http://127.0.0.1@evil.test",
            "http://127.0.0.1/path",
            "https://127.0.0.1",
            "http://127.0.0.1/?x=y",
        ] {
            assert!(OpenCode::new(endpoint, Path::new("/tmp")).is_err());
        }
        for id in ["../other", "ses_1?x=y", "", "a/b"] {
            assert!(validate_id(id).is_err());
        }
    }
}
