//! Native modal hosting above embedded webviews.

use gpui::{
    App, Bounds, Entity, Render, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowKind, WindowOptions, px, size,
};

/// Open a window-sized modal surface above every native child view in `parent_window`.
///
/// Native webviews are composited above their parent window's GPUI scene, so an in-window modal
/// can never cover them regardless of its elevation. A parent-anchored native popup establishes
/// the correct platform stacking order while still letting GPUI render the scrim and dialog.
pub fn open_native_modal<V: Render + 'static>(
    parent_window: &mut Window,
    cx: &mut App,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V> + 'static,
) -> Result<WindowHandle<V>, String> {
    use gpui::popup::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupOptions};

    let parent = parent_window.window_handle();
    let modal_size = parent_window.viewport_size();
    let rem_size = parent_window.rem_size();
    let display_id = parent_window.display(cx).map(|display| display.id());

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Default::default(),
                modal_size,
            ))),
            titlebar: None,
            focus: true,
            show: true,
            kind: WindowKind::AnchoredPopup(PopupOptions {
                parent,
                anchor_rect: Bounds::new(Default::default(), size(px(1.), px(1.))),
                anchor: PopupAnchor::TopLeft,
                gravity: PopupGravity::BottomRight,
                constraint_adjustment: PopupConstraintAdjustment::empty(),
                offset: Default::default(),
                // A modal covers the complete parent and owns its input without relying on a
                // menu-style grab. This also permits opening it from an active context menu.
                grab: false,
            }),
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            display_id,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        },
        move |window, cx| {
            window.set_rem_size(rem_size);
            build(window, cx)
        },
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_plugin::ui as plugin_ui;
    use ui::prelude::*;

    struct DialogHarness;
    impl Render for DialogHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            plugin_ui::DialogSurface::new("dialog-layout-test")
                .height_limit(px(180.))
                .debug_selector("DIALOG")
                .child(plugin_ui::dialog_header("Dialog title", div(), cx))
                .child(
                    plugin_ui::dialog_body()
                        .id("dialog-body")
                        .debug_selector(|| "DIALOG_BODY".into())
                        .overflow_y_scroll()
                        .child(div().h(px(400.)).flex_none()),
                )
                .child(
                    plugin_ui::dialog_actions(cx)
                        .debug_selector(|| "DIALOG_ACTIONS".into())
                        .child(plugin_ui::action("cancel", "Cancel"))
                        .child(plugin_ui::action("save", "Save")),
                )
        }
    }

    #[gpui::test]
    fn dialog_sections_span_surface_and_keep_actions_visible(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
        let (_, cx) = cx.add_window_view(|_, _| DialogHarness);
        cx.simulate_resize(size(px(420.), px(280.)));
        cx.run_until_parked();
        let dialog = cx.debug_bounds("DIALOG").unwrap();
        let body = cx.debug_bounds("DIALOG_BODY").unwrap();
        let actions = cx.debug_bounds("DIALOG_ACTIONS").unwrap();
        assert!(dialog.size.height <= px(180.));
        assert!(body.size.height > px(0.) && body.size.height < px(400.));
        assert_eq!(body.bottom(), actions.top());
        assert_eq!(actions.bottom(), dialog.bottom() - px(1.));
        assert_eq!(actions.left(), dialog.left() + px(1.));
        assert_eq!(actions.right(), dialog.right() - px(1.));
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let mut dividers: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| {
                    quad.border_color == cx.theme().colors().border
                        && quad.border_widths.left.as_f32() == 0.
                        && (quad.border_widths.top.as_f32() > 0.
                            || quad.border_widths.bottom.as_f32() > 0.)
                })
                .collect();
            // GPUI splits an outlined element into quads with different masks.
            dividers.dedup_by(|a, b| a.bounds == b.bounds);
            assert_eq!(dividers.len(), 2, "header and footer separators");
            for divider in dividers {
                assert_eq!(px(divider.bounds.origin.x.as_f32() / scale), actions.left());
                assert_eq!(px(divider.bounds.size.width.as_f32() / scale), actions.size.width);
            }
        });
    }
}
