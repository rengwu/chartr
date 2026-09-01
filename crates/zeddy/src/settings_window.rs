//! Chartr's singleton, application-wide Settings window.
//!
//! This follows Zed's `SettingsWindow` boundary: opening Settings focuses the
//! existing app-wide window, global settings notify every workspace live, and
//! the originating workspace is retained only for operations that truly need
//! runtime state (the backend and plugin catalog).

use gpui::{
    Anchor, AnyView, App, Bounds, ClickEvent, Context, DefiniteLength, ElementId, Entity,
    FocusHandle, Focusable, FontWeight, Hsla, KeyBinding, PathPromptOptions, Render, Role,
    SharedString, TextAlign, WeakEntity, Window, WindowBounds, WindowHandle, WindowOptions,
    actions, px, size,
};
use ui::{
    Banner, Button, ButtonSize, ColumnWidthConfig, DropdownMenu, DropdownStyle, Icon, PopoverMenu,
    RedistributableColumnsState, Severity, Switch, Table, TableResizeBehavior, Tooltip, prelude::*,
};

use crate::{
    app::Zeddy,
    components::{
        ContextMenu, SegmentedControl, SegmentedControlOption, selection_list, selection_row,
    },
    fonts::{Fonts, UI_LABEL_DEFAULT, UI_LABEL_LARGE, UI_LABEL_SMALL, UI_TEXT_DEFAULT},
    keymap::{KeymapAction, KeymapStore},
    mode::Mode,
    persistence::SidebarScope,
    settings::{
        self, AppearanceContent, GeneralContent, ResolvedSettings, SettingsPage, SettingsStore,
        TerminalContent, ThemeMode,
    },
    text_input::{InputEvent, TextInput},
};

actions!(settings_window, [Close]);

const SETTINGS_WINDOW_MIN_WIDTH: f32 = 720.;
const SETTINGS_CONTROL_COLUMN_WIDTH: f32 = 200.;
const SETTINGS_FIELD_VERTICAL_PADDING: f32 = 16.;

#[derive(Clone, Copy)]
enum ThemeTarget {
    Fixed,
    Light,
    Dark,
}

pub fn init(keymap: &KeymapStore, cx: &mut App) {
    #[cfg(target_os = "macos")]
    cx.bind_keys([KeyBinding::new("cmd-w", Close, Some("ChartrSettings"))]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-w", Close, Some("ChartrSettings"))]);

    cx.bind_keys([KeyBinding::new(
        keymap.key(KeymapAction::OpenSettings),
        crate::actions::settings::Open,
        Some("ChartrSettings"),
    )]);
}

/// Focus Zed-style: one Settings window for the application, never one per
/// workspace. Reopening also retargets workspace-scoped controls to the most
/// recent caller.
pub fn open(original_window: WindowHandle<Zeddy>, original: WeakEntity<Zeddy>, cx: &mut App) {
    open_with_origin(Some(original_window), original, cx);
}

fn open_with_origin(
    original_window: Option<WindowHandle<Zeddy>>,
    original: WeakEntity<Zeddy>,
    cx: &mut App,
) {
    let existing = cx.windows().into_iter().find_map(|window| window.downcast::<SettingsWindow>());
    if let Some(existing) = existing {
        existing
            .update(cx, |settings, window, cx| {
                settings.original_window = original_window;
                settings.original = original;
                window.activate_window();
                cx.notify();
            })
            .ok();
        return;
    }

    // Like Zed, defer creation so the originating workspace action is off the
    // stack before GPUI installs another root view.
    cx.defer(move |cx| {
        let bounds = Bounds::centered(None, size(px(900.), px(680.)), cx);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(crate::title_bar::options("Chartr — Settings")),
                app_owns_titlebar_drag: crate::title_bar::app_owns_drag(),
                focus: true,
                show: true,
                is_movable: true,
                kind: gpui::WindowKind::Normal,
                window_background: cx.theme().window_background_appearance(),
                window_min_size: Some(size(px(SETTINGS_WINDOW_MIN_WIDTH), px(420.))),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| SettingsWindow::new(original_window, original, window, cx));
                window.focus(&view.read(cx).focus_handle(cx), cx);
                view
            },
        );
        if let Err(error) = opened {
            eprintln!("Chartr could not open Settings: {error}");
        }
    });
}

pub struct SettingsWindow {
    original_window: Option<WindowHandle<Zeddy>>,
    original: WeakEntity<Zeddy>,
    page: SettingsPage,
    plugin_settings: Option<(String, AnyView)>,
    recording_keymap: Option<KeymapAction>,
    keymap_restart_required: bool,
    ui_font_size_input: Entity<TextInput>,
    terminal_font_size_input: Entity<TextInput>,
    hotkey_widths: Entity<RedistributableColumnsState>,
    title_bar: Entity<crate::title_bar::TitleBar>,
    focus: FocusHandle,
    problem: Option<String>,
}

