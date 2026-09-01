//! Small Chartr defaults around Zed's reusable UI components.
//!
//! Content and behavior stay with their owning feature; only visual contracts
//! shared across features belong here.

use gpui::{
    Action, AnyElement, App, ClickEvent, Context, Div, ElementId, Entity, Hsla, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, Window, px, relative,
};
use ui::{ButtonSize, ContextMenu as UiContextMenu, DynamicSpacing, IconPosition, prelude::*};

/// Chartr's shared context-menu builder.
///
/// Zed's menu rows are flush by default. This wrapper inserts a small,
/// non-selectable gap between adjacent actions while leaving separators and
/// headers as distinct group boundaries.
pub struct ContextMenu {
    inner: UiContextMenu,
    has_item_in_group: bool,
}

impl ContextMenu {
    pub fn build(
        window: &mut Window,
        cx: &mut App,
        build: impl FnOnce(Self, &mut Window, &mut Context<UiContextMenu>) -> Self,
    ) -> Entity<UiContextMenu> {
        UiContextMenu::build(window, cx, |menu, window, cx| {
            build(Self { inner: menu, has_item_in_group: false }, window, cx).inner
        })
    }

    fn before_item(mut self) -> Self {
        if self.has_item_in_group {
            self.inner = self.inner.custom_row(|_, _| div().h_1().into_any_element());
        }
        self.has_item_in_group = true;
        self
    }

    pub fn entry(
        self,
        label: impl Into<SharedString>,
        action: Option<Box<dyn Action>>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let mut this = self.before_item();
        this.inner = this.inner.entry(label, action, handler);
        this
    }

    pub fn toggleable_entry(
        self,
        label: impl Into<SharedString>,
        toggled: bool,
        position: IconPosition,
        action: Option<Box<dyn Action>>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let mut this = self.before_item();
        this.inner = this.inner.toggleable_entry(label, toggled, position, action, handler);
        this
    }

    pub fn separator(mut self) -> Self {
        self.inner = self.inner.separator();
        self.has_item_in_group = false;
        self
    }

    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.inner = self.inner.header(title);
        self.has_item_in_group = false;
        self
    }
}

impl FluentBuilder for ContextMenu {}

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

/// One mutually exclusive choice inside a [`SegmentedControl`].
pub struct SegmentedControlOption {
    id: ElementId,
    label: SharedString,
    selected: bool,
    on_click: Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

impl SegmentedControlOption {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self { id: id.into(), label: label.into(), selected, on_click: Box::new(on_click) }
    }
}

/// A compact radio-like control whose options share one outline and are split
/// by dividers. Selection uses the theme's neutral element surface instead of
/// its semantic accent tint so it remains balanced across light and dark
/// themes.
#[derive(IntoElement)]
pub struct SegmentedControl {
    label: SharedString,
    options: Vec<SegmentedControlOption>,
    disabled: bool,
}

impl SegmentedControl {
    pub fn new(
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = SegmentedControlOption>,
    ) -> Self {
        Self { label: label.into(), options: options.into_iter().collect(), disabled: false }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let option_count = self.options.len();
        let colors = cx.theme().colors();
        let border = colors.border.opacity(0.8);

        h_flex()
            .id(self.label.clone())
            .role(Role::RadioGroup)
            .aria_label(self.label)
            .rounded_md()
            .overflow_hidden()
            .border_1()
            .border_color(border)
            .when(self.disabled, |control| control.opacity(0.5))
            .children(self.options.into_iter().enumerate().map(|(index, option)| {
                let selected = option.selected;
                h_flex()
                    .id(option.id)
                    .role(Role::RadioButton)
                    .aria_selected(selected)
                    .h(ButtonSize::Default.rems())
                    .px_3()
                    .when(index + 1 < option_count, |item| item.border_r_1().border_color(border))
                    .when(selected, |item| item.bg(colors.ghost_element_selected))
                    .when(!selected && !self.disabled, |item| {
                        item.hover(|style| style.bg(colors.ghost_element_hover))
                            .active(|style| style.bg(colors.ghost_element_active))
                    })
                    .when_else(
                        self.disabled,
                        |item| item.cursor_not_allowed(),
                        |item| item.cursor_pointer().on_click(option.on_click),
                    )
                    .child(Label::new(option.label).size(LabelSize::Small).when(
                        !selected,
                        |label| {
                            label.color(if self.disabled { Color::Disabled } else { Color::Muted })
                        },
                    ))
            }))
    }
}

/// Optional state surfaces for a selection row embedded on a custom ground.
#[derive(Debug, Clone, Copy)]
pub struct SelectionRowBackgrounds {
    pub hover: Hsla,
    pub selected: Hsla,
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
    backgrounds: Option<SelectionRowBackgrounds>,
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
            backgrounds: None,
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

    pub fn backgrounds(mut self, backgrounds: SelectionRowBackgrounds) -> Self {
        self.backgrounds = Some(backgrounds);
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
        let colors = cx.theme().colors();
        let (selected_background, hover_background, active_background) = if let Some(backgrounds) =
            self.backgrounds
        {
            let interaction = if self.selected { backgrounds.selected } else { backgrounds.hover };
            (backgrounds.selected, interaction, interaction)
        } else {
            (colors.ghost_element_selected, colors.ghost_element_hover, colors.ghost_element_active)
        };

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
            .when(self.selected, |row| row.bg(selected_background))
            .hover(|style| style.bg(hover_background))
            .active(|style| style.bg(active_background))
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
