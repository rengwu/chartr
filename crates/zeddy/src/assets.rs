//! zeddy's assets: four icons, compiled in.
//!
//! Zed's `ui` components ask an [`AssetSource`] for an icon by path. zeddy uses
//! four of them, so they are embedded with `include_str!` rather than read from
//! a directory beside the binary — a GUI that cannot find its own icons at
//! runtime is a class of bug worth not having.
//!
//! Asking for anything else answers `None` rather than failing. A component
//! zeddy does not draw is not a missing asset, and an icon that silently does
//! not appear is a better failure than a window that does not open.

use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

pub struct Assets;

/// Icons zeddy draws, by the path `IconName::path` derives.
const ICONS: &[(&str, &str)] = &[
    ("icons/plus.svg", include_str!("../assets/icons/plus.svg")),
    ("icons/close.svg", include_str!("../assets/icons/close.svg")),
    ("icons/tab.svg", include_str!("../assets/icons/tab.svg")),
    ("icons/menu.svg", include_str!("../assets/icons/menu.svg")),
];

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(ICONS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, svg)| Cow::Borrowed(svg.as_bytes())))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(ICONS
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui::IconName;

    #[test]
    fn every_icon_zeddy_draws_is_embedded() {
        for icon in [IconName::Plus, IconName::Close, IconName::Tab, IconName::Menu] {
            let path = icon.path();
            assert!(
                Assets.load(&path).expect("load").is_some(),
                "{path} is drawn by zeddy but not embedded"
            );
        }
    }

    #[test]
    fn an_icon_zeddy_does_not_draw_is_absent_rather_than_an_error() {
        assert!(Assets.load("icons/nonexistent.svg").expect("load").is_none());
    }
}
