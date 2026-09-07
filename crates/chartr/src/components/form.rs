//! Shared sizing and alignment for single-line form controls.

use gpui::{AnyElement, App, ElementId, Entity, Focusable, MouseButton, SharedString, px};
use ui::{
    Button, ButtonLike, ButtonSize, ButtonStyle, Color, Icon, IconName, IconSize, Label, prelude::*,
};

use crate::{
    fonts::{UI_LABEL_DEFAULT, UI_LABEL_SMALL},
    text_input::TextInput,
};

/// The browser address bar and form controls share this font-relative height.
pub const FORM_CONTROL_SIZE: ButtonSize = ButtonSize::Medium;

pub fn form_button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    Button::new(id, label).size(FORM_CONTROL_SIZE).style(ButtonStyle::Outlined)
}

/// A full-width picker keeps its value aligned with neighboring text inputs.
pub fn form_picker(id: impl Into<ElementId>, value: impl Into<SharedString>) -> ButtonLike {
    let value = value.into();
    ButtonLike::new(id)
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
                .child(Icon::new(IconName::ChevronDown).size(IconSize::XSmall).color(Color::Muted)),
        )
}

pub fn input_field(id: impl Into<ElementId>, input: Entity<TextInput>, cx: &App) -> AnyElement {
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
        .on_mouse_down(MouseButton::Left, move |_, window, cx| focus.focus(window, cx))
        .child(input)
        .into_any_element()
}

pub fn form_row(
    label: impl Into<SharedString>,
    description: Option<&str>,
    control: AnyElement,
) -> AnyElement {
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
