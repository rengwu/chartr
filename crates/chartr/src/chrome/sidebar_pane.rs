//! The resizable pane shared by the session list and Inbox history.

use std::time::{Duration, Instant};

use gpui::{MouseButton, deferred};
use ui::prelude::*;

use super::DraggedSidebar;
use crate::mode::Mode;

const MIN_WIDTH: f32 = 108.;
pub(crate) const INBOX_MIN_WIDTH: f32 = 120.;
const MAX_WIDTH: f32 = 480.;
const RESIZE_DURATION: Duration = Duration::from_millis(250);

pub(crate) struct SidebarPane {
    width: f32,
    motion: Option<Resize>,
}

struct Resize {
    from: f32,
    started: Instant,
}

fn constrain(width: f32, mode: Mode) -> f32 {
    width.clamp(if mode == Mode::Inbox { INBOX_MIN_WIDTH } else { MIN_WIDTH }, MAX_WIDTH)
}

impl SidebarPane {
    pub fn new(width: f32, mode: Mode) -> Self {
        Self { width: constrain(width, mode), motion: None }
    }

    /// Persist the destination, even if the app closes during an auto-resize.
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn set_mode(&mut self, mode: Mode, now: Instant) {
        let width = constrain(self.width, mode);
        if width != self.width {
            self.motion = Some(Resize { from: self.sample(now).0, started: now });
            self.width = width;
        }
    }

    /// A manual drag takes over immediately, including during an auto-resize.
    pub fn resize(&mut self, width: f32, mode: Mode) {
        self.width = constrain(width, mode);
        self.motion = None;
    }

    fn sample(&self, now: Instant) -> (f32, bool) {
        let Some(motion) = &self.motion else {
            return (self.width, false);
        };
        let progress = now.saturating_duration_since(motion.started).as_secs_f32()
            / RESIZE_DURATION.as_secs_f32();
        if progress >= 1. {
            return (self.width, false);
        }
        let eased = 1. - (1. - progress).powi(3);
        (motion.from + (self.width - motion.from) * eased, true)
    }

    pub fn advance(&mut self, now: Instant, reduce_motion: bool) -> (f32, bool) {
        if reduce_motion {
            self.motion = None;
        }
        let sampled = self.sample(now);
        if !sampled.1 {
            self.motion = None;
        }
        sampled
    }
}

pub(crate) fn render(
    width: f32,
    controls: Option<(AnyElement, AnyElement)>,
    contents: AnyElement,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .id("shared-sidebar-pane")
        .relative()
        .w(px(width))
        .h_full()
        .flex_none()
        .bg(cx.theme().colors().panel_background)
        .when_some(controls, |pane, (space_switcher, view_menu)| {
            pane.child(
                h_flex()
                    .h(px(36.))
                    .flex_none()
                    .px_2()
                    .gap_1()
                    .justify_between()
                    .child(h_flex().min_w_0().flex_1().child(space_switcher))
                    .child(h_flex().gap_px().child(view_menu)),
            )
        })
        .child(div().flex_1().min_h_0().w_full().overflow_hidden().child(contents))
        .child(deferred(
            div()
                .id("sidebar-resize-handle")
                .absolute()
                // Keep the target outside the pane, clear of trailing row actions.
                .right(px(-6.))
                .top_0()
                .h_full()
                .w(px(6.))
                .cursor_col_resize()
                .on_drag(DraggedSidebar, |dragged, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| dragged.clone())
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_expands_the_shared_pane_and_keeps_its_width_on_return() {
        let now = Instant::now();
        let mut pane = SidebarPane::new(108., Mode::Sidebar);
        pane.set_mode(Mode::Inbox, now);
        assert_eq!(pane.width(), INBOX_MIN_WIDTH);
        assert_eq!(pane.advance(now, false), (108., true));
        let (midway, animating) = pane.advance(now + RESIZE_DURATION / 2, false);
        assert!(animating && midway > 108. && midway < INBOX_MIN_WIDTH);
        // Rapid mode changes must not jump back or restart the expansion.
        pane.set_mode(Mode::Sidebar, now + RESIZE_DURATION / 2);
        assert_eq!(pane.advance(now + RESIZE_DURATION / 2, false), (midway, true));
        assert_eq!(pane.advance(now + RESIZE_DURATION, false), (INBOX_MIN_WIDTH, false));
        pane.set_mode(Mode::Inbox, now + RESIZE_DURATION);
        assert_eq!(pane.advance(now + RESIZE_DURATION, false), (INBOX_MIN_WIDTH, false));
    }

    #[test]
    fn wide_panes_and_restored_inbox_do_not_animate() {
        let now = Instant::now();
        let mut pane = SidebarPane::new(400., Mode::Sidebar);
        pane.set_mode(Mode::Inbox, now);
        assert_eq!(pane.advance(now, false), (400., false));
        let mut restored = SidebarPane::new(108., Mode::Inbox);
        assert_eq!(restored.advance(now, false), (INBOX_MIN_WIDTH, false));
    }

    #[test]
    fn dragging_takes_over_and_obeys_each_modes_limits() {
        let now = Instant::now();
        let mut pane = SidebarPane::new(108., Mode::Sidebar);
        pane.set_mode(Mode::Inbox, now);
        pane.resize(410., Mode::Inbox);
        assert_eq!(pane.advance(now, false), (410., false));
        pane.resize(50., Mode::Inbox);
        assert_eq!(pane.width(), INBOX_MIN_WIDTH);
        pane.resize(50., Mode::Sidebar);
        assert_eq!(pane.width(), MIN_WIDTH);
        pane.resize(900., Mode::Inbox);
        assert_eq!(pane.width(), MAX_WIDTH);
    }

    #[test]
    fn reduced_motion_finishes_auto_resize() {
        let now = Instant::now();
        let mut pane = SidebarPane::new(108., Mode::Sidebar);
        pane.set_mode(Mode::Inbox, now);
        assert_eq!(pane.advance(now, true), (INBOX_MIN_WIDTH, false));
        assert_eq!(pane.advance(now, false), (INBOX_MIN_WIDTH, false));
    }
}
