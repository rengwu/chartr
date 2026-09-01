//! Chartr's singleton, application-wide Settings window.
//!
//! This follows Zed's `SettingsWindow` boundary: opening Settings focuses the
//! existing app-wide window, global settings notify every workspace live, and
//! the originating workspace is retained only for operations that truly need
//! runtime state (the backend and plugin catalog).

use gpui::{
    Anchor, AnyView, App, Bounds, Context, DefiniteLength, Entity, FocusHandle, Focusable,
    FontWeight, KeyBinding, PathPromptOptions, Render, Role, WeakEntity, Window, WindowBounds,
    WindowHandle, WindowOptions, actions, px, size,
};
use ui::{
    Banner, Button, ColumnWidthConfig, ContextMenu, DropdownMenu, DropdownStyle, Icon, IconButton,
    PopoverMenu, RedistributableColumnsState, Severity, Table, TableResizeBehavior, Tooltip,
    prelude::*,
};

use crate::{
    app::Zeddy,
    fonts::Fonts,
    keymap::{KeymapAction, KeymapStore},
    mode::Mode,
    persistence::SidebarScope,
    settings::{
        self, AppearanceContent, GeneralContent, ResolvedSettings, SettingsPage, SettingsStore,
        TerminalContent, ThemeMode,
    },
};