impl SettingsWindow {
    fn new(
        original_window: Option<WindowHandle<Zeddy>>,
        original: WeakEntity<Zeddy>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let resolved = cx.global::<SettingsStore>().resolved().clone();
        let ui_font_size = format_number(resolved.ui_font_size);
        let ui_font_size_input = cx.new(|cx| {
            let mut input = TextInput::new("Interface font size", cx);
            input.set_text(ui_font_size, false, cx);
            input.set_text_align(TextAlign::Center, cx);
            input
        });
        let terminal_font_size = format_number(resolved.terminal_font_size);
        let terminal_font_size_input = cx.new(|cx| {
            let mut input = TextInput::new("Terminal font size", cx);
            input.set_text(terminal_font_size, false, cx);
            input.set_text_align(TextAlign::Center, cx);
            input
        });
        cx.observe_global_in::<SettingsStore>(window, |this, window, cx| {
            this.sync_font_size_inputs(window, cx);
            cx.notify();
        })
        .detach();
        cx.observe_global_in::<KeymapStore>(window, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&ui_font_size_input, |this, input, _: &InputEvent, cx| {
            if let Ok(value) = input.read(cx).text().parse::<f32>()
                && value.is_finite()
                && (8. ..=32.).contains(&value)
            {
                this.set_ui_font_size(value, cx);
            }
        })
        .detach();
        cx.subscribe(&terminal_font_size_input, |this, input, _: &InputEvent, cx| {
            if let Ok(value) = input.read(cx).text().parse::<f32>()
                && value.is_finite()
                && (8. ..=72.).contains(&value)
            {
                this.set_terminal_font_size(value, cx);
            }
        })
        .detach();
        cx.on_focus_out(&ui_font_size_input.focus_handle(cx), window, |this, _, _, cx| {
            this.commit_ui_font_size_input(cx)
        })
        .detach();
        cx.on_focus_out(&terminal_font_size_input.focus_handle(cx), window, |this, _, _, cx| {
            this.commit_terminal_font_size_input(cx)
        })
        .detach();
        cx.on_window_closed(|cx, _| {
            if let Some(settings) =
                cx.windows().into_iter().find_map(|window| window.downcast::<SettingsWindow>())
                && cx.windows().len() == 1
            {
                cx.update_window(*settings, |_, window, _| window.remove_window()).ok();
            }
        })
        .detach();
        Self {
            original_window,
            original,
            page: SettingsPage::default(),
            plugin_settings: None,
            recording_keymap: None,
            keymap_restart_required: false,
            ui_font_size_input,
            terminal_font_size_input,
            hotkey_widths: cx.new(|_| {
                RedistributableColumnsState::new(
                    2,
                    vec![DefiniteLength::Fraction(0.68), DefiniteLength::Fraction(0.32)],
                    vec![TableResizeBehavior::Resizable, TableResizeBehavior::Resizable],
                )
            }),
            title_bar: cx.new(|_| crate::title_bar::TitleBar::new("settings-title-bar")),
            focus: cx.focus_handle(),
            problem: None,
        }
    }

    fn update_settings(
        &mut self,
        mutate: impl FnOnce(&mut settings::SettingsContent),
        apply_theme: bool,
        apply_fonts: bool,
        cx: &mut Context<Self>,
    ) {
        match settings::update_global(cx, mutate) {
            Ok(resolved) => {
                cx.set_reduce_motion(resolved.reduce_motion);
                if apply_theme {
                    settings::apply_theme(&resolved, cx);
                }
                if apply_fonts {
                    theme::set_theme_settings_provider(
                        Box::new(Fonts::from_settings(&resolved)),
                        cx,
                    );
                }
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    fn set_terminate_on_exit(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.update_settings(
            |content| {
                content
                    .general
                    .get_or_insert_with(GeneralContent::default)
                    .terminate_sessions_on_exit = Some(enabled);
            },
            false,
            false,
            cx,
        );
    }

    fn set_theme_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.update_settings(
            move |content| {
                let appearance = content.appearance.get_or_insert_with(AppearanceContent::default);
                appearance.theme_mode = Some(mode);
            },
            true,
            true,
            cx,
        );
    }

    fn set_reduce_motion(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.update_settings(
            move |content| {
                content.appearance.get_or_insert_with(AppearanceContent::default).reduce_motion =
                    Some(enabled);
            },
            false,
            false,
            cx,
        );
    }

    fn set_theme(&mut self, target: ThemeTarget, theme: String, cx: &mut Context<Self>) {
        self.update_settings(
            move |content| {
                let appearance = content.appearance.get_or_insert_with(AppearanceContent::default);
                match target {
                    ThemeTarget::Fixed => {
                        appearance.theme_mode = Some(ThemeMode::Fixed);
                        appearance.fixed_theme = Some(theme);
                    }
                    ThemeTarget::Light => {
                        appearance.theme_mode = Some(ThemeMode::System);
                        appearance.light_theme = Some(theme);
                    }
                    ThemeTarget::Dark => {
                        appearance.theme_mode = Some(ThemeMode::System);
                        appearance.dark_theme = Some(theme);
                    }
                }
            },
            true,
            true,
            cx,
        );
    }

    fn set_ui_font(&mut self, family: String, cx: &mut Context<Self>) {
        self.update_settings(
            move |content| {
                content.appearance.get_or_insert_with(AppearanceContent::default).ui_font_family =
                    Some(family);
            },
            false,
            true,
            cx,
        );
    }

    fn set_ui_font_size(&mut self, size: f32, cx: &mut Context<Self>) {
        let size = size.clamp(8., 32.);
        if cx.global::<SettingsStore>().resolved().ui_font_size == size {
            return;
        }
        self.update_settings(
            move |content| {
                content.appearance.get_or_insert_with(AppearanceContent::default).ui_font_size =
                    Some(size);
            },
            false,
            true,
            cx,
        );
    }

    fn adjust_ui_font_size(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = cx.global::<SettingsStore>().resolved().ui_font_size;
        self.set_ui_font_size(current + delta, cx);
    }

    fn set_terminal_font(&mut self, family: String, cx: &mut Context<Self>) {
        self.update_settings(
            move |content| {
                content.terminal.get_or_insert_with(TerminalContent::default).font_family =
                    Some(family);
            },
            false,
            false,
            cx,
        );
    }

    fn set_terminal_font_size(&mut self, size: f32, cx: &mut Context<Self>) {
        let size = size.clamp(8., 72.);
        if cx.global::<SettingsStore>().resolved().terminal_font_size == size {
            return;
        }
        self.update_settings(
            move |content| {
                content.terminal.get_or_insert_with(TerminalContent::default).font_size =
                    Some(size);
            },
            false,
            false,
            cx,
        );
    }

    fn adjust_terminal_font_size(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = cx.global::<SettingsStore>().resolved().terminal_font_size;
        self.set_terminal_font_size(current + delta, cx);
    }

    fn commit_ui_font_size_input(&mut self, cx: &mut Context<Self>) {
        let current = self.settings(cx).ui_font_size;
        let value = self
            .ui_font_size_input
            .read(cx)
            .text()
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(8., 32.))
            .unwrap_or(current);
        self.set_ui_font_size(value, cx);
        sync_number_text(&self.ui_font_size_input, value, cx);
    }

    fn commit_terminal_font_size_input(&mut self, cx: &mut Context<Self>) {
        let current = self.settings(cx).terminal_font_size;
        let value = self
            .terminal_font_size_input
            .read(cx)
            .text()
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(8., 72.))
            .unwrap_or(current);
        self.set_terminal_font_size(value, cx);
        sync_number_text(&self.terminal_font_size_input, value, cx);
    }

