//! Runtime items owned by a Chartr pane.
//!
//! The workspace model owns stable [`ItemId`](crate::workspace::ItemId) values;
//! this module owns the corresponding live object. Catalog entries are not
//! items. Opening a plugin creates one `PluginItem`, just as attaching a Herdr
//! session creates one `SessionItem`.

use gpui::AnyView;
use zeddy_plugin::PaneKey;

use crate::{session::Session, terminal::Fit};

pub enum Item {
    Session(SessionItem),
    Plugin(PluginItem),
}

impl Item {
    pub fn title(&self) -> String {
        match self {
            Self::Session(item) => item.session.title(),
            Self::Plugin(item) => item.title.clone(),
        }
    }

    pub fn status(&self) -> Option<zeddy_herdr::control::SessionStatus> {
        match self {
            Self::Session(item) => Some(item.session.info.status),
            Self::Plugin(_) => None,
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
            Self::Plugin(_) => None,
        }
    }

    pub fn as_session_mut(&mut self) -> Option<&mut SessionItem> {
        match self {
            Self::Session(item) => Some(item),
            Self::Plugin(_) => None,
        }
    }

    pub fn as_plugin(&self) -> Option<&PluginItem> {
        match self {
            Self::Plugin(item) => Some(item),
            Self::Session(_) => None,
        }
    }
}

pub struct SessionItem {
    pub session: Session,
    pub fit: Fit,
}

impl SessionItem {
    pub fn new(session: Session) -> Self {
        Self { session, fit: Fit::default() }
    }
}

pub struct PluginItem {
    pub contribution: PaneKey,
    pub title: String,
    pub view: AnyView,
    /// A session-specific plugin closes when this Herdr session ends.
    pub bound_session: Option<zeddy_herdr::PaneId>,
    pub can_clone: bool,
    pub restorable: bool,
}
