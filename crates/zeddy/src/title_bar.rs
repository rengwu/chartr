//! The small, app-drawn macOS title bar shared by Chartr windows.

use gpui::{ElementId, MouseButton, TitlebarOptions, point, px};
use ui::prelude::*;

const HEIGHT: f32 = 34.;

/// Use the native title bar everywhere except macOS, where Chartr draws the
/// background and AppKit keeps responsibility for the traffic-light controls.
pub fn options(fallback_title: &'static str) -> TitlebarOptions {
    TitlebarOptions {
        title: (!cfg!(target_os = "macos")).then(|| fallback_title.into()),
        appears_transparent: cfg!(target_os = "macos"),
        traffic_light_position: cfg!(target_os = "macos").then(|| point(px(9.), px(9.))),
    }
}

pub const fn app_owns_drag() -> bool {
    cfg!(target_os = "macos")
}

pub struct TitleBar {
    id: ElementId,
    should_move: bool,
}

impl TitleBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), should_move: false }
    }
}

/// Draw a deliberately empty title bar on macOS. The native traffic lights
/// sit above this surface, so the rest of the bar remains one drag target.
impl Render for TitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !cfg!(target_os = "macos") {
            return div().id(self.id.clone()).into_any_element();
        }

        let colors = cx.theme().colors();
        let background = if window.is_window_active() {
            colors.title_bar_background
        } else {
            colors.title_bar_inactive_background
        };

        h_flex()
            .id(self.id.clone())
            .w_full()
            .h(px(HEIGHT))
            .flex_none()
            .bg(background)
            .border_b_1()
            .border_color(colors.border)
            .on_mouse_down_out(cx.listener(|this, _, _, _| this.should_move = false))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.should_move = false))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| this.should_move = true))
            .on_mouse_move(cx.listener(|this, _, window, _| {
                if this.should_move {
                    this.should_move = false;
                    window.start_window_move();
                }
            }))
            .on_click(|event, window, _| {
                if event.click_count() == 2 {
                    window.titlebar_double_click();
                }
            })
            .into_any_element()
    }
}
