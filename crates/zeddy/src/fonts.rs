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

/// A picker choice and its embedded faces. Empty faces use Zed's or system assets.
pub struct BundledFont {
    pub family: &'static str,
    faces: &'static [&'static [u8]],
}

impl BundledFont {
    fn native_family(&self) -> &'static str {
        // Google's static optical-size instances retain these internal family
        // names. Keep the public choice stable while requesting the actual face.
        match self.family {
            "DM Sans" => "DM Sans 9pt",
            "Nunito Sans" => "Nunito Sans 12pt ExtraLight 12pt",
            family => family,
        }
    }
}

/// Interface choices share their catalog with bundled font registration.
pub const UI_FONTS: &[BundledFont] = &[
    BundledFont {
        family: "IBM Plex Sans",
        faces: &[], // Supplied by the pinned Zed asset bundle.
    },
    BundledFont {
        family: "System UI",
        faces: &[], // Resolved by the native platform.
    },
    BundledFont {
        family: "Asap",
        faces: &[
            include_bytes!("../assets/fonts/ui/asap/Asap-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/asap/Asap-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/asap/Asap-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/asap/Asap-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Barlow",
        faces: &[
            include_bytes!("../assets/fonts/ui/barlow/Barlow-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/barlow/Barlow-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/barlow/Barlow-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/barlow/Barlow-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Cabin",
        faces: &[
            include_bytes!("../assets/fonts/ui/cabin/Cabin-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/cabin/Cabin-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/cabin/Cabin-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/cabin/Cabin-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Comic Neue",
        faces: &[
            include_bytes!("../assets/fonts/ui/comicneue/ComicNeue-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/comicneue/ComicNeue-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/comicneue/ComicNeue-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/comicneue/ComicNeue-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "DM Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/dmsans/DMSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/dmsans/DMSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/dmsans/DMSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/dmsans/DMSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Fira Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/firasans/FiraSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/firasans/FiraSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/firasans/FiraSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/firasans/FiraSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Geist",
        faces: &[
            include_bytes!("../assets/fonts/ui/geist/Geist-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/geist/Geist-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Hind",
        faces: &[
            include_bytes!("../assets/fonts/ui/hind/Hind-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/hind/Hind-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Inter",
        faces: &[
            include_bytes!("../assets/fonts/ui/inter/Inter-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/inter/Inter-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/inter/Inter-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/inter/Inter-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Jim Nightshade",
        faces: &[include_bytes!("../assets/fonts/ui/jimnightshade/JimNightshade-Regular.ttf")],
    },
    BundledFont {
        family: "Karla",
        faces: &[
            include_bytes!("../assets/fonts/ui/karla/Karla-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/karla/Karla-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/karla/Karla-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/karla/Karla-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Lato",
        faces: &[
            include_bytes!("../assets/fonts/ui/lato/Lato-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/lato/Lato-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/lato/Lato-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/lato/Lato-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Merriweather Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/merriweathersans/MerriweatherSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/merriweathersans/MerriweatherSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/merriweathersans/MerriweatherSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/merriweathersans/MerriweatherSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Noto Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/notosans/NotoSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/notosans/NotoSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/notosans/NotoSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/notosans/NotoSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Nunito Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/nunitosans/NunitoSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/nunitosans/NunitoSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/nunitosans/NunitoSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/nunitosans/NunitoSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Open Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/opensans/OpenSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/opensans/OpenSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/opensans/OpenSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/opensans/OpenSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Oxygen",
        faces: &[
            include_bytes!("../assets/fonts/ui/oxygen/Oxygen-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/oxygen/Oxygen-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "PT Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/ptsans/PTSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/ptsans/PTSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/ptsans/PTSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/ptsans/PTSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "PT Serif",
        faces: &[
            include_bytes!("../assets/fonts/ui/ptserif/PTSerif-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/ptserif/PTSerif-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/ptserif/PTSerif-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/ptserif/PTSerif-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Playpen Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/playpensans/PlaypenSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/playpensans/PlaypenSans-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Poppins",
        faces: &[
            include_bytes!("../assets/fonts/ui/poppins/Poppins-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/poppins/Poppins-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/poppins/Poppins-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/poppins/Poppins-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Roboto",
        faces: &[
            include_bytes!("../assets/fonts/ui/roboto/Roboto-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/roboto/Roboto-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/roboto/Roboto-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/roboto/Roboto-Bold.ttf"),
        ],
    },
    BundledFont {
        family: "Work Sans",
        faces: &[
            include_bytes!("../assets/fonts/ui/worksans/WorkSans-Italic.ttf"),
            include_bytes!("../assets/fonts/ui/worksans/WorkSans-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/ui/worksans/WorkSans-Regular.ttf"),
            include_bytes!("../assets/fonts/ui/worksans/WorkSans-Bold.ttf"),
        ],
    },
];

/// Keep the terminal picker and bundled font registration in one catalog.
pub const TERMINAL_FONTS: &[BundledFont] = &[
    BundledFont {
        family: "IBM Plex Mono",
        faces: &[include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf")],
    },
    BundledFont {
        family: "Lilex",
        faces: &[], // Supplied by the pinned Zed asset bundle.
    },
    BundledFont {
        family: "Anonymous Pro",
        faces: &[
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-Bold.ttf"),
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-Italic.ttf"),
            include_bytes!("../assets/fonts/anonymouspro/AnonymousPro-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "Cousine",
        faces: &[
            include_bytes!("../assets/fonts/cousine/Cousine-Bold.ttf"),
            include_bytes!("../assets/fonts/cousine/Cousine-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/cousine/Cousine-Italic.ttf"),
            include_bytes!("../assets/fonts/cousine/Cousine-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "Cutive Mono",
        faces: &[include_bytes!("../assets/fonts/cutivemono/CutiveMono-Regular.ttf")],
    },
    BundledFont {
        family: "DM Mono",
        faces: &[
            include_bytes!("../assets/fonts/dmmono/DMMono-Italic.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-Medium.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-MediumItalic.ttf"),
            include_bytes!("../assets/fonts/dmmono/DMMono-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "Fira Code",
        faces: &[
            include_bytes!("../assets/fonts/firacode/FiraCode-Bold.ttf"),
            include_bytes!("../assets/fonts/firacode/FiraCode-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "Inconsolata",
        faces: &[
            include_bytes!("../assets/fonts/inconsolata/Inconsolata-Bold.ttf"),
            include_bytes!("../assets/fonts/inconsolata/Inconsolata-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "JetBrains Mono",
        faces: &[
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-Bold.ttf"),
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-Italic.ttf"),
            include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "PT Mono",
        faces: &[include_bytes!("../assets/fonts/ptmono/PTM55FT.ttf")],
    },
    BundledFont {
        family: "Red Hat Mono",
        faces: &[
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-Bold.ttf"),
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-Italic.ttf"),
            include_bytes!("../assets/fonts/redhatmono/RedHatMono-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "Roboto Mono",
        faces: &[
            include_bytes!("../assets/fonts/robotomono/RobotoMono-Bold.ttf"),
            include_bytes!("../assets/fonts/robotomono/RobotoMono-BoldItalic.ttf"),
            include_bytes!("../assets/fonts/robotomono/RobotoMono-Italic.ttf"),
            include_bytes!("../assets/fonts/robotomono/RobotoMono-Regular.ttf"),
        ],
    },
    BundledFont {
        family: "Source Code Pro",
        faces: &[
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-Bold.ttf"),
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-BoldIt.ttf"),
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-It.ttf"),
            include_bytes!("../assets/fonts/sourcecodepro/SourceCodePro-Regular.ttf"),
        ],
    },
    BundledFont {
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
        UI_FONTS
            .iter()
            .chain(TERMINAL_FONTS)
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
        let ui_family =
            UI_FONTS.iter().find(|font| font.family == settings.ui_font_family).map_or_else(
                || settings.ui_font_family.clone(),
                |font| font.native_family().to_owned(),
            );
        Self {
            ui: gpui::font(ui_family),
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
