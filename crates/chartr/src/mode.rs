//! Presentation modes. Sidebar and Tabs project the existing terminal layout;
//! Inbox projects durable provider identities over the same runtimes.
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
    /// Agent session history beside the selected session terminal.
    #[serde(alias = "conversations")]
    Inbox,
}

const TRANSITION_DURATION: Duration = Duration::from_millis(500);

fn transition_easing(progress: f32) -> f32 {
    // Exponential ease-out: a quick initial slide followed by a long settle.
    // Normalize the curve so it reaches the destination without a final snap.
    (1. - 2_f32.powf(-10. * progress)) / (1. - 2_f32.powi(-10))
}

/// Each surface owns its reveal amount; modes only choose the destinations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChromeVisibility {
    pub sidebar: f32,
    pub tabs: f32,
}

impl From<Mode> for ChromeVisibility {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Sidebar | Mode::Inbox => Self { sidebar: 1., tabs: 0. },
            Mode::Tabs => Self { sidebar: 0., tabs: 1. },
        }
    }
}

#[derive(Default)]
pub(crate) struct ModeTransition {
    motion: Option<Motion>,
    sidebar_mode: Mode,
    sidebar_motion: Option<SidebarMotion>,
}

// Spaces occupies page 0 and Chats page 1 within the shared sidebar.
struct SidebarMotion {
    from: f32,
    to: f32,
    started: Instant,
}

impl SidebarMotion {
    fn sample(&self, now: Instant) -> (f32, bool) {
        let progress = now.saturating_duration_since(self.started).as_secs_f32()
            / TRANSITION_DURATION.as_secs_f32();
        if progress >= 1. || self.from == self.to {
            return (self.to, false);
        }
        let eased = transition_easing(progress);
        (self.from + (self.to - self.from) * eased, true)
    }
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
        let eased = transition_easing(progress);
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
    /// Keep the outgoing sidebar's contents while Tabbed hides its pane.
    pub fn sidebar_mode(&self) -> Mode {
        self.sidebar_mode
    }

    /// Horizontal page position, independent of pane geometry and terminal resizing.
    pub fn sidebar_position(&self, now: Instant) -> (f32, bool) {
        self.sidebar_motion.as_ref().map_or((0., false), |motion| motion.sample(now))
    }

