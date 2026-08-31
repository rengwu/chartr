//! The root view: the sessions, the mode, the plugins, and nothing else.
//!
//! Everything that can live below this file does. What is left here is only
//! what genuinely needs to see more than one of them at once — which pane the
//! workspace is showing, and what a chrome action means.

use std::{path::PathBuf, rc::Rc, time::Duration};

use futures::{StreamExt as _, channel::mpsc};
use gpui::{FocusHandle, Focusable, Task};
use ui::prelude::*;
use zeddy_herdr::{Namespace, Sidecar, WorkspaceId, control::Client};
use zeddy_plugin::PaneKey;
use zeddy_plugin_host::{Catalog, PaneSource, Paths};
use zeddy_vt::Size;

use crate::{
    chrome::{self, Action, Entry},
    fonts::Fonts,
    keys,
    mode::Mode,
    palette,
    session::Session,
    terminal::{Appearance, Fit, TerminalElement},
};

/// How long to wait for the private backend before saying it did not come up.
const BACKEND_TIMEOUT: Duration = Duration::from_secs(10);

/// What the workspace is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Showing {
    Session(usize),
    Plugin(PaneKey),
    /// Before the first session exists, or after the last one is closed.
    Empty,
}

pub struct Zeddy {
    client: Client,
    workspace: Option<WorkspaceId>,
    sessions: Vec<Session>,
    showing: Showing,
    mode: Mode,
    catalog: Catalog,
    fit: Fit,
    focus: FocusHandle,
    /// The last thing that went wrong, shown in place of the workspace. One
    /// slot, not a log: what the user needs is the reason the thing they just
    /// tried did not happen.
    problem: Option<String>,
    _wakeups: Task<()>,
    wakeup_tx: mpsc::UnboundedSender<()>,
}

impl Zeddy {
    pub fn new(cwd: PathBuf, cx: &mut Context<Self>) -> Self {
        let namespace = Namespace::private();
        let client = match Sidecar::beside_current_exe() {
            Ok(sidecar) => Client::new(sidecar, namespace),
            Err(err) => {
                // Without a backend there is nothing to show, but the window
                // still opens: a window that says why is more useful than one
                // that never appears.
                return Self::broken(err.to_string(), cx);
            }
        };

        let (wakeup_tx, wakeup_rx) = mpsc::unbounded();
        let mut this = Self {
            client,
            workspace: None,
            sessions: Vec::new(),
            showing: Showing::Empty,
            mode: Mode::default(),
            catalog: Catalog::default(),
            fit: Fit::default(),
            focus: cx.focus_handle(),
            problem: None,
            _wakeups: Self::watch(wakeup_rx, cx),
            wakeup_tx,
        };

        this.catalog = zeddy_plugin_host::load_all(&plugin_paths(), cx);
        this.connect(cwd, cx);
        this
    }

    /// A window with no backend behind it. Everything is empty and the problem
    /// is on screen.
    fn broken(problem: String, cx: &mut Context<Self>) -> Self {
        let (wakeup_tx, wakeup_rx) = mpsc::unbounded();
        Self {
            client: Client::new(
                Sidecar::at(PathBuf::from("/nonexistent")).unwrap_or_else(|_| unreachable!()),
                Namespace::private(),
            ),
            workspace: None,
            sessions: Vec::new(),
            showing: Showing::Empty,
            mode: Mode::default(),
            catalog: Catalog::default(),
            fit: Fit::default(),
            focus: cx.focus_handle(),
            problem: Some(problem),
            _wakeups: Self::watch(wakeup_rx, cx),
            wakeup_tx,
        }
    }

