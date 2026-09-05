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
use gpui::{ElementId, EntityId, Pixels, Role, SharedString};
use ui::{ButtonLike, CommonAnimationExt, IconButton, Tab, TabPosition, Tooltip, prelude::*};
use zeddy_herdr::control::SessionStatus;

use crate::assets::PLUGIN_LAUNCHER_ICON_PATH;

const TAB_LABEL_MIN_WIDTH: f32 = 24.;

pub(crate) fn new_item_button(id: impl Into<ElementId>) -> IconButton {
    IconButton::new(id, IconName::Plus).icon_size(IconSize::Small)
}

pub(crate) fn new_plugin_pane_button(id: impl Into<ElementId>, icon_size: IconSize) -> ButtonLike {
    ButtonLike::new(id)
        .aria_label("New plugin pane")
        .tooltip(Tooltip::text("New plugin pane"))
        .child(Icon::from_path(PLUGIN_LAUNCHER_ICON_PATH).size(icon_size))
}

pub(crate) fn new_item_cell(button: impl IntoElement, cx: &App) -> AnyElement {
    h_flex()
        // Zed's inner TabBar row derives its height from its tabs. Keep an
        // empty strip at the same height instead of collapsing to the button.
        .h(Tab::container_height(cx))
        .flex_none()
        // Collapse this divider onto the last tab's border.
        .ml(px(-1.))
        .px(DynamicSpacing::Base04.rems(cx))
        .border_l_1()
        .border_color(cx.theme().colors().border)
        .child(button)
        .into_any_element()
}

fn tab_label(title: impl Into<SharedString>, selected: bool) -> impl IntoElement {
    h_flex().when(selected, |label| label.pr_px()).child(
        div()
            .min_w(px(TAB_LABEL_MIN_WIDTH))
            .child(Label::new(title).size(UI_LABEL_DEFAULT).truncate()),
    )
}

/// Resolve the Zed border shape shared by outer and pane-local tab strips.
pub(crate) fn tab_position(index: usize, count: usize, active_index: Option<usize>) -> TabPosition {
    if index == 0 {
        TabPosition::First
    } else if index + 1 == count {
        TabPosition::Last
    } else {
        TabPosition::Middle(index.cmp(&active_index.unwrap_or(index)))
    }
}

/// The common visual core for every workspace tab.
///
/// Zed's selected [`Tab`] replaces one horizontal pixel of padding with a
/// border. GPUI paints that border inside the box, so its intrinsic width is
/// one pixel smaller than the inactive state. Restore that pixel here to keep
/// selection from shifting the rest of either tab strip.
pub(crate) struct ItemTab<'a> {
    id: ElementId,
    title: SharedString,
    aria_label: SharedString,
    selected: bool,
    position: TabPosition,
    activity: Activity,
    icon_path: Option<SharedString>,
    grouped: bool,
    space: &'a str,
    key: ItemId,
    close_slot: Option<AnyElement>,
}

impl<'a> ItemTab<'a> {
    pub(crate) fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        selected: bool,
        position: TabPosition,
        space: &'a str,
        key: ItemId,
    ) -> Self {
        let title = title.into();
        Self {
            id: id.into(),
            aria_label: title.clone(),
            title,
            selected,
            position,
            activity: Activity::default(),
            icon_path: None,
            grouped: false,
            space,
            key,
            close_slot: None,
        }
    }

    pub(crate) fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = label.into();
        self
    }

    pub(crate) fn activity(mut self, activity: Activity) -> Self {
        self.activity = activity;
        self
    }

    pub(crate) fn icon_path(mut self, icon_path: Option<SharedString>) -> Self {
        self.icon_path = icon_path;
        self
    }

    pub(crate) fn grouped(mut self, grouped: bool) -> Self {
        self.grouped = grouped;
        self
    }

    pub(crate) fn close_slot(mut self, close_slot: Option<AnyElement>) -> Self {
        self.close_slot = close_slot;
        self
    }

    pub(crate) fn build(self, cx: &App) -> Tab {
        Tab::new(self.id)
            .role(Role::Tab)
            .aria_label(self.aria_label)
            .aria_selected(self.selected)
            .position(self.position)
            .toggle_state(self.selected)
            .start_slot(item_indicator(
                self.activity,
                self.icon_path,
                self.grouped,
                self.space,
                self.key,
                cx,
            ))
            .end_slot::<AnyElement>(self.close_slot)
            .child(tab_label(self.title, self.selected))
    }
}

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
    /// A plugin's package-owned Hugeicons SVG, or the embedded plugin-launcher
    /// icon. Sessions and groups use their live status indicator instead.
    pub icon_path: Option<SharedString>,
    /// Herdr's agent state. Plugins and grouped outer tabs have no aggregate
    /// session state of their own.
    pub status: Option<SessionStatus>,
    /// A non-agent process currently owns the foreground process group.
    pub process_running: bool,
    /// A session whose reader has stopped is still listed — closing it is the
    /// user's decision, not something that happens to them.
    pub ended: bool,
    /// Zed's terminal emulator received BEL since the terminal last handled input.
    pub bell: bool,
    pub selected: bool,
    pub closable: bool,
    pub grouped: bool,
}

