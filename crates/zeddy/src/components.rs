//! Small Chartr defaults around Zed's reusable UI components.
//!
//! Content and behavior stay with their owning feature; only visual contracts
//! shared across features belong here.

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, Window, px, relative,
};
use ui::{DynamicSpacing, prelude::*};

/// A vertical collection of selectable rows. The inter-row gap is part of the
/// collection rather than any individual row, so adjacent state backgrounds
/// are always separated consistently.
pub fn selection_list() -> Div {
    v_flex().gap_px()
}

/// Chartr's common selectable-row treatment. This mirrors Zed's sparse
/// `ListItem`, with one pixel removed from each vertical side. Zed only exposes
/// dense and sparse presets, so keeping the intermediate density here ensures
/// every Chartr list uses the same geometry and full-row hit target.
pub fn selection_row(id: impl Into<ElementId>, selected: bool) -> SelectionRow {
    SelectionRow::new(id, selected)
}

#[derive(IntoElement)]
pub struct SelectionRow {
    id: ElementId,
    selected: bool,
    aria_role: Option<Role>,
    aria_label: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl SelectionRow {
    fn new(id: impl Into<ElementId>, selected: bool) -> Self {
        Self {
            id: id.into(),
            selected,
            aria_role: None,
            aria_label: None,
            on_click: None,
            start_slot: None,
            end_slot: None,
            children: Vec::new(),
        }
    }

    pub fn aria_role(mut self, role: Role) -> Self {
        self.aria_role = Some(role);
        self
    }

    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn start_slot<E: IntoElement>(mut self, slot: impl Into<Option<E>>) -> Self {
        self.start_slot = slot.into().map(IntoElement::into_any_element);
        self
    }

    pub fn end_slot<E: IntoElement>(mut self, slot: impl Into<Option<E>>) -> Self {
        self.end_slot = slot.into().map(IntoElement::into_any_element);
        self
    }
}

impl ParentElement for SelectionRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SelectionRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sparse_padding = window.rem_size() * 0.25;
        let vertical_padding =
            if sparse_padding > px(1.) { sparse_padding - px(1.) } else { px(0.) };
        let has_end_slot = self.end_slot.is_some();

        h_flex()
            .id(self.id)
            .group("list_item")
            .w_full()
            .relative()
            .gap_1()
            .px(DynamicSpacing::Base06.rems(cx))
            .py(vertical_padding)
            .rounded_sm()
            .when_some(self.aria_role, |row, role| row.role(role).aria_selected(self.selected))
            .when_some(self.aria_label, |row, label| row.aria_label(label))
            .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
            .active(|style| style.bg(cx.theme().colors().ghost_element_active))
            .when(self.selected, |row| row.bg(cx.theme().colors().ghost_element_selected))
            .when_some(self.on_click, |row, on_click| row.cursor_pointer().on_click(on_click))
            .child(
                h_flex()
                    .flex_grow_1()
                    .flex_shrink_0()
                    .flex_basis(relative(0.25))
                    .gap(DynamicSpacing::Base06.rems(cx))
                    .overflow_hidden()
                    .children(self.start_slot)
                    .children(self.children),
            )
            .when(has_end_slot, |row| row.justify_between())
            .when_some(self.end_slot, |row, end_slot| {
                row.child(h_flex().flex_shrink_1().overflow_hidden().child(end_slot))
            })
    }
}
