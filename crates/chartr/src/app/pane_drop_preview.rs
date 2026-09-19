use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{Bounds, point, size};
use ui::prelude::*;

const INSET: f32 = 6.;
const DURATION: Duration = Duration::from_millis(140);

/// One window-space rectangle, shared by all pane targets so it can travel
/// between panes without being clipped by their individual content bounds.
#[derive(Clone, Default)]
pub(super) struct PaneDropPreview(Rc<RefCell<State>>);

#[derive(Default)]
struct State {
    target: Option<Bounds<Pixels>>,
    motion: Option<Motion>,
}

struct Motion {
    from: Bounds<Pixels>,
    to: Bounds<Pixels>,
    started: Instant,
}

impl Motion {
    fn bounds(&self, now: Instant) -> Bounds<Pixels> {
        let progress = (now.saturating_duration_since(self.started).as_secs_f32()
            / DURATION.as_secs_f32())
        .min(1.);
        let eased = 1. - (1. - progress).powi(3);
        Bounds::new(
            self.from.origin + (self.to.origin - self.from.origin) * eased,
            size(
                self.from.size.width + (self.to.size.width - self.from.size.width) * eased,
                self.from.size.height + (self.to.size.height - self.from.size.height) * eased,
            ),
        )
    }
}

impl State {
    fn advance(&mut self, now: Instant, reduce_motion: bool) -> Option<(Bounds<Pixels>, bool)> {
        let Some(target) = self.target else {
            self.motion = None;
            return None;
        };
        if self.motion.as_ref().is_none_or(|motion| motion.to != target) || reduce_motion {
            // Retarget from the current presentation bounds, even if the previous
            // transition has not finished. Repeated pointer events don't restart it.
            let from = if reduce_motion {
                target
            } else {
                self.motion.as_ref().map_or(target, |motion| motion.bounds(now))
            };
            self.motion = Some(Motion { from, to: target, started: now });
        }
        let motion = self.motion.as_ref().unwrap();
        let animating =
            motion.from != motion.to && now.saturating_duration_since(motion.started) < DURATION;
        Some((motion.bounds(now), animating))
    }
}

impl PaneDropPreview {
    #[cfg(test)]
    pub(super) fn target_bounds(&self) -> Option<Bounds<Pixels>> {
        self.0.borrow().target
    }

    pub(super) fn target(&self) -> impl IntoElement {
        let state = self.0.clone();
        gpui::canvas(
            |_, _, _| {},
            move |bounds, _, _, _| {
                // This canvas only paints when its drop target's group is hovered
                // and accepts the payload. Keep the actual drop hitbox full size.
                let inset_x = px(INSET).min(bounds.size.width / 2.);
                let inset_y = px(INSET).min(bounds.size.height / 2.);
                state.borrow_mut().target = Some(Bounds::new(
                    bounds.origin + point(inset_x, inset_y),
                    size(bounds.size.width - inset_x * 2., bounds.size.height - inset_y * 2.),
                ));
            },
        )
        .absolute()
        .size_full()
    }

    pub(super) fn overlay(&self) -> impl IntoElement {
        let prepare = self.0.clone();
        let paint = self.0.clone();
        gpui::canvas(
            // All prepaint callbacks precede painting. Visible targets then
            // nominate their bounds before this last workspace child paints.
            move |_, _, _| prepare.borrow_mut().target = None,
            move |_, _, window, cx| {
                if let Some((bounds, animating)) =
                    paint.borrow_mut().advance(cx.background_executor().now(), cx.reduce_motion())
                {
                    let mut quad = gpui::fill(bounds, cx.theme().colors().drop_target_background);
                    quad.corner_radii = px(4.).into();
                    window.paint_quad(quad);
                    if animating {
                        window.request_animation_frame();
                    }
                }
            },
        )
        .absolute()
        .size_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, width: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(6.)), size(px(width), px(388.)))
    }

    #[test]
    fn retargets_across_panes_without_jumping_or_restarting_on_pointer_motion() {
        let now = Instant::now();
        let mut state = State { target: Some(rect(6., 388.)), ..Default::default() };
        assert_eq!(state.advance(now, false), Some((rect(6., 388.), false)));
        state.target = Some(rect(406., 188.));
        assert_eq!(state.advance(now, false), Some((rect(6., 388.), true)));
        let halfway = now + DURATION / 2;
        let (presented, animating) = state.advance(halfway, false).unwrap();
        assert!(animating);
        assert_eq!(presented, rect(356., 213.));

        // Redirect back across the divider while also expanding to a full pane.
        state.target = Some(rect(6., 588.));
        assert_eq!(state.advance(halfway, false), Some((presented, true)));
        assert_eq!(state.advance(halfway + DURATION, false), Some((rect(6., 588.), false)));
    }

    #[test]
    fn reduced_motion_snaps_and_leaving_the_targets_clears_the_preview() {
        let now = Instant::now();
        let mut state = State { target: Some(rect(6., 388.)), ..Default::default() };
        state.advance(now, false);
        state.target = Some(rect(406., 188.));
        assert_eq!(state.advance(now, true), Some((rect(406., 188.), false)));
        state.target = None;
        assert_eq!(state.advance(now, false), None);
        assert!(state.motion.is_none());
        state.target = Some(rect(6., 388.));
        assert_eq!(state.advance(now, false), Some((rect(6., 388.), false)));
    }
}
