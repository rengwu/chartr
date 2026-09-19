//! Native AppKit regression check (requires a macOS desktop session):
//! cargo run --manifest-path vendor/zed-platform/gpui_macos/Cargo.toml --example passive_popup
//!
//! Run on the main thread, as required by AppKit. GPUI's simulated windows do
//! not exercise native focus eligibility or mouse-event routing.
#[cfg(target_os = "macos")]
fn main() {
    use cocoa::base::id;
    use gpui::{
        App, AppContext, Application, Bounds, Context, IntoElement, Render, Window, WindowBounds,
        WindowKind, WindowOptions, div, point, popup::PopupOptions, px, size,
    };
    use gpui_macos::MacPlatform;
    use objc::{
        msg_send,
        runtime::{BOOL, YES},
        sel, sel_impl,
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::rc::Rc;

    struct Empty;
    impl Render for Empty {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    fn native_input_properties(window: &Window) -> (bool, bool, bool) {
        let RawWindowHandle::AppKit(handle) =
            HasWindowHandle::window_handle(window).unwrap().as_raw()
        else {
            panic!("expected an AppKit window");
        };
        unsafe {
            let view = handle.ns_view.as_ptr() as id;
            let panel: id = msg_send![view, window];
            let ignores_mouse: BOOL = msg_send![panel, ignoresMouseEvents];
            let can_become_key: BOOL = msg_send![panel, canBecomeKeyWindow];
            let can_become_main: BOOL = msg_send![panel, canBecomeMainWindow];
            (ignores_mouse == YES, can_become_key == YES, can_become_main == YES)
        }
    }

    Application::with_platform(Rc::new(MacPlatform::new(false))).run(|cx: &mut App| {
        // Keep these windows hidden: querying native input policy does not need
        // to take focus away from the application the developer is using.
        let parent = cx
            .open_window(
                WindowOptions { show: false, focus: false, ..Default::default() },
                |_, cx| cx.new(|_| Empty),
            )
            .unwrap();
        parent
            .update(cx, |_, window, _| {
                assert_eq!(native_input_properties(window), (false, true, true));
            })
            .unwrap();

        for (focus, grab) in [(false, false), (false, true), (true, false), (true, true)] {
            let popup = cx
                .open_window(
                    WindowOptions {
                        show: false,
                        focus,
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                            point(px(0.), px(0.)),
                            size(px(80.), px(30.)),
                        ))),
                        kind: WindowKind::AnchoredPopup(PopupOptions {
                            parent: parent.into(),
                            anchor_rect: Bounds::new(point(px(10.), px(10.)), size(px(1.), px(1.))),
                            anchor: Default::default(),
                            gravity: Default::default(),
                            constraint_adjustment: Default::default(),
                            offset: Default::default(),
                            grab,
                        }),
                        ..Default::default()
                    },
                    |_, cx| cx.new(|_| Empty),
                )
                .unwrap();
            popup
                .update(cx, |_, window, _| {
                    let passive = !focus && !grab;
                    assert_eq!(
                        native_input_properties(window),
                        (passive, !passive, !passive),
                        "incorrect native input policy for focus={focus}, grab={grab}",
                    );
                    window.remove_window();
                })
                .unwrap();
        }
        println!("Native popup input checks passed (tooltip, menus, and normal window).");
        cx.quit();
    });
}

#[cfg(not(target_os = "macos"))]
fn main() {}
