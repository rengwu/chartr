//! One persistent Herdr session hosted by Zed's complete terminal engine.
//!
//! Herdr still owns the long-lived PTY. Chartr launches Herdr's interactive
//! `terminal attach` client inside a local PTY created by Zed, so Zed receives
//! ordinary terminal bytes and owns emulation, rendering, resizing, keyboard,
//! paste, selection, and mouse reporting as one coherent implementation.

use std::{
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use collections::HashMap;
use futures::{StreamExt as _, channel::mpsc};
use gpui::{App, AppContext as _, Context, Entity, Task};
use settings::Settings as _;
use task::Shell;
use terminal::{Terminal, TerminalBuilder, terminal_settings::TerminalSettings};
use util::paths::PathStyle;
use zeddy_herdr::{PaneId, control};

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
        let (input_tx, mut input_rx) = mpsc::unbounded::<Vec<u8>>();
        let input_task = cx.spawn(async move |_, cx| {
            while let Some(bytes) = input_rx.next().await {
                if weak_terminal.update(cx, |terminal, _| terminal.input(bytes)).is_err() {
                    return;
                }
            }
        });

        let endpoint = Arc::new(Mutex::new(SessionEndpoint { info: info.clone(), input_tx }));
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
        *self.endpoint.lock().unwrap() = replacement.endpoint.lock().unwrap().clone();
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
    input_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl SessionAccess {
    pub fn info(&self) -> zeddy_herdr::Result<control::Session> {
        let endpoint = self.0.upgrade().ok_or_else(session_unavailable)?;
        Ok(endpoint.lock().unwrap().info.clone())
    }

    pub fn send(&self, bytes: &[u8]) -> zeddy_herdr::Result<()> {
        let endpoint = self.0.upgrade().ok_or_else(session_unavailable)?;
        endpoint
            .lock()
            .unwrap()
            .input_tx
            .unbounded_send(bytes.to_vec())
            .map_err(|_| session_unavailable())
    }
}

fn session_unavailable() -> zeddy_herdr::Error {
    zeddy_herdr::Error::Protocol("terminal is no longer available".to_owned())
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
            terminal: zeddy_herdr::TerminalId("terminal".into()),
            workspace: zeddy_herdr::WorkspaceId("workspace".into()),
            label: "before".into(),
            running: None,
            status: control::SessionStatus::Unknown,
            agent: None,
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
            let endpoint = Arc::new(Mutex::new(SessionEndpoint { info: info.clone(), input_tx }));
            (
                Session { info, terminal, ended: None, endpoint, _input_task: Task::ready(()) },
                input_rx,
            )
        };
        let (mut session, mut original_rx) = attachment(info.clone());
        let access = session.access();
        access.send(b"first").unwrap();
        assert_eq!(futures::executor::block_on(original_rx.next()).unwrap(), b"first");
        let mut info = info;
        info.label = "after".into();
        session.update_info(info.clone());
        assert_eq!(access.info().unwrap().title(), "after");
        let (replacement, mut replacement_rx) = attachment(info);
        let replacement_terminal = replacement.terminal().entity_id();
        session.replace_attachment(replacement);
        assert_eq!(session.terminal().entity_id(), replacement_terminal);
        access.send(b"replacement").unwrap();
        assert_eq!(futures::executor::block_on(replacement_rx.next()).unwrap(), b"replacement");
        assert_eq!(futures::executor::block_on(original_rx.next()), None);
        drop(session);
        assert!(access.send(b"closed").is_err());
        assert!(access.info().is_err());
    }
}
