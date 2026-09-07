//! The control plane: herdr's socket API.
//!
//! One request per connection, NDJSON, blocking. Blocking is deliberate — the
//! calls are local, they take microseconds, and the alternative is an async
//! runtime around this deliberately small control surface.
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
    Error, Namespace, PaneId, Result, SUPPORTED_HERDR_VERSION, SUPPORTED_PROTOCOL, Sidecar,
    TerminalId, WorkspaceId,
    protocol::{
        self, Created, Empty, PaneCloseParams, PaneList, PaneListParams, Pong, Request, Response,
        ServerLiveHandoffParams, TabCreateParams, TabList, TabListParams, WorkspaceCreateParams,
        WorkspaceList,
    },
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

/// A pane as chartr talks about it, once herdr's vocabulary has been left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: PaneId,
    /// The persistent PTY identifier consumed by `herdr terminal attach`.
    pub terminal: TerminalId,
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
            terminal: TerminalId(pane.terminal_id),
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

/// A complete, namespace-safe invocation of Herdr's interactive attach CLI.
///
/// The UI layer decides how to host this command (currently in Zed's PTY),
/// while this crate remains the only layer that knows Herdr's executable,
/// arguments, or private environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectAttach {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
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

/// A connection-per-request client for chartr's private daemon.
#[derive(Debug, Clone)]
pub struct Client {
    sidecar: Sidecar,
    namespace: Namespace,
    deadline: Option<Instant>,
}

impl Client {
    pub fn new(sidecar: Sidecar, namespace: Namespace) -> Self {
        Self { sidecar, namespace, deadline: None }
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Bring the private daemon up if it is not already, then handshake it.
    ///
    /// Starting and adopting are the same call on purpose. chartr has no
    /// "is it running" question worth asking separately: the answer it acts on
    /// is "can I talk to it", and that is a `ping`.
    pub fn connect(&self, timeout: Duration) -> Result<()> {
        self.until(timeout).connect_inner(timeout)
    }

    fn until(&self, timeout: Duration) -> Self {
        let deadline = Instant::now() + timeout;
        Self {
            deadline: Some(self.deadline.map_or(deadline, |old| old.min(deadline))),
            ..self.clone()
        }
    }

    fn connect_inner(&self, timeout: Duration) -> Result<()> {
        self.namespace.prepare()?;
        match self.handshake() {
            Ok(()) => return Ok(()),
            // A private daemon may outlive the chartr build that started it
            // because it owns persistent PTYs. Replace only a daemon that
            // answered with an incompatible identity; an unrelated transient
            // handshake failure must not trigger an upgrade.
            Err(Error::IncompatibleDaemon { .. }) if self.answers() => {
                self.live_handoff()?;
                return self.reconnect(timeout);
            }
            Err(_) => {}
        }

        remaining(self.deadline.expect("connect has a deadline"))?;
        self.spawn_daemon()?;

        let deadline = self.deadline.expect("connect has a deadline");
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

    /// Replace the daemon at this private socket with the pinned sidecar while
    /// preserving its live PTYs.
    fn live_handoff(&self) -> Result<()> {
        let import_exe = self.sidecar.path().to_str().ok_or_else(|| {
            Error::Sidecar(format!(
                "Herdr sidecar path is not valid UTF-8: {}",
                self.sidecar.path().display()
            ))
        })?;
        let params = ServerLiveHandoffParams {
            import_exe,
            expected_protocol: SUPPORTED_PROTOCOL,
            expected_version: SUPPORTED_HERDR_VERSION,
        };
        let _: Empty = self.call("server.live_handoff", &params)?;
        Ok(())
    }

    /// Whether anything is accepting connections at this private socket.
    ///
    /// This is a socket-presence probe, not a daemon health check.
    pub fn answers(&self) -> bool {
        connect_socket(&self.namespace.socket(), self.until(REQUEST_TIMEOUT).deadline.unwrap())
            .is_ok()
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
        let mut child = command.spawn()?;
        let stop_deadline = Instant::now() + REQUEST_TIMEOUT;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= stop_deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "backend stop command timed out",
                )
                .into());
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        if !status.success() && self.answers() {
            return Err(Error::Backend {
                method: "server stop",
                code: String::new(),
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
                code: String::new(),
                message: "the private socket is still accepting connections".to_owned(),
            });
        }
        Ok(())
    }

