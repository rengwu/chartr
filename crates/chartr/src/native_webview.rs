//! Compatibility between GPUI's native windows and Wry child webviews.

use std::ops::Deref;

use gpui::{Bounds, Pixels};
#[cfg(not(target_os = "linux"))]
use wry::dpi::{LogicalPosition, LogicalSize, Position, Size as WrySize};
#[cfg(target_os = "linux")]
use wry::dpi::{Position, Size as WrySize};
use wry::{Rect, WebView, WebViewBuilder, raw_window_handle::HasWindowHandle};

/// A Wry child whose native lifecycle does not depend on another GTK tick.
pub(crate) struct ChildWebView(Option<WebView>);

impl Deref for ChildWebView {
    type Target = WebView;

    fn deref(&self) -> &WebView {
        self.0.as_ref().expect("live child webview")
    }
}

impl ChildWebView {
    pub(crate) fn set_visible(&self, visible: bool) -> wry::Result<()> {
        self.deref().set_visible(visible)?;
        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            use wry::WebViewExtUnix;

            // Wry queues XMapWindow/XUnmapWindow on GTK's connection. GPUI's
            // connection cannot flush it, and closing the final web pane stops
            // its GTK pump. Finish the request before the host repaints below it.
            self.webview().display().sync();
        }
        Ok(())
    }
}

impl Drop for ChildWebView {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            use wry::WebViewExtUnix;

            let display = self.webview().display();
            // Wry also buffers XDestroyWindow. The last Rc may belong to an old
            // GPUI frame and disappear after the pane's GTK pump has stopped.
            drop(self.0.take());
            display.sync();
        }
    }
}

/// Creates a hidden child view; its pane controls visibility and bounds.
pub(crate) fn build_child(
    builder: WebViewBuilder<'_>,
    parent: &impl HasWindowHandle,
) -> wry::Result<ChildWebView> {
    #[cfg(target_os = "linux")]
    {
        let handle = parent.window_handle()?;
        let raw = xlib_handle(handle.as_raw())?;
        // SAFETY: XCB and Xlib identify the same X11 window with the same XID.
        // This changes only its representation, not the window or connection.
        // The original parent remains borrowed throughout child construction.
        let parent = unsafe { wry::raw_window_handle::WindowHandle::borrow_raw(raw) };
        let webview = ChildWebView(Some(builder.build_as_child(&parent)?));
        // Wry installs its X11 container after applying `with_visible(false)`.
        // Hide it again now so inactive tabs don't leave a mapped child window.
        webview.set_visible(false)?;
        Ok(webview)
    }
    #[cfg(not(target_os = "linux"))]
    builder.build_as_child(parent).map(|webview| ChildWebView(Some(webview)))
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

/// Match CSS pixels to GPUI logical pixels, independently of GTK's desktop scale.
#[cfg(target_os = "linux")]
pub(crate) fn sync_content_scale(webview: &WebView, host_scale: f32) -> wry::Result<()> {
    use gtk::prelude::*;
    use wry::WebViewExtUnix;

    let widget = webview.webview();
    // Physical bounds only size the container. WebKit also multiplies page
    // content by the GTK widget scale, which can differ from the GPUI scale
    // (for example GDK_SCALE=2 with a 1x X11 host). Compensate with page zoom
    // so layout, canvas resolution, and pointer coordinates agree with GPUI.
    let zoom = f64::from(host_scale) / f64::from(widget.scale_factor());
    if widget.property::<f64>("zoom-level") != zoom {
        webview.zoom(zoom)?;
    }
    Ok(())
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

    // Run in a separate process for each GTK scale; GTK reads GDK_SCALE at init:
    // GDK_BACKEND=x11 GDK_SCALE=2 cargo test -p chartr --bin chartr
    //   gtk_content_tracks_host_scale -- --ignored --test-threads=1
    #[test]
    #[ignore = "requires an X11 display and a real WebKit renderer"]
    fn gtk_content_tracks_host_scale() {
        use gtk::prelude::*;
        use std::{
            sync::mpsc,
            time::{Duration, Instant},
        };
        use wry::{WebViewBuilderExtUnix, WebViewExtUnix};

        fn wait<T>(result: &mpsc::Receiver<T>) -> T {
            let deadline = Instant::now() + Duration::from_secs(15);
            loop {
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
                if let Ok(value) = result.try_recv() {
                    return value;
                }
                assert!(Instant::now() < deadline, "WebKit did not answer the layout probe");
                std::thread::sleep(Duration::from_millis(5));
            }
        }

        fn evaluate(webview: &WebView, script: &str) -> serde_json::Value {
            let (output, result) = mpsc::channel();
            webview
                .evaluate_script_with_callback(script, move |value| {
                    output.send(value).unwrap();
                })
                .unwrap();
            serde_json::from_str(&wait(&result)).expect("JavaScript result")
        }

        gtk::init().unwrap();
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_default_size(400, 300);
        let (loaded_tx, loaded_rx) = mpsc::channel();
        let webview = WebViewBuilder::new()
            .with_on_page_load_handler(move |event, _| {
                if matches!(event, wry::PageLoadEvent::Finished) {
                    let _ = loaded_tx.send(());
                }
            })
            .with_html("<html><body style='margin:0'><button style='width:120px;height:28px'>Probe</button></body></html>")
            .build_gtk(&window).unwrap();
        window.show_all();
        wait(&loaded_rx);
        let widget = webview.webview();
        let gtk_scale = f64::from(widget.scale_factor());
        if let Ok(expected) = std::env::var("GDK_SCALE") {
            assert_eq!(gtk_scale, expected.parse::<f64>().unwrap());
        }
        // Confirm this renderer actually applies GTK scaling before the fix.
        let initial = evaluate(&webview, "window.devicePixelRatio");
        assert_eq!(initial.as_f64().unwrap(), gtk_scale);

        // Include a fractional host scale and a return to 1x to exercise updates.
        for host_scale in [1.0_f32, 1.5, 2.0, 1.0] {
            sync_content_scale(&webview, host_scale).unwrap();
            let probe = evaluate(
                &webview,
                "(() => { const r = document.querySelector('button').getBoundingClientRect(); return [devicePixelRatio, innerWidth, r.width, r.height]; })()",
            );
            let dpr = probe[0].as_f64().unwrap();
            let host_scale = f64::from(host_scale);
            assert!((dpr - host_scale).abs() < 0.001, "GTK {gtk_scale}: {probe}");
            let physical_width = f64::from(widget.allocated_width()) * gtk_scale;
            assert!(
                (probe[1].as_f64().unwrap() * dpr - physical_width).abs() <= host_scale,
                "CSS viewport must fill the physical allocation: {probe}"
            );
            assert_eq!(probe[2].as_f64().unwrap() * dpr, 120.0 * host_scale);
            assert_eq!(probe[3].as_f64().unwrap() * dpr, 28.0 * host_scale);
        }
        window.close();
    }

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
