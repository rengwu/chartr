//! Small Chartr defaults around Zed's reusable UI components.
//!
//! Content and behavior stay with their owning feature; only visual contracts
//! shared across features belong here.

use gpui::{Div, ElementId};
use ui::{ListItem, ListItemSpacing, prelude::*};

/// A vertical collection of selectable rows. The inter-row gap is part of the
/// collection rather than any individual row, so adjacent state backgrounds
/// are always separated consistently.
pub fn selection_list() -> Div {
    v_flex().gap_px()
}

/// Chartr's common selectable-row treatment, backed by Zed's `ListItem` so
/// padding, corners, and interaction-state colors follow the component theme.
pub fn selection_row(id: impl Into<ElementId>, selected: bool) -> ListItem {
    ListItem::new(id).spacing(ListItemSpacing::Sparse).rounded().toggle_state(selected)
}
