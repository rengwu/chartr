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
    collections::{HashMap, HashSet},
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
        TabCreateParams, TabList, TabListParams, WorkspaceCreateParams, WorkspaceList,
    },
    stream::Attachment,
};

/// What an agent in a session is doing, once Herdr's vocabulary has been left
/// behind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SessionStatus {
    Idle,
    Working,
    Blocked,
    Done,
    /// No agent, or nothing known about one. The ordinary state of a shell.
    #[default]
    Unknown,
}

impl From<protocol::AgentStatus> for SessionStatus {
    fn from(status: protocol::AgentStatus) -> Self {
        match status {
            protocol::AgentStatus::Idle => Self::Idle,
            protocol::AgentStatus::Working => Self::Working,
            protocol::AgentStatus::Blocked => Self::Blocked,
            protocol::AgentStatus::Done => Self::Done,
            protocol::AgentStatus::Unknown => Self::Unknown,
        }
    }
}

/// A pane as zeddy talks about it, once herdr's vocabulary has been left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: PaneId,
    pub workspace: WorkspaceId,
    /// Herdr's persistent tab label/number, used when nothing is running.
    pub label: String,
    /// The detected agent or non-shell foreground process, if one is running.
    pub running: Option<String>,
    /// What the detected agent is doing. Ordinary processes remain Unknown;
    /// their presence is carried separately by `running`.
    pub status: SessionStatus,
    /// The agent herdr believes is running in the pane, if any.
    pub agent: Option<String>,
    pub cwd: Option<PathBuf>,
}

impl Session {
    fn from_pane(pane: protocol::Pane, label: Option<String>, running: Option<String>) -> Self {
        let label = label
            .or_else(|| pane.title.as_deref().and_then(non_blank).map(str::to_owned))
            .unwrap_or_else(|| pane.pane_id.clone());
        let agent = pane
            .display_agent
            .as_deref()
            .and_then(non_blank)
            .or_else(|| pane.agent.as_deref().and_then(non_blank))
            .map(str::to_owned);
        Self {
            id: PaneId(pane.pane_id),
            workspace: WorkspaceId(pane.workspace_id),
            label,
            running,
            status: pane.agent_status.into(),
            agent,
            cwd: pane.cwd.map(PathBuf::from),
        }
    }

    /// What the tab says: the live agent/process name, or its persistent label.
    pub fn title(&self) -> &str {
        self.running.as_deref().unwrap_or(&self.label)
    }

    /// Whether an ordinary (non-agent) process currently owns the PTY's
    /// foreground process group.
    pub fn process_running(&self) -> bool {
        self.agent.is_none() && self.running.is_some()
    }
}

