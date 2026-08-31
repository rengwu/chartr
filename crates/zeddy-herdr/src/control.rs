//! The control plane: herdr's socket API.
//!
//! One request per connection, NDJSON, blocking. Blocking is deliberate — the
//! calls are local, they take microseconds, and the alternative is an async
//! runtime in a crate whose entire job is six methods.
//!
//! Callers on the window thread should still not sit on these directly; the app
//! runs them on a background executor and delivers the answer back. This crate
//! does not decide that for them.

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    Error, Geometry, Namespace, PaneId, Result, SUPPORTED_HERDR_VERSION, SUPPORTED_PROTOCOL,
    Sidecar, WorkspaceId,
    protocol::{
        self, Created, Empty, PaneCloseParams, PaneList, PaneListParams, Pong, Request, Response,
        TabCreateParams, WorkspaceCreateParams, WorkspaceList,
    },
    stream::Attachment,
};

/// A pane as zeddy talks about it, once herdr's vocabulary has been left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: PaneId,
    pub workspace: WorkspaceId,
    /// What to put on the tab. herdr's title if it has one, the agent's name if
    /// it knows one, and the id only as a last resort — a tab always has a name.
    pub title: String,
    /// The agent herdr believes is running in the pane, if any.
    pub agent: Option<String>,
    pub cwd: Option<PathBuf>,
}

impl From<protocol::Pane> for Session {
    fn from(pane: protocol::Pane) -> Self {
        let title = pane
            .title
            .filter(|t| !t.trim().is_empty())
            .or_else(|| pane.display_agent.clone())
            .unwrap_or_else(|| pane.pane_id.clone());
        Self {
            id: PaneId(pane.pane_id),
            workspace: WorkspaceId(pane.workspace_id),
            title,
            agent: pane.display_agent,
            cwd: pane.cwd.map(PathBuf::from),
        }
    }
}

/// A connection-per-request client for zeddy's private daemon.
#[derive(Debug, Clone)]
pub struct Client {
    sidecar: Sidecar,
    namespace: Namespace,
}

