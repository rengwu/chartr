//! herdr's wire types — exactly the ones chartr sends or reads, and no more.
//!
//! herdr's socket API has ninety methods. chartr uses nine of them. Modelling
//! only those keeps the pin in [`crate::SUPPORTED_HERDR_VERSION`] honest: a
//! herdr release can change anything chartr does not name here without chartr
//! having an opinion about it.
//!
//! Every response field chartr does not require is `#[serde(default)]`, because
//! herdr adds fields between releases and a new one must not fail a parse.

use serde::{Deserialize, Serialize};

/// The control-plane request envelope. One of these per connection.
#[derive(Debug, Serialize)]
pub struct Request<P> {
    pub id: String,
    pub method: &'static str,
    pub params: P,
}

/// The control-plane response envelope: exactly one of `result` or `error`.
///
/// The explicit bound keeps serde's derive from also demanding `R: Default`,
/// which it would infer from the `default` attributes below.
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "R: Deserialize<'de>"))]
pub struct Response<R> {
    #[serde(default)]
    pub result: Option<R>,
    #[serde(default)]
    pub error: Option<ErrorBody>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub message: String,
}

/// Methods take a params object even when they take no parameters.
#[derive(Debug, Deserialize, Serialize)]
pub struct Empty {}

#[derive(Debug, Deserialize)]
pub struct PaneHistoryInfo {
    pub pane: PaneHistory,
}
#[derive(Debug, Deserialize)]
pub struct PaneHistory {
    pub scroll: PaneScroll,
}
#[derive(Debug, Deserialize)]
pub struct PaneScroll {
    pub max_offset_from_bottom: u64,
    pub viewport_rows: u64,
}
#[derive(Debug, Deserialize)]
pub struct PaneSelection {
    pub text: String,
}

/// `server.live_handoff` — replace an incompatible private daemon without
/// terminating the PTYs it owns.
#[derive(Debug, Serialize)]
pub struct ServerLiveHandoffParams<'a> {
    pub import_exe: &'a str,
    pub expected_protocol: u32,
    pub expected_version: &'a str,
}

/// `ping` — the handshake. Its answer is the version check.
#[derive(Debug, Deserialize)]
pub struct Pong {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub protocol: u32,
}

/// A herdr workspace: one project directory's worth of panes.
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub workspace_id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub pane_count: u32,
}

/// A herdr pane: one terminal, which is what chartr shows in one tab.
#[derive(Debug, Clone, Deserialize)]
pub struct Pane {
    pub pane_id: String,
    /// The server-owned PTY behind this pane. Herdr's interactive attach CLI
    /// addresses the terminal rather than the pane that currently displays it.
    pub terminal_id: String,
    #[serde(default)]
    pub workspace_id: String,
    /// The Herdr tab containing this pane. chartr keeps one session per Herdr
    /// tab, so this is also where the session's persistent fallback label
    /// lives.
    #[serde(default)]
    pub tab_id: String,
    /// herdr's own title for the pane, when it has worked one out.
    #[serde(default)]
    pub title: Option<String>,
    /// The command herdr believes is running — `claude`, `codex`, and so on.
    /// This is the whole reason chartr is an *agent* multiplexer and not a
    /// terminal multiplexer: the backend already knows what a pane is running.
    #[serde(default)]
    pub display_agent: Option<String>,
    /// Herdr's internal agent name, used only when it has no display name.
    #[serde(default)]
    pub agent: Option<String>,
    /// What Herdr believes the detected agent is doing. chartr consumes this
    /// instead of trying to infer agent state from terminal output itself.
    #[serde(default)]
    pub agent_status: AgentStatus,
    #[serde(default)]
    pub cwd: Option<String>,
}

/// What Herdr believes the agent in a pane is doing.
///
/// Unknown future values deliberately become [`Unknown`](Self::Unknown): a
/// newer daemon gaining another state must not make an older chartr unable to
/// list or attach the pane.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Blocked,
    Done,
    #[default]
    #[serde(other)]
    Unknown,
}

/// A Herdr tab: the persistent name and ordering container for one chartr
/// terminal session.
#[derive(Debug, Clone, Deserialize)]
pub struct Tab {
    pub tab_id: String,
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TabListParams<'a> {
    pub workspace_id: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct TabList {
    #[serde(default)]
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Serialize)]
pub struct PaneProcessParams<'a> {
    pub pane_id: &'a str,
}

/// The envelope returned by `pane.process_info`.
#[derive(Debug, Deserialize)]
pub struct PaneProcess {
    pub process_info: ProcessInfo,
}

/// The foreground process group of the PTY Herdr owns.
#[derive(Debug, Deserialize)]
pub struct ProcessInfo {
    #[serde(default)]
    pub shell_pid: u32,
    #[serde(default)]
    pub foreground_processes: Vec<Process>,
}

impl ProcessInfo {
    /// The program running in the pane, excluding the shell waiting at its own
    /// prompt.
    pub fn foreground_program(&self) -> Option<&Process> {
        self.foreground_processes.iter().find(|process| process.pid != self.shell_pid)
    }
}

#[derive(Debug, Deserialize)]
pub struct Process {
    #[serde(default)]
    pub pid: u32,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceCreateParams<'a> {
    pub cwd: &'a str,
    pub label: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct TabCreateParams<'a> {
    pub workspace_id: &'a str,
    pub cwd: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct PaneListParams<'a> {
    pub workspace_id: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct PaneCloseParams<'a> {
    pub pane_id: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceList {
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Deserialize)]
pub struct PaneList {
    #[serde(default)]
    pub panes: Vec<Pane>,
}

/// `workspace.create` and `tab.create` both answer with the pane they opened.
#[derive(Debug, Deserialize)]
pub struct Created {
    pub root_pane: Pane,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_response_carrying_an_error_parses_as_one() {
        let raw = r#"{"id":"1","error":{"code":"not_found","message":"no such pane"}}"#;
        let parsed: Response<PaneList> = serde_json::from_str(raw).expect("parses");
        assert!(parsed.result.is_none());
        assert_eq!(parsed.error.expect("error").message, "no such pane");
    }

    #[test]
    fn unknown_response_fields_do_not_fail_the_parse() {
        let raw = r#"{"id":"1","result":{"panes":[{"pane_id":"p1","terminal_id":"term1","invented_in_0_9":true}]}}"#;
        let parsed: Response<PaneList> = serde_json::from_str(raw).expect("parses");
        assert_eq!(parsed.result.expect("result").panes[0].pane_id, "p1");
    }
}
