//! zeddy's fonts.
//!
//! Zed's `ui` components read their font and size through a
//! [`ThemeSettingsProvider`], which the `theme_settings` crate normally fills
//! in from the user's settings file. zeddy has no settings file, so it answers
//! the five questions itself — and this is then also the one place the terminal
//! font is chosen, rather than a constant in the renderer.

use std::borrow::Cow;

use gpui::{App, Font, Pixels, Rems, Window, px};
use settings::Settings as _;
use terminal::terminal_settings::TerminalSettings;
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

/// A terminal picker choice and its embedded faces. Lilex is loaded by Zed's assets.
pub struct TerminalFont {
    pub family: &'static str,
    faces: &'static [&'static [u8]],
}

/// Keep the terminal picker and bundled font registration in one catalog.
pub const TERMINAL_FONTS: &[TerminalFont] = &[
    TerminalFont {
        family: "IBM Plex Mono",
        faces: &[include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf")],
    },
    TerminalFont {
        family: "Lilex",
        faces: &[], // Supplied by the pinned Zed asset bundle.
    },
    TerminalFont {
        family: "Anonymous Pro",
        faces: &[
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-Bold.ttf"),
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-Italic.ttf"),
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Cousine",
        faces: &[
            include_bytes!("../assets/fonts/cousine/Cousine-Bold.ttf"),
            include_bytes!("../assets/fonts/cousine/Cousine-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/cousine/Cousine-Italic.ttf"),
            include_bytes!("../assets/fonts/cousine/Cousine-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Cutive Mono",
        faces: &[include_bytes!("../assets/fonts/cutivemono/CutiveMono-Regular.ttf")],
    },
    TerminalFont {
        family: "DM Mono",
        faces: &[
            include_bytes!("../assets/fonts/dmmono/DMMono-Italic.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-Light.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-LightItalic.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-Medium.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-MediumItalic.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Fira Code",
        faces: &[
            include_bytes!("../assets/fonts/firacode/FiraCode-Bold.ttf"),
            include_bytes!("../assets/fonts/firacode/FiraCode-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Inconsolata",
        faces: &[
            include_bytes!("../assets/fonts/inconsolata/Inconsolata-Bold.ttf"),
            include_bytes!("../assets/fonts/inconsolata/Inconsolata-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "JetBrains Mono",
        faces: &[
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-Bold.ttf"),
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-Italic.ttf"),
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "PT Mono",
        faces: &[include_bytes!("../assets/fonts/ptmono/PTM55FT.ttf")],
    },
    TerminalFont {
        family: "Red Hat Mono",
        faces: &[
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-Bold.ttf"),
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-Italic.ttf"),
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Roboto Mono",
        faces: &[
            include_bytes!("../assets/fonts/robotomono/RobotoMono-Bold.ttf"),
            include_bytes!("../assets/fonts/robotomono/RobotoMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/robotomono/RobotoMono-Italic.ttf"),
            include_bytes!("../assets/fonts/robotomono/RobotoMono-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Source Code Pro",
        faces: &[
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-Bold.ttf"),
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-BoldIt.ttf"),
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-It.ttf"),
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-Regular.ttf"),
        ],
    },
    TerminalFont {
        family: "Space Mono",
        faces: &[
            include_bytes!("../assets/fonts/spacemono/SpaceMono-Bold.ttf"),
            include_bytes!("../assets/fonts/spacemono/SpaceMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/spacemono/SpaceMono-Italic.ttf"),
            include_bytes!("../assets/fonts/spacemono/SpaceMono-Regular.ttf"),
        ],
    },
];

pub fn load_bundled(cx: &App) -> anyhow::Result<()> {
    cx.text_system().add_fonts(
        TERMINAL_FONTS
            .iter()
            .flat_map(|font| font.faces.iter().map(|face| Cow::Borrowed(*face)))
            .collect(),
    )
}

/// Install Chartr's resolved typography at the two native Zed settings
/// boundaries that consume it. UI components use `ThemeSettingsProvider`,
/// while a standalone `TerminalElement` deliberately gives the terminal's own
/// font override precedence. Keeping both in sync lets TerminalView perform its
/// normal relayout and PTY resize when typography changes.
pub fn install(settings: &ResolvedSettings, cx: &mut App) {
    theme::set_theme_settings_provider(Box::new(Fonts::from_settings(settings)), cx);

    if let Some(mut terminal_settings) = TerminalSettings::try_get(cx).cloned() {
        terminal_settings.font_family = Some(settings.terminal_font_family.clone().into());
        terminal_settings.font_size = Some(px(settings.terminal_font_size));
        TerminalSettings::override_global(terminal_settings, cx);
    }

    cx.refresh_windows();
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
        self.buffer_size
    }

    fn ui_density(&self, _: &App) -> UiDensity {
        UiDensity::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn installs_terminal_typography_in_zeds_native_settings(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            let mut settings = ResolvedSettings::default();
            settings.terminal_font_family = "IBM Plex Mono".to_owned();
            settings.terminal_font_size = 19.;

            install(&settings, cx);

            let native = TerminalSettings::get_global(cx);
            assert_eq!(native.font_size, Some(px(19.)));
            assert_eq!(native.font_family.as_ref().map(AsRef::as_ref), Some("IBM Plex Mono"));
        });
    }
}
