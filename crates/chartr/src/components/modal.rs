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
