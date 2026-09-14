//! Opt-in CSS tokens sourced from the same theme and sizing as native plugin UI.
use gpui::{App, Window};
use theme::ActiveTheme;

pub(super) fn script(window: &Window, cx: &App) -> String {
    let theme = cx.theme();
    let colors = theme.colors();
    let fonts = theme::theme_settings(cx);
    let font = serde_json::to_string(fonts.ui_font(cx).family.as_ref()).unwrap();
    let mono = serde_json::to_string(fonts.buffer_font(cx).family.as_ref()).unwrap();
    let rem = f32::from(window.rem_size());
    let pixels = |value: gpui::Rems| format!("{}px", f32::from(value.to_pixels(window.rem_size())));
    let mut tokens = std::collections::BTreeMap::from([
        (
            "--chartr-color-scheme".to_owned(),
            if theme.appearance().is_light() { "light" } else { "dark" }.to_owned(),
        ),
        ("--chartr-ui-font".into(), format!("{font}, system-ui, sans-serif")),
        ("--chartr-mono-font".into(), format!("{mono}, ui-monospace, monospace")),
        ("--chartr-font-size".into(), pixels(chartr_plugin::ui::UI_TEXT_DEFAULT)),
        ("--chartr-font-small".into(), pixels(chartr_plugin::ui::UI_TEXT_SMALL)),
        ("--chartr-font-large".into(), pixels(chartr_plugin::ui::UI_TEXT_LARGE)),
        ("--chartr-control-height".into(), pixels(chartr_plugin::ui::FORM_CONTROL_SIZE.rems())),
        ("--chartr-icon-size".into(), pixels(ui::IconSize::Small.rems())),
        ("--chartr-icon-button-size".into(), pixels(ui::ButtonSize::Default.rems())),
        ("--chartr-radius".into(), format!("{}px", rem * 0.375)),
        ("--chartr-bg".into(), colors.editor_background.to_string()),
        ("--chartr-panel".into(), colors.panel_background.to_string()),
        ("--chartr-surface".into(), colors.elevated_surface_background.to_string()),
        ("--chartr-text".into(), colors.text.to_string()),
        ("--chartr-muted".into(), colors.text_muted.to_string()),
        ("--chartr-border".into(), colors.border.to_string()),
        ("--chartr-border-variant".into(), colors.border_variant.to_string()),
        ("--chartr-control".into(), colors.element_background.to_string()),
        ("--chartr-hover".into(), colors.element_hover.to_string()),
        ("--chartr-selected".into(), colors.element_selected.to_string()),
        ("--chartr-accent".into(), colors.text_accent.to_string()),
        ("--chartr-focus".into(), colors.border_focused.to_string()),
        ("--chartr-warning".into(), theme.status().warning.to_string()),
        ("--chartr-error".into(), theme.status().error.to_string()),
        ("--chartr-success".into(), theme.status().success.to_string()),
        (
            "--chartr-scrollbar".into(),
            colors.panel_background.blend(colors.text.alpha(0.7)).alpha(1.).to_string(),
        ),
        ("--chartr-scrollbar-hover".into(), colors.text.to_string()),
    ]);
    for step in 1..=6 {
        tokens.insert(format!("--chartr-space-{step}"), format!("{}px", rem * step as f32 * 0.25));
    }
    format!("window.__chartrApplyTheme({});", serde_json::to_string(&tokens).unwrap())
}
