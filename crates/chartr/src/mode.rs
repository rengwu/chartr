//! Presentation modes. Sidebar and Tabs project the existing terminal layout;
//! Conversations projects durable provider identities over the same runtimes.
//! Switching presentation never starts or stops a session.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Where the session list is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// A vertical list down the left. Wide enough for a directory, an agent
    /// name, and a status — the mode for many long-lived sessions.
    #[default]
    Sidebar,
    /// A horizontal strip across the top. Familiar, and denser per session —
    /// the mode for a handful of things you are switching between quickly.
    Tabs,
    /// History and rich chat over the existing agent runtimes.
    Conversations,
}

const TRANSITION_DURATION: Duration = Duration::from_millis(500);

/// Each surface owns its reveal amount; modes only choose the destinations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChromeVisibility {
    pub sidebar: f32,
    pub tabs: f32,
}

impl From<Mode> for ChromeVisibility {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Sidebar => Self { sidebar: 1., tabs: 0. },
            Mode::Tabs => Self { sidebar: 0., tabs: 1. },
            Mode::Conversations => Self { sidebar: 0., tabs: 0. },
        }
    }
}

#[derive(Default)]
pub(crate) struct ModeTransition {
    motion: Option<Motion>,
}

struct Motion {
    from: ChromeVisibility,
    to: ChromeVisibility,
    started: Instant,
}

impl Motion {
    fn sample(&self, now: Instant) -> (ChromeVisibility, bool) {
        let elapsed = now.saturating_duration_since(self.started);
        if elapsed >= TRANSITION_DURATION || self.from == self.to {
            return (self.to, false);
        }
        let progress = elapsed.as_secs_f32() / TRANSITION_DURATION.as_secs_f32();
        // Exponential ease-out: a quick initial slide followed by a long settle.
        // Normalize the curve so it reaches the destination without a final snap.
        let eased = (1. - 2_f32.powf(-10. * progress)) / (1. - 2_f32.powi(-10));
        (
            ChromeVisibility {
                sidebar: self.from.sidebar + (self.to.sidebar - self.from.sidebar) * eased,
                tabs: self.from.tabs + (self.to.tabs - self.from.tabs) * eased,
            },
            true,
        )
    }
}

impl ModeTransition {
    pub fn advance(
        &mut self,
        mode: Mode,
        now: Instant,
        reduce_motion: bool,
    ) -> (ChromeVisibility, bool) {
        let target = ChromeVisibility::from(mode);
        if self.motion.as_ref().is_none_or(|motion| motion.to != target) || reduce_motion {
            // Restore directly on first render. Redirect an interrupted slide from
            // its presentation position instead of jumping to the previous mode.
            let from = if reduce_motion {
                target
            } else {
                self.motion.as_ref().map_or(target, |motion| motion.sample(now).0)
            };
            self.motion = Some(Motion { from, to: target, started: now });
        }
        self.motion.as_ref().unwrap().sample(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_without_motion_and_slides_both_surfaces_together() {
        for (from, to) in [(Mode::Sidebar, Mode::Tabs), (Mode::Tabs, Mode::Sidebar)] {
            let now = Instant::now();
            let mut transition = ModeTransition::default();
            assert_eq!(transition.advance(from, now, false), (from.into(), false));
            assert_eq!(transition.advance(to, now, false), (from.into(), true));
            let (visible, animating) = transition.advance(to, now + TRANSITION_DURATION / 2, false);
            assert!(animating);
            assert!(visible.sidebar > 0. && visible.sidebar < 1.);
            assert!(visible.tabs > 0. && visible.tabs < 1.);
            assert!((visible.sidebar + visible.tabs - 1.).abs() < f32::EPSILON);
            assert_eq!(
                transition.advance(to, now + TRANSITION_DURATION, false),
                (to.into(), false)
            );
        }
    }

    #[test]
    fn quick_reversals_keep_the_current_position() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Sidebar, now, false);
        transition.advance(Mode::Tabs, now, false);
        let midway = now + TRANSITION_DURATION / 3;
        let (visible, _) = transition.advance(Mode::Tabs, midway, false);
        assert_eq!(transition.advance(Mode::Sidebar, midway, false), (visible, true));
        assert_eq!(
            transition.advance(Mode::Sidebar, midway + TRANSITION_DURATION, false),
            (Mode::Sidebar.into(), false)
        );
    }

    #[test]
    fn reduced_motion_finishes_an_in_flight_transition() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Sidebar, now, false);
        transition.advance(Mode::Tabs, now, false);
        assert_eq!(
            transition.advance(Mode::Tabs, now + TRANSITION_DURATION / 2, true),
            (Mode::Tabs.into(), false)
        );
        assert_eq!(transition.advance(Mode::Sidebar, now, true), (Mode::Sidebar.into(), false));
    }
}
