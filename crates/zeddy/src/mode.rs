//! The two ways zeddy arranges the same sessions.
//!
//! Both modes show one session at a time and switch between the same list. The
//! difference is only where the list lives, so the mode is one enum and not two
//! layouts: nothing below the chrome knows which one is showing, and toggling
//! never touches a session.

/// Where the session list is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// A vertical list down the left. Wide enough for a directory, an agent
    /// name, and a status — the mode for many long-lived sessions.
    #[default]
    Sidebar,
    /// A horizontal strip across the top. Familiar, and denser per session —
    /// the mode for a handful of things you are switching between quickly.
    Tabs,
}

impl Mode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Sidebar => Self::Tabs,
            Self::Tabs => Self::Sidebar,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_twice_is_the_identity() {
        assert_eq!(Mode::Sidebar.toggled().toggled(), Mode::Sidebar);
        assert_eq!(Mode::Tabs.toggled().toggled(), Mode::Tabs);
    }

    #[test]
    fn the_two_modes_are_the_only_two() {
        assert_eq!(Mode::default(), Mode::Sidebar);
    }
}
