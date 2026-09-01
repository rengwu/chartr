//! zeddy's fonts.
//!
//! Zed's `ui` components read their font and size through a
//! [`ThemeSettingsProvider`], which the `theme_settings` crate normally fills
//! in from the user's settings file. zeddy has no settings file, so it answers
//! the five questions itself — and this is then also the one place the terminal
//! font is chosen, rather than a constant in the renderer.

use std::borrow::Cow;

use gpui::{App, Font, Pixels, Rems, Window, px};
use theme::{ThemeSettingsProvider, UiDensity};
use ui::LabelSize;

use crate::settings::ResolvedSettings;

/// The families zeddy asks for, and the sizes it draws them at.
pub struct Fonts {
    ui: Font,
    buffer: Font,
    ui_size: Pixels,
    buffer_size: Pixels,
}

/// IBM Plex Sans comes from Zed's asset bundle; Mono is bundled below because
/// Zed does not ship that face.
const MONOSPACE_FAMILY: &str = "IBM Plex Mono";

/// Chartr's semantic interface type scale. These values are relative to the
/// configured `ui_font_size`, whose default is 14 px, so the default scale is
/// exactly 14/12/10 px while still respecting the user's interface scale.
pub const UI_TEXT_LARGE: Rems = Rems(1.);
pub const UI_TEXT_DEFAULT: Rems = Rems(12. / 14.);
pub const UI_TEXT_SMALL: Rems = Rems(10. / 14.);

pub const UI_LABEL_LARGE: LabelSize = LabelSize::Custom(UI_TEXT_LARGE);
pub const UI_LABEL_DEFAULT: LabelSize = LabelSize::Custom(UI_TEXT_DEFAULT);
pub const UI_LABEL_SMALL: LabelSize = LabelSize::Custom(UI_TEXT_SMALL);

const IBM_PLEX_MONO: &[u8] =
    include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf");

pub fn load_bundled(cx: &App) -> anyhow::Result<()> {
    cx.text_system().add_fonts(vec![Cow::Borrowed(IBM_PLEX_MONO)])
}

impl Default for Fonts {
    fn default() -> Self {
        Self::from_settings(&ResolvedSettings::default())
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

    /// Install the configured interface type scale on a window and return the
    /// font its root should inherit. This is the same boundary as Zed's
    /// `setup_ui_font`: `ui_font_size` is the root rem, so every UI component
    /// and semantic `LabelSize` resolves from one user-controlled scale.
    pub fn setup_ui(window: &mut Window, cx: &App) -> Font {
        let settings = theme::theme_settings(cx);
        window.set_rem_size(settings.ui_font_size(cx));
        settings.ui_font(cx).clone()
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
        let defaults = ResolvedSettings::default();
        assert!(!MONOSPACE_FAMILY.is_empty() && !defaults.ui_font_family.is_empty());
    }

    #[test]
    fn the_default_monospace_is_a_real_bundled_font() {
        assert!(IBM_PLEX_MONO.starts_with(&[0, 1, 0, 0]));
        assert!(IBM_PLEX_MONO.len() > 100_000);
    }
}