actions!(settings_window, [Close]);

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
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Chartr — Settings".into()),
                    ..Default::default()
                }),
                focus: true,
                show: true,
                is_movable: true,
                kind: gpui::WindowKind::Normal,
                window_background: cx.theme().window_background_appearance(),
                window_min_size: Some(size(px(640.), px(420.))),
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
    hotkey_widths: Entity<RedistributableColumnsState>,
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
        cx.observe_global_in::<SettingsStore>(window, |_, _, cx| cx.notify()).detach();
        cx.observe_global_in::<KeymapStore>(window, |_, _, cx| cx.notify()).detach();
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
            hotkey_widths: cx.new(|_| {
                RedistributableColumnsState::new(
                    2,
                    vec![DefiniteLength::Fraction(0.68), DefiniteLength::Fraction(0.32)],
                    vec![TableResizeBehavior::Resizable, TableResizeBehavior::Resizable],
                )
            }),
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

    fn adjust_ui_font_size(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = cx.global::<SettingsStore>().resolved().ui_font_size;
        self.update_settings(
            move |content| {
                content.appearance.get_or_insert_with(AppearanceContent::default).ui_font_size =
                    Some((current + delta).clamp(8., 32.));
            },
            false,
            true,
            cx,
        );
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

    fn adjust_terminal_font_size(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = cx.global::<SettingsStore>().resolved().terminal_font_size;
        self.update_settings(
            move |content| {
                content.terminal.get_or_insert_with(TerminalContent::default).font_size =
                    Some((current + delta).clamp(8., 72.));
            },
            false,
            false,
            cx,
        );
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
                .child(Label::new(plugin.clone()).size(LabelSize::XSmall).color(Color::Muted))
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
        let toggle = cx.listener(move |this, _, _, cx| this.set_terminate_on_exit(!terminate, cx));
        let use_sidebar = cx.listener(|this, _, _, cx| this.set_mode(Mode::Sidebar, cx));
        let use_tabs = cx.listener(|this, _, _, cx| this.set_mode(Mode::Tabs, cx));
        let show_all =
            cx.listener(|this, _, _, cx| this.set_sidebar_scope(SidebarScope::AllSpaces, cx));
        let show_active =
            cx.listener(|this, _, _, cx| this.set_sidebar_scope(SidebarScope::ActiveSpace, cx));
        v_flex()
            .gap_4()
            .child(Label::new("Chartr").size(LabelSize::Large))
            .child(
                Label::new(format!(
                    "Version {} · configuration namespace chartr-zeddy",
                    env!("CARGO_PKG_VERSION")
                ))
                .size(LabelSize::Small)
                .color(Color::Muted),
            )
            .child(
                h_flex()
                    .justify_between()
                    .gap_4()
                    .child(
                        v_flex()
                            .child(Label::new("Terminate sessions on exit").size(LabelSize::Small))
                            .child(
                                Label::new("Normal app exit detaches and leaves sessions running.")
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        Button::new(
                            "terminate-sessions-on-exit",
                            if terminate { "On" } else { "Off" },
                        )
                        .toggle_state(terminate)
                        .on_click(toggle),
                    ),
            )
            .child(setting_label("Presentation"))
            .child(
                h_flex()
                    .justify_between()
                    .gap_4()
                    .child(
                        v_flex().child(Label::new("Session list").size(LabelSize::Small)).child(
                            Label::new("Show sessions in a sidebar or a tab strip.")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("presentation-sidebar", "Sidebar")
                                    .disabled(!runtime_available)
                                    .toggle_state(mode == Mode::Sidebar)
                                    .on_click(use_sidebar),
                            )
                            .child(
                                Button::new("presentation-tabs", "Tabbed")
                                    .disabled(!runtime_available)
                                    .toggle_state(mode == Mode::Tabs)
                                    .on_click(use_tabs),
                            ),
                    ),
            )
            .child(setting_label("Sidebar"))
            .child(
                h_flex()
                    .justify_between()
                    .gap_4()
                    .child(
                        v_flex().child(Label::new("Spaces shown").size(LabelSize::Small)).child(
                            Label::new("Show every space or only the currently active space.")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("sidebar-all-spaces", "All spaces")
                                    .disabled(!runtime_available)
                                    .toggle_state(sidebar_scope == SidebarScope::AllSpaces)
                                    .on_click(show_all),
                            )
                            .child(
                                Button::new("sidebar-active-space", "Active space only")
                                    .disabled(!runtime_available)
                                    .toggle_state(sidebar_scope == SidebarScope::ActiveSpace)
                                    .on_click(show_active),
                            ),
                    ),
            )
            .into_any_element()
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
        v_flex()
            .gap_3()
            .child(setting_label("Theme mode"))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("theme-fixed", "Fixed")
                            .toggle_state(mode == ThemeMode::Fixed)
                            .on_click(fixed_mode),
                    )
                    .child(
                        Button::new("theme-system", "Match system")
                            .toggle_state(mode == ThemeMode::System)
                            .on_click(system_mode),
                    ),
            )
            .when(mode == ThemeMode::Fixed, |view| {
                view.child(setting_label("Theme")).child(fixed_picker)
            })
            .when(mode == ThemeMode::System, |view| {
                view.child(
                    h_flex()
                        .gap_6()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(setting_label("Light theme"))
                                .child(light_picker),
                        )
                        .child(
                            v_flex().gap_1().child(setting_label("Dark theme")).child(dark_picker),
                        ),
                )
            })
            .child(setting_label("Interface font"))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        PopoverMenu::new("ui-font-menu")
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
                                                let _ = set.update(cx, |this, cx| {
                                                    this.set_ui_font(family.to_owned(), cx)
                                                });
                                            })
                                        },
                                    )
                                }))
                            }),
                    )
                    .child(
                        IconButton::new("ui-font-smaller", IconName::Dash)
                            .tooltip(Tooltip::text("Decrease interface font size"))
                            .on_click(smaller),
                    )
                    .child(
                        Label::new(format!("{} px", settings.ui_font_size)).size(LabelSize::Small),
                    )
                    .child(
                        IconButton::new("ui-font-larger", IconName::Plus)
                            .tooltip(Tooltip::text("Increase interface font size"))
                            .on_click(larger),
                    ),
            )
            .into_any_element()
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
        v_flex()
            .gap_3()
            .child(setting_label("Terminal font"))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        PopoverMenu::new("terminal-font-menu")
                            .trigger(
                                Button::new("terminal-font-family", settings.terminal_font_family)
                                    .end_icon(Icon::new(IconName::ChevronDown)),
                            )
                            .anchor(Anchor::BottomLeft)
                            .menu(move |window, cx| {
                                let font = font.clone();
                                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                                    ["IBM Plex Mono", "Lilex", ".ZedMono"].into_iter().fold(
                                        menu,
                                        |menu, family| {
                                            let set = font.clone();
                                            menu.entry(family, None, move |_, cx| {
                                                let _ = set.update(cx, |this, cx| {
                                                    this.set_terminal_font(family.to_owned(), cx)
                                                });
                                            })
                                        },
                                    )
                                }))
                            }),
                    )
                    .child(
                        IconButton::new("terminal-font-smaller", IconName::Dash)
                            .tooltip(Tooltip::text("Decrease terminal font size"))
                            .on_click(smaller),
                    )
                    .child(
                        Label::new(format!("{} px", settings.terminal_font_size))
                            .size(LabelSize::Small),
                    )
                    .child(
                        IconButton::new("terminal-font-larger", IconName::Plus)
                            .tooltip(Tooltip::text("Increase terminal font size"))
                            .on_click(larger),
                    ),
            )
            .child(setting_label("Free sessions directory"))
            .child(
                Button::new(
                    "choose-free-sessions-directory",
                    settings.ad_hoc_directory.as_ref().map_or_else(
                        || "Home directory".to_owned(),
                        |path| path.display().to_string(),
                    ),
                )
                .on_click(choose_directory),
            )
            .child(setting_value("Backend", self.backend_label(cx)))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("settings-retry-backend", "Retry")
                            .disabled(!runtime_available)
                            .on_click(retry),
                    )
                    .child(
                        Button::new("settings-restart-backend", "Restart Backend")
                            .disabled(!runtime_available)
                            .on_click(restart),
                    ),
            )
            .into_any_element()
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
                    Label::new(action.title()).size(LabelSize::Small).into_any_element(),
                    Button::new(
                        format!("record-hotkey-{}", action.id()),
                        if recording == Some(action) {
                            "Press shortcut…".to_owned()
                        } else {
                            keymap.key(action).to_owned()
                        },
                    )
                    .label_size(LabelSize::Default)
                    .toggle_state(recording == Some(action))
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
                        .child(Label::new(problem).size(LabelSize::Small)),
                )
            })
            .when(self.keymap_restart_required, |view| {
                view.child(Banner::new().child(
                    Label::new(
                        "Shortcut changes are saved. Restart Chartr to rebuild the application keymap.",
                    )
                    .size(LabelSize::Small),
                ))
            })
            .child(
                Label::new(
                    "Click a shortcut, then press one key chord. Conflicts in the Chartr context are rejected.",
                )
                .size(LabelSize::XSmall)
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
        let rows: Vec<_> = descriptors
            .into_iter()
            .map(|descriptor| {
                let manifest = descriptor.manifest;
                let enabled = descriptor.enabled;
                let has_settings = descriptor.has_settings;
                let id = manifest.id.clone();
                let control_id = id.clone();
                let configured = settings.plugin(&id);
                let toggle = cx.listener(move |this, _, _, cx| {
                    this.set_plugin_enabled(id.clone(), !enabled, cx)
                });
                let trust = match manifest.kind {
                    zeddy_plugin::manifest::Kind::Native => {
                        "Native — fully trusted code".to_owned()
                    }
                    zeddy_plugin::manifest::Kind::Web => {
                        let project = match manifest.permissions.project_files {
                            zeddy_plugin::manifest::ProjectAccess::None => "no project files",
                            zeddy_plugin::manifest::ProjectAccess::Read => "read project files",
                            zeddy_plugin::manifest::ProjectAccess::ReadWrite => {
                                "read/write project files"
                            }
                        };
                        let mut grants = vec![project.to_owned()];
                        if !manifest.permissions.network.is_empty() {
                            grants.push(format!(
                                "network: {}",
                                manifest.permissions.network.join(", ")
                            ));
                        }
                        if manifest.permissions.process {
                            grants.push("process actions".to_owned());
                        }
                        if manifest.permissions.session {
                            grants.push("bound-session actions".to_owned());
                        }
                        format!("Web — {}", grants.join(" · "))
                    }
                };
                let unsafe_control =
                    (manifest.kind == zeddy_plugin::manifest::Kind::Web).then(|| {
                        let id = manifest.id.clone();
                        let change = cx.listener(move |this, _, _, cx| {
                            this.set_plugin_unsafe(id.clone(), !configured.unsafe_filesystem, cx)
                        });
                        Button::new(
                            format!("plugin-unsafe-{}", manifest.id),
                            if configured.unsafe_filesystem {
                                "Unsafe filesystem granted"
                            } else {
                                "Grant unsafe filesystem"
                            },
                        )
                        .disabled(!origin_available)
                        .toggle_state(configured.unsafe_filesystem)
                        .on_click(change)
                    });
                let configure = has_settings.then(|| {
                    let id = manifest.id.clone();
                    Button::new(format!("plugin-settings-{}", manifest.id), "Configure")
                        .disabled(!origin_available)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open_plugin_settings(id.clone(), window, cx)
                        }))
                });
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .rounded_md()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                v_flex()
                                    .child(Label::new(manifest.name).size(LabelSize::Small))
                                    .child(
                                        Label::new(manifest.id)
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    ),
                            )
                            .child(
                                Button::new(
                                    format!("plugin-enabled-{control_id}"),
                                    if enabled { "Enabled" } else { "Disabled" },
                                )
                                .disabled(!origin_available)
                                .toggle_state(enabled)
                                .on_click(toggle),
                            ),
                    )
                    .child(Label::new(trust).size(LabelSize::XSmall).color(Color::Muted))
                    .when_some(configure, |row, control| row.child(control))
                    .when_some(unsafe_control, |row, control| row.child(control))
            })
            .collect();
        let rejected: Vec<_> = rejected
            .into_iter()
            .map(|rejected| {
                Banner::new().severity(Severity::Error).child(
                    Label::new(format!("{}: {}", rejected.dir.display(), rejected.why))
                        .size(LabelSize::XSmall),
                )
            })
            .collect();
        let _ = window;
        v_flex()
            .gap_2()
            .when(!origin_available, |view| {
                view.child(
                    Banner::new().child(
                        Label::new(
                            "Open Settings from a Chartr workspace to manage runtime plugins.",
                        )
                        .size(LabelSize::Small),
                    ),
                )
            })
            .when(rows.is_empty() && rejected.is_empty(), |view| {
                view.child(Label::new("No plugins installed.").color(Color::Muted))
            })
            .children(rows)
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
        let selected = self.page;
        let navigation: Vec<_> = SettingsPage::ALL
            .into_iter()
            .map(|page| {
                div()
                    .id(format!("settings-page-{}", page.slug()))
                    .role(Role::Tab)
                    .aria_label(page.title())
                    .aria_selected(page == selected)
                    .mx_1()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(page == selected, |row| {
                        row.bg(cx.theme().colors().element_selected)
                            .text_color(cx.theme().colors().text)
                    })
                    .when(page != selected, |row| {
                        row.text_color(cx.theme().colors().text_muted)
                            .hover(|row| row.bg(cx.theme().colors().element_hover))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = page;
                        if page != SettingsPage::Plugins {
                            this.plugin_settings = None;
                        }
                        cx.notify();
                    }))
                    .child(Label::new(page.title()).size(LabelSize::Small))
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
            .bg(cx.theme().colors().background)
            .text_color(cx.theme().colors().text)
            .on_action(cx.listener(|_, _: &Close, window, _| window.remove_window()))
            .on_action(cx.listener(|_, _: &crate::actions::settings::Open, window, _| {
                window.activate_window()
            }))
            .on_key_down(cx.listener(|this, event, window, cx| this.on_key(event, window, cx)))
            .child(
                h_flex()
                    .size_full()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(176.))
                            .h_full()
                            .py_3()
                            .border_r_1()
                            .border_color(cx.theme().colors().border)
                            .bg(cx.theme().colors().surface_background)
                            .child(
                                div().px_3().pb_2().child(
                                    Label::new("Settings")
                                        .size(LabelSize::Large)
                                        .weight(FontWeight::SEMIBOLD),
                                ),
                            )
                            .child(div().px_3().py_1().child(
                                Label::new("Options").size(LabelSize::XSmall).color(Color::Muted),
                            ))
                            .children(navigation),
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
                                    .child(Label::new(page_title).size(LabelSize::Large))
                                    .when_some(unreadable, |view, problem| {
                                        view.child(
                                            Banner::new()
                                                .severity(Severity::Error)
                                                .child(Label::new(problem).size(LabelSize::Small)),
                                        )
                                    })
                                    .when_some(self.problem.clone(), |view, problem| {
                                        view.child(
                                            Banner::new()
                                                .severity(Severity::Error)
                                                .child(Label::new(problem).size(LabelSize::Small)),
                                        )
                                    })
                                    .child(content),
                            ),
                    ),
            )
    }
}

fn setting_label(label: &'static str) -> AnyElement {
    Label::new(label).size(LabelSize::Small).color(Color::Muted).into_any_element()
}

fn setting_value(label: &'static str, value: String) -> AnyElement {
    h_flex()
        .justify_between()
        .gap_4()
        .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
        .child(Label::new(value).size(LabelSize::Small))
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
}
