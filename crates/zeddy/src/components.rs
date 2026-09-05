//! Shared Chartr controls, organized by interaction and presentation.

mod modal;
mod popup;
mod selection;

pub use modal::open_native_modal;
pub use popup::{ContextMenu, PopupMenu, popup_right_click_menu};
pub use selection::{
    SegmentedControl, SegmentedControlOption, SelectionRowBackgrounds, selection_list,
    selection_row,
};
