//! One persistent Herdr session hosted by Zed's complete terminal engine.
//!
//! Herdr still owns the long-lived PTY. Chartr launches Herdr's interactive
//! `terminal attach` client inside a local PTY created by Zed, so Zed receives
//! ordinary terminal bytes and owns emulation, rendering, resizing, keyboard,
//! paste, selection, and mouse reporting as one coherent implementation.

use std::time::Duration;

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
    input_tx: mpsc::UnboundedSender<Vec<u8>>,
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

        Self { info, terminal, ended: None, input_tx, _input_task: input_task }
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
        SessionAccess { info: self.info.clone(), input_tx: self.input_tx.clone() }
    }
}

/// Thread-safe capability passed to session-bound web plugins.
#[derive(Clone)]
pub struct SessionAccess {
    pub info: control::Session,
    input_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl SessionAccess {
    pub fn send(&self, bytes: &[u8]) -> zeddy_herdr::Result<()> {
        self.input_tx
            .unbounded_send(bytes.to_vec())
            .map_err(|_| zeddy_herdr::Error::Protocol("terminal is no longer available".to_owned()))
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").field("id", &self.info.id).finish_non_exhaustive()
    }
}
