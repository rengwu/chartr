//! One persistent Herdr session hosted by Zed's complete terminal engine.
//!
//! Herdr still owns the long-lived PTY. chartr launches Herdr's interactive
//! `terminal attach` client inside a local PTY created by Zed, so Zed receives
//! ordinary terminal bytes and owns emulation, rendering, resizing, keyboard,
//! paste, selection, and mouse reporting as one coherent implementation.

mod launch;

use std::{
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use chartr_herdr::{PaneId, control};
use collections::HashMap;
use futures::{StreamExt as _, channel::mpsc};
use gpui::{App, AppContext as _, Context, Entity, Task};
use settings::Settings as _;
use task::Shell;
use terminal::{Terminal, TerminalBuilder, terminal_settings::TerminalSettings};
use util::paths::PathStyle;

/// The local attach client's outcome, once it stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    /// The attach client closed. The server-owned session may still be alive.
    Closed,
}

/// One attached session. Dropping the terminal ends only the local Herdr
/// client; Herdr's server-owned PTY remains available for the next attachment.
pub struct Session {
    pub info: control::Session,
    terminal: Entity<Terminal>,
    ended: Option<Ended>,
    endpoint: Arc<Mutex<SessionEndpoint>>,
    _input_task: Task<()>,
}

impl Session {
    /// Build Zed's PTY around Herdr's namespace-safe direct-attach command.
    pub fn attach_builder(
        client: &control::Client,
        info: &control::Session,
        window_id: u64,
        cx: &App,
    ) -> Task<anyhow::Result<TerminalBuilder>> {
        let attach = client.direct_attach(&info.terminal);
        let settings = TerminalSettings::get_global(cx).clone();

        let shell = Shell::WithArguments {
            program: attach.program.to_string_lossy().into_owned(),
            args: attach.args,
            title_override: Some(info.title().to_owned()),
        };
        let env: HashMap<String, String> = attach.env.into_iter().collect();

        TerminalBuilder::new(
            info.cwd.clone(),
            None,
            shell,
            env,
            settings.cursor_shape,
            settings.alternate_scroll,
            settings.max_scroll_history_lines,
            settings.path_hyperlink_regexes,
            Duration::from_millis(settings.path_hyperlink_timeout_ms),
            false,
            window_id,
            None,
            cx,
            Vec::new(),
            PathStyle::local(),
        )
    }

    pub fn from_builder(
        info: control::Session,
        builder: TerminalBuilder,
        cx: &mut Context<crate::space::Space>,
    ) -> Self {
        let terminal = cx.new(|cx| builder.subscribe(cx));
        let weak_terminal = terminal.downgrade();
        let (input_tx, mut input_rx) = mpsc::unbounded::<Input>();
        let executor = cx.background_executor().clone();
        let input_task = cx.spawn(async move |space, cx| {
            while let Some(input) = input_rx.next().await {
                let (bytes, launch) = match input {
                    Input::Raw(bytes) => (bytes, None),
                    Input::Launch(launch) => (launch.start.clone(), Some(launch)),
                };
                if weak_terminal.update(cx, |terminal, _| terminal.input(bytes)).is_err() {
                    return;
                }
                if let Some(launch) = launch {
                    let deadline = executor.now() + Duration::from_secs(30);
                    while !launch.files.ready() && executor.now() < deadline {
                        executor.timer(Duration::from_millis(10)).await;
                    }
                    if !launch.files.ready() {
                        let _ = space.update(cx, |space, cx| {
                            space.report_launch_error(
                                "The agent launch did not start; its prompt was not sent.".into(),
                                cx,
                            );
                        });
                        continue;
                    }
                    launch.files.acknowledge();
                    if !launch.input.is_empty()
                        && weak_terminal
                            .update(cx, |terminal, _| terminal.input(launch.input))
                            .is_err()
                    {
                        return;
                    }
                }
            }
        });

        let endpoint = Arc::new(Mutex::new(SessionEndpoint {
            info: info.clone(),
            input_tx,
            launch_scripts: Vec::new(),
        }));
        Self { info, terminal, ended: None, endpoint, _input_task: input_task }
    }

    pub fn id(&self) -> &PaneId {
        &self.info.id
    }

    pub fn terminal(&self) -> Entity<Terminal> {
        self.terminal.clone()
    }

    pub fn ended(&self) -> Option<Ended> {
        self.ended.clone()
    }

    pub fn mark_ended(&mut self) {
        self.ended = Some(Ended::Closed);
    }

    /// The live title inferred by the control plane: detected agent, foreground
    /// process, then Herdr's persistent tab label.
    pub fn title(&self) -> String {
        self.info.title().to_owned()
    }

    pub fn access(&self) -> SessionAccess {
        SessionAccess(Arc::downgrade(&self.endpoint))
    }

    pub fn update_info(&mut self, info: control::Session) {
        self.endpoint.lock().unwrap().info = info.clone();
        self.info = info;
    }

    /// Keep plugin capabilities attached to the same persistent session when
    /// its local terminal and input task are replaced.
    pub fn replace_attachment(&mut self, mut replacement: Self) {
        let mut endpoint = self.endpoint.lock().unwrap();
        let mut next = replacement.endpoint.lock().unwrap().clone();
        next.launch_scripts = std::mem::take(&mut endpoint.launch_scripts);
        *endpoint = next;
        drop(endpoint);
        replacement.endpoint = self.endpoint.clone();
        *self = replacement;
    }
}