    fn sync_font_size_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let settings = self.settings(cx);
        if !self.ui_font_size_input.focus_handle(cx).is_focused(window) {
            sync_number_text(&self.ui_font_size_input, settings.ui_font_size, cx);
        }
        if !self.terminal_font_size_input.focus_handle(cx).is_focused(window) {
            sync_number_text(&self.terminal_font_size_input, settings.terminal_font_size, cx);
        }
    }

    fn pick_free_sessions_directory(&mut self, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Use for Free sessions".into()),
        });
        cx.spawn(async move |this, cx| {
            let outcome = chosen.await;
            let _ = this.update(cx, |this, cx| match outcome {
                Ok(Ok(Some(paths))) if !paths.is_empty() => {
                    let path = paths[0].clone();
                    match settings::update_global(cx, |content| {
                        content
                            .terminal
                            .get_or_insert_with(TerminalContent::default)
                            .ad_hoc_directory = Some(path.clone());
                    }) {
                        Ok(_) => {
                            if let Some(origin) = this.original.upgrade() {
                                origin.update(cx, |origin, cx| {
                                    origin.settings_set_free_sessions_directory(path, cx)
                                });
                            }
                            this.problem = None;
                        }
                        Err(error) => this.problem = Some(error.to_string()),
                    }
                    cx.notify();
                }
                Ok(Ok(_)) | Err(_) => {}
                Ok(Err(error)) => {
                    this.problem = Some(error.to_string());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn cycle_page(&mut self, backwards: bool, cx: &mut Context<Self>) {
        let current = SettingsPage::ALL.iter().position(|page| *page == self.page).unwrap_or(0);
        let next = if backwards {
            current.checked_sub(1).unwrap_or(SettingsPage::ALL.len() - 1)
        } else {
            (current + 1) % SettingsPage::ALL.len()
        };
        self.page = SettingsPage::ALL[next];
        self.plugin_settings = None;
        cx.notify();
    }

    fn on_key(&mut self, event: &gpui::KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(action) = self.recording_keymap {
            cx.stop_propagation();
            if event.keystroke.key == "escape" {
                self.recording_keymap = None;
                cx.notify();
                return;
            }
            if matches!(
                event.keystroke.key.as_str(),
                "shift" | "control" | "alt" | "cmd" | "super" | "fn"
            ) {
                return;
            }
            let key = event.keystroke.unparse();
            match cx.update_global::<KeymapStore, _>(|keymap, _| keymap.set(action, key)) {
                Ok(()) => {
                    self.recording_keymap = None;
                    self.keymap_restart_required = true;
                    self.problem = None;
                }
                Err(error) => self.problem = Some(error.to_string()),
            }
            cx.notify();
            return;
        }
        if event.keystroke.modifiers.control && event.keystroke.key == "tab" {
            cx.stop_propagation();
            self.cycle_page(event.keystroke.modifiers.shift, cx);
        }
    }

    fn set_plugin_enabled(&mut self, plugin: String, enabled: bool, cx: &mut Context<Self>) {
        let Some(origin) = self.original.upgrade() else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
            cx.notify();
            return;
        };
        match origin.update(cx, |origin, cx| {
            origin.settings_set_plugin_enabled(plugin.clone(), enabled, cx)
        }) {
            Ok(()) => {
                if !enabled && self.plugin_settings.as_ref().is_some_and(|(id, _)| id == &plugin) {
                    self.plugin_settings = None;
                }
                self.problem = None;
            }
            Err(error) => self.problem = Some(error),
        }
        cx.notify();
    }

    fn set_plugin_unsafe(&mut self, plugin: String, enabled: bool, cx: &mut Context<Self>) {
        let Some(origin) = self.original.upgrade() else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
            cx.notify();
            return;
        };
        match origin
            .update(cx, |origin, cx| origin.settings_set_plugin_unsafe(plugin.clone(), enabled, cx))
        {
            Ok(()) => {
                if self.plugin_settings.as_ref().is_some_and(|(id, _)| id == &plugin) {
                    self.plugin_settings = None;
                }
                self.problem = None;
            }
            Err(error) => self.problem = Some(error),
        }
        cx.notify();
    }

    fn open_plugin_settings(
        &mut self,
        plugin: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(origin) = self.original.upgrade() else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
            cx.notify();
            return;
        };
        let view = origin.update(cx, |origin, cx| origin.settings_plugin_view(&plugin, window, cx));
        if let Some(view) = view {
            self.plugin_settings = Some((plugin, view));
            self.problem = None;
        }
        cx.notify();
    }

    fn backend_label(&self, cx: &App) -> String {
        self.original
            .upgrade()
            .map(|origin| origin.read(cx).settings_backend_label())
            .unwrap_or_else(|| "Workspace unavailable".to_owned())
    }

    fn sidebar_scope(&self, cx: &App) -> Option<SidebarScope> {
        self.original.upgrade().map(|origin| origin.read(cx).settings_sidebar_scope())
    }

    fn mode(&self, cx: &App) -> Option<Mode> {
        self.original.upgrade().map(|origin| origin.read(cx).settings_mode())
    }

    fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if let Some(origin) = self.original.upgrade() {
            origin.update(cx, |origin, cx| origin.settings_set_mode(mode, cx));
            self.problem = None;
        } else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
        }
        cx.notify();
    }

    fn set_sidebar_scope(&mut self, scope: SidebarScope, cx: &mut Context<Self>) {
        if let Some(origin) = self.original.upgrade() {
            origin.update(cx, |origin, cx| origin.settings_set_sidebar_scope(scope, cx));
            self.problem = None;
        } else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
        }
        cx.notify();
    }

    fn retry_backend(&mut self, cx: &mut Context<Self>) {
        if let Some(origin) = self.original.upgrade() {
            origin.update(cx, |origin, cx| origin.settings_retry_backend(cx));
            self.problem = None;
        } else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
        }
        cx.notify();
    }

    fn restart_backend(&mut self, cx: &mut Context<Self>) {
        let Some(origin) = self.original_window else {
            self.problem = Some("The originating Chartr window is no longer available.".into());
            cx.notify();
            return;
        };
        if origin
            .update(cx, |origin, window, cx| origin.settings_restart_backend(window, cx))
            .is_err()
        {
            self.problem = Some("The originating Chartr window is no longer available.".into());
        } else {
            self.problem = None;
        }
        cx.notify();
    }

    fn settings(&self, cx: &App) -> ResolvedSettings {
        cx.global::<SettingsStore>().resolved().clone()
    }

    fn content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if self.page == SettingsPage::Plugins
            && let Some((plugin, view)) = self.plugin_settings.as_ref()
        {
            let back = cx.listener(|this, _, _, cx| {
                this.plugin_settings = None;
                cx.notify();
            });
            return v_flex()
                .gap_3()
                .child(Button::new("plugin-settings-back", "Back to plugins").on_click(back))
                .child(Label::new(plugin.clone()).size(UI_LABEL_SMALL).color(Color::Muted))
                .child(div().min_h(px(320.)).child(view.clone()))
                .into_any_element();
        }
        match self.page {
            SettingsPage::General => self.general_page(cx),
            SettingsPage::Appearance => self.appearance_page(window, cx),
            SettingsPage::Terminal => self.terminal_page(cx),
            SettingsPage::Hotkeys => self.hotkeys_page(cx),
            SettingsPage::Plugins => self.plugins_page(window, cx),
        }
    }

    fn general_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let terminate = self.settings(cx).terminate_sessions_on_exit;
        let mode = self.mode(cx);
        let sidebar_scope = self.sidebar_scope(cx);
        let runtime_available = mode.is_some() && sidebar_scope.is_some();
        let mode = mode.unwrap_or_default();
        let sidebar_scope = sidebar_scope.unwrap_or_default();
        let terminate_setting = cx.weak_entity();
        let use_sidebar = cx.listener(move |this, _, _, cx| {
            if runtime_available {
                this.set_mode(Mode::Sidebar, cx);
            }
        });
        let use_tabs = cx.listener(move |this, _, _, cx| {
            if runtime_available {
                this.set_mode(Mode::Tabs, cx);
            }
        });
        let show_all = cx.listener(move |this, _, _, cx| {
            if runtime_available {
                this.set_sidebar_scope(SidebarScope::AllSpaces, cx);
            }
        });
        let show_active = cx.listener(move |this, _, _, cx| {
            if runtime_available {
                this.set_sidebar_scope(SidebarScope::ActiveSpace, cx);
            }
        });
        settings_fields(
            vec![
                setting_field(
                    "Terminate sessions on exit",
                    "End running sessions when Chartr exits instead of leaving them detached.",
                    Switch::new("terminate-sessions-on-exit", terminate.into())
                        .tab_index(0isize)
                        .aria_label("Terminate sessions on exit")
                        .aria_description(
                            "End running sessions when Chartr exits instead of leaving them detached.",
                        )
                        .on_click(move |state, _, cx| {
                            let terminate = state.selected();
                            let _ = terminate_setting.update(cx, |this, cx| {
                                this.set_terminate_on_exit(terminate, cx)
                            });
                        }),
                ),
                setting_field(
                    "Session list",
                    "Choose where sessions appear in the workspace.",
                    SegmentedControl::new(
                        "Session list presentation",
                        [
                            SegmentedControlOption::new(
                                "presentation-sidebar",
                                "Sidebar",
                                mode == Mode::Sidebar,
                                use_sidebar,
                            ),
                            SegmentedControlOption::new(
                                "presentation-tabs",
                                "Tabbed",
                                mode == Mode::Tabs,
                                use_tabs,
                            ),
                        ],
                    )
                    .disabled(!runtime_available),
                ),
                setting_field(
                    "Spaces shown",
                    "Show every space in the sidebar or only the active one.",
                    SegmentedControl::new(
                        "Spaces shown in the sidebar",
                        [
                            SegmentedControlOption::new(
                                "sidebar-all-spaces",
                                "All spaces",
                                sidebar_scope == SidebarScope::AllSpaces,
                                show_all,
                            ),
                            SegmentedControlOption::new(
                                "sidebar-active-space",
                                "Active only",
                                sidebar_scope == SidebarScope::ActiveSpace,
                                show_active,
                            ),
                        ],
                    )
                    .disabled(!runtime_available),
                ),
            ],
            cx.theme().colors().border_variant,
        )
    }

    fn theme_dropdown(
        &self,
        id: &'static str,
        label: &'static str,
        current: String,
        target: ThemeTarget,
        themes: Vec<theme::ThemeMeta>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let weak = cx.weak_entity();
        let selected = current.clone();
        let menu = ContextMenu::build(window, cx, move |menu, _, _| {
            let mut menu = menu;
            let has_dark = themes.iter().any(|theme| theme.appearance == theme::Appearance::Dark);
            let has_light = themes.iter().any(|theme| theme.appearance == theme::Appearance::Light);

            for (appearance, heading) in [
                (theme::Appearance::Dark, "Dark themes"),
                (theme::Appearance::Light, "Light themes"),
            ] {
                let choices: Vec<_> =
                    themes.iter().filter(|theme| theme.appearance == appearance).collect();
                if choices.is_empty() {
                    continue;
                }
                if has_dark && has_light {
                    if appearance == theme::Appearance::Light {
                        menu = menu.separator();
                    }
                    menu = menu.header(heading);
                }
                for choice in choices {
                    let name = choice.name.to_string();
                    let checked = name == selected;
                    let update = weak.clone();
                    menu = menu.toggleable_entry(
                        name.clone(),
                        checked,
                        IconPosition::End,
                        None,
                        move |_, cx| {
                            let name = name.clone();
                            let _ = update.update(cx, |this, cx| this.set_theme(target, name, cx));
                        },
                    );
                }
            }
            menu
        });

        DropdownMenu::new(id, current, menu)
            .style(DropdownStyle::Outlined)
            .attach(Anchor::BottomLeft)
            .aria_label(label)
            .into_any_element()
    }

    fn appearance_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let settings = self.settings(cx);
        let mode = settings.theme_mode;
        let mut themes = theme::ThemeRegistry::global(cx).list();
        themes.sort_unstable_by(|a, b| {
            a.appearance.is_light().cmp(&b.appearance.is_light()).then(a.name.cmp(&b.name))
        });
        let light_themes = themes
            .iter()
            .filter(|theme| theme.appearance == theme::Appearance::Light)
            .cloned()
            .collect();
        let dark_themes = themes
            .iter()
            .filter(|theme| theme.appearance == theme::Appearance::Dark)
            .cloned()
            .collect();
        let fixed_mode = cx.listener(|this, _, _, cx| this.set_theme_mode(ThemeMode::Fixed, cx));
        let system_mode = cx.listener(|this, _, _, cx| this.set_theme_mode(ThemeMode::System, cx));
        let reduce_motion = settings.reduce_motion;
        let reduce_motion_setting = cx.weak_entity();
        let font = cx.weak_entity();
        let smaller = cx.listener(|this, _, _, cx| this.adjust_ui_font_size(-1., cx));
        let larger = cx.listener(|this, _, _, cx| this.adjust_ui_font_size(1., cx));
        let fixed_picker = self.theme_dropdown(
            "fixed-theme-menu",
            "Theme",
            settings.fixed_theme.clone(),
            ThemeTarget::Fixed,
            themes,
            window,
            cx,
        );
        let light_picker = self.theme_dropdown(
            "light-theme-menu",
            "Light theme",
            settings.light_theme.clone(),
            ThemeTarget::Light,
            light_themes,
            window,
            cx,
        );
        let dark_picker = self.theme_dropdown(
            "dark-theme-menu",
            "Dark theme",
            settings.dark_theme.clone(),
            ThemeTarget::Dark,
            dark_themes,
            window,
            cx,
        );
        let font_picker = PopoverMenu::new("ui-font-menu")
            .trigger(
                Button::new("ui-font-family", settings.ui_font_family)
                    .end_icon(Icon::new(IconName::ChevronDown)),
            )
            .anchor(Anchor::BottomLeft)
            .menu(move |window, cx| {
                let font = font.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    ["IBM Plex Sans", ".ZedSans", "System UI"].into_iter().fold(
                        menu,
                        |menu, family| {
                            let set = font.clone();
                            menu.entry(family, None, move |_, cx| {
                                let _ = set
                                    .update(cx, |this, cx| this.set_ui_font(family.to_owned(), cx));
                            })
                        },
                    )
                }))
            });
        let font_size = number_field(
            "ui-font-size",
            "Interface font size",
            "Adjust the size of interface text.",
            self.ui_font_size_input.clone(),
            smaller,
            larger,
            cx,
        );

        let mut fields = vec![setting_field(
            "Theme mode",
            "Use one theme at all times or follow the system appearance.",
            SegmentedControl::new(
                "Theme mode",
                [
                    SegmentedControlOption::new(
                        "theme-fixed",
                        "Fixed",
                        mode == ThemeMode::Fixed,
                        fixed_mode,
                    ),
                    SegmentedControlOption::new(
                        "theme-system",
                        "System",
                        mode == ThemeMode::System,
                        system_mode,
                    ),
                ],
            ),
        )];
        match mode {
            ThemeMode::Fixed => fields.push(setting_field(
                "Theme",
                "Choose the theme used throughout the interface.",
                fixed_picker,
            )),
            ThemeMode::System => {
                fields.push(setting_field(
                    "Light theme",
                    "Choose the theme used while the system is in light mode.",
                    light_picker,
                ));
                fields.push(setting_field(
                    "Dark theme",
                    "Choose the theme used while the system is in dark mode.",
                    dark_picker,
                ));
            }
        }
        fields.extend([
            setting_field(
                "Font family",
                "Choose the typeface used throughout the interface.",
                font_picker,
            ),
            setting_field("Font size", "Adjust the size of interface text.", font_size),
            setting_field(
                "Reduce motion",
                "Disable movement animations when space cards are sorted.",
                Switch::new("reduce-motion", reduce_motion.into())
                    .tab_index(0isize)
                    .aria_label("Reduce motion")
                    .aria_description("Disable movement animations when space cards are sorted.")
                    .on_click(move |state, _, cx| {
                        let reduce_motion = state.selected();
                        let _ = reduce_motion_setting
                            .update(cx, |this, cx| this.set_reduce_motion(reduce_motion, cx));
                    }),
            ),
        ]);

        settings_fields(fields, cx.theme().colors().border_variant)
    }

    fn terminal_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let settings = self.settings(cx);
        let font = cx.weak_entity();
        let smaller = cx.listener(|this, _, _, cx| this.adjust_terminal_font_size(-1., cx));
        let larger = cx.listener(|this, _, _, cx| this.adjust_terminal_font_size(1., cx));
        let choose_directory = cx.listener(|this, _, _, cx| this.pick_free_sessions_directory(cx));
        let retry = cx.listener(|this, _, _, cx| this.retry_backend(cx));
        let restart = cx.listener(|this, _, _, cx| this.restart_backend(cx));
        let runtime_available = self.original.upgrade().is_some();
        let font_picker = PopoverMenu::new("terminal-font-menu")
            .trigger(
                Button::new("terminal-font-family", settings.terminal_font_family)
                    .end_icon(Icon::new(IconName::ChevronDown)),
            )
            .anchor(Anchor::BottomLeft)
            .menu(move |window, cx| {
                let font = font.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    ["IBM Plex Mono", "Lilex", ".ZedMono"].into_iter().fold(menu, |menu, family| {
                        let set = font.clone();
                        menu.entry(family, None, move |_, cx| {
                            let _ = set.update(cx, |this, cx| {
                                this.set_terminal_font(family.to_owned(), cx)
                            });
                        })
                    })
                }))
            });
        let font_size = number_field(
            "terminal-font-size",
            "Terminal font size",
            "Adjust the size of terminal text.",
            self.terminal_font_size_input.clone(),
            smaller,
            larger,
            cx,
        );
        let directory = settings
            .ad_hoc_directory
            .as_ref()
            .map_or_else(|| "Home directory".to_owned(), |path| path.display().to_string());

        settings_fields(
            vec![
                setting_field(
                    "Font family",
                    "Choose the typeface used in terminal sessions.",
                    font_picker,
                ),
                setting_field("Font size", "Adjust the size of terminal text.", font_size),
                setting_field(
                    "Free sessions directory",
                    "Choose the working directory used when a Free session starts.",
                    Button::new("choose-free-sessions-directory", directory)
                        .start_icon(Icon::new(IconName::FolderOpen).color(Color::Muted))
                        .end_icon(Icon::new(IconName::ChevronRight).color(Color::Muted))
                        .truncate(true)
                        .tooltip(Tooltip::text("Choose Free sessions directory"))
                        .on_click(choose_directory),
                ),
                setting_field(
                    "Backend status",
                    "Show the session backend connected to this workspace.",
                    Label::new(self.backend_label(cx)).size(UI_LABEL_DEFAULT),
                ),
                setting_field(
                    "Retry connection",
                    "Try to reconnect after a backend connection failure.",
                    Button::new("settings-retry-backend", "Retry")
                        .disabled(!runtime_available)
                        .on_click(retry),
                ),
                setting_field(
                    "Restart backend",
                    "Stop and start the backend process for this workspace.",
                    Button::new("settings-restart-backend", "Restart")
                        .disabled(!runtime_available)
                        .on_click(restart),
                ),
            ],
            cx.theme().colors().border_variant,
        )
    }

    fn hotkeys_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let recording = self.recording_keymap;
        let keymap = cx.global::<KeymapStore>();
        let table = KeymapAction::ALL.into_iter().fold(
            Table::new(2)
                .striped()
                .width_config(ColumnWidthConfig::redistributable(self.hotkey_widths.clone()))
                .header(vec!["Action", "Shortcut"]),
            |table, action| {
                let capture = cx.listener(move |this, _, _, cx| {
                    this.recording_keymap = Some(action);
                    this.problem = None;
                    cx.notify();
                });
                table.row(vec![
                    Label::new(action.title()).size(UI_LABEL_DEFAULT).into_any_element(),
                    Button::new(
                        format!("record-hotkey-{}", action.id()),
                        if recording == Some(action) {
                            "Press shortcut…".to_owned()
                        } else {
                            keymap.key(action).to_owned()
                        },
                    )
                    .toggle_state(recording == Some(action))
                    .selected_style(ButtonStyle::Filled)
                    .selected_label_color(Color::Default)
                    .on_click(capture)
                    .into_any_element(),
                ])
            },
        );
        let keymap_problem = keymap.problem().map(str::to_owned);
        v_flex()
            .gap_2()
            .when_some(keymap_problem, |view, problem| {
                view.child(
                    Banner::new()
                        .severity(Severity::Error)
                        .child(Label::new(problem).size(UI_LABEL_DEFAULT)),
                )
            })
            .when(self.keymap_restart_required, |view| {
                view.child(Banner::new().child(
                    Label::new(
                        "Shortcut changes are saved. Restart Chartr to rebuild the application keymap.",
                    )
                    .size(UI_LABEL_DEFAULT),
                ))
            })
            .child(
                Label::new(
                    "Click a shortcut, then press one key chord. Conflicts in the Chartr context are rejected.",
                )
                .size(UI_LABEL_SMALL)
                .color(Color::Muted),
            )
            .child(table)
            .into_any_element()
    }

    fn plugins_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let origin_available = self.original.upgrade().is_some();
        let (descriptors, rejected) = self
            .original
            .upgrade()
            .map(|origin| origin.read(cx).settings_plugins())
            .unwrap_or_default();
        let settings = self.settings(cx);
        let mut fields = Vec::new();
        for descriptor in descriptors {
            let manifest = descriptor.manifest;
            let name = manifest.name.clone();
            let id = manifest.id.clone();
            let enabled = descriptor.enabled;
            let has_settings = descriptor.has_settings;
            let unsafe_filesystem = settings.plugin(&id).unsafe_filesystem;
            let is_web = manifest.kind == zeddy_plugin::manifest::Kind::Web;
            let access = match manifest.kind {
                zeddy_plugin::manifest::Kind::Native => {
                    format!("Identifier: {id}. Runs as fully trusted native code.")
                }
                zeddy_plugin::manifest::Kind::Web => {
                    let project = match manifest.permissions.project_files {
                        zeddy_plugin::manifest::ProjectAccess::None => "no project files",
                        zeddy_plugin::manifest::ProjectAccess::Read => "read project files",
                        zeddy_plugin::manifest::ProjectAccess::ReadWrite => {
                            "read and write project files"
                        }
                    };
                    let mut grants = vec![project.to_owned()];
                    if !manifest.permissions.network.is_empty() {
                        grants.push(format!(
                            "network access to {}",
                            manifest.permissions.network.join(", ")
                        ));
                    }
                    if manifest.permissions.process {
                        grants.push("process actions".to_owned());
                    }
                    if manifest.permissions.session {
                        grants.push("bound-session actions".to_owned());
                    }
                    format!("Identifier: {id}. Access: {}.", grants.join(", "))
                }
            };

            let enabled_name = format!("{name} — Enabled");
            let enabled_description = access;
            let enabled_id = id.clone();
            let enabled_setting = cx.weak_entity();
            let enabled_control = Switch::new(format!("plugin-enabled-{id}"), enabled.into())
                .disabled(!origin_available)
                .tab_index(0isize)
                .aria_label(enabled_name.clone())
                .aria_description(enabled_description.clone())
                .on_click(move |state, _, cx| {
                    let enabled = state.selected();
                    let _ = enabled_setting.update(cx, |this, cx| {
                        this.set_plugin_enabled(enabled_id.clone(), enabled, cx)
                    });
                });
            fields.push(setting_field(enabled_name, enabled_description, enabled_control));

            if has_settings {
                let settings_id = id.clone();
                fields.push(setting_field(
                    format!("{name} — Configuration"),
                    "Open this plugin's own settings.",
                    Button::new(format!("plugin-settings-{id}"), "Configure")
                        .disabled(!origin_available)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open_plugin_settings(settings_id.clone(), window, cx)
                        })),
                ));
            }

            if is_web {
                let unsafe_name = format!("{name} — Unsafe filesystem access");
                let unsafe_description =
                    "Allow access to files outside the plugin's declared project permissions.";
                let unsafe_id = id.clone();
                let unsafe_setting = cx.weak_entity();
                let unsafe_control =
                    Switch::new(format!("plugin-unsafe-{id}"), unsafe_filesystem.into())
                        .disabled(!origin_available)
                        .tab_index(0isize)
                        .aria_label(unsafe_name.clone())
                        .aria_description(unsafe_description)
                        .on_click(move |state, _, cx| {
                            let enabled = state.selected();
                            let _ = unsafe_setting.update(cx, |this, cx| {
                                this.set_plugin_unsafe(unsafe_id.clone(), enabled, cx)
                            });
                        });
                fields.push(setting_field(unsafe_name, unsafe_description, unsafe_control));
            }
        }
        let has_fields = !fields.is_empty();
        let rejected: Vec<_> = rejected
            .into_iter()
            .map(|rejected| {
                Banner::new().severity(Severity::Error).child(
                    Label::new(format!("{}: {}", rejected.dir.display(), rejected.why))
                        .size(UI_LABEL_SMALL),
                )
            })
            .collect();
        let _ = window;
        v_flex()
            .gap_4()
            .when(!origin_available, |view| {
                view.child(
                    Banner::new().child(
                        Label::new(
                            "Open Settings from a Chartr workspace to manage runtime plugins.",
                        )
                        .size(UI_LABEL_DEFAULT),
                    ),
                )
            })
            .when(!has_fields && rejected.is_empty(), |view| {
                view.child(Label::new("No plugins installed.").color(Color::Muted))
            })
            .when(has_fields, |view| {
                view.child(settings_fields(fields, cx.theme().colors().border_variant))
            })
            .children(rejected)
            .into_any_element()
    }
}

