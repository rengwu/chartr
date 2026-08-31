//! herdr's wire types — exactly the ones zeddy sends or reads, and no more.
//!
//! herdr's socket API has ninety methods. zeddy uses six of them. Modelling
//! only those keeps the pin in [`crate::SUPPORTED_HERDR_VERSION`] honest: a
//! herdr release can change anything zeddy does not name here without zeddy
//! having an opinion about it.
//!
//! Every response field zeddy does not require is `#[serde(default)]`, because
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
#[derive(Debug, Serialize)]
pub struct Empty {}

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

/// A herdr pane: one terminal, which is what zeddy shows in one tab.
#[derive(Debug, Clone, Deserialize)]
pub struct Pane {
    pub pane_id: String,
    #[serde(default)]
    pub workspace_id: String,
    /// herdr's own title for the pane, when it has worked one out.
    #[serde(default)]
    pub title: Option<String>,
    /// The command herdr believes is running — `claude`, `codex`, and so on.
    /// This is the whole reason zeddy is an *agent* multiplexer and not a
    /// terminal multiplexer: the backend already knows what a pane is running.
    #[serde(default)]
    pub display_agent: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
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

// --- the data plane ------------------------------------------------------

/// A line of `herdr terminal session control`'s stdout.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum StreamMessage {
    #[serde(rename = "terminal.frame")]
    Frame(RawFrame),
    #[serde(rename = "terminal.closed")]
    Closed(Closed),
}

/// A repaint, as it arrives: base64 ANSI plus the geometry it was painted for.
///
/// Only the first frame after an attach or a resize is `full`. Every other one
/// is a diff against what the frames before it drew, which is why
/// [`crate::stream::Frames`] refuses a stream with a gap in `seq` rather than
/// painting a plausible-looking wrong screen.
#[derive(Debug, Clone, Deserialize)]
pub struct RawFrame {
    pub bytes: String,
    #[serde(default)]
    pub full: bool,
    #[serde(default)]
    pub seq: u64,
    #[serde(default)]
    pub width: u16,
    #[serde(default)]
    pub height: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Closed {
    #[serde(default)]
    pub reason: String,
}

/// A line written to `herdr terminal session control`'s stdin.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StreamCommand {
    #[serde(rename = "terminal.input")]
    Input { bytes: String },
    #[serde(rename = "terminal.resize")]
    Resize { cols: u16, rows: u16 },
    #[serde(rename = "terminal.release")]
    Release,
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
        let raw = r#"{"id":"1","result":{"panes":[{"pane_id":"p1","invented_in_0_9":true}]}}"#;
        let parsed: Response<PaneList> = serde_json::from_str(raw).expect("parses");
        assert_eq!(parsed.result.expect("result").panes[0].pane_id, "p1");
    }

    #[test]
    fn stream_messages_are_tagged_by_type() {
        let raw = r#"{"type":"terminal.frame","bytes":"aGk=","full":true,"seq":0,"width":80,"height":24}"#;
        match serde_json::from_str::<StreamMessage>(raw).expect("parses") {
            StreamMessage::Frame(frame) => assert!(frame.full && frame.seq == 0),
            StreamMessage::Closed(_) => panic!("that was a frame"),
        }
    }

    #[test]
    fn commands_serialise_the_way_herdr_reads_them() {
        let json = serde_json::to_string(&StreamCommand::Resize { cols: 120, rows: 40 })
            .expect("serialises");
        assert_eq!(json, r#"{"type":"terminal.resize","cols":120,"rows":40}"#);
    }
}
