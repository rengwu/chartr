//! Observe native children through a separate X11 connection, without pumping GTK.
use super::*;
use wry::raw_window_handle::{WindowHandle, XlibWindowHandle};
use x11_dl::xlib;

struct Parent {
    xlib: xlib::Xlib,
    display: *mut xlib::Display,
    window: xlib::Window,
}

impl Parent {
    fn new() -> Self {
        let xlib = xlib::Xlib::open().unwrap();
        // SAFETY: this test exclusively owns the connection and parent window.
        unsafe {
            let display = (xlib.XOpenDisplay)(std::ptr::null());
            assert!(!display.is_null(), "requires an X11 display");
            let root = (xlib.XDefaultRootWindow)(display);
            let window = (xlib.XCreateSimpleWindow)(display, root, 0, 0, 400, 300, 0, 0, 0);
            (xlib.XMapWindow)(display, window);
            (xlib.XSync)(display, 0);
            Self { xlib, display, window }
        }
    }

    fn children(&self) -> Vec<xlib::Window> {
        let (mut root, mut parent, mut children, mut count) = (0, 0, std::ptr::null_mut(), 0);
        // SAFETY: XQueryTree allocates the returned array; copy it before XFree.
        unsafe {
            assert_ne!(
                (self.xlib.XQueryTree)(
                    self.display,
                    self.window,
                    &mut root,
                    &mut parent,
                    &mut children,
                    &mut count
                ),
                0
            );
            if children.is_null() {
                return Vec::new();
            }
            let result = std::slice::from_raw_parts(children, count as usize).to_vec();
            (self.xlib.XFree)(children.cast());
            result
        }
    }

    fn mapped_children(&self) -> Vec<xlib::Window> {
        self.children()
            .into_iter()
            .filter(|child| {
                // SAFETY: these children belong to the live parent and no GTK events
                // run concurrently with this test's queries.
                unsafe {
                    let mut attributes = std::mem::zeroed();
                    assert_ne!(
                        (self.xlib.XGetWindowAttributes)(self.display, *child, &mut attributes),
                        0
                    );
                    attributes.map_state == xlib::IsViewable
                }
            })
            .collect()
    }
}

impl Drop for Parent {
    fn drop(&mut self) {
        // SAFETY: all child handles are dropped before the owning connection.
        unsafe {
            (self.xlib.XDestroyWindow)(self.display, self.window);
            (self.xlib.XCloseDisplay)(self.display);
        }
    }
}

#[test]
#[ignore = "requires an X11 display and GTK; run with --test-threads=1"]
fn native_child_shutdown_reaches_x11_without_another_gtk_tick() {
    gtk::init().unwrap();
    let parent = Parent::new();
    // SAFETY: the Parent outlives this borrowed handle and every child view.
    let window = unsafe { WindowHandle::borrow_raw(XlibWindowHandle::new(parent.window).into()) };
    let webview = Rc::new(
        crate::native_webview::build_child(
            WebViewBuilder::new().with_html("<p>Wayfinder</p>").with_visible(false),
            &window,
        )
        .unwrap(),
    );
    webview.set_visible(true).unwrap();
    gtk::gdk::Display::default().unwrap().sync();
    assert_eq!(parent.mapped_children().len(), 1, "visible child is over the host");

    let visibility = NativeViewLeaseOwner::default();
    let previous_frame = VisibleWebView {
        webview: Rc::downgrade(&webview),
        lease: visibility.acquire(),
        frame: None,
        visible: true,
    };
    // Switching to a terminal drops the old element state without closing the
    // plugin. That hide must also reach X11 before any later GTK iteration.
    drop(previous_frame);
    assert!(parent.mapped_children().is_empty(), "inactive webview still covers the host");
    webview.set_visible(true).unwrap();
    assert_eq!(parent.mapped_children().len(), 1);

    let handle = NativeWebViewHandle::default();
    handle.install(webview.clone());
    // GPUI can retain the previous frame's Rc after the item closes. Shutdown
    // must hide the child immediately, even when the pane's GTK pump stops.
    handle.shutdown();
    assert!(handle.get().is_none());
    assert!(parent.mapped_children().is_empty(), "closed webview still covers the host");

    drop(webview);
    assert!(parent.children().is_empty(), "destroy request never reached X11");
}
