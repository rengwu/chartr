//! Conversation identity and provider observations, independent of pane layout.
//!
//! This crate never starts, stops or owns a terminal. The host supplies verified
//! runtime observations and adapters address the conversation already running.

mod opencode;
mod store;
mod transcripts;

pub use opencode::OpenCode;
pub use opencode::endpoints_for_process;
pub use store::Store;
pub use transcripts::ProviderPaths;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use chartr_agent::{MessageTransport, Provider};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeSession {
    pub id: String,
    pub path: Option<PathBuf>,
}

/// A current host observation; neither a row selection nor a layout position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub runtime: String,
    pub terminal: String,
    pub provider: Provider,
    pub native: Option<NativeSession>,
    pub cwd: Option<PathBuf>,
    pub space: Option<SpaceIdentity>,
    pub title: Option<String>,
    pub status: Status,
    pub pid: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Unknown,
    Idle,
    Working,
    Waiting,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub text: String,
    #[serde(default)]
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Question { id: String, prompt: String, options: Vec<(String, String)> },
    Permission { id: String, permission: String, patterns: Vec<String> },
    TerminalRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    pub message_id: String,
    pub text: String,
    #[serde(default)]
    pub prior_user_messages: Option<Vec<String>>,
}

impl Delivery {
    pub fn matches(&self, message: &Message) -> bool {
        match &self.prior_user_messages {
            Some(prior) => {
                message.role == Role::User
                    && !prior.contains(&message.id)
                    && message.text.trim_end() == self.text.trim_end()
            }
            None => message.id == self.message_id,
        }
    }
}

/// The owning Chartr space, independent of the CLI's working directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceIdentity {
    pub key: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub provider: Provider,
    pub native: Option<NativeSession>,
    pub title: String,
    pub custom_title: Option<String>,
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub space: Option<SpaceIdentity>,
    pub updated: u64,
    pub draft: String,
    pub archived: bool,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub delivery: Option<Delivery>,
    /// Only reconciled, live observations grant a runtime or an input route.
    #[serde(skip)]
    pub runtime: Option<String>,
    #[serde(skip)]
    pub terminal: Option<String>,
    #[serde(skip)]
    pub status: Status,
    #[serde(skip)]
    pub endpoint: Option<String>,
    #[serde(skip)]
    pub problem: Option<String>,
    #[serde(skip)]
    pub requests: Vec<Request>,
}

impl Conversation {
    pub fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }

    pub fn can_send(&self) -> bool {
        (self.provider.transport() == MessageTransport::TerminalPrompt
            || (self.provider.transport() == MessageTransport::OpenCodeApi
                && self.endpoint.is_some()))
            && self.native.is_some()
            && self.runtime.is_some()
            && self.status == Status::Idle
            && self.delivery.is_none()
    }

    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        query.is_empty()
            || self.display_title().to_lowercase().contains(&query)
            || self.provider.name().to_lowercase().contains(&query)
            || self
                .cwd
                .as_ref()
                .is_some_and(|p| p.to_string_lossy().to_lowercase().contains(&query))
    }
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn prompt_title(messages: &[Message]) -> Option<String> {
    let text = messages.iter().find(|m| m.role == Role::User && !m.text.trim().is_empty())?;
    let compact = text.text.split_whitespace().collect::<Vec<_>>().join(" ");
    Some(compact.chars().take(100).collect())
}
