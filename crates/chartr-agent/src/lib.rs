//! Canonical agent identities and capabilities shared by launch, history, and Herdr.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageTransport {
    TerminalPrompt,
    OpenCodeApi,
    TerminalOnly,
}

pub struct Definition {
    pub provider: Provider,
    pub name: &'static str,
    pub slug: &'static str,
    pub aliases: &'static [&'static str],
    pub transport: MessageTransport,
    pub positional_prompt: bool,
}

// Generate both the identity type and its complete catalog from one declaration.
// A new provider cannot exist without aliases and explicit capabilities.
macro_rules! providers {
    ($( $variant:ident => ($name:literal, $slug:literal, $aliases:expr, $transport:ident, $positional:literal) ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Provider { $( $variant, )+ }
        pub const DEFINITIONS: &[Definition] = &[
            $( Definition { provider: Provider::$variant, name: $name, slug: $slug,
                aliases: $aliases, transport: MessageTransport::$transport,
                positional_prompt: $positional }, )+
        ];
    };
}
providers! {
    Codex => ("Codex", "codex", &["codex", "codex-cli"], TerminalPrompt, true),
    Claude => ("Claude", "claude", &["claude", "claude-code", "claude code"], TerminalPrompt, true),
    Grok => ("Grok", "grok", &["grok", "grok-build"], TerminalOnly, false),
    OpenCode => ("OpenCode", "opencode", &["opencode", "open code"], OpenCodeApi, false),
}

impl Provider {
    pub fn all() -> impl Iterator<Item = Self> {
        DEFINITIONS.iter().map(|entry| entry.provider)
    }

    pub fn definition(self) -> &'static Definition {
        DEFINITIONS.iter().find(|entry| entry.provider == self).expect("registered provider")
    }

    /// Human/provider labels. Never infers identity from a substring of a title.
    pub fn detect(name: &str) -> Option<Self> {
        DEFINITIONS
            .iter()
            .find(|entry| entry.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name)))
            .map(|entry| entry.provider)
    }

    /// Executable paths and argv[0], including version-named processes whose
    /// argv[0] still names their provider (handled by checking both at the caller).
    pub fn executable(path: &str) -> Option<Self> {
        Self::detect(Path::new(path).file_name()?.to_str()?)
    }

    /// Wire integration IDs are canonical and case-sensitive.
    pub fn from_slug(slug: &str) -> Option<Self> {
        DEFINITIONS.iter().find(|entry| entry.slug == slug).map(|entry| entry.provider)
    }

    pub fn name(self) -> &'static str {
        self.definition().name
    }
    pub fn slug(self) -> &'static str {
        self.definition().slug
    }
    pub fn transport(self) -> MessageTransport {
        self.definition().transport
    }
    pub fn needs_process_identity(self) -> bool {
        self.transport() != MessageTransport::TerminalOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alias_has_one_identity_across_labels_paths_and_integrations() {
        for entry in DEFINITIONS {
            assert_eq!(Provider::from_slug(entry.slug), Some(entry.provider));
            for alias in entry.aliases {
                assert_eq!(Provider::detect(&alias.to_uppercase()), Some(entry.provider));
                assert_eq!(
                    Provider::executable(&format!("/opt/agents/{alias}")),
                    Some(entry.provider)
                );
                assert_eq!(
                    DEFINITIONS.iter().filter(|other| other.aliases.contains(alias)).count(),
                    1
                );
            }
        }
        for helper in ["node", "git", "codex-helper", "my claude task", "2.1.267"] {
            assert_eq!(Provider::detect(helper), None);
            assert_eq!(Provider::executable(helper), None);
        }
        assert!(Provider::from_slug("claude-code").is_none());
    }
}
