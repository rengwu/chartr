//! Semantic plugin UI. Keep page chrome, type, actions, and dialogs here.
//! Plugins provide content and behavior, not copies of these visual recipes.

use ::ui::{Button, ButtonSize, ButtonStyle, Color, Icon, Label, LabelSize, TintColor, prelude::*};
use gpui::{
    App, ClickEvent, ElementId, Focusable, IntoElement, MouseDownEvent, ParentElement, RenderOnce,
    SharedString, Window, px, relative,
};

pub const UI_TEXT_LARGE: gpui::Rems = gpui::Rems(1.);
pub const UI_TEXT_DEFAULT: gpui::Rems = gpui::Rems(12. / 14.);
pub const UI_TEXT_SMALL: gpui::Rems = gpui::Rems(10. / 14.);
pub const UI_LABEL_LARGE: LabelSize = LabelSize::Custom(UI_TEXT_LARGE);
pub const UI_LABEL_DEFAULT: LabelSize = LabelSize::Custom(UI_TEXT_DEFAULT);
pub const UI_LABEL_SMALL: LabelSize = LabelSize::Custom(UI_TEXT_SMALL);
pub const FORM_CONTROL_SIZE: ButtonSize = ButtonSize::Medium;

#[derive(IntoElement)]
pub struct Text {
    label: Label,
}
impl Text {
    pub fn color(mut self, color: Color) -> Self {
        self.label = self.label.color(color);
        self
    }
    pub fn truncate(mut self) -> Self {
        self.label = self.label.truncate();
        self
    }
}
impl RenderOnce for Text {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.label
    }
}
pub fn label(text: impl Into<SharedString>) -> Text {
    Text { label: Label::new(text).size(UI_LABEL_DEFAULT) }
}
pub fn heading(text: impl Into<SharedString>) -> Text {
    Text { label: Label::new(text).size(UI_LABEL_LARGE) }
}
pub fn caption(text: impl Into<SharedString>) -> Text {
    Text { label: Label::new(text).size(UI_LABEL_SMALL) }
}

/// No general styling or sizing API: action density and treatment are semantic.
///
/// ```compile_fail
/// use chartr_plugin::{gpui::Styled, ui::action};
/// action("save", "Save").bg(chartr_plugin::gpui::rgb(0));
/// ```
///
/// ```compile_fail
/// use chartr_plugin::ui::action;
/// action("save", "Save").size(ui::ButtonSize::Small);
/// ```
#[derive(IntoElement)]
pub struct ActionButton {
    button: Button,
}

impl ActionButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self { button: form_button(id, label) }
    }
    pub fn primary(mut self) -> Self {
        self.button = self.button.style(ButtonStyle::Filled);
        self
    }
    pub fn destructive(mut self) -> Self {
        self.button = self.button.style(ButtonStyle::Tinted(TintColor::Error));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }
    pub fn start_icon(mut self, icon: Icon) -> Self {
        self.button = self.button.start_icon(icon);
        self
    }
    pub fn end_icon(mut self, icon: Icon) -> Self {
        self.button = self.button.end_icon(icon);
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(handler);
        self
    }
}

impl RenderOnce for ActionButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.button
    }
}
impl ::ui::Clickable for ActionButton {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click(handler)
    }
    fn cursor_style(mut self, cursor: gpui::CursorStyle) -> Self {
        self.button = self.button.cursor_style(cursor);
        self
    }
}
impl ::ui::Toggleable for ActionButton {
    fn toggle_state(mut self, selected: bool) -> Self {
        self.button = self.button.toggle_state(selected);
        self
    }
}

pub fn action(id: impl Into<ElementId>, text: impl Into<SharedString>) -> ActionButton {
    ActionButton::new(id, text)
}

#[derive(IntoElement)]
pub struct IconAction {
    button: ::ui::IconButton,
}
impl IconAction {
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.button = self.button.aria_label(label);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(handler);
        self
    }
}
impl ::ui::Clickable for IconAction {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click(handler)
    }
    fn cursor_style(mut self, cursor: gpui::CursorStyle) -> Self {
        self.button = self.button.cursor_style(cursor);
        self
    }
}
impl ::ui::Toggleable for IconAction {
    fn toggle_state(mut self, selected: bool) -> Self {
        self.button = self.button.toggle_state(selected);
        self
    }
}
impl RenderOnce for IconAction {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.button
    }
}
pub fn icon_action(id: impl Into<ElementId>, icon: ::ui::IconName) -> IconAction {
    IconAction { button: ::ui::IconButton::new(id, icon).icon_size(::ui::IconSize::Small) }
}