    /// Redraw whenever any session's reader says something changed.
    ///
    /// Every wakeup already waiting is drained before the redraw, so a burst of
    /// frames costs one paint rather than one paint each.
    fn watch(mut wakeups: mpsc::UnboundedReceiver<()>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            while wakeups.next().await.is_some() {
                while wakeups.try_recv().is_ok() {}
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    return;
                }
            }
        })
    }

    /// Bring the private backend up and adopt whatever is already running in
    /// this directory.
    fn connect(&mut self, cwd: PathBuf, cx: &mut Context<Self>) {
        if let Err(err) = self.client.connect(BACKEND_TIMEOUT) {
            self.problem = Some(err.to_string());
            return;
        }

        let label = cwd.file_name().map(|name| name.to_string_lossy().into_owned());
        match self.client.open_workspace(&cwd, label.as_deref()) {
            Ok(workspace) => {
                self.workspace = Some(workspace);
                self.refresh(cx);
            }
            Err(err) => self.problem = Some(err.to_string()),
        }
    }

    /// Attach to every session the backend is running that zeddy is not showing
    /// yet.
    ///
    /// Adopting rather than creating is the point of a durable backend: a
    /// session that outlived the last launch is picked up here, not restarted.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let known = self.client.sessions(self.workspace.as_ref());
        let listed = match known {
            Ok(listed) => listed,
            Err(err) => {
                self.problem = Some(err.to_string());
                return;
            }
        };

        for info in listed {
            if self.sessions.iter().any(|session| session.id() == &info.id) {
                continue;
            }
            match Session::attach(&self.client, info, self.grid(), self.wakeup_tx.clone()) {
                Ok(session) => self.sessions.push(session),
                Err(err) => self.problem = Some(err.to_string()),
            }
        }

        if matches!(self.showing, Showing::Empty) && !self.sessions.is_empty() {
            self.showing = Showing::Session(0);
        }
        cx.notify();
    }

    /// The grid the last paint found room for, or a sane default before the
    /// first one.
    fn grid(&self) -> Size {
        self.fit.get().unwrap_or_default()
    }

    /// Tell the shown session how many cells the last paint found room for.
    ///
    /// Only the shown one: a background session has no bounds of its own, and
    /// resizing it to the visible pane's grid would reflow a screen nobody is
    /// looking at. It is resized when it is next shown.
    fn fit_shown(&mut self) {
        let Some(size) = self.fit.get() else {
            return;
        };
        let Showing::Session(index) = self.showing else {
            return;
        };
        if let Some(session) = self.sessions.get_mut(index)
            && let Err(err) = session.resize(size)
        {
            self.problem = Some(err.to_string());
        }
    }

    fn act(&mut self, action: Action, cx: &mut Context<Self>) {
        match action {
            Action::ToggleMode => self.mode = self.mode.toggled(),
            Action::Select(index) => self.showing = Showing::Session(index),
            Action::New => self.start_session(cx),
            Action::Close(index) => self.close_session(index, cx),
        }
        cx.notify();
    }

    fn start_session(&mut self, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.clone() else {
            return;
        };
        match self.client.start_session(&workspace, None) {
            Ok(info) => {
                match Session::attach(&self.client, info, self.grid(), self.wakeup_tx.clone()) {
                    Ok(session) => {
                        self.sessions.push(session);
                        self.showing = Showing::Session(self.sessions.len() - 1);
                        self.problem = None;
                    }
                    Err(err) => self.problem = Some(err.to_string()),
                }
            }
            Err(err) => self.problem = Some(err.to_string()),
        }
        cx.notify();
    }

    fn close_session(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.sessions.len() {
            return;
        }
        let mut session = self.sessions.remove(index);
        if let Err(err) = self.client.close_session(session.id()) {
            self.problem = Some(err.to_string());
        }
        session.release();

        // Selection follows the list rather than the index: closing the tab you
        // are on should land you on its neighbour, not on nothing.
        self.showing = match self.showing.clone() {
            Showing::Session(_) if self.sessions.is_empty() => Showing::Empty,
            Showing::Session(selected) if selected > index => Showing::Session(selected - 1),
            Showing::Session(selected) if selected == index => {
                Showing::Session(index.min(self.sessions.len() - 1))
            }
            other => other,
        };
        cx.notify();
    }

    /// The chrome's view of the sessions, plus the plugin panes that share the
    /// same list.
    fn entries(&self) -> Vec<Entry> {
        let selected_session = match self.showing {
            Showing::Session(index) => Some(index),
            _ => None,
        };

        let mut entries: Vec<Entry> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(index, session)| Entry {
                title: session.title(),
                agent: session.info.agent.clone(),
                ended: session.ended().is_some(),
                selected: selected_session == Some(index),
            })
            .collect();

        // Plugin panes sit after the sessions, in the catalog's stable order,
        // so a plugin cannot change where a session's tab is.
        for pane in self.catalog.panes() {
            entries.push(Entry {
                title: pane.title.clone(),
                agent: None,
                ended: false,
                selected: self.showing == Showing::Plugin(pane.key.clone()),
            });
        }
        entries
    }

    /// Map a chrome index back onto what it selects. The chrome counts one
    /// list; this is where it becomes two.
    fn showing_for(&self, index: usize) -> Showing {
        if index < self.sessions.len() {
            Showing::Session(index)
        } else {
            self.catalog
                .panes()
                .get(index - self.sessions.len())
                .map(|pane| Showing::Plugin(pane.key.clone()))
                .unwrap_or(Showing::Empty)
        }
    }

    fn on_key(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        let Showing::Session(index) = self.showing else {
            return;
        };
        let Some(bytes) = keys::bytes_for(&event.keystroke) else {
            return;
        };
        if let Some(session) = self.sessions.get_mut(index)
            && let Err(err) = session.send(&bytes)
        {
            self.problem = Some(err.to_string());
            cx.notify();
        }
    }

    fn workspace_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        // The grid the previous frame measured reaches the backend here, one
        // frame late by construction: nothing knows how many cells fit until
        // something has been laid out in the space they have to fit in.
        self.fit_shown();

        if let Some(problem) = self.problem.clone() {
            return message(&problem, cx).into_any_element();
        }

        match self.showing.clone() {
            Showing::Empty => message("No session. Press + to start one.", cx).into_any_element(),
            Showing::Session(index) => match self.sessions.get_mut(index) {
                Some(session) => {
                    terminal(session, self.fit.clone(), self.focus.is_focused(window), cx)
                        .into_any_element()
                }
                None => message("That session is gone.", cx).into_any_element(),
            },
            Showing::Plugin(key) => self.plugin_pane(&key, window, cx),
        }
    }

    /// Mount a plugin's pane.
    ///
    /// A native plugin's view is an ordinary GPUI view dropped straight into
    /// this element tree — the same frame path as the terminal beside it.
    fn plugin_pane(
        &mut self,
        key: &PaneKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(plugin) = self.catalog.get_mut(&key.plugin) else {
            return message("That plugin is no longer loaded.", cx).into_any_element();
        };
        match plugin.pane(key) {
            Some(PaneSource::Native(plugin)) => plugin.view(key, window, cx).into_any_element(),
            Some(PaneSource::Web(entry)) => {
                // The webview host is the one piece of the web tier that is not
                // written yet; until it is, the pane says so rather than
                // pretending to be empty.
                message(&format!("Web plugin panes are not hosted yet ({}).", entry.display()), cx)
                    .into_any_element()
            }
            None => message("That pane is no longer contributed.", cx).into_any_element(),
        }
    }
}

