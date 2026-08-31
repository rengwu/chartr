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
    pub unsafe_filesystem: Option<bool>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginSettings {
    pub enabled: bool,
    pub unsafe_filesystem: bool,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self { enabled: true, unsafe_filesystem: false }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct GeneralContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate_sessions_on_exit: Option<bool>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct AppearanceContent {
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

    pub fn content(&self) -> &SettingsContent {
        &self.content
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

/// Register Chartr's named semantic theme pair, then select the resolved
/// fixed/system variant. Both are ordinary Zed `Theme` values, so every Zed
/// component consumes the same tokens as Chartr's product views.
pub fn init_themes(settings: &ResolvedSettings, cx: &mut gpui::App) {
    let registry = ThemeRegistry::global(cx);
    if let Ok(source) = registry.get("One Dark") {
        let mut dark = (*source).clone();
        dark.id = "chartr_dark".to_owned();
        dark.name = CHARTR_DARK.into();
        let light = chartr_light(&dark);
        registry.insert_themes([dark, light]);
    }
    apply_theme(settings, cx);
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
    let selected = gpui::rgb(0xdce6f5).into();
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
    colors.text = text;
    colors.text_muted = muted;
    colors.text_placeholder = muted;
    colors.text_disabled = muted;
    colors.icon = text;
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

    #[test]
    fn a_missing_file_resolves_to_fixed_chartr_dark() {
        let scratch = tempfile::tempdir().unwrap();
        let store = SettingsStore::load(scratch.path().join(SETTINGS_FILE));
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
    }

    #[test]
    fn an_update_is_atomic_and_survives_relaunch() {
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join(SETTINGS_FILE);
        let mut store = SettingsStore::load(&file);
        store
            .update(|content| {
                content.terminal.get_or_insert_default().font_size = Some(17.);
            })
            .unwrap();
        let relaunched = SettingsStore::load(&file);
        assert_eq!(relaunched.resolved().terminal_font_size, 17.);
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
                        unsafe_filesystem: Some(true),
                        ..PluginSettingsContent::default()
                    },
                );
            })
            .unwrap();
        let relaunched = SettingsStore::load(file);
        let notes = relaunched.resolved().plugin("com.example.notes");
        assert!(!notes.enabled);
        assert!(notes.unsafe_filesystem);
        assert_eq!(
            relaunched.resolved().plugin("com.example.other"),
            PluginSettings::default(),
            "there is no global unsafe grant"
        );
    }
}
