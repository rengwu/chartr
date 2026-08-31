//! The two chromes, and the one thing they have in common.
//!
//! A chrome is a list of sessions with one of them selected. Sidebar mode draws
//! that list down the left; tabs mode draws it across the top. Neither knows
//! anything else about the app, which is what keeps the two implementations to
//! a screenful each: they take [`Entry`] values and emit stable item keys.

pub mod sidebar;
pub mod tabs;

use std::rc::Rc;

use crate::workspace::{ItemId, PaneId};
use gpui::EntityId;
use ui::{Tab, prelude::*};

/// One row in the sidebar, or one tab in the strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub space: EntityId,
    pub space_key: String,
    pub key: ItemId,
    pub pane: PaneId,
    pub index: usize,
    pub title: String,
    /// The agent herdr believes is running, when it knows one. In sidebar mode
    /// this is a second line; in tabs mode there is no room and it is dropped.
    pub agent: Option<String>,
    /// A session whose reader has stopped is still listed — closing it is the
    /// user's decision, not something that happens to them.
    pub ended: bool,
    pub selected: bool,
    pub closable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceEntries {
    pub id: EntityId,
    pub name: String,
    pub active: bool,
    pub removable: bool,
    pub available: bool,
    pub panes: Vec<PaneEntries>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneEntries {
    pub id: PaneId,
    pub entries: Vec<Entry>,
}

/// What the user did to the chrome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Select {
        space: Option<EntityId>,
        item: ItemId,
    },
    Close {
        space: Option<EntityId>,
        item: ItemId,
    },
    MoveItem {
        space: EntityId,
        item: ItemId,
        source: PaneId,
        source_index: usize,
        target: PaneId,
        target_index: usize,
    },
    CloseGroup {
        space: EntityId,
    },
    CloseSpace {
        space: EntityId,
    },
    RenameSpace {
        space: EntityId,
    },
    LocateSpace {
        space: EntityId,
    },
    NewInSpace {
        space: EntityId,
    },
    New,
    ToggleMode,
    ToggleSidebarScope,
}

/// How a chrome reports what the user did.
///
/// `Rc` because both chromes hand the same callback to every row they draw,
/// and a `cx.listener` closure is not `Clone`.
pub type Emit = Rc<dyn Fn(Action, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct DraggedSidebar;

impl Render for DraggedSidebar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

#[derive(Clone)]
pub struct DraggedItem {
    pub space: String,
    pub pane: PaneId,
    pub index: usize,
    pub item: ItemId,
    pub title: String,
    pub selected: bool,
}

impl Render for DraggedItem {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Tab::new(("dragged-item", self.item.get() as usize))
            .toggle_state(self.selected)
            .child(Label::new(self.title.clone()).size(LabelSize::Small))
    }
}

/// The dot that carries a session's state, in the one place both chromes agree
/// on what it means.
pub fn status_dot(entry: &Entry, cx: &App) -> impl IntoElement {
    let color = if entry.ended {
        cx.theme().status().error
    } else if entry.agent.is_some() {
        cx.theme().status().success
    } else {
        cx.theme().colors().text_muted
    };
    div().size(px(6.)).rounded_full().bg(color).flex_none()
}