#[derive(Clone, Copy)]
pub enum NoticeKind {
    Information,
    Error,
}

#[derive(IntoElement)]
pub struct Notice {
    message: SharedString,
    kind: NoticeKind,
    actions: Vec<gpui::AnyElement>,
}
impl Notice {
    pub fn new(message: impl Into<SharedString>, kind: NoticeKind) -> Self {
        Self { message: message.into(), kind, actions: Vec::new() }
    }
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}
impl RenderOnce for Notice {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (color, background) = match self.kind {
            NoticeKind::Error => (Color::Error, cx.theme().status().error.opacity(0.1)),
            NoticeKind::Information => (Color::Muted, cx.theme().colors().element_background),
        };
        div()
            .w_full()
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().colors().border_variant)
            .bg(background)
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .justify_between()
                    .child(label(self.message).color(color))
                    .children(self.actions),
            )
    }
}

pub fn notice(message: impl Into<SharedString>, error: bool) -> Notice {
    Notice::new(message, if error { NoticeKind::Error } else { NoticeKind::Information })
}

#[derive(IntoElement)]
pub struct PageHeader {
    title: SharedString,
    description: Option<SharedString>,
    actions: Vec<gpui::AnyElement>,
}
impl PageHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self { title: title.into(), description: None, actions: Vec::new() }
    }
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn action(mut self, action: ActionButton) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}
impl RenderOnce for PageHeader {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_2()
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .gap_3()
                    .child(heading(self.title))
                    .child(h_flex().gap_2().children(self.actions)),
            )
            .children(self.description.map(|description| label(description).color(Color::Muted)))
    }
}

/// Shared modal paint and input boundary; the caller chooses content, not colors.
#[derive(IntoElement)]
pub struct ModalOverlay {
    id: ElementId,
    children: Vec<gpui::AnyElement>,
    dismiss: Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>,
}
impl ModalOverlay {
    pub fn new(
        id: impl Into<ElementId>,
        dismiss: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self { id: id.into(), children: Vec::new(), dismiss: Box::new(dismiss) }
    }
}
impl ParentElement for ModalOverlay {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}
impl RenderOnce for ModalOverlay {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .absolute()
            .inset_0()
            .flex()
            .items_start()
            .justify_center()
            .pt_8()
            .bg(gpui::black().opacity(0.35))
            .on_mouse_down(gpui::MouseButton::Left, self.dismiss)
            .children(self.children)
    }
}

#[derive(IntoElement)]
pub struct DialogSurface {
    id: ElementId,
    width: gpui::Pixels,
    height_limit: Option<gpui::Pixels>,
    children: Vec<gpui::AnyElement>,
    selector: Option<SharedString>,
}
impl DialogSurface {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: px(640.),
            height_limit: None,
            children: Vec::new(),
            selector: None,
        }
    }
    pub fn compact(mut self) -> Self {
        self.width = px(440.);
        self
    }
    pub fn height_limit(mut self, height: gpui::Pixels) -> Self {
        self.height_limit = Some(height);
        self
    }
    pub fn debug_selector(mut self, selector: impl Into<SharedString>) -> Self {
        self.selector = Some(selector.into());
        self
    }
}
impl ParentElement for DialogSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}
impl RenderOnce for DialogSurface {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .id(self.id)
            .w(self.width)
            .max_w(relative(0.92))
            .max_h(relative(0.9))
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().elevated_surface_background)
            .shadow_lg()
            .overflow_hidden()
            .when_some(self.height_limit, |view, height| view.max_h(height))
            .when_some(self.selector, |view, selector| {
                view.debug_selector(move || selector.to_string())
            })
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .children(self.children)
    }
}

