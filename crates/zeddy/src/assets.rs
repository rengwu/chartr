//! Chartr's icon overrides, compiled in, with Zed's assets as a fallback.
//!
//! Zed's `ui` components ask an [`AssetSource`] for icons by their `IconName`
//! path. Chartr overrides the handful it owns (including its Hugeicons browser
//! controls), then delegates every other asset to Zed's bundled catalog.

use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

pub struct Assets;

/// The shared icon for every entry point and placeholder associated with opening a plugin pane.
pub const PLUGIN_LAUNCHER_ICON_PATH: &str = "icons/full_screen.svg";

/// Icons zeddy draws, by the path `IconName::path` derives.
const ICONS: &[(&str, &str)] = &[
    ("icons/plus.svg", include_str!("../assets/icons/plus.svg")),
    (PLUGIN_LAUNCHER_ICON_PATH, include_str!("../assets/icons/full_screen.svg")),
    ("icons/close.svg", include_str!("../assets/icons/close.svg")),
    ("icons/tab.svg", include_str!("../assets/icons/tab.svg")),
    ("icons/menu.svg", include_str!("../assets/icons/menu.svg")),
    ("icons/arrow_left.svg", include_str!("../assets/icons/arrow_left.svg")),
    ("icons/arrow_right.svg", include_str!("../assets/icons/arrow_right.svg")),
    ("icons/rotate_cw.svg", include_str!("../assets/icons/rotate_cw.svg")),
    ("icons/stop.svg", include_str!("../assets/icons/stop.svg")),
    ("icons/lock.svg", include_str!("../assets/icons/lock.svg")),
    ("icons/public.svg", include_str!("../assets/icons/public.svg")),
    ("icons/agent_ai_programming.svg", include_str!("../assets/icons/agent_ai_programming.svg")),
    ("icons/agent_chat_gpt.svg", include_str!("../assets/icons/agent_chat_gpt.svg")),
    ("icons/agent_claude.svg", include_str!("../assets/icons/agent_claude.svg")),
    ("icons/agent_copilot.svg", include_str!("../assets/icons/agent_copilot.svg")),
    ("icons/agent_deepseek.svg", include_str!("../assets/icons/agent_deepseek.svg")),
    ("icons/agent_google_gemini.svg", include_str!("../assets/icons/agent_google_gemini.svg")),
    ("icons/agent_grok.svg", include_str!("../assets/icons/agent_grok.svg")),
    ("icons/agent_mistral.svg", include_str!("../assets/icons/agent_mistral.svg")),
    ("icons/agent_pi.svg", include_str!("../assets/icons/agent_pi.svg")),
    ("icons/agent_qwen.svg", include_str!("../assets/icons/agent_qwen.svg")),
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
    fn chartrs_plugin_launcher_icon_is_embedded() {
        assert!(Assets.load(PLUGIN_LAUNCHER_ICON_PATH).expect("load").is_some());
    }

    #[test]
    fn agent_brand_icons_are_embedded() {
        for path in [
            "icons/agent_ai_programming.svg",
            "icons/agent_chat_gpt.svg",
            "icons/agent_claude.svg",
            "icons/agent_copilot.svg",
            "icons/agent_deepseek.svg",
            "icons/agent_google_gemini.svg",
            "icons/agent_grok.svg",
            "icons/agent_mistral.svg",
            "icons/agent_pi.svg",
            "icons/agent_qwen.svg",
        ] {
            assert!(Assets.load(path).expect("load").is_some(), "missing {path}");
        }
    }

    #[test]
    fn other_assets_fall_back_to_zeds_catalog() {
        assert!(Assets.load(&IconName::Check.path()).expect("load").is_some());
    }
}
