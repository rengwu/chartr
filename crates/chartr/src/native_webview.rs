//! Compatibility between GPUI's native windows and Wry child webviews.

use gpui::{Bounds, Pixels};
#[cfg(not(target_os = "linux"))]
use wry::dpi::{LogicalPosition, LogicalSize, Position, Size as WrySize};
#[cfg(target_os = "linux")]
use wry::dpi::{Position, Size as WrySize};
use wry::{Rect, WebView, WebViewBuilder, raw_window_handle::HasWindowHandle};

/// Creates a hidden child view; its pane controls visibility and bounds.
pub(crate) fn build_child(
    builder: WebViewBuilder<'_>,
    parent: &impl HasWindowHandle,
) -> wry::Result<WebView> {
    #[cfg(target_os = "linux")]
    {
        let handle = parent.window_handle()?;
        let raw = xlib_handle(handle.as_raw())?;
        // SAFETY: XCB and Xlib identify the same X11 window with the same XID.
        // This changes only its representation, not the window or connection.
        // The original parent remains borrowed throughout child construction.
        let parent = unsafe { wry::raw_window_handle::WindowHandle::borrow_raw(raw) };
        let webview = builder.build_as_child(&parent)?;
        // Wry installs its X11 container after applying `with_visible(false)`.
        // Hide it again now so inactive tabs don't leave a mapped child window.
        webview.set_visible(false)?;
        Ok(webview)
    }
    #[cfg(not(target_os = "linux"))]
    builder.build_as_child(parent)
}

#[cfg(target_os = "linux")]
fn xlib_handle(
    handle: wry::raw_window_handle::RawWindowHandle,
) -> wry::Result<wry::raw_window_handle::RawWindowHandle> {
    use wry::raw_window_handle::{RawWindowHandle, XlibWindowHandle};

    match handle {
        RawWindowHandle::Xcb(handle) => {
            // GPUI exposes XCB, while Wry 0.56 accepts only Xlib child parents.
            let mut xlib = XlibWindowHandle::new(handle.window.get().into());
            xlib.visual_id = handle.visual_id.map_or(0, |id| id.get().into());
            Ok(xlib.into())
        }
        RawWindowHandle::Xlib(_) => Ok(handle),
        _ => Err(wry::Error::UnsupportedWindowHandle),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeFrame {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl NativeFrame {
    pub(crate) fn snapped(bounds: Bounds<Pixels>, scale: f32) -> Self {
        let scale = if cfg!(target_os = "linux") { scale } else { 1.0 };
        let left = (bounds.left().as_f32() * scale).round() as i32;
        let top = (bounds.top().as_f32() * scale).round() as i32;
        let right = (bounds.right().as_f32() * scale).round() as i32;
        let bottom = (bounds.bottom().as_f32() * scale).round() as i32;
        Self { x: left, y: top, width: (right - left).max(0), height: (bottom - top).max(0) }
    }

    pub(crate) fn wry(self) -> Rect {
        #[cfg(target_os = "linux")]
        {
            // GTK's GDK_SCALE can differ from GPUI's scale. Physical bounds
            // keep the X11 child inside its pane regardless of that setting.
            use wry::dpi::{PhysicalPosition, PhysicalSize};
            Rect {
                position: Position::Physical(PhysicalPosition::new(self.x, self.y)),
                size: WrySize::Physical(PhysicalSize::new(self.width as u32, self.height as u32)),
            }
        }
        #[cfg(not(target_os = "linux"))]
        Rect {
            position: Position::Logical(LogicalPosition::new(f64::from(self.x), f64::from(self.y))),
            size: WrySize::Logical(LogicalSize::new(f64::from(self.width), f64::from(self.height))),
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{num::NonZeroU32, ptr::NonNull};
    use wry::raw_window_handle::{
        RawWindowHandle, WaylandWindowHandle, XcbWindowHandle, XlibWindowHandle,
    };

    #[test]
    fn xcb_preserves_window_and_optional_visual_ids() {
        for window in [1, 0x12345678, u32::MAX] {
            for visual in [None, NonZeroU32::new(42)] {
                let mut xcb = XcbWindowHandle::new(NonZeroU32::new(window).unwrap());
                xcb.visual_id = visual;
                let RawWindowHandle::Xlib(xlib) = xlib_handle(xcb.into()).unwrap() else {
                    panic!("Wry needs an Xlib window handle");
                };
                assert_eq!(xlib.window, u64::from(window));
                assert_eq!(xlib.visual_id, visual.map_or(0, |id| u64::from(id.get())));
            }
        }
    }

    #[test]
    fn existing_xlib_handle_is_preserved() {
        let mut xlib = XlibWindowHandle::new(123);
        xlib.visual_id = 456;
        assert_eq!(xlib_handle(xlib.into()).unwrap(), RawWindowHandle::Xlib(xlib));
    }

    #[test]
    fn unsupported_parent_is_rejected_before_entering_wry() {
        let wayland = WaylandWindowHandle::new(NonNull::dangling());
        assert!(matches!(xlib_handle(wayland.into()), Err(wry::Error::UnsupportedWindowHandle)));
    }

    #[test]
    fn native_bounds_use_gpui_pixels_even_when_gtk_has_a_different_scale() {
        let bounds = Bounds::new(
            gpui::point(gpui::px(12.5), gpui::px(34.5)),
            gpui::size(gpui::px(100.5), gpui::px(200.5)),
        );
        let frame = NativeFrame::snapped(bounds, 2.0).wry();
        for gtk_scale in [1.0, 2.0, 3.0] {
            assert_eq!(
                frame.position.to_physical::<i32>(gtk_scale),
                wry::dpi::PhysicalPosition::new(25, 69)
            );
            assert_eq!(
                frame.size.to_physical::<u32>(gtk_scale),
                wry::dpi::PhysicalSize::new(201, 401)
            );
        }
    }
}