impl Focusable for SettingsWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ui_font = Fonts::setup_ui(window, cx);
        let selected = self.page;
        let navigation: Vec<_> = SettingsPage::ALL
            .into_iter()
            .map(|page| {
                selection_row(format!("settings-page-{}", page.slug()), page == selected)
                    .aria_role(Role::Tab)
                    .aria_label(page.title())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = page;
                        if page != SettingsPage::Plugins {
                            this.plugin_settings = None;
                        }
                        cx.notify();
                    }))
                    .child(
                        Label::new(page.title())
                            .size(UI_LABEL_DEFAULT)
                            .when(page != selected, |label| label.color(Color::Muted)),
                    )
            })
            .collect();
        let unreadable = cx.global::<SettingsStore>().unreadable().map(str::to_owned);
        let page_title = self.page.title();
        let content = self.content(window, cx);

        div()
            .id("settings-window")
            .key_context("ChartrSettings")
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .font(ui_font)
            .text_size(UI_TEXT_DEFAULT)
            .bg(cx.theme().colors().background)
            .text_color(cx.theme().colors().text)
            .on_action(cx.listener(|_, _: &Close, window, _| window.remove_window()))
            .on_action(cx.listener(|_, _: &crate::actions::settings::Open, window, _| {
                window.activate_window()
            }))
            .on_key_down(cx.listener(|this, event, window, cx| this.on_key(event, window, cx)))
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(240.))
                            .h_full()
                            .py_3()
                            .px_1()
                            .border_r_1()
                            .border_color(cx.theme().colors().border)
                            .bg(cx.theme().colors().surface_background)
                            .child(
                                div().px_3().pb_2().child(
                                    Label::new("Settings")
                                        .size(UI_LABEL_LARGE)
                                        .weight(FontWeight::SEMIBOLD),
                                ),
                            )
                            .child(selection_list().px_2().children(navigation)),
                    )
                    .child(
                        div()
                            .id("settings-content-scroll")
                            .flex_1()
                            .h_full()
                            .min_w_0()
                            .overflow_y_scroll()
                            .child(
                                v_flex()
                                    .w_full()
                                    .max_w(px(720.))
                                    .p_6()
                                    .gap_4()
                                    .child(Label::new(page_title).size(UI_LABEL_LARGE))
                                    .when_some(unreadable, |view, problem| {
                                        view.child(
                                            Banner::new()
                                                .severity(Severity::Error)
                                                .child(Label::new(problem).size(UI_LABEL_DEFAULT)),
                                        )
                                    })
                                    .when_some(self.problem.clone(), |view, problem| {
                                        view.child(
                                            Banner::new()
                                                .severity(Severity::Error)
                                                .child(Label::new(problem).size(UI_LABEL_DEFAULT)),
                                        )
                                    })
                                    .child(content),
                            ),
                    ),
            )
    }
}

