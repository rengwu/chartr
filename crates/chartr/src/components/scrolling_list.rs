//! Shared scrolling treatment for Spaces and chat history.

use gpui::{
    Bounds, BoxShadow, ContentMask, ScrollHandle, canvas, fill, linear_color_stop, linear_gradient,
    point, relative, size,
};
use ui::{ScrollAxes, Scrollbars, WithScrollbar, prelude::*};

pub(crate) fn scrollbar_thumb_colors(colors: &theme::ThemeColors) -> [gpui::Hsla; 3] {
    [
        colors.panel_background.blend(colors.text.alpha(0.7)).alpha(1.),
        colors.text.alpha(1.),
        colors.text.alpha(1.),
    ]
}

/// A tracked list with the shared top fade and hover-only vertical scrollbar.
pub(crate) fn scrolling_list(
    scrollbar_id: impl Into<gpui::ElementId>,
    content: gpui::Stateful<gpui::Div>,
    scroll_handle: &ScrollHandle,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let colors = cx.theme().colors();
    let [thumb, hovered_thumb, active_thumb] = scrollbar_thumb_colors(colors);
    let scroll_background = colors.panel_background;
    let fade_handle = scroll_handle.clone();
    v_flex()
        .relative()
        .flex_1()
        .min_h_0()
        .w_full()
        .child(content.flex_1().min_h_0().overflow_y_scroll().track_scroll(scroll_handle))
        .child(
            // Read the clamped offset at paint time. This passive overlay
            // adds no hitbox, layout shift, timer, or scroll subscription.
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| {
                    let strength = (-fade_handle.offset().y / px(12.)).clamp(0., 1.);
                    if strength == 0. {
                        return;
                    }
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        // A small, theme-colored blurred veil gives the
                        // edge a frosted appearance. GPUI has no element
                        // backdrop blur; this needs no offscreen capture.
                        let veil = Bounds::new(
                            point(bounds.left(), bounds.top() - px(8.)),
                            size(bounds.size.width, px(8.)),
                        );
                        window.paint_drop_shadows(
                            veil,
                            Default::default(),
                            &[BoxShadow::new(
                                px(0.),
                                px(4.),
                                scroll_background.opacity(0.35 * strength),
                            )
                            .blur_radius(px(12.))],
                        );
                        window.paint_quad(fill(
                            bounds,
                            linear_gradient(
                                180.,
                                linear_color_stop(scroll_background.opacity(strength), 0.),
                                linear_color_stop(scroll_background.opacity(0.), 1.),
                            ),
                        ));
                    });
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h(px(28.))
            .max_h(relative(1.)),
        )
        .custom_scrollbars(
            Scrollbars::on_hover(ScrollAxes::Vertical)
                .id(scrollbar_id)
                .thumb_colors(thumb, hovered_thumb, active_thumb)
                .tracked_scroll_handle(scroll_handle)
                .notify_content(),
            window,
            cx,
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Modifiers, MouseButton, Render, TestAppContext};

    struct Harness {
        handle: ScrollHandle,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            v_flex().size(px(200.)).child(scrolling_list(
                "test-scrollbar",
                div().id("test-list").child(div().h(px(1000.)).w_full()),
                &self.handle,
                window,
                cx,
            ))
        }
    }

    #[gpui::test]
    fn fade_tracks_scroll_and_hover_thumb_remains_draggable(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
        });
        let handle = ScrollHandle::new();
        let (_, cx) = cx.add_window_view(|_, _| Harness { handle: handle.clone() });
        let outside = point(px(250.), px(250.));
        cx.simulate_mouse_move(outside, None, Modifiers::none());
        let resting = cx.update(|window, _| window.painted_quads().len());
        handle.set_offset(point(px(0.), px(-50.)));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        let faded = cx.update(|window, _| window.painted_quads().len());
        assert_eq!(faded, resting + 1, "scrolling paints the top fade");

        let thumb = point(px(193.), px(25.));
        cx.simulate_mouse_move(thumb, None, Modifiers::none());
        assert_eq!(cx.update(|window, _| window.painted_quads().len()), faded + 1);
        let before = handle.offset().y;
        cx.simulate_mouse_down(thumb, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside, MouseButton::Left, Modifiers::none());
        assert!(handle.offset().y < before, "the fade must not intercept thumb dragging");
        cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::none());
        assert_eq!(cx.update(|window, _| window.painted_quads().len()), faded);

        handle.set_offset(point(px(0.), px(0.)));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert_eq!(cx.update(|window, _| window.painted_quads().len()), resting);
    }
}