/// Thread-safe capability passed to session-bound web plugins.
#[derive(Clone)]
pub struct SessionAccess(Weak<Mutex<SessionEndpoint>>);

#[derive(Clone)]
struct SessionEndpoint {
    info: control::Session,
    input_tx: mpsc::UnboundedSender<Input>,
    // Keep unsent/unconsumed launch scripts alive across attachment replacement.
    // The dedicated shell removes its script; dropping the session cleans up failures.
    launch_scripts: Vec<Arc<launch::LaunchFiles>>,
}

enum Input {
    Raw(Vec<u8>),
    Launch(launch::StagedLaunch),
}

impl SessionAccess {
    pub fn info(&self) -> chartr_herdr::Result<control::Session> {
        let endpoint = self.0.upgrade().ok_or_else(session_unavailable)?;
        Ok(endpoint.lock().unwrap().info.clone())
    }

    pub fn send(&self, bytes: &[u8]) -> chartr_herdr::Result<()> {
        let endpoint = self.0.upgrade().ok_or_else(session_unavailable)?;
        endpoint
            .lock()
            .unwrap()
            .input_tx
            .unbounded_send(Input::Raw(bytes.to_vec()))
            .map_err(|_| session_unavailable())
    }

    /// Launch commands are shell input; ordinary terminal keystrokes remain raw.
    pub fn send_shell_launch(
        &self,
        launch: &chartr_plugin::TerminalLaunch,
    ) -> chartr_herdr::Result<()> {
        let endpoint = self.0.upgrade().ok_or_else(session_unavailable)?;
        let mut endpoint = endpoint.lock().unwrap();
        let staged = launch::stage(launch).map_err(|error| {
            chartr_herdr::Error::Protocol(format!("Preparing agent launch: {error}"))
        })?;
        let files = staged.files.clone();
        endpoint
            .input_tx
            .unbounded_send(Input::Launch(staged))
            .map_err(|_| session_unavailable())?;
        endpoint.launch_scripts.retain(|files| files.pending());
        endpoint.launch_scripts.push(files);
        Ok(())
    }
}

fn session_unavailable() -> chartr_herdr::Error {
    chartr_herdr::Error::Protocol("terminal is no longer available".to_owned())
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").field("id", &self.info.id).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn plugin_access_observes_updates_replacement_input_and_session_close(
        cx: &mut gpui::TestAppContext,
    ) {
        let info = control::Session {
            id: PaneId("session".into()),
            terminal: chartr_herdr::TerminalId("terminal".into()),
            workspace: chartr_herdr::WorkspaceId("workspace".into()),
            label: "before".into(),
            running: None,
            status: control::SessionStatus::Unknown,
            agent: None,
            agent_session: None,
            conversation_title: None,
            foreground_pid: None,
            cwd: None,
        };
        let mut attachment = |info: control::Session| {
            let terminal = cx.new(|cx| {
                TerminalBuilder::new_display_only(
                    terminal::terminal_settings::CursorShape::default(),
                    terminal::terminal_settings::AlternateScroll::On,
                    None,
                    0,
                    cx.background_executor(),
                    PathStyle::local(),
                )
                .subscribe(cx)
            });
            let (input_tx, input_rx) = mpsc::unbounded();
            let endpoint = Arc::new(Mutex::new(SessionEndpoint {
                info: info.clone(),
                input_tx,
                launch_scripts: Vec::new(),
            }));
            (
                Session { info, terminal, ended: None, endpoint, _input_task: Task::ready(()) },
                input_rx,
            )
        };
        let (mut session, mut original_rx) = attachment(info.clone());
        let access = session.access();
        access.send(b"first").unwrap();
        assert!(
            matches!(futures::executor::block_on(original_rx.next()), Some(Input::Raw(bytes)) if bytes == b"first")
        );
        let mut info = info;
        info.label = "after".into();
        session.update_info(info.clone());
        assert_eq!(access.info().unwrap().title(), "after");
        access
            .send_shell_launch(&chartr_plugin::TerminalLaunch {
                command: format!("printf '%s' '{}'", "x".repeat(5000)),
                input: Vec::new(),
            })
            .unwrap();
        let script = session.endpoint.lock().unwrap().launch_scripts[0].script.to_path_buf();
        let Some(Input::Launch(staged)) = futures::executor::block_on(original_rx.next()) else {
            panic!("expected launch")
        };
        assert!(staged.start.starts_with(b"exec /bin/sh ") && staged.start.len() < 1024);
        drop(staged);
        assert!(script.exists());
        let (replacement, mut replacement_rx) = attachment(info);
        let replacement_terminal = replacement.terminal().entity_id();
        session.replace_attachment(replacement);
        assert!(script.exists(), "reattaching must not discard a pending launch");
        assert_eq!(session.terminal().entity_id(), replacement_terminal);
        access.send(b"replacement").unwrap();
        assert!(
            matches!(futures::executor::block_on(replacement_rx.next()), Some(Input::Raw(bytes)) if bytes == b"replacement")
        );
        assert!(futures::executor::block_on(original_rx.next()).is_none());
        drop(session);
        assert!(!script.exists(), "closing the session cleans up an unconsumed launch");
        assert!(access.send(b"closed").is_err());
        assert!(access.info().is_err());
    }
}
