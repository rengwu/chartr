//! The two chromes, and the one thing they have in common.
//!
//! A chrome is a list of outer workspace tabs with one selected. A one-item tab
//! is standalone; a multi-item pane workspace is one grouped entry. Sidebar
//! mode draws the list down the left and tabs mode draws it across the top.
//! Neither owns workspace state: both take [`Entry`] values and emit stable ids.

pub mod sidebar;
pub mod tabs;

use std::rc::Rc;

use crate::{
    fonts::UI_LABEL_DEFAULT,
    workspace::{ItemId, PaneId, WorkspaceTabId},
};
use gpui::{EntityId, Pixels};
use ui::{CommonAnimationExt, prelude::*};
use zeddy_herdr::control::SessionStatus;

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
    /// Herdr's agent state. Plugins and grouped outer tabs have no aggregate
    /// session state of their own.
    pub status: Option<SessionStatus>,
    /// A non-agent process currently owns the foreground process group.
    pub process_running: bool,
    /// A session whose reader has stopped is still listed — closing it is the
    /// user's decision, not something that happens to them.
    pub ended: bool,
    pub selected: bool,
    pub closable: bool,
    pub grouped: bool,
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
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select { space: Option<EntityId>, item: ItemId },
    Close { space: Option<EntityId>, item: ItemId },
    CloseGroup { space: EntityId, tab: WorkspaceTabId },
    UngroupPane { space: EntityId, tab: WorkspaceTabId },
    MoveWorkspaceTab { space: EntityId, tab: WorkspaceTabId, target_index: usize },
    BeginSpaceDrag { at: Pixels },
    CloseSpace { space: EntityId },
    RenameSpace { space: EntityId },
    LocateSpace { space: EntityId },
    NewInSpace { space: EntityId },
    New,
    OpenSettings,
}

/// A whole sidebar space card in flight.
///
/// Space sorting deliberately has its own payload type. Session rows nested in
/// the card continue to carry [`DraggedItem`], so GPUI dispatches the two drag
/// gestures to different listeners without either surface inspecting or
/// rejecting the other's values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraggedSpace(pub EntityId);

impl Render for DraggedSpace {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
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
    pub top_level: bool,
    /// The drag represents the whole outer workspace tab, not its
    /// representative item. Grouped tabs may be sorted by outer chrome, but
    /// cannot be dropped into an individual pane as though they were one item.
    pub grouped: bool,
}

impl Render for DraggedItem {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        dragged_item_pill(self.grouped, cx)
    }
}

const DRAGGED_ITEM_PILL_WIDTH: f32 = 44.;
const DRAGGED_GROUP_PILL_WIDTH: f32 = 60.;
const DRAGGED_ITEM_PILL_HEIGHT: f32 = 22.;

fn dragged_item_pill_width(grouped: bool) -> f32 {
    if grouped { DRAGGED_GROUP_PILL_WIDTH } else { DRAGGED_ITEM_PILL_WIDTH }
}

fn dragged_item_pill(grouped: bool, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(dragged_item_pill_width(grouped)))
        .h(px(DRAGGED_ITEM_PILL_HEIGHT))
        .rounded_full()
        .border_1()
        .border_color(colors.border)
        .bg(colors.elevated_surface_background)
        .shadow_md()
        .child(Label::new(if grouped { "group" } else { "tab" }).size(UI_LABEL_DEFAULT))
}

/// Builds the one drag preview used by every Chartr tab surface.
///
/// GPUI positions a drag view at `pointer - offset_within_source`, which is
/// perfect when the preview has the source element's dimensions. Chartr's
/// sidebar rows and outer tabs are often much wider than the compact preview,
/// though, so using the source offset makes the visible ghost trail behind the
/// pointer. Translating the compact preview by that same offset and half of
/// its own size locks its center to GPUI's current-frame pointer position.
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = dragged_item_pill_width(self.dragged.grouped);
        div()
            .pl(self.source_offset.x - px(width / 2.))
            .pt(self.source_offset.y - px(DRAGGED_ITEM_PILL_HEIGHT / 2.))
            .child(dragged_item_pill(self.dragged.grouped, cx))
    }
}

/// The fixed status mark used by sidebar rows, outer tabs, and pane-local tabs.
///
/// Herdr owns agent detection and state. Chartr only maps those states to the
/// same visual language the earlier clients used, using Zed's own icons and
/// animation primitive. A plain foreground process gets a slower neutral
/// spinner so it cannot be mistaken for an agent actively working.
pub fn status_indicator(
    status: Option<SessionStatus>,
    process_running: bool,
    ended: bool,
    grouped: bool,
    space: &str,
    key: ItemId,
    cx: &App,
) -> AnyElement {
    let slot = || div().flex_none().size(px(12.)).flex().items_center().justify_center();
    let icon = |name, color| Icon::new(name).size(IconSize::XSmall).color(color);

    if ended {
        return slot().child(icon(IconName::XCircle, Color::Error)).into_any_element();
    }
    if grouped {
        return slot().child(icon(IconName::Split, Color::Muted)).into_any_element();
    }

    match status {
        Some(SessionStatus::Working) => {
            slot()
                .child(icon(IconName::LoadCircle, Color::Accent).with_keyed_rotate_animation(
                    format!("working-status-{space}-{}", key.get()),
                    2,
                ))
                .into_any_element()
        }
        Some(SessionStatus::Blocked) => {
            slot().child(icon(IconName::DebugPause, Color::Warning)).into_any_element()
        }
        Some(SessionStatus::Done) => {
            slot().child(icon(IconName::Check, Color::Success)).into_any_element()
        }
        Some(SessionStatus::Idle | SessionStatus::Unknown) if process_running => {
            slot()
                .child(icon(IconName::LoadCircle, Color::Muted).with_keyed_rotate_animation(
                    format!("process-status-{space}-{}", key.get()),
                    5,
                ))
                .into_any_element()
        }
        Some(SessionStatus::Idle | SessionStatus::Unknown) => slot()
            .child(
                div().size(px(5.)).rounded_full().bg(cx.theme().colors().text_muted.opacity(0.28)),
            )
            .into_any_element(),
        None => slot().into_any_element(),
    }
}
