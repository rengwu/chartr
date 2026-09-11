use super::*;

/// Identity must come from a current Herdr observation, never a pane title.
#[derive(Clone, Debug)]
pub struct AgentInputTarget {
    pub pane: PaneId,
    pub terminal: TerminalId,
    pub provider: String,
    pub native_id: String,
    pub pid: u32,
}

#[derive(Debug)]
pub struct InputFailure {
    pub uncertain: bool,
    pub message: String,
}

impl Client {
    pub fn check_agent_input(&self, target: &AgentInputTarget) -> Result<()> {
        let client = self.until(Duration::from_secs(3));
        client.check_input_binding(target)?;
        let result: serde_json::Value = client.call("pane.read", &serde_json::json!({
            "pane_id":target.pane.0,"source":"visible","format":"ansi","strip_ansi":false,"lines":24
        }))?;
        let screen = result["read"]["text"]
            .as_str()
            .ok_or_else(|| Error::Protocol("The agent prompt could not be read".into()))?;
        if !empty_prompt(&target.provider, screen) {
            return Err(Error::Protocol("The terminal has a draft, menu, or unfinished prompt. Clear or finish it in terminal, then send again. Your chat draft is kept.".into()));
        }
        // Revalidate after reading the prompt: the pane may have changed agents.
        client.check_input_binding(target)
    }

    fn check_input_binding(&self, target: &AgentInputTarget) -> Result<()> {
        if !chartr_agent::Provider::from_slug(&target.provider).is_some_and(|provider| {
            provider.transport() == chartr_agent::MessageTransport::TerminalPrompt
        }) {
            return Err(Error::Protocol("This agent has no terminal message adapter".into()));
        }
        #[derive(serde::Deserialize)]
        struct ResultPane {
            pane: protocol::Pane,
        }
        let result: ResultPane =
            self.call("pane.get", &serde_json::json!({"pane_id":target.pane.0}))?;
        let pane = result.pane;
        if pane.terminal_id != target.terminal.0
            || pane.agent.as_deref() != Some(&target.provider)
            || !pane.agent_session.as_ref().is_some_and(|native| {
                native.kind == "id"
                    && native.agent == target.provider
                    && native.value == target.native_id
            })
        {
            return Err(Error::Protocol("The terminal's conversation changed. Select its current conversation before sending.".into()));
        }
        if !matches!(pane.agent_status, protocol::AgentStatus::Idle | protocol::AgentStatus::Done) {
            return Err(Error::Protocol("The agent is busy or waiting for a terminal interaction. Wait for its prompt before sending.".into()));
        }
        let processes: protocol::PaneProcess =
            self.call("pane.process_info", &serde_json::json!({"pane_id":target.pane.0}))?;
        if !processes
            .process_info
            .foreground_processes
            .iter()
            .any(|p| p.pid == target.pid && p.pid != processes.process_info.shell_pid)
        {
            return Err(Error::Protocol(
                "The agent process changed. Wait for its conversation to reconnect.".into(),
            ));
        }
        Ok(())
    }

    /// One bracketed paste + Enter in the existing PTY. Never starts or resumes
    /// a second agent. The caller confirms acceptance from the native transcript.
    pub fn send_agent_input(
        &self,
        target: &AgentInputTarget,
        text: &str,
    ) -> std::result::Result<(), InputFailure> {
        let rejected = |e: Error| InputFailure { uncertain: false, message: e.to_string() };
        if text.trim().is_empty()
            || text.len() > 64 * 1024
            || text.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
            || text.trim_start().starts_with('/')
        {
            return Err(InputFailure { uncertain: false, message: "Send plain text up to 64 KiB here. Run slash commands and terminal controls in terminal.".into() });
        }
        self.check_agent_input(target).map_err(rejected)?;
        let result: Result<serde_json::Value> = self.until(Duration::from_secs(3)).call(
            "pane.send_input",
            &serde_json::json!({"pane_id":target.pane.0,"text":text,"keys":["Enter"]}),
        );
        result.map(|_| ()).map_err(|e| InputFailure { uncertain: true, message: e.to_string() })
    }
}

