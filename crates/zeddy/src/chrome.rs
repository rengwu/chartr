//! The two chromes, and the one thing they have in common.
//!
//! A chrome is a list of sessions with one of them selected. Sidebar mode draws
//! that list down the left; tabs mode draws it across the top. Neither knows
//! anything else about the app, which is what keeps the two implementations to
//! a screenful each: they take [`Entry`] values and emit indices.

pub mod sidebar;
pub mod tabs;

use std::rc::Rc;

use ui::prelude::*;

/// One row in the sidebar, or one tab in the strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub title: String,
    /// The agent herdr believes is running, when it knows one. In sidebar mode
    /// this is a second line; in tabs mode there is no room and it is dropped.
    pub agent: Option<String>,
    /// A session whose reader has stopped is still listed — closing it is the
    /// user's decision, not something that happens to them.
    pub ended: bool,
    pub selected: bool,
}

/// What the user did to the chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Select(usize),
    Close(usize),
    New,
    ToggleMode,
}

/// How a chrome reports what the user did.
///
/// `Rc` because both chromes hand the same callback to every row they draw,
/// and a `cx.listener` closure is not `Clone`.
pub type Emit = Rc<dyn Fn(Action, &mut Window, &mut App)>;

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