    pub fn advance(
        &mut self,
        mode: Mode,
        now: Instant,
        reduce_motion: bool,
    ) -> (ChromeVisibility, bool) {
        if mode != Mode::Tabs {
            self.sidebar_mode = mode;
        }
        let sidebar_target = if self.sidebar_mode() == Mode::Inbox { 1. } else { 0. };
        // Only Spaces <-> Chats slides the contents. Any switch involving Tabs
        // settles the page immediately, even if the pane is still moving.
        let snap_sidebar = reduce_motion
            || mode == Mode::Tabs
            || self.motion.as_ref().is_none_or(|motion| motion.to.tabs == 1.);
        if snap_sidebar
            || self.sidebar_motion.as_ref().is_none_or(|motion| motion.to != sidebar_target)
        {
            let from = if snap_sidebar { sidebar_target } else { self.sidebar_position(now).0 };
            self.sidebar_motion = Some(SidebarMotion { from, to: sidebar_target, started: now });
        }
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
        for (from, to) in [
            (Mode::Sidebar, Mode::Tabs),
            (Mode::Tabs, Mode::Sidebar),
            (Mode::Inbox, Mode::Tabs),
            (Mode::Tabs, Mode::Inbox),
        ] {
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
    fn sidebar_and_inbox_keep_the_same_pane_visible() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        let visible = ChromeVisibility { sidebar: 1., tabs: 0. };
        for mode in [Mode::Sidebar, Mode::Inbox, Mode::Sidebar] {
            assert_eq!(transition.advance(mode, now, false), (visible, false));
            assert_eq!(transition.sidebar_mode(), mode);
        }
    }

    #[test]
    fn inbox_contents_stay_in_the_sidebar_through_its_exit_to_tabs() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Inbox, now, false);
        for elapsed in [Duration::ZERO, TRANSITION_DURATION / 4, TRANSITION_DURATION / 2] {
            let (visible, animating) = transition.advance(Mode::Tabs, now + elapsed, false);
            assert!(animating && visible.sidebar > 0.);
            assert_eq!(transition.sidebar_mode(), Mode::Inbox);
        }
        assert_eq!(
            transition.advance(Mode::Tabs, now + TRANSITION_DURATION, false),
            (Mode::Tabs.into(), false),
        );
    }

    #[test]
    fn interrupted_sidebar_exits_use_the_explicitly_selected_sidebar_content() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Inbox, now, false);
        transition.advance(Mode::Tabs, now, false);
        let midway = now + TRANSITION_DURATION / 3;
        let (visible, _) = transition.advance(Mode::Tabs, midway, false);
        assert_eq!(transition.advance(Mode::Inbox, midway, false), (visible, true));
        assert_eq!(transition.sidebar_mode(), Mode::Inbox);
        assert_eq!(transition.advance(Mode::Sidebar, midway, false), (visible, true));
        assert_eq!(transition.sidebar_mode(), Mode::Sidebar);
        transition.advance(Mode::Tabs, midway, false);
        assert_eq!(transition.sidebar_mode(), Mode::Sidebar);
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
    fn sidebar_pages_slide_in_both_directions_without_resizing_the_pane() {
        for (from, to, start, end) in
            [(Mode::Sidebar, Mode::Inbox, 0., 1.), (Mode::Inbox, Mode::Sidebar, 1., 0.)]
        {
            let now = Instant::now();
            let mut transition = ModeTransition::default();
            transition.advance(from, now, false);
            assert_eq!(transition.sidebar_position(now), (start, false));
            assert_eq!(transition.advance(to, now, false), (to.into(), false));
            assert_eq!(transition.sidebar_position(now), (start, true));
            let midway = now + TRANSITION_DURATION / 2;
            transition.advance(to, midway, false);
            let (position, animating) = transition.sidebar_position(midway);
            assert!(animating && position > 0. && position < 1.);
            assert!((position - end).abs() < (position - start).abs());
            let finished = now + TRANSITION_DURATION;
            transition.advance(to, finished, false);
            assert_eq!(transition.sidebar_position(finished), (end, false));
        }
    }

    #[test]
    fn sidebar_page_reversals_preserve_the_current_position() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Sidebar, now, false);
        transition.advance(Mode::Inbox, now, false);
        let midway = now + TRANSITION_DURATION / 3;
        let (position, _) = transition.sidebar_position(midway);
        transition.advance(Mode::Sidebar, midway, false);
        assert_eq!(transition.sidebar_position(midway), (position, true));
        let later = midway + TRANSITION_DURATION / 3;
        let (returning, _) = transition.sidebar_position(later);
        assert!(returning < position);
        transition.advance(Mode::Inbox, later, false);
        assert_eq!(transition.sidebar_position(later), (returning, true));
    }

    #[test]
    fn sidebar_pages_do_not_animate_when_switching_to_or_from_tabs() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Sidebar, now, false);
        transition.advance(Mode::Inbox, now, false);
        let midway = now + TRANSITION_DURATION / 3;
        assert!(transition.sidebar_position(midway).1);
        transition.advance(Mode::Tabs, midway, false);
        assert_eq!(transition.sidebar_position(midway), (1., false));
        transition.advance(Mode::Sidebar, midway, false);
        assert_eq!(transition.sidebar_position(midway), (0., false));
        transition.advance(Mode::Tabs, midway, false);
        let hidden = midway + TRANSITION_DURATION;
        transition.advance(Mode::Tabs, hidden, false);
        transition.advance(Mode::Inbox, hidden, false);
        assert_eq!(transition.sidebar_position(hidden), (1., false));
    }

    #[test]
    fn reduced_motion_finishes_sidebar_page_switches() {
        let now = Instant::now();
        let mut transition = ModeTransition::default();
        transition.advance(Mode::Inbox, now, false);
        transition.advance(Mode::Sidebar, now, false);
        let midway = now + TRANSITION_DURATION / 2;
        transition.advance(Mode::Sidebar, midway, true);
        assert_eq!(transition.sidebar_position(midway), (0., false));
        transition.advance(Mode::Inbox, midway, true);
        assert_eq!(transition.sidebar_position(midway), (1., false));
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

#[cfg(test)]
mod inbox_migration_tests {
    use super::*;
    #[test]
    fn conversations_preferences_migrate_to_inbox() {
        assert_eq!(serde_json::from_str::<Mode>("\"conversations\"").unwrap(), Mode::Inbox);
        assert_eq!(serde_json::to_string(&Mode::Inbox).unwrap(), "\"inbox\"");
    }
}