    /// Wait for an already-started daemon to answer, without starting another.
    pub fn reconnect(&self, timeout: Duration) -> Result<()> {
        let bounded = self.until(timeout);
        let deadline = bounded.deadline.expect("reconnect has a deadline");
        let mut backoff = Duration::from_millis(25);
        loop {
            match bounded.handshake() {
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
    /// A version mismatch is an error and not a warning. Direct attachment rides
    /// herdr's command line, so a daemon chartr did not ship is a daemon chartr
    /// cannot promise to render.
    pub fn handshake(&self) -> Result<()> {
        let pong: Pong = self.call("ping", &Empty {})?;
        if pong.version != SUPPORTED_HERDR_VERSION || pong.protocol != SUPPORTED_PROTOCOL {
            return Err(Error::IncompatibleDaemon {
                version: pong.version,
                protocol: pong.protocol,
            });
        }
        Ok(())
    }

    /// Every pane the private daemon is running, or every pane in one workspace.
    pub fn sessions(&self, workspace: Option<&WorkspaceId>) -> Result<Vec<Session>> {
        let bounded = self.until(REQUEST_TIMEOUT);
        let params = PaneListParams { workspace_id: workspace.map(|w| w.0.as_str()) };
        let list: PaneList = bounded.call("pane.list", &params)?;
        Ok(bounded.describe(list.panes))
    }

    /// Decorate Herdr panes with the same live titles used by chartr-rs:
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
        match self.call::<_, serde_json::Value>("pane.close", &PaneCloseParams { pane_id: &pane.0 })
        {
            Ok(_) => Ok(()),
            // Closing is a desired-state operation. If another request or the
            // pane's process won the race, the requested state already holds.
            Err(error) if error.is_backend_code("pane_not_found") => Ok(()),
            Err(error) => Err(error),
        }
    }

    /// Complete launch specification for Herdr's native interactive terminal
    /// client.
    ///
    /// `--takeover` makes chartr the sole controller after a relaunch instead
    /// of failing because a dead or superseded frontend still owns the stream.
    /// `/usr/bin/env -u` is the standard process adapter on both supported
    /// desktop targets; it removes inherited Herdr selectors before executing
    /// the exact vendored sidecar. The terminal host therefore consumes this
    /// value without knowing or reconstructing Herdr's namespace rules.
    pub fn direct_attach(&self, terminal: &TerminalId) -> DirectAttach {
        let mut env = HashMap::new();
        let mut args = Vec::new();
        for (key, value) in self.namespace.env() {
            let key = key.to_string_lossy().into_owned();
            match value {
                Some(value) => {
                    env.insert(key, value.to_string_lossy().into_owned());
                }
                None => {
                    args.push("-u".to_owned());
                    args.push(key);
                }
            }
        }
        args.push(self.sidecar.path().to_string_lossy().into_owned());
        args.extend([
            "terminal".to_owned(),
            "attach".to_owned(),
            terminal.0.clone(),
            "--takeover".to_owned(),
        ]);
        DirectAttach { program: PathBuf::from("/usr/bin/env"), args, env }
    }

    /// Send one request, read one response, close the connection.
    fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &'static str,
        params: &P,
    ) -> Result<R> {
        let socket = self.namespace.socket();
        let deadline = self
            .deadline
            .unwrap_or(Instant::now() + REQUEST_TIMEOUT)
            .min(Instant::now() + REQUEST_TIMEOUT);
        let mut stream = connect_socket(&socket, deadline).map_err(|err| {
            Error::Transport(std::io::Error::new(
                err.kind(),
                format!("{}: {err}", socket.display()),
            ))
        })?;
        stream.set_nonblocking(true)?;

        let request = Request { id: next_id(), method, params };
        let mut line = serde_json::to_vec(&request)
            .map_err(|err| Error::Protocol(format!("cannot encode {method}: {err}")))?;
        line.push(b'\n');
        let mut pending = line.as_slice();
        while !pending.is_empty() {
            remaining(deadline)?;
            match stream.write(pending) {
                Ok(0) => return Err(std::io::Error::from(std::io::ErrorKind::WriteZero).into()),
                Ok(n) => pending = &pending[n..],
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    wait_socket(&stream, rustix::event::PollFlags::OUT, deadline)?;
                }
                Err(error) => return Err(error.into()),
            }
        }

        let reply = read_reply(&stream, deadline)?;
        if reply.trim().is_empty() {
            return Err(Error::Protocol(format!("{method} got no answer")));
        }

        let response: Response<R> = serde_json::from_str(&reply)
            .map_err(|err| Error::Protocol(format!("cannot read the answer to {method}: {err}")))?;
        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (None, Some(error)) => {
                Err(Error::Backend { method, code: error.code, message: error.message })
            }
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

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REPLY_BYTES: usize = 8 * 1024 * 1024;

fn remaining(deadline: Instant) -> std::io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "backend request timed out")
        })
}

