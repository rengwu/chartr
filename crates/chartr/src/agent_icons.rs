//! Shared provider glyphs for the Agent launcher and live workspace sessions.
//!
//! Icon inference is presentation-only; it does not grant a process provider
//! capabilities or use user-editable session titles as identity.

pub(crate) const GENERIC_AGENT_ICON: &str = "icons/agent_ai_programming.svg";

pub(crate) fn session_icon(agent: Option<&str>, running: Option<&str>) -> Option<&'static str> {
    agent.and_then(known_agent_icon).or_else(|| {
        let executable = std::path::Path::new(running?).file_name()?.to_str()?;
        known_agent_icon(executable)
    })
}

pub(crate) fn known_agent_icon(value: &str) -> Option<&'static str> {
    let value = value.to_ascii_lowercase();
    let terms: Vec<_> = value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .collect();
    let has = |aliases: &[&str]| terms.iter().any(|term| aliases.contains(term));
    let compact: String = value.chars().filter(char::is_ascii_alphanumeric).collect();

    if has(&["claude", "anthropic"]) {
        Some("icons/agent_claude.svg")
    } else if has(&["codex", "openai", "chatgpt"]) {
        Some("icons/agent_chat_gpt.svg")
    } else if has(&["grok", "xai"]) {
        Some("icons/agent_grok.svg")
    } else if compact.contains("opencode") {
        Some("icons/agent_opencode.svg")
    } else if compact.contains("deepseek") {
        Some("icons/agent_deepseek.svg")
    } else if has(&["antigravity"]) || compact == "googleantigravity" {
        Some("icons/agent_antigravity.svg")
    } else if has(&["kimi", "moonshot"]) {
        Some("icons/agent_kimi.svg")
    } else if has(&["gemini"]) {
        Some("icons/agent_google_gemini.svg")
    } else if has(&["mistral"]) {
        Some("icons/agent_mistral.svg")
    } else if has(&["qwen"]) {
        Some("icons/agent_qwen.svg")
    } else if has(&["copilot"]) {
        Some("icons/agent_copilot.svg")
    } else if has(&["pi"]) {
        Some("icons/agent_pi.svg")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::AssetSource;

    #[test]
    fn popular_harnesses_resolve_to_embedded_provider_icons() {
        for (name, icon) in [
            ("claude", "agent_claude"),
            ("codex", "agent_chat_gpt"),
            ("grok", "agent_grok"),
            ("pi", "agent_pi"),
            ("antigravity", "agent_antigravity"),
            ("kimi", "agent_kimi"),
            ("deepseek", "agent_deepseek"),
            ("opencode", "agent_opencode"),
        ] {
            let expected = format!("icons/{icon}.svg");
            assert_eq!(session_icon(Some(name), Some("node")), Some(expected.as_str()));
            assert_eq!(
                session_icon(None, Some(&format!("/opt/bin/{name}"))),
                Some(expected.as_str())
            );
            assert_eq!(session_icon(Some(&name.to_uppercase()), None), Some(expected.as_str()));
            assert!(crate::assets::Assets.load(&expected).unwrap().is_some());
        }
    }

    #[test]
    fn identity_takes_precedence_and_plain_processes_keep_the_terminal_fallback() {
        assert_eq!(
            session_icon(Some("claude-code"), Some("codex")),
            Some("icons/agent_claude.svg")
        );
        assert_eq!(session_icon(Some("kimi-cli"), Some("python")), Some("icons/agent_kimi.svg"));
        assert_eq!(session_icon(Some("Grok Build"), None), Some("icons/agent_grok.svg"));
        for process in
            [None, Some("zsh"), Some("node"), Some("python"), Some("/projects/claude/python")]
        {
            assert_eq!(session_icon(None, process), None);
        }
        assert_eq!(session_icon(Some("custom-wrapper"), Some("node")), None);
        // Fresh metadata clears the previous provider icon when an agent exits.
        assert!(session_icon(Some("codex"), Some("codex")).is_some());
        assert_eq!(session_icon(None, None), None);
    }
}
