//! Chartr's icon overrides, compiled in, with Zed's assets as a fallback.
//!
//! Zed's `ui` components ask an [`AssetSource`] for icons by their `IconName`
//! path. Chartr overrides the handful it owns (including its Hugeicons browser
//! controls), then delegates every other asset to Zed's bundled catalog.

use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

pub struct Assets;

/// Icons zeddy draws, by the path `IconName::path` derives.
const ICONS: &[(&str, &str)] = &[
    ("icons/plus.svg", include_str!("../assets/icons/plus.svg")),
    ("icons/blockchain_01.svg", include_str!("../assets/icons/blockchain_01.svg")),
    ("icons/close.svg", include_str!("../assets/icons/close.svg")),
    ("icons/tab.svg", include_str!("../assets/icons/tab.svg")),
    ("icons/menu.svg", include_str!("../assets/icons/menu.svg")),
    ("icons/arrow_left.svg", include_str!("../assets/icons/arrow_left.svg")),
    ("icons/arrow_right.svg", include_str!("../assets/icons/arrow_right.svg")),
    ("icons/rotate_cw.svg", include_str!("../assets/icons/rotate_cw.svg")),
    ("icons/stop.svg", include_str!("../assets/icons/stop.svg")),
    ("icons/lock.svg", include_str!("../assets/icons/lock.svg")),
    ("icons/public.svg", include_str!("../assets/icons/public.svg")),
];

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, svg)) = ICONS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(svg.as_bytes())));
        }
        zed_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut assets = zed_assets::Assets.list(path)?;
        for (name, _) in ICONS.iter().filter(|(name, _)| name.starts_with(path)) {
            let name = SharedString::from(*name);
            if !assets.contains(&name) {
                assets.push(name);
            }
        }
        Ok(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui::IconName;

    #[test]
    fn every_icon_zeddy_draws_is_embedded() {
        assert!(Assets.load("icons/blockchain_01.svg").expect("load").is_some());
        for icon in [
            IconName::Plus,
            IconName::Close,
            IconName::Tab,
            IconName::Menu,
            IconName::ArrowLeft,
            IconName::ArrowRight,
            IconName::RotateCw,
            IconName::Stop,
            IconName::Lock,
            IconName::Public,
        ] {
            let path = icon.path();
            assert!(
                Assets.load(&path).expect("load").is_some(),
                "{path} is drawn by zeddy but not embedded"
            );
        }
    }

    #[test]
    fn other_assets_fall_back_to_zeds_catalog() {
        assert!(Assets.load(&IconName::Check.path()).expect("load").is_some());
    }
}
