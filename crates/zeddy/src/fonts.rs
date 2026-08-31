//! zeddy's fonts.
//!
//! Zed's `ui` components read their font and size through a
//! [`ThemeSettingsProvider`], which the `theme_settings` crate normally fills
//! in from the user's settings file. zeddy has no settings file, so it answers
//! the five questions itself — and this is then also the one place the terminal
//! font is chosen, rather than a constant in the renderer.

use std::borrow::Cow;

use gpui::{App, Font, Pixels, px};
use theme::{ThemeSettingsProvider, UiDensity};

use crate::settings::ResolvedSettings;

/// The families zeddy asks for, and the sizes it draws them at.
pub struct Fonts {
    ui: Font,
    buffer: Font,
    ui_size: Pixels,
    buffer_size: Pixels,
}

/// Chartr's typography defaults. IBM Plex Sans comes from Zed's asset bundle;
/// Mono is bundled below because Zed does not ship that face.
const UI_FAMILY: &str = "IBM Plex Sans";
const MONOSPACE_FAMILY: &str = "IBM Plex Mono";

const IBM_PLEX_MONO: &[u8] =
    include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf");

pub fn load_bundled(cx: &App) -> anyhow::Result<()> {
    cx.text_system().add_fonts(vec![Cow::Borrowed(IBM_PLEX_MONO)])
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            ui: gpui::font(UI_FAMILY),
            buffer: gpui::font(MONOSPACE_FAMILY),
            ui_size: px(14.),
            buffer_size: px(13.),
        }
    }
}

impl Fonts {
    pub fn from_settings(settings: &ResolvedSettings) -> Self {
        Self {
            ui: gpui::font(settings.ui_font_family.clone()),
            buffer: gpui::font(settings.terminal_font_family.clone()),
            ui_size: px(settings.ui_font_size),
            buffer_size: px(settings.terminal_font_size),
        }
    }

    /// The terminal's font and the line height to draw it at.
    ///
    /// The ratio is the one every terminal uses and nobody writes down: a line
    /// box about 1.4× the point size, which leaves box-drawing characters
    /// touching and leaves text legible.
    pub fn terminal(&self) -> (Font, Pixels, Pixels) {
        let size = self.buffer_size;
        (self.buffer.clone(), size, (size * 1.4).round())
    }
}

/// Whether the platform can actually rasterise the bundled terminal face.
pub fn text_renders(cx: &App) -> bool {
    cx.text_system().all_font_names().iter().any(|name| name == MONOSPACE_FAMILY)
}

impl ThemeSettingsProvider for Fonts {
    fn ui_font<'a>(&'a self, _: &'a App) -> &'a Font {
        &self.ui
    }

    fn buffer_font<'a>(&'a self, _: &'a App) -> &'a Font {
        &self.buffer
    }

    fn ui_font_size(&self, _: &App) -> Pixels {
        self.ui_size
    }

    fn buffer_font_size(&self, _: &App) -> Pixels {
        self.terminal().1
    }

    fn ui_density(&self, _: &App) -> UiDensity {
        UiDensity::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_terminal_line_box_leaves_room_for_descenders() {
        let (_, size, line_height) = Fonts::default().terminal();
        assert!(line_height > size, "glyphs would clip");
        assert!(line_height < size * 2., "the grid would look double-spaced");
    }

    #[test]
    fn zeddy_names_a_family_on_every_platform() {
        assert!(!MONOSPACE_FAMILY.is_empty() && !UI_FAMILY.is_empty());
    }

    #[test]
    fn the_default_monospace_is_a_real_bundled_font() {
        assert!(IBM_PLEX_MONO.starts_with(&[0, 1, 0, 0]));
        assert!(IBM_PLEX_MONO.len() > 100_000);
    }
}