fn connect_socket(path: &Path, deadline: Instant) -> std::io::Result<UnixStream> {
    let socket = socket2::Socket::new(socket2::Domain::UNIX, socket2::Type::STREAM, None)?;
    socket.connect_timeout(&socket2::SockAddr::unix(path)?, remaining(deadline)?)?;
    Ok(std::os::fd::OwnedFd::from(socket).into())
}

fn wait_socket(
    stream: &UnixStream,
    flags: rustix::event::PollFlags,
    deadline: Instant,
) -> std::io::Result<()> {
    loop {
        let timeout = rustix::event::Timespec::try_from(remaining(deadline)?)
            .map_err(std::io::Error::other)?;
        let mut fds = [rustix::event::PollFd::new(stream, flags)];
        match rustix::event::poll(&mut fds, Some(&timeout)) {
            Ok(0) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "backend request timed out",
                ));
            }
            Ok(_) => return Ok(()),
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(error.into()),
        }
    }
}

/// Wait against one absolute deadline, so partial replies cannot keep a request
/// alive forever. Polling avoids changing socket options after a peer closes
/// (which can fail with EINVAL on macOS). Also bound replies without a newline.
fn read_reply(stream: &UnixStream, deadline: Instant) -> std::io::Result<String> {
    stream.set_nonblocking(true)?;
    let mut reader = BufReader::new(stream);
    let mut reply = Vec::new();
    loop {
        remaining(deadline)?;
        let buffer = match reader.fill_buf() {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                wait_socket(stream, rustix::event::PollFlags::IN, deadline)?;
                continue;
            }
            result => result?,
        };
        if buffer.is_empty() {
            break;
        }
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(buffer.len(), |at| at + 1);
        if reply.len() + count > MAX_REPLY_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "backend reply exceeds 8 MiB",
            ));
        }
        reply.extend_from_slice(&buffer[..count]);
        reader.consume(count);
        if newline.is_some() {
            break;
        }
    }
    String::from_utf8(reply)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
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
    use std::os::unix::net::UnixListener;

    use super::*;

    #[test]
    fn stalled_daemons_cannot_exceed_connect_or_reconnect_deadlines() {
        for connect in [Client::connect, Client::reconnect] {
            let temp = tempfile::tempdir().unwrap();
            let namespace = Namespace::rooted(temp.path().join("private"));
            namespace.prepare().unwrap();
            let sidecar = temp.path().join("herdr");
            std::fs::write(&sidecar, "fixture").unwrap();
            let listener = UnixListener::bind(namespace.socket()).unwrap();
            let (release, wait) = std::sync::mpsc::channel::<()>();
            let server = std::thread::spawn(move || {
                let (_stream, _) = listener.accept().unwrap();
                let _ = wait.recv_timeout(Duration::from_secs(2));
            });
            let client = Client::new(Sidecar::at(&sidecar).unwrap(), namespace);
            let start = Instant::now();
            assert!(connect(&client, Duration::from_millis(50)).is_err());
            assert!(start.elapsed() < Duration::from_secs(1));
            let _ = release.send(());
            server.join().unwrap();
        }
    }

    #[test]
    fn stalled_request_writes_are_bounded_too() {
        let temp = tempfile::tempdir().unwrap();
        let namespace = Namespace::rooted(temp.path().join("private"));
        namespace.prepare().unwrap();
        let sidecar = temp.path().join("herdr");
        std::fs::write(&sidecar, "fixture").unwrap();
        let listener = UnixListener::bind(namespace.socket()).unwrap();
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let server = std::thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            let _ = wait.recv_timeout(Duration::from_secs(2));
        });
        let client = Client::new(Sidecar::at(&sidecar).unwrap(), namespace);
        let start = Instant::now();
        let result: Result<Empty> =
            client.until(Duration::from_millis(100)).call("test", &"x".repeat(2 * 1024 * 1024));
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_secs(1));
        let _ = release.send(());
        server.join().unwrap();
    }

    #[test]
    fn partial_replies_do_not_extend_the_absolute_deadline() {
        let (reader, mut writer) = UnixStream::pair().unwrap();
        let server = std::thread::spawn(move || {
            for _ in 0..200 {
                if writer.write_all(b" ").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let start = Instant::now();
        assert!(read_reply(&reader, start + Duration::from_millis(50)).is_err());
        assert!(start.elapsed() < Duration::from_secs(1));
        drop(reader);
        server.join().unwrap();
    }

    #[test]
    fn oversized_replies_are_rejected() {
        let (reader, mut writer) = UnixStream::pair().unwrap();
        let server = std::thread::spawn(move || {
            let _ = writer.write_all(&vec![b' '; MAX_REPLY_BYTES + 1]);
        });
        let error = read_reply(&reader, Instant::now() + Duration::from_secs(2)).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData, "{error:?}");
        drop(reader);
        server.join().unwrap();
    }

    fn pane(id: &str, title: Option<&str>, agent: Option<&str>) -> protocol::Pane {
        protocol::Pane {
            pane_id: id.to_owned(),
            terminal_id: format!("term-{id}"),
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
        assert_eq!(Session::from(pane("p1", Some("   "), None)).title(), "p1");
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
            "terminal_id": "term1",
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
    fn direct_attach_uses_the_terminal_id_and_private_namespace() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let executable = tmp.path().join("herdr");
        std::fs::write(&executable, []).expect("sidecar fixture");
        let namespace = Namespace::rooted(tmp.path().join("private/herdr"));
        let client = Client::new(Sidecar::at(&executable).expect("sidecar"), namespace.clone());

        let attach = client.direct_attach(&TerminalId("terminal-7".to_owned()));

        assert_eq!(attach.program, PathBuf::from("/usr/bin/env"));
        assert!(attach.args.windows(2).any(|pair| pair == ["-u", "HERDR_PANE_ID"]));
        assert!(attach.args.windows(2).any(|pair| pair == ["-u", "HERDR_SESSION"]));
        assert_eq!(
            &attach.args[attach.args.len() - 5..],
            &[
                executable.to_string_lossy().into_owned(),
                "terminal".to_owned(),
                "attach".to_owned(),
                "terminal-7".to_owned(),
                "--takeover".to_owned(),
            ]
        );
        assert_eq!(
            attach.env.get("HERDR_SOCKET_PATH"),
            Some(&namespace.socket().to_string_lossy().into_owned())
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
    fn closing_an_already_absent_pane_is_successful() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let namespace = Namespace::rooted(tmp.path().join("private"));
        namespace.prepare().expect("prepare namespace");
        let herdr = tmp.path().join("herdr");
        std::fs::write(&herdr, b"sidecar fixture").expect("sidecar fixture");
        let listener = UnixListener::bind(namespace.socket()).expect("test daemon socket");
        let server = std::thread::spawn(move || {
            let (mut request, _) = listener.accept().expect("close request");
            let payload = read_test_request(&mut request);
            assert_eq!(payload["method"], "pane.close");
            assert_eq!(payload["params"]["pane_id"], "w1:p1");
            writeln!(
                request,
                r#"{{"id":"1","error":{{"code":"pane_not_found","message":"pane w1:p1 not found"}}}}"#
            )
            .expect("not-found response");
        });

        let client = Client::new(Sidecar::at(&herdr).expect("sidecar"), namespace);
        client.close_session(&PaneId("w1:p1".to_owned())).expect("absence is the desired state");
        server.join().expect("test daemon");
    }

    #[test]
    fn closing_preserves_other_backend_failures() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let namespace = Namespace::rooted(tmp.path().join("private"));
        namespace.prepare().expect("prepare namespace");
        let herdr = tmp.path().join("herdr");
        std::fs::write(&herdr, b"sidecar fixture").expect("sidecar fixture");
        let listener = UnixListener::bind(namespace.socket()).expect("test daemon socket");
        let server = std::thread::spawn(move || {
            let (mut request, _) = listener.accept().expect("close request");
            let _ = read_test_request(&mut request);
            writeln!(
                request,
                r#"{{"id":"1","error":{{"code":"permission_denied","message":"not allowed"}}}}"#
            )
            .expect("failure response");
        });

        let client = Client::new(Sidecar::at(&herdr).expect("sidecar"), namespace);
        let error = client
            .close_session(&PaneId("w1:p1".to_owned()))
            .expect_err("a real rejection must remain visible");
        assert!(error.is_backend_code("permission_denied"));
        assert_eq!(error.to_string(), "herdr rejected pane.close: not allowed (permission_denied)");
        server.join().expect("test daemon");
    }

    #[test]
    fn connect_live_handoffs_an_incompatible_private_daemon() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let namespace = Namespace::rooted(tmp.path().join("private"));
        namespace.prepare().expect("prepare namespace");
        let herdr = tmp.path().join("herdr");
        std::fs::write(&herdr, b"sidecar fixture").expect("sidecar fixture");
        let listener = UnixListener::bind(namespace.socket()).expect("test daemon socket");
        let expected_exe = herdr.to_string_lossy().into_owned();

        let server = std::thread::spawn(move || {
            let (mut first_ping, _) = listener.accept().expect("first ping");
            let request = read_test_request(&mut first_ping);
            assert_eq!(request["method"], "ping");
            writeln!(first_ping, r#"{{"id":"1","result":{{"version":"0.8.0","protocol":19}}}}"#)
                .expect("old ping response");

            // `answers` deliberately performs only an OS-level connect.
            let (_probe, _) = listener.accept().expect("socket probe");

            let (mut handoff, _) = listener.accept().expect("handoff request");
            let request = read_test_request(&mut handoff);
            assert_eq!(request["method"], "server.live_handoff");
            assert_eq!(request["params"]["import_exe"], expected_exe);
            assert_eq!(request["params"]["expected_protocol"], SUPPORTED_PROTOCOL);
            assert_eq!(request["params"]["expected_version"], SUPPORTED_HERDR_VERSION);
            writeln!(handoff, r#"{{"id":"2","result":{{}}}}"#).expect("handoff response");

            let (mut final_ping, _) = listener.accept().expect("final ping");
            let request = read_test_request(&mut final_ping);
            assert_eq!(request["method"], "ping");
            writeln!(
                final_ping,
                r#"{{"id":"3","result":{{"version":"{SUPPORTED_HERDR_VERSION}","protocol":{SUPPORTED_PROTOCOL}}}}}"#
            )
            .expect("new ping response");
        });

        let client = Client::new(Sidecar::at(&herdr).expect("sidecar"), namespace);
        client.connect(Duration::from_secs(1)).expect("handoff reaches the pinned daemon");
        server.join().expect("test daemon");
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

    fn read_test_request(stream: &mut UnixStream) -> serde_json::Value {
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).expect("read request");
        serde_json::from_str(&line).expect("request JSON")
    }
}
