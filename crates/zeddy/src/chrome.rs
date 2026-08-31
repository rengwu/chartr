//! The two chromes, and the one thing they have in common.
//!
//! A chrome is a list of outer workspace tabs with one selected. A one-item tab
//! is standalone; a multi-item pane workspace is one grouped entry. Sidebar
//! mode draws the list down the left and tabs mode draws it across the top.
//! Neither owns workspace state: both take [`Entry`] values and emit stable ids.

pub mod sidebar;
pub mod tabs;

use std::rc::Rc;

use crate::workspace::{ItemId, PaneId, WorkspaceTabId};
use gpui::EntityId;
use ui::{Tab, prelude::*};

/// One row in the sidebar, or one tab in the strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub space: EntityId,
    pub space_key: String,
    pub key: ItemId,
    pub tab: WorkspaceTabId,
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
    pub grouped: bool,
    pub item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceEntries {
    pub id: EntityId,
    pub name: String,
    pub active: bool,
    pub removable: bool,
    pub available: bool,
    pub entries: Vec<Entry>,
}

/// What the user did to the chrome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Select { space: Option<EntityId>, item: ItemId },
    Close { space: Option<EntityId>, item: ItemId },
    CloseGroup { space: EntityId, tab: WorkspaceTabId },
    MoveWorkspaceTab { space: EntityId, tab: WorkspaceTabId, target_index: usize },
    CloseSpace { space: EntityId },
    RenameSpace { space: EntityId },
    LocateSpace { space: EntityId },
    NewInSpace { space: EntityId },
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
    pub tab: WorkspaceTabId,
    pub pane: PaneId,
    pub index: usize,
    pub item: ItemId,
    pub title: String,
    pub selected: bool,
    pub top_level: bool,
}

impl Render for DraggedItem {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Tab::new(("dragged-item", self.item.get() as usize))
            .toggle_state(self.selected)
            .child(Label::new(self.title.clone()).size(LabelSize::Small))
    }
}

/// Builds the one drag preview used by every Chartr tab surface.
///
/// GPUI positions a drag view at `pointer - offset_within_source`, which is
/// perfect when the preview has the source element's dimensions. Chartr's
/// sidebar rows and outer tabs are often much wider than the compact preview,
/// though, so using the source offset makes the visible ghost trail behind the
/// pointer. Translating the compact preview by that same offset locks its
/// visible origin to GPUI's current-frame pointer position.
pub(crate) fn dragged_item_preview(
    dragged: &DraggedItem,
    source_offset: gpui::Point<gpui::Pixels>,
    cx: &mut App,
) -> gpui::Entity<DraggedItemPreview> {
    let dragged = dragged.clone();
    cx.new(|_| DraggedItemPreview { dragged, source_offset })
}

pub(crate) struct DraggedItemPreview {
    dragged: DraggedItem,
    source_offset: gpui::Point<gpui::Pixels>,
}

impl Render for DraggedItemPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().relative().left(self.source_offset.x).top(self.source_offset.y).child(
            Tab::new(("dragged-item-preview", self.dragged.item.get() as usize))
                .toggle_state(self.dragged.selected)
                .child(Label::new(self.dragged.title.clone()).size(LabelSize::Small)),
        )
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
