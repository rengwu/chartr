//! zeddy's fonts.
//!
//! Zed's `ui` components read their font and size through a
//! [`ThemeSettingsProvider`], which the `theme_settings` crate normally fills
//! in from the user's settings file. zeddy has no settings file, so it answers
//! the five questions itself — and this is then also the one place the terminal
//! font is chosen, rather than a constant in the renderer.

use gpui::{App, Font, Pixels, px};
use theme::{ThemeSettingsProvider, UiDensity};

/// The families zeddy asks for, and the sizes it draws them at.
pub struct Fonts {
    ui: Font,
    buffer: Font,
}

/// The UI face. GPUI resolves this to the platform's own system font.
const UI_FAMILY: &str = ".SystemUIFont";

/// The monospace face the terminal is drawn in: the one every one of these
/// platforms ships, so it is there without zeddy bundling a font file.
const MONOSPACE_FAMILY: &str = if cfg!(target_os = "macos") {
    "Menlo"
} else if cfg!(target_os = "windows") {
    "Consolas"
} else {
    "DejaVu Sans Mono"
};

impl Default for Fonts {
    fn default() -> Self {
        Self { ui: gpui::font(UI_FAMILY), buffer: gpui::font(MONOSPACE_FAMILY) }
    }
}

impl Fonts {
    /// The terminal's font and the line height to draw it at.
    ///
    /// The ratio is the one every terminal uses and nobody writes down: a line
    /// box about 1.4× the point size, which leaves box-drawing characters
    /// touching and leaves text legible.
    pub fn terminal(&self) -> (Font, Pixels, Pixels) {
        let size = px(13.);
        (self.buffer.clone(), size, (size * 1.4).round())
    }
}

/// Whether the platform can actually rasterise text.
///
/// GPUI answers `all_font_names` with its own hardcoded fallback list even when
/// the platform text system is the one that draws nothing, so "is the list
/// empty" is not the question. The question is whether a family the operating
/// system really ships is in it.
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
        px(14.)
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
}
