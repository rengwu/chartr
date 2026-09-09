//! chartr's singleton, application-wide Settings window.
//!
//! This follows Zed's `SettingsWindow` boundary: opening Settings focuses the
//! existing app-wide window, global settings notify every workspace live, and
//! the originating workspace is retained only for operations that truly need
//! runtime state (the backend and plugin catalog).

mod appearance;
mod pages;
mod plugins;

use gpui::{
    Anchor, AnyView, App, Bounds, ClickEvent, Context, DefiniteLength, ElementId, Entity,
    FocusHandle, Focusable, FontWeight, Hsla, KeyBinding, PathPromptOptions, Render, Role,
    SharedString, TextAlign, WeakEntity, Window, WindowBounds, WindowHandle, WindowOptions,
    actions, px, size,
};
use ui::{
    Banner, Button, ColumnWidthConfig, DropdownMenu, DropdownStyle, Icon, IconButton, PopoverMenu,
    RedistributableColumnsState, Severity, Switch, Table, TableResizeBehavior, Tooltip, prelude::*,
};

use crate::{
    app::WorkspaceWindow,
    components::{
        ContextMenu, FORM_CONTROL_SIZE, SegmentedControl, SegmentedControlOption, form_button,
        input_field, selection_list, selection_row,
    },
    fonts::{self, Fonts, UI_LABEL_DEFAULT, UI_LABEL_LARGE, UI_LABEL_SMALL, UI_TEXT_DEFAULT},
    keymap::{KeymapAction, KeymapStore},
    mode::Mode,
    settings::{
        self, AppearanceContent, GeneralContent, ResolvedSettings, SettingsPage, SettingsStore,
        TerminalContent, ThemeMode,
    },
    text_input::{InputEvent, TextInput},
};

actions!(chartr_settings_window, [Close]);

const SETTINGS_WINDOW_MIN_WIDTH: f32 = 720.;
const SETTINGS_SIDEBAR_WIDTH: f32 = 200.;
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
    cx.bind_keys([KeyBinding::new("cmd-w", Close, Some("chartrSettings"))]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-w", Close, Some("chartrSettings"))]);

    let key = keymap.key(KeymapAction::OpenSettings);
    if !key.is_empty() {
        cx.bind_keys([KeyBinding::new(
            key,
            crate::actions::settings::Open,
            Some("chartrSettings"),
        )]);
    }
}

/// Focus Zed-style: one Settings window for the application, never one per
/// workspace. Reopening also retargets workspace-scoped controls to the most
/// recent caller.
pub fn open(
    original_window: WindowHandle<WorkspaceWindow>,
    original: WeakEntity<WorkspaceWindow>,
    cx: &mut App,
) {
    open_with_origin(Some(original_window), original, cx);
}

pub fn open_plugin(
    original_window: WindowHandle<WorkspaceWindow>,
    original: WeakEntity<WorkspaceWindow>,
    plugin: Option<String>,
    cx: &mut App,
) {
    open(original_window, original, cx);
    cx.defer(move |cx| {
        let existing =
            cx.windows().into_iter().find_map(|window| window.downcast::<SettingsWindow>());
        if let Some(existing) = existing {
            let _ = existing.update(cx, |settings, window, cx| {
                settings.page = SettingsPage::Plugins;
                settings.plugin_settings = None;
                settings.plugin_information = None;
                if let Some(plugin) = plugin {
                    settings.open_plugin_settings(plugin, window, cx);
                }
                cx.notify();
            });
        }
    });
}

fn open_with_origin(
    original_window: Option<WindowHandle<WorkspaceWindow>>,
    original: WeakEntity<WorkspaceWindow>,
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
                titlebar: Some(crate::title_bar::options("chartr — Settings")),
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
            eprintln!("chartr could not open Settings: {error}");
        }
    });
}