pub fn dialog_header(
    title: impl Into<SharedString>,
    close: impl IntoElement,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .w_full()
        .justify_between()
        .pb_3()
        .border_b_1()
        .border_color(cx.theme().colors().border)
        .child(heading(title))
        .child(close)
}
pub fn dialog_body() -> gpui::Div {
    v_flex().w_full().gap_3()
}
pub fn dialog_actions(cx: &App) -> gpui::Div {
    h_flex()
        .w_full()
        .justify_end()
        .gap_2()
        .pt_3()
        .border_t_1()
        .border_color(cx.theme().colors().border)
}

/// Ordinary plugin pane foundations; embedded settings use `SettingsPage` instead.
pub fn pane_surface(id: impl Into<ElementId>, cx: &App) -> gpui::Stateful<gpui::Div> {
    v_flex().id(id).size_full().min_h_0().relative().bg(cx.theme().colors().editor_background)
}

pub fn card(cx: &App) -> gpui::Div {
    v_flex()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().surface_background)
        .overflow_hidden()
}

pub fn outlined_content(cx: &App) -> gpui::Div {
    div().p_3().rounded_md().border_1().border_color(cx.theme().colors().border_variant)
}

pub fn template_chip(unavailable: bool, cx: &App) -> gpui::Div {
    div()
        .rounded_sm()
        .border_1()
        .border_color(if unavailable {
            cx.theme().status().error
        } else {
            cx.theme().colors().border
        })
        .bg(cx.theme().colors().element_background)
        .text_color(cx.theme().colors().text)
}

pub fn data_row(row: gpui::Div, index: usize, held: bool, cx: &App) -> gpui::Div {
    let colors = cx.theme().colors();
    row.border_1()
        .border_color(gpui::transparent_black())
        .when(index % 2 == 1, |row| row.bg(colors.element_background))
        .when(held, |row| row.border_color(colors.drop_target_border).shadow_md())
}

pub fn table_header(row: gpui::Div, cx: &App) -> gpui::Div {
    row.pb_2().border_b_1().border_color(cx.theme().colors().border)
}

pub fn separated_row(row: gpui::Div, cx: &App) -> gpui::Div {
    row.border_b_1().border_color(cx.theme().colors().border_variant)
}

pub fn form_button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    Button::new(id, label).size(FORM_CONTROL_SIZE).style(ButtonStyle::Outlined)
}

/// A full-width picker keeps its value aligned with neighboring text inputs.
pub fn form_picker(id: impl Into<ElementId>, value: impl Into<SharedString>) -> ::ui::ButtonLike {
    let value = value.into();
    ::ui::ButtonLike::new(id)
        .size(FORM_CONTROL_SIZE)
        .style(ButtonStyle::Outlined)
        .full_width()
        .aria_label(value.clone())
        .child(
            h_flex()
                .w_full()
                .min_w_0()
                .justify_between()
                .gap_2()
                .text_left()
                .child(Label::new(value).size(UI_LABEL_DEFAULT).truncate())
                .child(
                    Icon::new(::ui::IconName::ChevronDown)
                        .size(::ui::IconSize::XSmall)
                        .color(Color::Muted),
                ),
        )
}

pub fn input_field<V: gpui::Render + gpui::Focusable>(
    id: impl Into<ElementId>,
    input: gpui::Entity<V>,
    cx: &App,
) -> gpui::AnyElement {
    let focus = input.focus_handle(cx);
    h_flex()
        .id(id)
        .w_full()
        .min_w_0()
        .h(FORM_CONTROL_SIZE.rems())
        .flex_none()
        .px_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(cx.theme().colors().editor_background)
        .track_focus(&focus)
        .in_focus(|field| field.border_color(cx.theme().colors().border_focused))
        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| focus.focus(window, cx))
        .child(input)
        .into_any_element()
}

pub fn form_row(
    label: impl Into<SharedString>,
    description: Option<&str>,
    control: gpui::AnyElement,
) -> gpui::AnyElement {
    h_flex()
        .w_full()
        .flex_none()
        .items_start()
        .gap_4()
        .child(
            h_flex()
                .w(px(112.))
                .flex_none()
                .min_h(FORM_CONTROL_SIZE.rems())
                .child(Label::new(label).size(UI_LABEL_DEFAULT)),
        )
        .child(v_flex().flex_1().min_w_0().gap_1().child(control).when_some(
            description,
            |field, description| {
                field.child(
                    Label::new(description.to_owned()).size(UI_LABEL_SMALL).color(Color::Muted),
                )
            },
        ))
        .into_any_element()
}