/// Interpret SGR dim styling so Codex's ghost placeholder is not mistaken for
/// a draft. Only the recognized empty prompt and footer shapes grant input.
fn styled_line(line: &str) -> Vec<(char, bool)> {
    let mut chars = line.chars().peekable();
    let mut dim = false;
    let mut out = Vec::new();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next() != Some('[') {
                return Vec::new();
            }
            let mut code = String::new();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    if c == 'm' {
                        // Color arguments contain numbers such as 2; they must
                        // not be confused with the standalone dim attribute.
                        let values: Vec<u16> =
                            code.split(';').map(|v| v.parse().unwrap_or(0)).collect();
                        let mut i = 0;
                        while i < values.len() {
                            match values[i] {
                                0 | 22 => dim = false,
                                2 => dim = true,
                                38 | 48 | 58 if values.get(i + 1) == Some(&2) => i += 4,
                                38 | 48 | 58 if values.get(i + 1) == Some(&5) => i += 2,
                                _ => {}
                            }
                            i += 1;
                        }
                    }
                    break;
                }
                code.push(c);
            }
        } else if c != '\r' {
            out.push((c, dim));
        }
    }
    out
}

fn empty_prompt(provider: &str, screen: &str) -> bool {
    let lines: Vec<_> = screen.lines().map(styled_line).collect();
    let marker = if provider == "codex" {
        '›'
    } else if provider == "claude" {
        '❯'
    } else {
        return false;
    };
    let Some(index) = lines.iter().rposition(|line| {
        line.iter().find(|(c, _)| !c.is_whitespace()).is_some_and(|(c, _)| *c == marker)
    }) else {
        return false;
    };
    let line = &lines[index];
    let marker_index = line.iter().position(|(c, _)| *c == marker).unwrap();
    if line[marker_index + 1..]
        .iter()
        .any(|(c, dim)| !c.is_whitespace() && !(provider == "codex" && *dim))
    {
        return false;
    }
    let tail: Vec<String> = lines[index + 1..]
        .iter()
        .map(|line| line.iter().map(|(c, _)| *c).collect::<String>().trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    if provider == "claude" {
        tail.first().is_some_and(|line| line.chars().all(|c| c == '─' || c == '━'))
            && tail[1..].iter().all(|line| {
                line.starts_with('⏵')
                    || line.starts_with("⏸ manual mode on")
                    || line.starts_with("? for shortcuts")
                    || line == "/rc"
                    || line.contains("tokens")
                    || line.contains("Context left")
            })
    } else {
        !tail.is_empty()
            && tail.iter().all(|line| {
                line.contains(" · ")
                    || line.contains("context left")
                    || line.contains("% left")
                    || line.starts_with("? for shortcuts")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_empty_prompts_accept_a_message() {
        assert!(empty_prompt(
            "codex",
            "old reply\n\x1b[1m›\x1b[0m \x1b[2mAsk Codex to do anything\x1b[0m\n\nmodel · /project"
        ));
        assert!(!empty_prompt("codex", "› existing draft\nmodel · /project"));
        assert!(!empty_prompt("codex", "› \n  another draft line\nmodel · /project"));
        assert!(!empty_prompt("codex", "› Approve command\n1. Yes\n2. No"));
        assert!(!empty_prompt(
            "codex",
            "› \x1b[38;2;153;153;153mreal draft\x1b[0m\nmodel · /project"
        ));
        assert!(empty_prompt("claude", "────────\n❯ \n────────\n/rc\n⏵⏵ auto mode on"));
        assert!(empty_prompt(
            "claude",
            "────────\n❯ \n────────\n⏸ manual mode on · ? for shortcuts · ← for agents"
        ));
        assert!(!empty_prompt("claude", "❯ existing draft\n────────\n? for shortcuts"));
        assert!(!empty_prompt("claude", "❯ Yes\n2. No"));
        assert!(!empty_prompt("claude", "❯\n  wrapped draft\n────────"));
    }
}
