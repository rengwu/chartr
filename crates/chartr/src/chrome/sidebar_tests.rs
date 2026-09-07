use gpui::{Context, Modifiers, MouseButton, Render, ScrollHandle, TestAppContext, point};
use ui::{ScrollAxes, Scrollbars, WithScrollbar, prelude::*};

struct ScrollbarHarness {
    handle: ScrollHandle,
}

impl Render for ScrollbarHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size(px(200.))
            .child(
                div()
                    .id("sessions")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.handle)
                    // A space heading must not steal the thumb's mouse-down event.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(div().h(px(1000.)).w_full()),
            )
            .custom_scrollbars(
                Scrollbars::on_hover(ScrollAxes::Vertical)
                    .id("spaces-scrollbar")
                    .tracked_scroll_handle(&self.handle)
                    .notify_content(),
                window,
                cx,
            )
    }
}

#[gpui::test]
fn sidebar_scrollbar_drags_past_the_container_and_releases(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
    });
    let handle = ScrollHandle::new();
    let (_, cx) = cx.add_window_view(|_, _| ScrollbarHarness { handle: handle.clone() });
    cx.simulate_mouse_move(point(px(250.), px(250.)), None, Modifiers::none());
    let hidden_quad_count = cx.update(|window, _| window.painted_quads().len());
    cx.simulate_mouse_move(point(px(50.), px(50.)), None, Modifiers::none());
    assert_eq!(
        cx.update(|window, _| window.painted_quads().len()),
        hidden_quad_count + 1,
        "entering the container must paint the thumb",
    );
    let thumb = point(px(193.), px(20.));
    cx.simulate_mouse_move(thumb, None, Modifiers::none());
    cx.simulate_mouse_down(thumb, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(point(px(193.), px(60.)), MouseButton::Left, Modifiers::none());
    let inside_offset = handle.offset().y;
    assert!(inside_offset < px(-100.), "thumb drag must scroll inside the container");

    let outside = point(px(250.), px(150.));
    cx.simulate_mouse_move(outside, MouseButton::Left, Modifiers::none());
    assert!(handle.offset().y < inside_offset, "drag must continue outside the container");
    assert_eq!(
        cx.update(|window, _| window.painted_quads().len()),
        hidden_quad_count + 1,
        "the thumb must remain visible throughout the drag",
    );
    cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::none());
    assert_eq!(
        cx.update(|window, _| window.painted_quads().len()),
        hidden_quad_count,
        "releasing outside must hide the thumb",
    );
    let released_offset = handle.offset();
    cx.simulate_mouse_move(point(px(250.), px(50.)), None, Modifiers::none());
    assert_eq!(handle.offset(), released_offset, "release must end scrolling");
}

#[gpui::test]
fn sidebar_scrollbar_has_contrast_in_every_theme(cx: &mut TestAppContext) {
    fn luminance(color: gpui::Hsla) -> f32 {
        let color = gpui::Rgba::from(color);
        let linear =
            |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::settings::init_themes(&crate::settings::ResolvedSettings::default(), cx);
        let registry = theme::ThemeRegistry::global(cx);
        for name in registry.list_names() {
            let theme = registry.get(&name).unwrap();
            let colors = &theme.styles.colors;
            let sidebar = crate::settings::sidebar_theme_colors(&theme);
            for thumb in super::sidebar::scrollbar_thumb_colors(colors) {
                assert_eq!(thumb.a, 1.);
                for background in
                    [colors.panel_background, sidebar.card_active, sidebar.card_inactive]
                {
                    let a = luminance(thumb);
                    let b = luminance(background);
                    let contrast = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                    assert!(contrast >= 3., "{name}: scrollbar contrast is only {contrast:.2}:1");
                }
            }
        }
    });
}
