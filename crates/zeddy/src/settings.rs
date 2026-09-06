//! Chartr's user-global settings store.
//!
//! Like Zed, the serialized content is sparse and optional while runtime
//! consumers receive a complete resolved value. Like Chartr-rs, a malformed
//! operator-owned file is reported and never overwritten, and successful
//! updates replace the file atomically.

use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
};

use gpui::{BorrowAppContext, Hsla};
use serde::{Deserialize, Serialize};
use theme::{Appearance, GlobalTheme, SystemAppearance, Theme, ThemeRegistry};

pub const SETTINGS_FILE: &str = "settings.toml";
pub const CHARTR_DARK: &str = "Chartr Dark";
pub const CHARTR_LIGHT: &str = "Chartr Light";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsPage {
    #[default]
    General,
    Appearance,
    Terminal,
    Hotkeys,
    Plugins,
}

impl SettingsPage {
    pub const ALL: [Self; 5] =
        [Self::General, Self::Appearance, Self::Terminal, Self::Hotkeys, Self::Plugins];

    pub fn title(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Terminal => "Terminal",
            Self::Hotkeys => "Hotkeys",
            Self::Plugins => "Plugins",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Appearance => "appearance",
            Self::Terminal => "terminal",
            Self::Hotkeys => "hotkeys",
            Self::Plugins => "plugins",
        }
    }
}

const HEADER: &str = "\
# Chartr-zeddy user settings. Omitted fields use Chartr's defaults.
# This namespace is intentionally isolated from previous Chartr installations.
";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    Fixed,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSettings {
    pub terminate_sessions_on_exit: bool,
    pub middle_click_closes_tab: bool,
    pub middle_click_closes_sidebar_tab: bool,
    pub reduce_motion: bool,
    pub theme_mode: ThemeMode,
    pub fixed_theme: String,
    pub light_theme: String,
    pub dark_theme: String,
    pub ui_font_family: String,
    pub ui_font_size: f32,
    pub terminal_font_family: String,
    pub terminal_font_size: f32,
    pub ad_hoc_directory: Option<PathBuf>,
    pub plugins: BTreeMap<String, PluginSettings>,
}

