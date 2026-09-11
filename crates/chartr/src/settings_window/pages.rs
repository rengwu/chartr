//! General, terminal, and keyboard settings pages.

use super::*;

impl SettingsWindow {
    pub(super) fn general_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let settings = self.settings(cx);
        let terminate = settings.terminate_sessions_on_exit;
        let middle_click_closes_tab = settings.middle_click_closes_tab;
        let middle_click_closes_sidebar_tab = settings.middle_click_closes_sidebar_tab;
        let show_view_mode_picker = settings.show_view_mode_picker;
        let show_status_bar = settings.show_status_bar;
        let status_bar_setting = cx.weak_entity();
        let mode = self.mode(cx);
        let show_space_picker = self.show_space_picker(cx).unwrap_or(true);
        let runtime_available = mode.is_some();
        let mode = mode.unwrap_or_default();
        let terminate_setting = cx.weak_entity();
        let middle_click_setting = cx.weak_entity();
        let sidebar_middle_click_setting = cx.weak_entity();
        let space_picker_setting = cx.weak_entity();
        let view_mode_picker_setting = cx.weak_entity();
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
        let mut fields = vec![
            setting_field(
                "Terminate sessions on exit",
                "End running sessions when chartr exits instead of leaving them detached.",
                Switch::new("terminate-sessions-on-exit", terminate.into())
                    .tab_index(0isize)
                    .aria_label("Terminate sessions on exit")
                    .aria_description(
                        "End running sessions when chartr exits instead of leaving them detached.",
                    )
                    .on_click(move |state, _, cx| {
                        let terminate = state.selected();
                        let _ = terminate_setting
                            .update(cx, |this, cx| this.set_terminate_on_exit(terminate, cx));
                    }),
            ),
            setting_field(
                "Middle click to close tab",
                "Close tabs in tabbed mode and pane tab bars with the middle mouse button.",
                Switch::new("middle-click-closes-tab", middle_click_closes_tab.into())
                    .tab_index(0isize)
                    .aria_label("Middle click to close tab")
                    .aria_description(
                        "Close tabs in tabbed mode and pane tab bars with the middle mouse button.",
                    )
                    .on_click(move |state, _, cx| {
                        let enabled = state.selected();
                        let _ = middle_click_setting
                            .update(cx, |this, cx| this.set_middle_click_closes_tab(enabled, cx));
                    }),
            ),
        ];
        if middle_click_closes_tab {
            fields.push(setting_field(
                "Middle click to close tab on sidebar",
                "Also close tabs from sidebar session rows with the middle mouse button.",
                Switch::new(
                    "middle-click-closes-sidebar-tab",
                    middle_click_closes_sidebar_tab.into(),
                )
                .tab_index(0isize)
                .aria_label("Middle click to close tab on sidebar")
                .aria_description(
                    "Also close tabs from sidebar session rows with the middle mouse button.",
                )
                .on_click(move |state, _, cx| {
                    let enabled = state.selected();
                    let _ = sidebar_middle_click_setting.update(cx, |this, cx| {
                        this.set_middle_click_closes_sidebar_tab(enabled, cx)
                    });
                }),
            ));
        }
        fields.push(setting_field(
            "Show space picker in sidebar mode",
            "Show the current space selector in the title bar while using the sidebar.",
            Switch::new("show-space-picker-in-sidebar-mode", show_space_picker.into())
                .tab_index(0isize)
                .disabled(!runtime_available)
                .aria_label("Show space picker in sidebar mode")
                .aria_description(
                    "Show the current space selector in the title bar while using the sidebar.",
                )
                .on_click(move |state, _, cx| {
                    let show = state.selected();
                    let _ = space_picker_setting
                        .update(cx, |this, cx| this.set_show_space_picker(show, cx));
                }),
        ));
        fields.push(setting_field(
            "Show view mode picker",
            "Show the Sidebar/Tabbed toggle in the workspace title bar.",
            Switch::new("show-view-mode-picker", show_view_mode_picker.into())
                .tab_index(0isize)
                .aria_label("Show view mode picker")
                .aria_description("Show the Sidebar/Tabbed toggle in the workspace title bar.")
                .on_click(move |state, _, cx| {
                    let show = state.selected();
                    let _ = view_mode_picker_setting
                        .update(cx, |this, cx| this.set_show_view_mode_picker(show, cx));
                }),
        ));
        fields.push(setting_field(
            "Show status bar",
            "Show persistent services and running sessions at the bottom of the workspace.",
            Switch::new("show-status-bar", show_status_bar.into())
                .tab_index(0isize)
                .aria_label("Show status bar")
                .on_click(move |state, _, cx| {
                    let show = state.selected();
                    let _ = status_bar_setting
                        .update(cx, |this, cx| this.set_show_status_bar(show, cx));
                }),
        ));
        fields.push(setting_field(
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
        ));
        settings_fields(fields, cx.theme().colors().border_variant)
    }

    pub(super) fn terminal_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
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
                settings_button("terminal-font-family", settings.terminal_font_family)
                    .end_icon(Icon::new(IconName::ChevronDown)),
            )
            .anchor(Anchor::BottomLeft)
            .menu(move |window, cx| {
                let font = font.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    fonts::TERMINAL_FONTS.iter().fold(menu, |menu, terminal_font| {
                        let family = terminal_font.family;
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
                    settings_button("choose-free-sessions-directory", directory)
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
                    settings_button("settings-retry-backend", "Retry")
                        .disabled(!runtime_available)
                        .on_click(retry),
                ),
                setting_field(
                    "Restart backend",
                    "Stop and start the backend process for this workspace.",
                    settings_button("settings-restart-backend", "Restart")
                        .disabled(!runtime_available)
                        .on_click(restart),
                ),
            ],
            cx.theme().colors().border_variant,
        )
    }

    pub(super) fn hotkeys_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
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
                    settings_button(
                        format!("record-hotkey-{}", action.id()),
                        if recording == Some(action) {
                            "Press shortcut…".to_owned()
                        } else {
                            keymap.shortcut_label(action).to_owned()
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
                view.child(chartr_plugin::ui::notice(problem, true))
            })
            .child(table)
            .into_any_element()
    }
}