pub struct SettingsWindow {
    original_window: Option<WindowHandle<WorkspaceWindow>>,
    original: WeakEntity<WorkspaceWindow>,
    page: SettingsPage,
    plugin_settings: Option<(String, Option<AnyView>)>,
    plugin_information: Option<String>,
    git_install_open: bool,
    git_url_input: Entity<TextInput>,
    plugin_operation: Option<String>,
    plugin_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    plugin_restart_required: bool,
    recording_keymap: Option<KeymapAction>,
    ui_font_size_input: Entity<TextInput>,
    terminal_font_size_input: Entity<TextInput>,
    hotkey_widths: Entity<RedistributableColumnsState>,
    title_bar: Entity<crate::title_bar::TitleBar>,
    focus: FocusHandle,
    problem: Option<String>,
}

impl SettingsWindow {
    fn new(
        original_window: Option<WindowHandle<WorkspaceWindow>>,
        original: WeakEntity<WorkspaceWindow>,
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
        let git_url_input = cx.new(|cx| TextInput::new("https://github.com/owner/plugin.git", cx));
        cx.on_release(|this, _| {
            if let Some(cancel) = &this.plugin_cancel {
                cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        })
        .detach();
        cx.observe_global_in::<SettingsStore>(window, |this, window, cx| {
            this.sync_font_size_inputs(window, cx);
            if this.plugin_settings.as_ref().is_some_and(|(id, _)| {
                let settings = cx.global::<SettingsStore>().resolved().plugin(id);
                !settings.enabled || settings.uninstalled
            }) {
                this.plugin_settings = None;
            }
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
            plugin_information: None,
            git_install_open: false,
            git_url_input,
            plugin_operation: None,
            plugin_cancel: None,
            plugin_restart_required: crate::plugin_installer::has_pending(
                &crate::app::plugin_paths(),
            ),
            recording_keymap: None,
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
                    fonts::install(&resolved, cx);
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

    fn set_middle_click_closes_tab(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.update_settings(
            |content| {
                content
                    .general
                    .get_or_insert_with(GeneralContent::default)
                    .middle_click_closes_tab = Some(enabled);
            },
            false,
            false,
            cx,
        );
    }

    fn set_middle_click_closes_sidebar_tab(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.update_settings(
            |content| {
                content
                    .general
                    .get_or_insert_with(GeneralContent::default)
                    .middle_click_closes_sidebar_tab = Some(enabled);
            },
            false,
            false,
            cx,
        );
    }

    fn set_show_status_bar(&mut self, show: bool, cx: &mut Context<Self>) {
        self.update_settings(
            |content| {
                content.general.get_or_insert_default().show_status_bar = Some(show);
            },
            false,
            false,
            cx,
        );
    }

    fn set_show_view_mode_picker(&mut self, show: bool, cx: &mut Context<Self>) {
        self.update_settings(
            |content| {
                content.general.get_or_insert_with(GeneralContent::default).show_view_mode_picker =
                    Some(show);
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
            true,
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
            true,
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
        self.plugin_information = None;
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
            let previous_key = cx.global::<KeymapStore>().key(action).to_owned();
            match cx.update_global::<KeymapStore, _>(|keymap, _| keymap.set(action, key)) {
                Ok(()) => {
                    let new_key = cx.global::<KeymapStore>().key(action).to_owned();
                    crate::actions::rebind(action, &previous_key, &new_key, cx);
                    self.recording_keymap = None;
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

    fn backend_label(&self, cx: &App) -> String {
        self.original
            .upgrade()
            .map(|origin| origin.read(cx).settings_backend_label())
            .unwrap_or_else(|| "Workspace unavailable".to_owned())
    }

    fn mode(&self, cx: &App) -> Option<Mode> {
        self.original.upgrade().map(|origin| origin.read(cx).settings_mode())
    }

    fn show_space_picker(&self, cx: &App) -> Option<bool> {
        self.original.upgrade().map(|origin| origin.read(cx).settings_show_space_picker())
    }

    fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if let Some(origin) = self.original.upgrade() {
            origin.update(cx, |origin, cx| origin.settings_set_mode(mode, cx));
            self.problem = None;
        } else {
            self.problem = Some("The originating chartr window is no longer available.".into());
        }
        cx.notify();
    }

    fn set_show_space_picker(&mut self, show: bool, cx: &mut Context<Self>) {
        if let Some(origin) = self.original.upgrade() {
            origin.update(cx, |origin, cx| origin.settings_set_show_space_picker(show, cx));
            self.problem = None;
        } else {
            self.problem = Some("The originating chartr window is no longer available.".into());
        }
        cx.notify();
    }

    fn retry_backend(&mut self, cx: &mut Context<Self>) {
        if let Some(origin) = self.original.upgrade() {
            origin.update(cx, |origin, cx| origin.settings_retry_backend(cx));
            self.problem = None;
        } else {
            self.problem = Some("The originating chartr window is no longer available.".into());
        }
        cx.notify();
    }

    fn restart_backend(&mut self, cx: &mut Context<Self>) {
        let Some(origin) = self.original_window else {
            self.problem = Some("The originating chartr window is no longer available.".into());
            cx.notify();
            return;
        };
        if origin
            .update(cx, |origin, window, cx| origin.settings_restart_backend(window, cx))
            .is_err()
        {
            self.problem = Some("The originating chartr window is no longer available.".into());
        } else {
            self.problem = None;
        }
        cx.notify();
    }

    fn settings(&self, cx: &App) -> ResolvedSettings {
        cx.global::<SettingsStore>().resolved().clone()
    }

    fn content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if self.page == SettingsPage::Plugins && self.plugin_information.is_some() {
            return self.plugin_information_page(cx);
        }
        if self.page == SettingsPage::Plugins && self.plugin_settings.is_some() {
            return self.plugin_configuration_page(window, cx);
        }
        match self.page {
            SettingsPage::General => self.general_page(cx),
            SettingsPage::Appearance => self.appearance_page(window, cx),
            SettingsPage::Terminal => self.terminal_page(cx),
            SettingsPage::Hotkeys => self.hotkeys_page(cx),
            SettingsPage::Plugins => self.plugins_page(window, cx),
        }
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
                            this.plugin_information = None;
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
        let show_page_title = self.page != SettingsPage::Plugins
            || (self.plugin_settings.is_none() && self.plugin_information.is_none());
        let content = self.content(window, cx);

        div()
            .id("settings-window")
            .key_context("chartrSettings")
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
                            .w(px(SETTINGS_SIDEBAR_WIDTH))
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
                                    .when(show_page_title, |view| {
                                        view.child(Label::new(page_title).size(UI_LABEL_LARGE))
                                    })
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

/// Settings actions keep a visible boundary at rest so they cannot be mistaken for labels.
fn settings_button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    form_button(id, label)
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
        .h(FORM_CONTROL_SIZE.rems())
        .flex_none()
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
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            let settings = SettingsStore::bare();
            fonts::install(settings.resolved(), cx);
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
                .update(cx, |settings, _, cx| {
                    settings.set_show_status_bar(false, cx);
                    settings.set_terminate_on_exit(true, cx);
                    settings.set_middle_click_closes_tab(true, cx);
                    settings.set_middle_click_closes_sidebar_tab(true, cx);
                })
                .unwrap();
            let resolved = cx.global::<SettingsStore>().resolved();
            assert!(!resolved.show_status_bar);
            assert!(resolved.terminate_sessions_on_exit);
            assert!(resolved.middle_click_closes_tab);
            assert!(resolved.middle_click_closes_sidebar_tab);
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
            assert_eq!(theme::theme_settings(cx).buffer_font_size(cx), px(16.));
            assert_eq!(
                <terminal::terminal_settings::TerminalSettings as ::settings::Settings>::get_global(
                    cx,
                )
                .font_size,
                Some(px(16.))
            );
        });
    }
    #[gpui::test]
    fn closing_settings_cancels_plugin_preparation(cx: &mut TestAppContext) {
        init_test(cx);
        cx.update(|cx| open_with_origin(None, WeakEntity::new_invalid(), cx));
        cx.run_until_parked();
        let settings = cx
            .windows()
            .into_iter()
            .find_map(|window| window.downcast::<SettingsWindow>())
            .unwrap();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        cx.update(|cx| {
            settings
                .update(cx, |settings, window, _| {
                    settings.plugin_cancel = Some(cancel.clone());
                    window.remove_window();
                })
                .unwrap();
        });
        cx.run_until_parked();
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
    }
}