impl From<protocol::Pane> for Session {
    fn from(pane: protocol::Pane) -> Self {
        let running = pane
            .display_agent
            .as_deref()
            .and_then(non_blank)
            .or_else(|| pane.agent.as_deref().and_then(non_blank))
            .map(str::to_owned);
        Self::from_pane(pane, None, running)
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

    /// Whether anything is accepting connections at this private socket.
    ///
    /// Supervision deliberately asks the operating system rather than pinging
    /// the daemon: a crashed daemon cannot answer a health check, while both a
    /// removed socket and a stale one refuse this connect.
    pub fn answers(&self) -> bool {
        UnixStream::connect(self.namespace.socket()).is_ok()
    }

    /// Start exactly one clean replacement for a daemon that has already died.
    ///
    /// This clears herdr's saved runtime shape before spawning. It does not
    /// wait or retry; [`reconnect`](Self::reconnect) is intentionally the only
    /// wait path so recovery cannot turn into a hidden spawn loop.
    pub fn restart(&self) -> Result<()> {
        self.namespace.prepare()?;
        if self.answers() {
            self.stop_daemon()?;
        }
        self.clear_saved_shape()?;
        self.spawn_daemon()
    }

    /// Ask this exact private daemon to stop and wait for its socket to close.
    pub fn stop_daemon(&self) -> Result<()> {
        let mut command = Command::new(self.sidecar.path());
        command
            .args(["server", "stop"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        apply(&mut command, &self.namespace);
        let status = command.status()?;
        if !status.success() && self.answers() {
            return Err(Error::Backend {
                method: "server stop",
                message: format!("herdr exited with {status}"),
            });
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.answers() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
        if self.answers() {
            return Err(Error::Backend {
                method: "server stop",
                message: "the private socket is still accepting connections".to_owned(),
            });
        }
        Ok(())
    }

    /// Wait for an already-started daemon to answer, without starting another.
    pub fn reconnect(&self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        let mut backoff = Duration::from_millis(25);
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

    /// Remove only the private daemon's saved workspace shape.
    pub fn clear_saved_shape(&self) -> Result<()> {
        let shape = self.namespace.saved_shape();
        match std::fs::remove_file(&shape) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(Error::Transport(std::io::Error::new(
                error.kind(),
                format!("cannot clear {}: {error}", shape.display()),
            ))),
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
        Ok(self.describe(list.panes))
    }

    /// Decorate Herdr panes with the same live titles used by Chartr-rs:
    /// detected agent, foreground process, then persistent tab label.
    ///
    /// These are presentation questions. A failed `tab.list` or
    /// `pane.process_info` must not hide an otherwise attachable terminal, so
    /// each lookup degrades to the pane metadata already in hand.
    fn describe(&self, panes: Vec<protocol::Pane>) -> Vec<Session> {
        let workspaces: HashSet<_> = panes.iter().map(|pane| pane.workspace_id.as_str()).collect();
        let mut tabs = HashMap::new();
        for workspace in workspaces {
            let params = TabListParams { workspace_id: workspace };
            if let Ok(list) = self.call::<_, TabList>("tab.list", &params) {
                tabs.extend(list.tabs.into_iter().map(|tab| (tab.tab_id.clone(), tab)));
            }
        }

        panes
            .into_iter()
            .map(|pane| {
                let agent = pane
                    .display_agent
                    .as_deref()
                    .and_then(non_blank)
                    .or_else(|| pane.agent.as_deref().and_then(non_blank))
                    .map(str::to_owned);
                let running = agent.clone().or_else(|| {
                    let params = protocol::PaneProcessParams { pane_id: &pane.pane_id };
                    self.call::<_, protocol::PaneProcess>("pane.process_info", &params)
                        .ok()?
                        .process_info
                        .foreground_program()?
                        .name
                        .as_deref()
                        .and_then(non_blank)
                        .map(str::to_owned)
                });
                let label = tabs.get(&pane.tab_id).map(tab_label);
                Session::from_pane(pane, label, running)
            })
            .collect()
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
        Ok(self.create_workspace(cwd, label)?.workspace)
    }

    /// Create a workspace and return its root session.
    ///
    /// herdr creates a workspace and its first pane as one operation. Callers
    /// that are implementing "new session" need that pane rather than only its
    /// workspace id, or they would create a second pane and lose the first.
    pub fn create_workspace(&self, cwd: &Path, label: Option<&str>) -> Result<Session> {
        let params = WorkspaceCreateParams { cwd: &cwd.to_string_lossy(), label };
        let created: Created = self.call("workspace.create", &params)?;
        Ok(self.describe(vec![created.root_pane]).remove(0))
    }

    /// Start one more session in a workspace that is already open.
    pub fn start_session(&self, workspace: &WorkspaceId, cwd: Option<&str>) -> Result<Session> {
        let params = TabCreateParams { workspace_id: &workspace.0, cwd };
        let created: Created = self.call("tab.create", &params)?;
        Ok(self.describe(vec![created.root_pane]).remove(0))
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

fn non_blank(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

fn tab_label(tab: &protocol::Tab) -> String {
    match tab.label.as_deref().and_then(non_blank) {
        Some(label) => label.to_owned(),
        None if tab.number > 0 => tab.number.to_string(),
        None => tab.tab_id.clone(),
    }
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
            tab_id: "w1:t1".to_owned(),
            title: title.map(str::to_owned),
            display_agent: agent.map(str::to_owned),
            agent: None,
            agent_status: protocol::AgentStatus::Unknown,
            cwd: None,
        }
    }

    #[test]
    fn a_tab_prefers_an_agent_then_falls_back_to_title_and_id() {
        assert_eq!(Session::from(pane("p1", Some("build"), Some("claude"))).title(), "claude");
        assert_eq!(Session::from(pane("p1", None, Some("claude"))).title(), "claude");
        assert_eq!(Session::from(pane("p1", Some("build"), None)).title(), "build");
        assert_eq!(Session::from(pane("p1", None, None)).title(), "p1");
    }

    #[test]
    fn a_blank_title_is_not_a_title() {
        assert_eq!(Session::from(pane("p1", Some("   "), Some("codex"))).title(), "codex");
    }

    #[test]
    fn herdr_agent_status_is_translated_without_inference() {
        let mut info = pane("p1", None, Some("claude"));
        info.agent_status = protocol::AgentStatus::Blocked;
        assert_eq!(Session::from(info).status, SessionStatus::Blocked);
    }

    #[test]
    fn an_unknown_future_agent_status_degrades_to_unknown() {
        let pane: protocol::Pane = serde_json::from_value(serde_json::json!({
            "pane_id": "p1",
            "agent_status": "meditating"
        }))
        .expect("pane");
        assert_eq!(pane.agent_status, protocol::AgentStatus::Unknown);
    }

    #[test]
    fn a_tab_label_falls_back_to_its_number_then_id() {
        let mut tab = protocol::Tab {
            tab_id: "w1:t2".to_owned(),
            number: 2,
            label: Some("build".to_owned()),
        };
        assert_eq!(tab_label(&tab), "build");
        tab.label = Some("  ".to_owned());
        assert_eq!(tab_label(&tab), "2");
        tab.number = 0;
        assert_eq!(tab_label(&tab), "w1:t2");
    }

    #[test]
    fn process_info_excludes_the_waiting_shell() {
        let process = protocol::ProcessInfo {
            shell_pid: 10,
            foreground_processes: vec![
                protocol::Process { pid: 10, name: Some("zsh".to_owned()) },
                protocol::Process { pid: 11, name: Some("htop".to_owned()) },
            ],
        };
        assert_eq!(
            process.foreground_program().and_then(|process| process.name.as_deref()),
            Some("htop")
        );
    }

    #[test]
    fn an_ordinary_foreground_process_is_distinct_from_an_agent() {
        let session = Session::from_pane(
            pane("p1", None, None),
            Some("1".to_owned()),
            Some("htop".to_owned()),
        );
        assert_eq!(session.title(), "htop");
        assert!(session.process_running());
        assert_eq!(session.status, SessionStatus::Unknown);
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

    #[test]
    fn a_clean_restart_removes_only_the_saved_shape() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let namespace = Namespace::rooted(tmp.path().join("private"));
        namespace.prepare().expect("prepare");
        let shape = namespace.saved_shape();
        std::fs::create_dir_all(shape.parent().expect("shape directory")).expect("directory");
        std::fs::write(&shape, b"stale shape").expect("shape");
        let neighbor = shape.parent().expect("shape directory").join("config.toml");
        std::fs::write(&neighbor, b"managed config").expect("config");
        let herdr = tmp.path().join("herdr");
        std::fs::write(&herdr, b"#!/bin/sh\n").expect("sidecar");
        let client = Client::new(Sidecar::at(&herdr).expect("sidecar"), namespace);

        client.clear_saved_shape().expect("clear shape");

        assert!(!shape.exists());
        assert_eq!(std::fs::read(&neighbor).expect("config remains"), b"managed config");
        client.clear_saved_shape().expect("already absent is harmless");
    }
}