impl Client {
    pub fn new(sidecar: Sidecar, namespace: Namespace) -> Self {
        Self { sidecar, namespace }
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Bring the private daemon up if it is not already, then handshake it.
    ///
    /// Starting and adopting are the same call on purpose. zeddy has no
    /// "is it running" question worth asking separately: the answer it acts on
    /// is "can I talk to it", and that is a `ping`.
    pub fn connect(&self, timeout: Duration) -> Result<()> {
        self.namespace.prepare()?;
        if self.handshake().is_ok() {
            return Ok(());
        }
        self.spawn_daemon()?;

        let deadline = Instant::now() + timeout;
        let mut backoff = Duration::from_millis(10);
        loop {
            match self.handshake() {
                Ok(()) => return Ok(()),
                Err(err) if Instant::now() + backoff >= deadline => return Err(err),
                Err(_) => {
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(Duration::from_millis(200));
                }
            }
        }
    }

    /// `ping`, checked against the version this client was written for.
    ///
    /// A version mismatch is an error and not a warning. The frame stream rides
    /// herdr's command line, so a daemon zeddy did not ship is a daemon zeddy
    /// cannot promise to render.
    pub fn handshake(&self) -> Result<()> {
        let pong: Pong = self.call("ping", &Empty {})?;
        if pong.version != SUPPORTED_HERDR_VERSION || pong.protocol != SUPPORTED_PROTOCOL {
            return Err(Error::Backend {
                method: "ping",
                message: format!(
                    "daemon is herdr {} (protocol {}); zeddy ships {SUPPORTED_HERDR_VERSION} \
                     (protocol {SUPPORTED_PROTOCOL})",
                    pong.version, pong.protocol
                ),
            });
        }
        Ok(())
    }

    /// Every pane the private daemon is running, or every pane in one workspace.
    pub fn sessions(&self, workspace: Option<&WorkspaceId>) -> Result<Vec<Session>> {
        let params = PaneListParams { workspace_id: workspace.map(|w| w.0.as_str()) };
        let list: PaneList = self.call("pane.list", &params)?;
        Ok(list.panes.into_iter().map(Session::from).collect())
    }

    /// Every workspace, so the sidebar has something to list.
    pub fn workspaces(&self) -> Result<Vec<protocol::Workspace>> {
        let list: WorkspaceList = self.call("workspace.list", &Empty {})?;
        Ok(list.workspaces)
    }

    /// The workspace already open on a directory, if there is one.
    ///
    /// herdr's workspace listing does not carry a working directory, but its
    /// panes do, so a workspace is identified by where its sessions are.
    ///
    /// Both sides are resolved before they are compared. On macOS `/tmp` and
    /// `/var` are symlinks into `/private`, so the path a process reports and
    /// the path its user typed are routinely different spellings of one
    /// directory — and a comparison that misses opens a second workspace on the
    /// same folder every launch.
    pub fn workspace_at(&self, cwd: &Path) -> Result<Option<WorkspaceId>> {
        let cwd = resolved(cwd);
        Ok(self
            .sessions(None)?
            .into_iter()
            .find(|session| session.cwd.as_deref().map(resolved) == Some(cwd.clone()))
            .map(|session| session.workspace))
    }

    /// The workspace for a directory: the one already open on it, or a new one.
    ///
    /// Adopting rather than always creating is the point of a backend that
    /// outlives the window. Without it, every launch would leave behind another
    /// workspace holding the previous launch's sessions.
    pub fn open_workspace(&self, cwd: &Path, label: Option<&str>) -> Result<WorkspaceId> {
        if let Some(existing) = self.workspace_at(cwd)? {
            return Ok(existing);
        }
        let params = WorkspaceCreateParams { cwd: &cwd.to_string_lossy(), label };
        let created: Created = self.call("workspace.create", &params)?;
        Ok(Session::from(created.root_pane).workspace)
    }

    /// Start one more session in a workspace that is already open.
    pub fn start_session(&self, workspace: &WorkspaceId, cwd: Option<&str>) -> Result<Session> {
        let params = TabCreateParams { workspace_id: &workspace.0, cwd };
        let created: Created = self.call("tab.create", &params)?;
        Ok(created.root_pane.into())
    }

    /// End a session. The pane and whatever is running in it both go.
    pub fn close_session(&self, pane: &PaneId) -> Result<()> {
        let _: serde_json::Value =
            self.call("pane.close", &PaneCloseParams { pane_id: &pane.0 })?;
        Ok(())
    }

    /// Attach to a session's byte stream at a given geometry.
    ///
    /// The client hands its own sidecar and namespace to the attachment, so the
    /// stream cannot end up pointed at a herdr the control plane is not talking
    /// to.
    pub fn attach(&self, pane: &PaneId, geometry: Geometry) -> Result<Attachment> {
        Attachment::open(&self.sidecar, &self.namespace, pane, geometry)
    }

    /// Send one request, read one response, close the connection.
    fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &'static str,
        params: &P,
    ) -> Result<R> {
        let socket = self.namespace.socket();
        let mut stream = UnixStream::connect(&socket).map_err(|err| {
            Error::Transport(std::io::Error::new(
                err.kind(),
                format!("{}: {err}", socket.display()),
            ))
        })?;

        let request = Request { id: next_id(), method, params };
        let mut line = serde_json::to_vec(&request)
            .map_err(|err| Error::Protocol(format!("cannot encode {method}: {err}")))?;
        line.push(b'\n');
        stream.write_all(&line)?;
        stream.flush()?;

        let mut reply = String::new();
        BufReader::new(&stream).read_line(&mut reply)?;
        if reply.trim().is_empty() {
            return Err(Error::Protocol(format!("{method} got no answer")));
        }

        let response: Response<R> = serde_json::from_str(&reply)
            .map_err(|err| Error::Protocol(format!("cannot read the answer to {method}: {err}")))?;
        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (None, Some(error)) => Err(Error::Backend {
                method,
                message: if error.code.is_empty() {
                    error.message
                } else {
                    format!("{} ({})", error.message, error.code)
                },
            }),
            (None, None) => {
                Err(Error::Protocol(format!("{method} answered with neither result nor error")))
            }
        }
    }

    /// Launch the private daemon detached, with its log as its only output.
    fn spawn_daemon(&self) -> Result<()> {
        let log = std::fs::File::create(self.namespace.log())?;
        let mut command = Command::new(self.sidecar.path());
        command
            .arg("server")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log));
        apply(&mut command, &self.namespace);
        command.spawn()?;
        Ok(())
    }
}

/// Place a command in a namespace: set what it pins, remove what it must not
/// inherit.
pub(crate) fn apply(command: &mut Command, namespace: &Namespace) {
    for (key, value) in namespace.env() {
        match value {
            Some(value) => command.env(key, value),
            None => command.env_remove(key),
        };
    }
}

/// A path with symlinks resolved, or the path itself where it cannot be — a
/// directory that no longer exists is not a directory a workspace is open on,
/// and the comparison above should simply not match it.
fn resolved(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_owned())
}

/// Request ids only have to be unique within one connection, and there is one
/// request per connection, so a counter is enough and a UUID would be theatre.
fn next_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pane(id: &str, title: Option<&str>, agent: Option<&str>) -> protocol::Pane {
        protocol::Pane {
            pane_id: id.to_owned(),
            workspace_id: "w1".to_owned(),
            title: title.map(str::to_owned),
            display_agent: agent.map(str::to_owned),
            cwd: None,
        }
    }

    #[test]
    fn a_tab_falls_back_from_title_to_agent_to_id() {
        assert_eq!(Session::from(pane("p1", Some("build"), Some("claude"))).title, "build");
        assert_eq!(Session::from(pane("p1", None, Some("claude"))).title, "claude");
        assert_eq!(Session::from(pane("p1", None, None)).title, "p1");
    }

    #[test]
    fn a_blank_title_is_not_a_title() {
        assert_eq!(Session::from(pane("p1", Some("   "), Some("codex"))).title, "codex");
    }

    #[test]
    fn a_missing_socket_names_the_socket_it_looked_for() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let namespace = Namespace::rooted(tmp.path());
        let herdr = tmp.path().join("herdr");
        std::fs::write(&herdr, b"#!/bin/sh\n").expect("write");
        let client = Client::new(Sidecar::at(&herdr).expect("sidecar"), namespace.clone());

        let err = client.handshake().expect_err("nothing is listening");
        assert!(err.to_string().contains(&namespace.socket().display().to_string()), "{err}");
    }
}