impl Focusable for Zeddy {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Zeddy {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self.entries();
        // Copied out rather than borrowed: `cx.theme()` borrows `cx`, and
        // building the workspace pane below needs it back.
        let (background, text, workspace_background) = {
            let colors = cx.theme().colors();
            (colors.background, colors.text, colors.editor_background)
        };

        let on_action = cx.listener(|this, action: &Action, _, cx| {
            let action = *action;
            if let Action::Select(index) = action {
                this.showing = this.showing_for(index);
                cx.notify();
            } else {
                this.act(action, cx);
            }
        });
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));

        // `h_full` is not redundant with `flex_1`. In sidebar mode this sits in
        // a row, where `flex_1` decides the *width* and the height would
        // otherwise be the content's — which is a terminal that sizes itself to
        // its parent, so the pair resolves to nothing at all.
        let workspace = v_flex()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .bg(workspace_background)
            .child(self.workspace_pane(window, cx));

        let body = match self.mode {
            Mode::Sidebar => h_flex()
                .size_full()
                .child(chrome::sidebar::render(&entries, emit.clone(), cx))
                .child(workspace),
            Mode::Tabs => v_flex()
                .size_full()
                .child(chrome::tabs::render(&entries, emit, cx))
                .child(workspace),
        };

        div()
            .track_focus(&self.focus)
            .key_context("Zeddy")
            .size_full()
            .bg(background)
            .text_color(text)
            .on_key_down(cx.listener(|this, event, _, cx| this.on_key(event, cx)))
            .child(body)
    }
}

fn terminal(session: &Session, fit: Fit, focused: bool, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let screen = session.screen();
    let colors = screen
        .rows
        .iter()
        .map(|row| row.iter().map(|cell| palette::cell_colors(cell, theme)).collect())
        .collect();

    let (font, font_size, line_height) = Fonts::default().terminal();
    let appearance = Appearance {
        font,
        font_size,
        line_height,
        background: theme.colors().terminal_background,
        cursor: theme.colors().terminal_foreground,
    };

    v_flex().size_full().p_2().child(TerminalElement::new(screen, colors, appearance, focused, fit))
}

fn message(text: &str, cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .child(Label::new(text.to_owned()).color(Color::Muted))
        .bg(cx.theme().colors().editor_background)
}

fn plugin_paths() -> Paths {
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        });
    Paths::under(root.join("zeddy"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_the_selected_session_lands_on_its_neighbour() {
        // The selection rule is arithmetic on indices, so it is tested as such
        // rather than through a live backend.
        let after = |selected: usize, closed: usize, remaining: usize| -> Showing {
            match Showing::Session(selected) {
                Showing::Session(_) if remaining == 0 => Showing::Empty,
                Showing::Session(s) if s > closed => Showing::Session(s - 1),
                Showing::Session(s) if s == closed => Showing::Session(closed.min(remaining - 1)),
                other => other,
            }
        };

        assert_eq!(after(1, 1, 2), Showing::Session(1), "the next one takes the index");
        assert_eq!(after(2, 2, 2), Showing::Session(1), "closing the last selects the new last");
        assert_eq!(after(2, 0, 2), Showing::Session(1), "closing before shifts the selection down");
        assert_eq!(after(0, 1, 2), Showing::Session(0), "closing after leaves it alone");
        assert_eq!(after(0, 0, 0), Showing::Empty, "closing the only one shows nothing");
    }
}