fn format_number(value: f32) -> String {
    value.to_string()
}

fn sync_number_text(input: &Entity<TextInput>, value: f32, cx: &mut Context<SettingsWindow>) {
    let value = format_number(value);
    if input.read(cx).text() != value {
        input.update(cx, |input, cx| input.set_text(value, false, cx));
    }
}

fn number_field(
    id: &'static str,
    label: &'static str,
    description: &'static str,
    input: Entity<TextInput>,
    decrement: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    increment: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> AnyElement {
    let id: ElementId = id.into();
    let colors = cx.theme().colors();
    let border = colors.border_variant;
    let background = colors.surface_background;
    let hover = colors.element_hover;
    let focus = input.focus_handle(cx);

    let decrement = h_flex()
        .id((id.clone(), "decrement"))
        .role(Role::Button)
        .aria_label("Decrement")
        .tab_index(0isize)
        .w(px(32.))
        .h_full()
        .justify_center()
        .cursor_pointer()
        .rounded_l_sm()
        .border_1()
        .border_color(border)
        .bg(background)
        .hover(|style| style.bg(hover))
        .on_click(decrement)
        .child(Icon::new(IconName::Dash).size(IconSize::Small));
    let increment = h_flex()
        .id((id.clone(), "increment"))
        .role(Role::Button)
        .aria_label("Increment")
        .tab_index(0isize)
        .w(px(32.))
        .h_full()
        .justify_center()
        .cursor_pointer()
        .rounded_r_sm()
        .border_1()
        .border_color(border)
        .bg(background)
        .hover(|style| style.bg(hover))
        .on_click(increment)
        .child(Icon::new(IconName::Plus).size(IconSize::Small));

    h_flex()
        .id(id)
        .role(Role::SpinButton)
        .aria_label(label)
        .aria_description(description)
        .h(ButtonSize::Default.rems())
        .gap_1()
        .child(decrement)
        .child(
            h_flex()
                .w(px(64.))
                .h_full()
                .px_2()
                .border_y_1()
                .border_color(border)
                .bg(background)
                .track_focus(&focus)
                .in_focus(|field| field.border_1().border_color(colors.border_focused))
                .child(input),
        )
        .child(increment)
        .into_any_element()
}

fn settings_fields(fields: Vec<AnyElement>, separator: Hsla) -> AnyElement {
    v_flex()
        .w_full()
        .children(fields.into_iter().enumerate().map(|(index, field)| {
            div()
                .w_full()
                .when(index > 0, |row| row.border_t_1().border_color(separator))
                .child(field)
        }))
        .into_any_element()
}

fn setting_field(
    name: impl Into<SharedString>,
    description: impl Into<SharedString>,
    control: impl IntoElement,
) -> AnyElement {
    h_flex()
        .w_full()
        .items_start()
        .gap_6()
        .py(px(SETTINGS_FIELD_VERTICAL_PADDING))
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_1()
                .child(Label::new(name).size(UI_LABEL_DEFAULT))
                .child(Label::new(description).size(UI_LABEL_SMALL).color(Color::Muted)),
        )
        .child(
            h_flex().w(px(SETTINGS_CONTROL_COLUMN_WIDTH)).flex_none().justify_end().child(control),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    struct WorkspacePlaceholder;

    impl Render for WorkspacePlaceholder {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            gpui::Empty
        }
    }

    fn init_test(cx: &mut TestAppContext) {
        cx.update(|cx| {
            theme::init(theme::LoadThemes::JustBase, cx);
            let settings = SettingsStore::bare();
            theme::set_theme_settings_provider(
                Box::new(Fonts::from_settings(settings.resolved())),
                cx,
            );
            cx.set_global(settings);
            let keymap = KeymapStore::bare();
            init(&keymap, cx);
            cx.set_global(keymap);
        });
    }

    fn settings_window_count(cx: &TestAppContext) -> usize {
        cx.windows().into_iter().filter_map(|window| window.downcast::<SettingsWindow>()).count()
    }

    #[gpui::test]
    fn reopening_settings_reuses_the_application_window(cx: &mut TestAppContext) {
        init_test(cx);
        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        assert_eq!(settings_window_count(cx), 1);

        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        assert_eq!(settings_window_count(cx), 1);
    }

    #[gpui::test]
    fn settings_closes_when_the_last_workspace_window_closes(cx: &mut TestAppContext) {
        init_test(cx);
        let workspace = cx.add_window(|_, _| WorkspacePlaceholder);
        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        assert_eq!(settings_window_count(cx), 1);

        cx.update(|cx| {
            workspace.update(cx, |_, window, _| window.remove_window()).unwrap();
        });
        cx.run_until_parked();
        assert_eq!(settings_window_count(cx), 0);
    }

    #[gpui::test]
    fn platform_close_shortcut_closes_only_the_settings_window(cx: &mut TestAppContext) {
        init_test(cx);
        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        let settings = cx
            .windows()
            .into_iter()
            .find(|window| window.downcast::<SettingsWindow>().is_some())
            .unwrap();
        let mut window = gpui::VisualTestContext::from_window(settings, cx);
        #[cfg(target_os = "macos")]
        window.simulate_keystrokes("cmd-w");
        #[cfg(not(target_os = "macos"))]
        window.simulate_keystrokes("ctrl-w");
        window.run_until_parked();
        assert_eq!(settings_window_count(cx), 0);
    }

    #[gpui::test]
    fn edits_use_the_application_global_settings_store(cx: &mut TestAppContext) {
        init_test(cx);
        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        let settings = cx
            .windows()
            .into_iter()
            .find_map(|window| window.downcast::<SettingsWindow>())
            .unwrap();
        cx.update(|cx| {
            settings
                .update(cx, |settings, _, cx| settings.set_terminate_on_exit(true, cx))
                .unwrap();
            assert!(cx.global::<SettingsStore>().resolved().terminate_sessions_on_exit);
        });
    }

    #[gpui::test]
    fn font_size_inputs_update_the_application_settings(cx: &mut TestAppContext) {
        init_test(cx);
        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        let settings = cx
            .windows()
            .into_iter()
            .find_map(|window| window.downcast::<SettingsWindow>())
            .unwrap();
        let (ui_input, terminal_input) = cx.update(|cx| {
            let settings = settings.read(cx).unwrap();
            (settings.ui_font_size_input.clone(), settings.terminal_font_size_input.clone())
        });

        cx.update(|cx| {
            ui_input.update(cx, |input, cx| input.set_text("18", false, cx));
            terminal_input.update(cx, |input, cx| input.set_text("16", false, cx));
        });
        cx.run_until_parked();

        cx.update(|cx| {
            let resolved = cx.global::<SettingsStore>().resolved();
            assert_eq!(resolved.ui_font_size, 18.);
            assert_eq!(resolved.terminal_font_size, 16.);
        });
    }
}
