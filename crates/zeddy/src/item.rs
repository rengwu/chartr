//! Runtime items owned by a Chartr pane.
//!
//! The workspace model owns stable [`ItemId`](crate::workspace::ItemId) values;
//! this module owns the corresponding live object. Catalog entries are not
//! items. Opening a plugin creates one `PluginItem`, just as attaching a Herdr
//! session creates one `SessionItem`.

use gpui::{AnyView, Entity};
use zeddy_plugin::PaneKey;

use crate::session::Session;

/// A plugin view plus any host-owned resource teardown that must happen when
/// the item leaves the workspace. GPUI may retain a view entity briefly after
/// its last render, so native resources must not rely on entity destruction as
/// their close signal.
pub struct PluginView {
    view: AnyView,
    close: Option<Box<dyn FnOnce()>>,
}

impl PluginView {
    pub fn new(view: AnyView) -> Self {
        Self { view, close: None }
    }

    pub fn with_close(view: AnyView, close: impl FnOnce() + 'static) -> Self {
        Self { view, close: Some(Box::new(close)) }
    }

    pub fn any_view(&self) -> &AnyView {
        &self.view
    }

    pub fn clone_view(&self) -> AnyView {
        self.view.clone()
    }
}

impl Drop for PluginView {
    fn drop(&mut self) {
        if let Some(close) = self.close.take() {
            close();
        }
    }
}

pub enum Item {
    Session(SessionItem),
    Plugin(PluginItem),
    /// A temporary, space-owned item that presents the plugin surface picker.
    /// Selecting a contribution replaces this item at the same stable id, so
    /// the resulting plugin stays in this tab (and pane, if it was moved).
    PluginLauncher {
        /// Session-bound contributions bind to the terminal that was active
        /// when the launcher tab was created.
        bound_session: Option<zeddy_herdr::PaneId>,
    },
}

impl Item {
    pub fn title(&self) -> String {
        match self {
            Self::Session(item) => item.session.title(),
            Self::Plugin(item) => item.title.clone(),
            Self::PluginLauncher { .. } => "New Plugin Pane".to_owned(),
        }
    }

    pub fn status(&self) -> Option<zeddy_herdr::control::SessionStatus> {
        match self {
            Self::Session(item) => Some(item.session.info.status),
            Self::Plugin(_) | Self::PluginLauncher { .. } => None,
        }
    }

    pub fn process_running(&self) -> bool {
        matches!(self, Self::Session(item) if item.session.info.process_running())
    }

    pub fn ended(&self) -> bool {
        matches!(self, Self::Session(item) if item.session.ended().is_some())
    }

    pub fn as_session(&self) -> Option<&SessionItem> {
        match self {
            Self::Session(item) => Some(item),
            Self::Plugin(_) | Self::PluginLauncher { .. } => None,
        }
    }

    pub fn as_session_mut(&mut self) -> Option<&mut SessionItem> {
        match self {
            Self::Session(item) => Some(item),
            Self::Plugin(_) | Self::PluginLauncher { .. } => None,
        }
    }

    pub fn as_plugin(&self) -> Option<&PluginItem> {
        match self {
            Self::Plugin(item) => Some(item),
            Self::Session(_) | Self::PluginLauncher { .. } => None,
        }
    }

    pub fn is_plugin_launcher(&self) -> bool {
        matches!(self, Self::PluginLauncher { .. })
    }

    pub fn plugin_launcher_bound_session(&self) -> Option<&zeddy_herdr::PaneId> {
        match self {
            Self::PluginLauncher { bound_session } => bound_session.as_ref(),
            Self::Session(_) | Self::Plugin(_) => None,
        }
    }
}

pub struct SessionItem {
    pub session: Session,
    view: Option<Entity<terminal_view::TerminalView>>,
    bell: bool,
}

impl SessionItem {
    pub fn new(session: Session) -> Self {
        Self { session, view: None, bell: false }
    }

    pub fn terminal_view(&self) -> Option<Entity<terminal_view::TerminalView>> {
        self.view.clone()
    }

    pub fn install_terminal_view(&mut self, view: Entity<terminal_view::TerminalView>) {
        self.view = Some(view);
    }

    pub fn clear_terminal_view(&mut self) {
        self.view = None;
        self.bell = false;
    }

    pub fn bell(&self) -> bool {
        self.bell
    }

    pub fn set_bell(&mut self, bell: bool) -> bool {
        if self.bell == bell {
            return false;
        }
        self.bell = bell;
        true
    }
}

pub struct PluginItem {
    pub contribution: PaneKey,
    pub title: String,
    pub view: PluginView,
    /// A session-specific plugin closes when this Herdr session ends.
    pub bound_session: Option<zeddy_herdr::PaneId>,
    pub can_clone: bool,
    pub restorable: bool,
}