impl Entry {
    pub(crate) fn activity(&self) -> Activity {
        Activity {
            status: self.status,
            process_running: self.process_running,
            ended: self.ended,
            bell: self.bell,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Activity {
    pub status: Option<SessionStatus>,
    pub process_running: bool,
    pub ended: bool,
    pub bell: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceEntries {
    pub id: EntityId,
    pub name: String,
    /// The synthetic folderless space is a fixed sidebar section rather than
    /// one of the sortable space cards.
    pub is_free: bool,
    pub active: bool,
    pub removable: bool,
    pub available: bool,
    pub entries: Vec<Entry>,
}

/// What the user did to the chrome.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ActivateSpace { space: EntityId },
    Select { space: Option<EntityId>, item: ItemId },
    Close { space: Option<EntityId>, item: ItemId },
    CloseGroup { space: EntityId, tab: WorkspaceTabId },
    UngroupPane { space: EntityId, tab: WorkspaceTabId },
    RenameGroup { space: EntityId, tab: WorkspaceTabId },
    MoveWorkspaceTab { space: EntityId, tab: WorkspaceTabId, target_index: usize },
    BeginSpaceDrag { at: Pixels },
    CloseSpace { space: EntityId },
    RenameSpace { space: EntityId },
    OpenSpaceFolder { space: EntityId },
    LocateSpace { space: EntityId },
    SwitchToTabs,
    SwitchToSidebar,
    NewSpace,
    NewInSpace { space: EntityId },
    NewPluginPaneInSpace { space: EntityId },
    New,
    NewPluginPane,
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

/// The fixed leading mark used by sidebar rows, outer tabs, and pane-local tabs.
///
/// Sessions show live Herdr/process state and plugins show the Hugeicon named
/// by their manifest. A plain foreground process gets a slower neutral spinner
/// so it cannot be mistaken for an agent actively working.
pub fn item_indicator(
    activity: Activity,
    icon_path: Option<SharedString>,
    grouped: bool,
    space: &str,
    key: ItemId,
    cx: &App,
) -> AnyElement {
    let slot = || div().flex_none().size(px(12.)).flex().items_center().justify_center();
    let icon = |name, color| Icon::new(name).size(IconSize::XSmall).color(color);

    if activity.ended {
        return slot().child(icon(IconName::XCircle, Color::Error)).into_any_element();
    }
    if grouped {
        return slot().child(icon(IconName::Split, Color::Muted)).into_any_element();
    }
    if activity.bell {
        return slot().child(icon(IconName::BellRing, Color::Warning)).into_any_element();
    }

    match activity.status {
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
        Some(SessionStatus::Idle | SessionStatus::Unknown) if activity.process_running => {
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
        None => match icon_path {
            Some(path) => {
                let icon = if path.starts_with("icons/") {
                    Icon::from_path(path)
                } else {
                    Icon::from_external_svg(path)
                };
                slot().child(icon.size(IconSize::XSmall).color(Color::Muted)).into_any_element()
            }
            None => slot().into_any_element(),
        },
    }
}