impl Default for ResolvedSettings {
    fn default() -> Self {
        Self {
            terminate_sessions_on_exit: false,
            middle_click_closes_tab: true,
            middle_click_closes_sidebar_tab: true,
            reduce_motion: false,
            theme_mode: ThemeMode::Fixed,
            fixed_theme: CHARTR_DARK.to_owned(),
            light_theme: CHARTR_LIGHT.to_owned(),
            dark_theme: CHARTR_DARK.to_owned(),
            ui_font_family: "IBM Plex Sans".to_owned(),
            ui_font_size: 14.,
            terminal_font_family: "IBM Plex Mono".to_owned(),
            terminal_font_size: 13.,
            ad_hoc_directory: None,
            plugins: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct SettingsContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general: Option<GeneralContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<AppearanceContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<TerminalContent>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub plugins: BTreeMap<String, PluginSettingsContent>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct PluginSettingsContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uninstalled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsafe_filesystem: Option<bool>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginSettings {
    pub enabled: bool,
    pub uninstalled: bool,
    pub unsafe_filesystem: bool,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self { enabled: true, uninstalled: false, unsafe_filesystem: false }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct GeneralContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate_sessions_on_exit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_click_closes_tab: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_click_closes_sidebar_tab: Option<bool>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct AppearanceContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_motion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_mode: Option<ThemeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_font_size: Option<f32>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct TerminalContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_hoc_directory: Option<PathBuf>,
    #[serde(flatten)]
    extra: toml::Table,
}

impl SettingsContent {
    pub fn resolve(&self) -> ResolvedSettings {
        let defaults = ResolvedSettings::default();
        let general = self.general.as_ref();
        let appearance = self.appearance.as_ref();
        let terminal = self.terminal.as_ref();
        ResolvedSettings {
            terminate_sessions_on_exit: general
                .and_then(|content| content.terminate_sessions_on_exit)
                .unwrap_or(defaults.terminate_sessions_on_exit),
            middle_click_closes_tab: general
                .and_then(|content| content.middle_click_closes_tab)
                .unwrap_or(defaults.middle_click_closes_tab),
            middle_click_closes_sidebar_tab: general
                .and_then(|content| content.middle_click_closes_sidebar_tab)
                .unwrap_or(defaults.middle_click_closes_sidebar_tab),
            reduce_motion: appearance
                .and_then(|content| content.reduce_motion)
                .unwrap_or(defaults.reduce_motion),
            theme_mode: appearance
                .and_then(|content| content.theme_mode)
                .unwrap_or(defaults.theme_mode),
            fixed_theme: appearance
                .and_then(|content| content.fixed_theme.clone())
                .unwrap_or(defaults.fixed_theme),
            light_theme: appearance
                .and_then(|content| content.light_theme.clone())
                .unwrap_or(defaults.light_theme),
            dark_theme: appearance
                .and_then(|content| content.dark_theme.clone())
                .unwrap_or(defaults.dark_theme),
            ui_font_family: appearance
                .and_then(|content| content.ui_font_family.clone())
                .unwrap_or(defaults.ui_font_family),
            ui_font_size: appearance
                .and_then(|content| content.ui_font_size)
                .filter(|size| size.is_finite() && *size >= 8. && *size <= 32.)
                .unwrap_or(defaults.ui_font_size),
            terminal_font_family: terminal
                .and_then(|content| content.font_family.clone())
                .unwrap_or(defaults.terminal_font_family),
            terminal_font_size: terminal
                .and_then(|content| content.font_size)
                .filter(|size| size.is_finite() && *size >= 8. && *size <= 72.)
                .unwrap_or(defaults.terminal_font_size),
            ad_hoc_directory: terminal.and_then(|content| content.ad_hoc_directory.clone()),
            plugins: self
                .plugins
                .iter()
                .map(|(id, content)| {
                    (
                        id.clone(),
                        PluginSettings {
                            enabled: content.enabled.unwrap_or(true),
                            uninstalled: content.uninstalled.unwrap_or(false),
                            unsafe_filesystem: content.unsafe_filesystem.unwrap_or(false),
                        },
                    )
                })
                .collect(),
        }
    }
}

impl ResolvedSettings {
    pub fn plugin(&self, id: &str) -> PluginSettings {
        self.plugins.get(id).copied().unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct SettingsStore {
    file: Option<PathBuf>,
    content: SettingsContent,
    resolved: ResolvedSettings,
    unreadable: Option<String>,
}

// Zed keeps settings as application-global state and has every window observe
// that store. Chartr does the same so the dedicated Settings window and every
// workspace always render one authoritative value.
impl gpui::Global for SettingsStore {}

pub fn update_global(
    cx: &mut gpui::App,
    mutate: impl FnOnce(&mut SettingsContent),
) -> Result<ResolvedSettings, io::Error> {
    cx.update_global::<SettingsStore, _>(|store, _| store.update(mutate).cloned())
}

impl SettingsStore {
    pub fn load(file: impl Into<PathBuf>) -> Self {
        let file = file.into();
        let mut content = SettingsContent::default();
        let mut unreadable = None;
        match fs::read_to_string(&file) {
            Ok(text) => match toml::from_str(&text) {
                Ok(read) => content = read,
                Err(error) => {
                    unreadable = Some(format!(
                        "Chartr could not read {}, so defaults are active: {error}",
                        file.display()
                    ))
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                unreadable = Some(format!("Chartr could not read {}: {error}", file.display()))
            }
        }
        let resolved = content.resolve();
        Self { file: Some(file), content, resolved, unreadable }
    }

    pub fn bare() -> Self {
        let content = SettingsContent::default();
        let resolved = content.resolve();
        Self { file: None, content, resolved, unreadable: None }
    }

    pub fn resolved(&self) -> &ResolvedSettings {
        &self.resolved
    }

    pub fn unreadable(&self) -> Option<&str> {
        self.unreadable.as_deref()
    }

    pub fn update(
        &mut self,
        mutate: impl FnOnce(&mut SettingsContent),
    ) -> Result<&ResolvedSettings, io::Error> {
        let mut candidate = self.content.clone();
        mutate(&mut candidate);
        self.save(&candidate)?;
        self.resolved = candidate.resolve();
        self.content = candidate;
        Ok(&self.resolved)
    }

    fn save(&self, content: &SettingsContent) -> io::Result<()> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        if let Some(error) = &self.unreadable {
            return Err(io::Error::other(format!(
                "{error}; Chartr will not overwrite settings it cannot read"
            )));
        }
        let parent = file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let encoded = toml::to_string_pretty(content).map_err(io::Error::other)?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(format!("{HEADER}\n{encoded}").as_bytes())?;
        staged.flush()?;
        staged.as_file().sync_all()?;
        staged.persist(file).map_err(|error| error.error)?;
        Ok(())
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::bare()
    }
}

pub fn settings_file() -> Result<PathBuf, crate::spaces::Error> {
    Ok(crate::spaces::config_root()?.join(SETTINGS_FILE))
}

/// Register Chartr's theme catalog, then select the resolved fixed/system
/// variant. Every entry is an ordinary Zed `Theme`, so Chartr's terminal and
/// every Zed UI component consume one registry and one set of tokens.
pub fn init_themes(settings: &ResolvedSettings, cx: &mut gpui::App) {
    let registry = ThemeRegistry::global(cx);
    let dark_source = registry.get("One Dark").ok();
    if let Some(dark_source) = &dark_source {
        let light_source = registry
            .get("One Light")
            .map(|theme| (*theme).clone())
            .unwrap_or_else(|_| chartr_light(dark_source));
        registry.insert_themes(THEME_PALETTES.map(|palette| {
            let source = if palette.appearance == Appearance::Light {
                &light_source
            } else {
                dark_source.as_ref()
            };
            catalog_theme(source, palette)
        }));
    }
    if let Some(source) = dark_source {
        let dark = chartr_dark(&source);
        let light = chartr_light(&dark);
        registry.insert_themes([dark, light]);
    }
    apply_theme(settings, cx);
}

/// The same operator-facing catalog Chartr-rs exposes. Its palette values are
/// copied from that implementation: Ayu, Gruvbox, and One follow Zed's bundled
/// themes; Catppuccin follows its official semantic palette; VS Code follows
/// the workbench colors. Chartr only adapts those established values into
/// Zed's richer semantic token model.
#[derive(Clone, Copy)]
struct ThemePalette {
    name: &'static str,
    appearance: Appearance,
    surface: u32,
    sidebar: u32,
    border: u32,
    text: u32,
    muted: u32,
    card: u32,
    card_open: u32,
    ring: u32,
    selected: u32,
    hover: u32,
    notice: u32,
    accent: u32,
    done: u32,
    idle: u32,
    quiet: u32,
    terminal_foreground: u32,
}

const THEME_PALETTES: [ThemePalette; 13] = [
    ThemePalette::new(
        "Ayu Dark",
        Appearance::Dark,
        0x0d1016,
        0x1f2127,
        0x3f4043,
        0xbfbdb6,
        0x8a8986,
        0x1f2127,
        0x3e4043,
        0x1b4a6e,
        0x3e4043,
        0x2d2f34,
        0xef7177,
        0x5ac1fe,
        0xaad84c,
        0xfeb454,
        0x696a6a,
        0xbfbdb6,
    ),
    ThemePalette::new(
        "Ayu Light",
        Appearance::Light,
        0xfcfcfc,
        0xececed,
        0xcfd1d2,
        0x5c6166,
        0x8b8e92,
        0xececed,
        0xcfd0d2,
        0xc4daf6,
        0xcfd0d2,
        0xdfe0e1,
        0xef7271,
        0x3b9ee5,
        0x85b304,
        0xf1ad49,
        0xa9acae,
        0x5c6166,
    ),
    ThemePalette::new(
        "Ayu Mirage",
        Appearance::Dark,
        0x242835,
        0x353944,
        0x53565d,
        0xcccac2,
        0x9a9a98,
        0x353944,
        0x53565d,
        0x24556f,
        0x53565d,
        0x43464f,
        0xf18779,
        0x72cffe,
        0xd5fe80,
        0xfecf72,
        0x7b7d7f,
        0xcccac2,
    ),
    ThemePalette::new(
        "Catppuccin Frappé",
        Appearance::Dark,
        0x303446,
        0x292c3c,
        0x51576d,
        0xc6d0f5,
        0xa5adce,
        0x414559,
        0x51576d,
        0xca9ee6,
        0x51576d,
        0x414559,
        0xe78284,
        0xca9ee6,
        0xa6d189,
        0xe5c890,
        0x737994,
        0xc6d0f5,
    ),
    ThemePalette::new(
        "Catppuccin Latte",
        Appearance::Light,
        0xeff1f5,
        0xe6e9ef,
        0xbcc0cc,
        0x4c4f69,
        0x6c6f85,
        0xccd0da,
        0xbcc0cc,
        0x8839ef,
        0xbcc0cc,
        0xccd0da,
        0xd20f39,
        0x8839ef,
        0x40a02b,
        0xdf8e1d,
        0x9ca0b0,
        0x4c4f69,
    ),
    ThemePalette::new(
        "Catppuccin Macchiato",
        Appearance::Dark,
        0x24273a,
        0x1e2030,
        0x494d64,
        0xcad3f5,
        0xa5adcb,
        0x363a4f,
        0x494d64,
        0xc6a0f6,
        0x494d64,
        0x363a4f,
        0xed8796,
        0xc6a0f6,
        0xa6da95,
        0xeed49f,
        0x6e738d,
        0xcad3f5,
    ),
    ThemePalette::new(
        "Catppuccin Mocha",
        Appearance::Dark,
        0x1e1e2e,
        0x181825,
        0x45475a,
        0xcdd6f4,
        0xa6adc8,
        0x313244,
        0x45475a,
        0xcba6f7,
        0x45475a,
        0x313244,
        0xf38ba8,
        0xcba6f7,
        0xa6e3a1,
        0xf9e2af,
        0x6c7086,
        0xcdd6f4,
    ),
    ThemePalette::new(
        "Gruvbox Dark",
        Appearance::Dark,
        0x282828,
        0x3a3735,
        0x5b534d,
        0xfbf1c7,
        0xc5b597,
        0x3a3735,
        0x5b524c,
        0x303a36,
        0x5b524c,
        0x494340,
        0xfb4a35,
        0x83a598,
        0xb7bb26,
        0xf9bd2f,
        0x998b78,
        0xebdbb2,
    ),
    ThemePalette::new(
        "Gruvbox Light",
        Appearance::Light,
        0xfbf1c7,
        0xecddb4,
        0xc8b899,
        0x282828,
        0x5f5650,
        0xecddb4,
        0xc8b899,
        0xab9965,
        0xc8b899,
        0xddcca7,
        0x9d0308,
        0x0b6678,
        0x797410,
        0xb57615,
        0x897b6e,
        0x282828,
    ),
    ThemePalette::new(
        "One Dark",
        Appearance::Dark,
        0x282c33,
        0x2f343e,
        0x464b57,
        0xdce0e5,
        0xa9afbc,
        0x2e343e,
        0x454a56,
        0x47679e,
        0x454a56,
        0x363c46,
        0xd07277,
        0x74ade8,
        0xa1c181,
        0xdec184,
        0x878a98,
        0xabb2bf,
    ),
    ThemePalette::new(
        "One Light",
        Appearance::Light,
        0xfafafa,
        0xebebec,
        0xc9c9ca,
        0x242529,
        0x58585a,
        0xebebec,
        0xcacaca,
        0x7d82e8,
        0xcacaca,
        0xdfdfe0,
        0xd36151,
        0x5c78e2,
        0x669f59,
        0xa48819,
        0x7e8086,
        0x2a2c33,
    ),
    ThemePalette::new(
        "VSCode Dark Modern",
        Appearance::Dark,
        0x1f1f1f,
        0x181818,
        0x2b2b2b,
        0xcccccc,
        0x9d9d9d,
        0x313131,
        0x313131,
        0x0078d4,
        0x313131,
        0x2b2b2b,
        0xf85149,
        0x0078d4,
        0x2ea043,
        0xe2c08d,
        0x6e7681,
        0xcccccc,
    ),
    ThemePalette::new(
        "VSCode Dark Plus",
        Appearance::Dark,
        0x1e1e1e,
        0x252526,
        0x3f3f46,
        0xd4d4d4,
        0x969696,
        0x2d2d30,
        0x37373d,
        0x007acc,
        0x37373d,
        0x2a2d2e,
        0xf44747,
        0x007acc,
        0x6a9955,
        0xdcdcaa,
        0x707070,
        0xd4d4d4,
    ),
];

/// Sidebar-only colors whose layering is too specific to borrow safely from
/// Zed's general element tokens. These values are deliberately explicit: this
/// table is the one hand-tuning point for every theme Chartr exposes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarThemeColors {
    pub card_inactive: Hsla,
    pub card_active: Hsla,
    pub session_hover: Hsla,
    pub session_active: Hsla,
}

#[derive(Clone, Copy)]
struct SidebarThemePalette {
    name: &'static str,
    card_inactive: u32,
    card_active: u32,
    session_hover: u32,
    session_active: u32,
}

impl SidebarThemePalette {
    const fn new(
        name: &'static str,
        card_inactive: u32,
        card_active: u32,
        session_hover: u32,
        session_active: u32,
    ) -> Self {
        Self { name, card_inactive, card_active, session_hover, session_active }
    }

    fn colors(self) -> SidebarThemeColors {
        let color = |value| gpui::rgb(value).into();
        SidebarThemeColors {
            card_inactive: color(self.card_inactive),
            card_active: color(self.card_active),
            session_hover: color(self.session_hover),
            session_active: color(self.session_active),
        }
    }
}

//                                          card       card       session    session
// Theme                                    inactive   active     hover      active
const SIDEBAR_THEME_PALETTES: [SidebarThemePalette; 15] = [
    SidebarThemePalette::new("Ayu Dark", 0x23252a, 0x26282e, 0x27292f, 0x2d2f34),
    SidebarThemePalette::new("Ayu Light", 0xe9e9ea, 0xe6e6e7, 0xe4e5e6, 0xdfe0e1),
    SidebarThemePalette::new("Ayu Mirage", 0x393c47, 0x3c404a, 0x3d414b, 0x43464f),
    SidebarThemePalette::new("Catppuccin Frappé", 0x2f3243, 0x35394b, 0x373b4d, 0x414559),
    SidebarThemePalette::new("Catppuccin Latte", 0xe0e3ea, 0xd9dde5, 0xd6dae2, 0xccd0da),
    SidebarThemePalette::new("Catppuccin Macchiato", 0x242738, 0x2a2d40, 0x2c3043, 0x363a4f),
    SidebarThemePalette::new("Catppuccin Mocha", 0x1e1f2d, 0x252535, 0x272838, 0x313244),
    SidebarThemePalette::new("Gruvbox Dark", 0x3e3a38, 0x423d3b, 0x433e3c, 0x494340),
    SidebarThemePalette::new("Gruvbox Light", 0xf0e6c9, 0xf0e6c9, 0xe3d3ac, 0xddcca7),
    SidebarThemePalette::new("One Dark", 0x313640, 0x333842, 0x333943, 0x363c46),
    SidebarThemePalette::new("One Light", 0xe8e8e9, 0xe5e5e6, 0xe4e4e5, 0xdfdfe0),
    SidebarThemePalette::new("VSCode Dark Modern", 0x1d1d1d, 0x222222, 0x232323, 0x2b2b2b),
    SidebarThemePalette::new("VSCode Dark Plus", 0x262728, 0x28292a, 0x282a2b, 0x2a2d2e),
    SidebarThemePalette::new(CHARTR_DARK, 0x313640, 0x333842, 0x333943, 0x363c46),
    SidebarThemePalette::new(CHARTR_LIGHT, 0xf9fafb, 0xf4f5f7, 0xf1f3f5, 0xe8ebef),
];

pub fn sidebar_theme_colors(theme: &Theme) -> SidebarThemeColors {
    SIDEBAR_THEME_PALETTES
        .iter()
        .find(|palette| palette.name == theme.name.as_ref())
        .copied()
        .map(SidebarThemePalette::colors)
        .unwrap_or_else(|| {
            let colors = &theme.styles.colors;
            SidebarThemeColors {
                card_inactive: colors.element_background,
                card_active: colors.element_active,
                session_hover: colors.ghost_element_hover,
                session_active: colors.ghost_element_selected,
            }
        })
}

impl ThemePalette {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        name: &'static str,
        appearance: Appearance,
        surface: u32,
        sidebar: u32,
        border: u32,
        text: u32,
        muted: u32,
        card: u32,
        card_open: u32,
        ring: u32,
        selected: u32,
        hover: u32,
        notice: u32,
        accent: u32,
        done: u32,
        idle: u32,
        quiet: u32,
        terminal_foreground: u32,
    ) -> Self {
        Self {
            name,
            appearance,
            surface,
            sidebar,
            border,
            text,
            muted,
            card,
            card_open,
            ring,
            selected,
            hover,
            notice,
            accent,
            done,
            idle,
            quiet,
            terminal_foreground,
        }
    }
}

fn catalog_theme(source: &Theme, palette: ThemePalette) -> Theme {
    let mut theme = source.clone();
    theme.id =
        format!("chartr_catalog_{}", palette.name.to_ascii_lowercase().replace([' ', 'é'], "_"));
    theme.name = palette.name.into();
    theme.appearance = palette.appearance;

    let color = |value| gpui::rgb(value).into();
    let surface = color(palette.surface);
    let sidebar = color(palette.sidebar);
    let border = color(palette.border);
    let text = color(palette.text);
    let muted = color(palette.muted);
    let card = color(palette.card);
    let card_open = color(palette.card_open);
    let ring = color(palette.ring);
    let selected = color(palette.selected);
    let hover = color(palette.hover);
    let notice = color(palette.notice);
    let accent = color(palette.accent);
    let done = color(palette.done);
    let idle = color(palette.idle);
    let quiet = color(palette.quiet);
    let terminal_foreground = color(palette.terminal_foreground);

    let colors = &mut theme.styles.colors;
    colors.background = surface;
    colors.surface_background = sidebar;
    // Context menus use `ghost_element_hover` for their rows. Several palettes
    // intentionally give cards and hovered rows the same color, so using
    // `card` here makes pointer hover invisible. The sidebar surface keeps the
    // menu distinct from both its hover/selected rows and its outline.
    colors.elevated_surface_background = sidebar;
    colors.element_background = card;
    colors.element_hover = hover;
    colors.element_active = card_open;
    colors.element_selected = card_open;
    colors.element_selection_background = selected;
    colors.ghost_element_hover = hover;
    colors.ghost_element_active = card_open;
    colors.ghost_element_selected = card_open;
    colors.drop_target_background = selected;
    colors.drop_target_border = ring;
    colors.border = border;
    colors.border_variant = border;
    colors.border_focused = ring;
    colors.border_selected = ring;
    colors.text = text;
    colors.text_muted = muted;
    colors.text_placeholder = quiet;
    colors.text_disabled = quiet;
    // Selected controls should read through surface contrast, not a saturated
    // blue foreground. Semantic accents remain available to links, focus
    // rings, and status colors below.
    colors.text_accent = text;
    colors.icon = text;
    colors.icon_muted = muted;
    colors.icon_placeholder = muted;
    colors.icon_disabled = quiet;
    colors.icon_accent = text;
    colors.title_bar_background = sidebar;
    colors.title_bar_inactive_background = card;
    colors.toolbar_background = sidebar;
    colors.tab_bar_background = card;
    colors.tab_inactive_background = card;
    colors.tab_active_background = surface;
    colors.panel_background = sidebar;
    colors.panel_focused_border = ring;
    colors.panel_indent_guide = border;
    colors.panel_indent_guide_hover = muted;
    colors.panel_indent_guide_active = ring;
    colors.panel_overlay_background = card;
    colors.panel_overlay_hover = hover;
    colors.pane_group_border = border;
    colors.editor_background = surface;
    colors.editor_foreground = text;
    colors.editor_gutter_background = surface;
    colors.editor_subheader_background = card;
    colors.terminal_background = surface;
    colors.terminal_ansi_background = surface;
    colors.terminal_foreground = terminal_foreground;
    colors.terminal_bright_foreground = text;
    colors.terminal_dim_foreground = muted;
    colors.link_text_hover = accent;
    colors.version_control_added = done;
    colors.version_control_deleted = notice;
    colors.version_control_modified = idle;

    let status = &mut theme.styles.status;
    status.error = notice;
    status.error_border = notice;
    status.warning = idle;
    status.warning_border = idle;
    status.success = done;
    status.success_border = done;
    status.info = accent;
    status.info_border = accent;
    status.hidden = quiet;
    status.ignored = quiet;
    theme
}

fn chartr_dark(source: &Theme) -> Theme {
    let mut dark = source.clone();
    dark.id = "chartr_dark".to_owned();
    dark.name = CHARTR_DARK.into();
    dark.appearance = Appearance::Dark;

    let colors = &mut dark.styles.colors;
    let border = gpui::rgb(0x505866).into();
    let border_variant = gpui::rgb(0x414956).into();
    colors.text_accent = colors.text;
    colors.icon_accent = colors.icon;
    colors.border = border;
    colors.border_variant = border_variant;
    colors.pane_group_border = border;
    colors.panel_indent_guide = border_variant;
    colors.scrollbar_track_border = border_variant;
    dark
}

fn chartr_light(dark: &Theme) -> Theme {
    let mut light = dark.clone();
    light.id = "chartr_light".to_owned();
    light.name = CHARTR_LIGHT.into();
    light.appearance = Appearance::Light;
    let colors = &mut light.styles.colors;
    let canvas = gpui::rgb(0xf7f8fa).into();
    let surface = gpui::rgb(0xffffff).into();
    let raised = gpui::rgb(0xf1f3f5).into();
    let hover = gpui::rgb(0xe8ebef).into();
    let selected = gpui::rgb(0xdfe3e8).into();
    let border = gpui::rgb(0xd4d8de).into();
    let text = gpui::rgb(0x24272d).into();
    let muted = gpui::rgb(0x66707d).into();

    colors.background = canvas;
    colors.surface_background = surface;
    colors.elevated_surface_background = surface;
    colors.element_background = raised;
    colors.element_hover = hover;
    colors.element_active = selected;
    colors.element_selected = selected;
    colors.ghost_element_hover = hover;
    colors.ghost_element_active = selected;
    colors.ghost_element_selected = selected;
    colors.border = border;
    colors.border_variant = border;
    colors.pane_group_border = border;
    colors.panel_indent_guide = border;
    colors.scrollbar_track_border = border;
    colors.text = text;
    colors.text_accent = text;
    colors.text_muted = muted;
    colors.text_placeholder = muted;
    colors.text_disabled = muted;
    colors.icon = text;
    colors.icon_accent = text;
    colors.icon_muted = muted;
    colors.icon_placeholder = muted;
    colors.icon_disabled = muted;
    colors.title_bar_background = canvas;
    colors.title_bar_inactive_background = raised;
    colors.toolbar_background = surface;
    colors.tab_bar_background = raised;
    colors.tab_inactive_background = raised;
    colors.tab_active_background = surface;
    colors.panel_background = surface;
    colors.editor_background = surface;
    colors.editor_foreground = text;
    colors.editor_gutter_background = surface;
    colors.editor_subheader_background = raised;
    colors.terminal_background = surface;
    colors.terminal_ansi_background = surface;
    colors.terminal_foreground = text;
    colors.terminal_bright_foreground = text;
    colors.terminal_dim_foreground = muted;
    light
}

pub fn apply_theme(settings: &ResolvedSettings, cx: &mut gpui::App) {
    let name = match settings.theme_mode {
        ThemeMode::Fixed => settings.fixed_theme.as_str(),
        ThemeMode::System => match *SystemAppearance::global(cx) {
            Appearance::Light => settings.light_theme.as_str(),
            Appearance::Dark => settings.dark_theme.as_str(),
        },
    };
    if let Ok(theme) = ThemeRegistry::global(cx).get(name) {
        GlobalTheme::update_theme(cx, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn the_chartr_rs_theme_catalog_is_registered_as_zed_themes(cx: &mut TestAppContext) {
        cx.update(|cx| {
            theme::init(theme::LoadThemes::JustBase, cx);
            init_themes(&ResolvedSettings::default(), cx);
            let registry = ThemeRegistry::global(cx);

            for palette in THEME_PALETTES {
                let registered = registry.get(palette.name).unwrap();
                assert_eq!(registered.appearance, palette.appearance);
                assert_ne!(
                    registered.styles.colors.elevated_surface_background,
                    registered.styles.colors.border_variant,
                    "{} must retain a visible elevated-surface border",
                    palette.name,
                );
                assert_ne!(
                    registered.styles.colors.elevated_surface_background,
                    registered.styles.colors.ghost_element_hover,
                    "{} must retain a visible context-menu hover state",
                    palette.name,
                );
                assert_eq!(
                    registered.styles.colors.text_placeholder,
                    gpui::rgb(palette.quiet).into(),
                    "{} must render placeholders with the lower-emphasis quiet tone",
                    palette.name,
                );
            }
            for palette in SIDEBAR_THEME_PALETTES {
                let registered = registry.get(palette.name).unwrap();
                let colors = sidebar_theme_colors(&registered);
                assert_eq!(colors, palette.colors());
                assert_ne!(
                    colors.card_active, colors.session_active,
                    "{} needs a visible selected session inside an active card",
                    palette.name,
                );
            }
            assert_eq!(registry.get(CHARTR_DARK).unwrap().appearance, Appearance::Dark);
            assert_eq!(registry.get(CHARTR_LIGHT).unwrap().appearance, Appearance::Light);
        });
    }

    #[gpui::test]
    fn chartr_dark_keeps_structural_borders_clear_of_its_surfaces(cx: &mut TestAppContext) {
        cx.update(|cx| {
            theme::init(theme::LoadThemes::JustBase, cx);
            init_themes(&ResolvedSettings::default(), cx);
            let theme = ThemeRegistry::global(cx).get(CHARTR_DARK).unwrap();
            let colors = &theme.styles.colors;

            assert!((colors.border.l - colors.editor_background.l).abs() >= 0.15);
            assert!((colors.border_variant.l - colors.elevated_surface_background.l).abs() >= 0.08);
            assert_eq!(colors.pane_group_border, colors.border);
        });
    }

    #[gpui::test]
    fn active_control_foregrounds_are_neutral_across_the_theme_catalog(cx: &mut TestAppContext) {
        cx.update(|cx| {
            theme::init(theme::LoadThemes::JustBase, cx);
            init_themes(&ResolvedSettings::default(), cx);
            let registry = ThemeRegistry::global(cx);

            for name in THEME_PALETTES
                .map(|palette| palette.name)
                .into_iter()
                .chain([CHARTR_DARK, CHARTR_LIGHT])
            {
                let theme = registry.get(name).unwrap();
                let colors = &theme.styles.colors;
                assert_eq!(colors.text_accent, colors.text, "{name} has tinted active text");
                assert_eq!(colors.icon_accent, colors.icon, "{name} has tinted active icons");
            }
        });
    }

    #[test]
    fn a_missing_file_resolves_to_shipped_defaults() {
        let scratch = tempfile::tempdir().unwrap();
        let store = SettingsStore::load(scratch.path().join(SETTINGS_FILE));
        assert!(!store.resolved().terminate_sessions_on_exit);
        assert!(store.resolved().middle_click_closes_tab);
        assert!(store.resolved().middle_click_closes_sidebar_tab);
        assert_eq!(store.resolved().theme_mode, ThemeMode::Fixed);
        assert_eq!(store.resolved().fixed_theme, CHARTR_DARK);
        assert!(!scratch.path().join(SETTINGS_FILE).exists());
    }

    #[test]
    fn sparse_settings_merge_with_complete_defaults() {
        let content: SettingsContent = toml::from_str(
            "[appearance]\nui_font_size = 16\n[terminal]\nfont_family = 'Monaspace Neon'\n",
        )
        .unwrap();
        let resolved = content.resolve();
        assert_eq!(resolved.ui_font_size, 16.);
        assert_eq!(resolved.ui_font_family, "IBM Plex Sans");
        assert_eq!(resolved.terminal_font_family, "Monaspace Neon");
        assert_eq!(resolved.fixed_theme, CHARTR_DARK);
        assert!(!resolved.reduce_motion);
        assert!(resolved.middle_click_closes_tab);
        assert!(resolved.middle_click_closes_sidebar_tab);
    }

    #[test]
    fn an_update_is_atomic_and_survives_relaunch() {
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join(SETTINGS_FILE);
        let mut store = SettingsStore::load(&file);
        store
            .update(|content| {
                content.terminal.get_or_insert_default().font_size = Some(17.);
                content.appearance.get_or_insert_default().reduce_motion = Some(true);
                let general = content.general.get_or_insert_default();
                general.middle_click_closes_tab = Some(true);
                general.middle_click_closes_sidebar_tab = Some(true);
            })
            .unwrap();
        let relaunched = SettingsStore::load(&file);
        assert_eq!(relaunched.resolved().terminal_font_size, 17.);
        assert!(relaunched.resolved().reduce_motion);
        assert!(relaunched.resolved().middle_click_closes_tab);
        assert!(relaunched.resolved().middle_click_closes_sidebar_tab);
        assert!(fs::read_to_string(file).unwrap().starts_with("# Chartr-zeddy"));
    }

    #[test]
    fn malformed_operator_settings_are_never_overwritten() {
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join(SETTINGS_FILE);
        fs::write(&file, "[[[ not toml\n").unwrap();
        let mut store = SettingsStore::load(&file);
        assert!(store.unreadable().is_some());
        assert!(store.update(|_| {}).is_err());
        assert_eq!(fs::read_to_string(file).unwrap(), "[[[ not toml\n");
    }

    #[test]
    fn unknown_keys_survive_known_updates() {
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join(SETTINGS_FILE);
        fs::write(&file, "future_root = 'kept'\n[appearance]\nfuture_color = 'also kept'\n")
            .unwrap();
        let mut store = SettingsStore::load(&file);
        store
            .update(|content| {
                content.appearance.get_or_insert_default().ui_font_size = Some(15.);
            })
            .unwrap();
        let written = fs::read_to_string(file).unwrap();
        assert!(written.contains("future_root"));
        assert!(written.contains("future_color"));
    }

    #[test]
    fn invalid_font_sizes_fall_back_without_destroying_user_content() {
        let content: SettingsContent =
            toml::from_str("[appearance]\nui_font_size = 2\n[terminal]\nfont_size = 1000\n")
                .unwrap();
        assert_eq!(content.resolve().ui_font_size, 14.);
        assert_eq!(content.resolve().terminal_font_size, 13.);
        assert_eq!(content.appearance.unwrap().ui_font_size, Some(2.));
    }

    #[test]
    fn plugin_grants_are_per_plugin_and_survive_relaunch() {
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join(SETTINGS_FILE);
        let mut store = SettingsStore::load(&file);
        store
            .update(|content| {
                content.plugins.insert(
                    "com.example.notes".to_owned(),
                    PluginSettingsContent {
                        enabled: Some(false),
                        uninstalled: Some(true),
                        unsafe_filesystem: Some(true),
                        ..PluginSettingsContent::default()
                    },
                );
            })
            .unwrap();
        let relaunched = SettingsStore::load(file);
        let notes = relaunched.resolved().plugin("com.example.notes");
        assert!(!notes.enabled);
        assert!(notes.uninstalled);
        assert!(notes.unsafe_filesystem);
        assert_eq!(
            relaunched.resolved().plugin("com.example.other"),
            PluginSettings::default(),
            "there is no global unsafe grant"
        );
    }
}
