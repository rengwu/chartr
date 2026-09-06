//! Space switcher, problem notices, and window controls.

use super::*;

impl Zeddy {
    fn space_switcher(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let current = self
            .active
            .as_ref()
            .map(|space| space.read(cx).name().to_owned())
            .unwrap_or_else(|| "No space".to_owned());
        let active_id = self.active.as_ref().map(Entity::entity_id);
        let weak = cx.weak_entity();
        let spaces: Vec<_> = self
            .spaces
            .iter()
            .map(|space| {
                let read = space.read(cx);
                (space.clone(), read.name().to_owned(), read.kind())
            })
            .collect();

        PopupMenu::new("space-switcher")
            .trigger(
                ButtonLike::new("space-switcher-trigger")
                    .aria_label("Current space")
                    .aria_value(current.clone())
                    .selected_style(ButtonStyle::Filled)
                    .child(div().min_w_0().max_w(px(148.)).child(Label::new(current).truncate()))
                    .child(
                        Icon::new(IconName::ChevronUpDown)
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    ),
            )
            .anchor(Anchor::TopLeft)
            .menu(move |window, cx| {
                let weak = weak.clone();
                let spaces = spaces.clone();
                Some(ContextMenu::build_popup(window, cx, move |menu| {
                    let add = weak.clone();
                    let mut menu = menu.entry("New Space…", None, move |window, cx| {
                        let _ = add.update(cx, |this, cx| this.pick_a_folder(window, cx));
                    });

                    let registered: Vec<_> = spaces
                        .iter()
                        .filter(|(_, _, kind)| *kind == SpaceKind::Registered)
                        .collect();
                    if !registered.is_empty() {
                        menu = menu.separator().header("Project Spaces");
                    }
                    for (space, name, _) in registered {
                        let target = space.clone();
                        let select = weak.clone();
                        menu = menu.toggleable_entry(
                            name.clone(),
                            active_id == Some(space.entity_id()),
                            IconPosition::End,
                            None,
                            move |window, cx| {
                                let _ = select.update(cx, |this, cx| {
                                    this.activate(target.clone(), window, cx)
                                });
                            },
                        );
                    }

                    menu = menu.separator();
                    for (space, name, _kind) in
                        spaces.iter().filter(|(_, _, kind)| *kind == SpaceKind::AdHoc)
                    {
                        let target = space.clone();
                        let select = weak.clone();
                        menu = menu.toggleable_entry(
                            name.clone(),
                            active_id == Some(space.entity_id()),
                            IconPosition::End,
                            None,
                            move |window, cx| {
                                let _ = select.update(cx, |this, cx| {
                                    this.activate(target.clone(), window, cx)
                                });
                            },
                        );
                    }
                    menu
                }))
            })
            .into_any_element()
    }

    pub(super) fn visible_space_switcher(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.mode == Mode::Sidebar && !self.show_space_picker {
            gpui::Empty.into_any_element()
        } else {
            self.space_switcher(window, cx)
        }
    }

    fn current_error_keys(&self, cx: &App) -> Vec<ErrorNoticeKey> {
        let mut errors = Vec::new();
        match &self.backend {
            Backend::Recovering(message) => errors.push(ErrorNoticeKey {
                source: "Terminal backend".to_owned(),
                message: message.clone(),
                severity: ErrorSeverity::Warning,
            }),
            Backend::Failed(message) => errors.push(ErrorNoticeKey {
                source: "Terminal backend".to_owned(),
                message: message.clone(),
                severity: ErrorSeverity::Error,
            }),
            Backend::Starting | Backend::Ready => {}
        }
        if let Some(message) = &self.problem {
            errors.push(ErrorNoticeKey {
                source: "Chartr".to_owned(),
                message: message.clone(),
                severity: ErrorSeverity::Warning,
            });
        }
        for space in &self.spaces {
            let space = space.read(cx);
            if let Some(message) = space.problem() {
                errors.push(ErrorNoticeKey {
                    source: format!("Space · {}", space.name()),
                    message: message.to_owned(),
                    severity: ErrorSeverity::Warning,
                });
            }
        }
        errors
    }

    pub(super) fn error_notices(&mut self, cx: &App) -> Vec<ErrorNotice> {
        let current = self.current_error_keys(cx);
        let now = cx.background_executor().now();
        reconcile_error_notices(current, &mut self.error_seen_at, &mut self.dismissed_errors, now)
    }

    fn dismiss_errors(
        &mut self,
        errors: impl IntoIterator<Item = ErrorNoticeKey>,
        cx: &mut Context<Self>,
    ) {
        self.dismissed_errors.extend(errors);
        cx.notify();
    }

    fn error_menu(&self, notices: Vec<ErrorNotice>, cx: &Context<Self>) -> AnyElement {
        let count = notices.len();
        let has_error = notices.iter().any(|notice| notice.key.severity == ErrorSeverity::Error);
        let backend_failed = matches!(self.backend, Backend::Failed(_));
        let weak = cx.weak_entity();
        let label = match count {
            0 => "No problems".to_owned(),
            1 => "1 problem".to_owned(),
            count => format!("{count} problems"),
        };
        PopupMenu::new("error-menu")
            .trigger_with_tooltip(
                IconButton::new("error-menu-trigger", IconName::BellRing)
                    .icon_size(IconSize::Small)
                    .icon_color(if has_error { Color::Error } else { Color::Warning })
                    .aria_label(label.clone())
                    .disabled(notices.is_empty()),
                Tooltip::text(label),
            )
            .anchor(Anchor::TopRight)
            .menu(move |window, cx| {
                if notices.is_empty() {
                    return None;
                }
                let now = cx.background_executor().now();
                let retry = weak.clone();
                let restart = weak.clone();
                let clear = weak.clone();
                let dismiss = weak.clone();
                let menu_notices = notices.clone();
                let clear_keys =
                    menu_notices.iter().map(|notice| notice.key.clone()).collect::<Vec<_>>();
                let notice_count = menu_notices.len();
                Some(ContextMenu::build_popup(window, cx, move |menu| {
                    let mut menu = menu.popup_width(px(520.)).custom_row(move |_, _| {
                        let clear = clear.clone();
                        let clear_keys = clear_keys.clone();
                        let clear_button = Button::new("clear-all-problems", "Clear all")
                            .size(ButtonSize::Compact)
                            .label_size(UI_LABEL_SMALL)
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                let _ = clear.update(cx, |this, cx| {
                                    this.dismiss_errors(clear_keys.clone(), cx)
                                });
                                window.remove_window();
                            })
                            .into_any_element();
                        error_menu_header(notice_count, clear_button)
                    });
                    for (index, notice) in menu_notices.iter().cloned().enumerate() {
                        let dismiss = dismiss.clone();
                        menu = menu.custom_row(move |_, cx| {
                            let dismiss = dismiss.clone();
                            let key = notice.key.clone();
                            let dismiss_button =
                                IconButton::new(("dismiss-problem", index), IconName::Close)
                                    .icon_size(IconSize::XSmall)
                                    .aria_label("Dismiss problem")
                                    .tooltip(Tooltip::text("Dismiss problem"))
                                    .on_click(move |_, window, cx| {
                                        cx.stop_propagation();
                                        let _ = dismiss.update(cx, |this, cx| {
                                            this.dismiss_errors([key.clone()], cx)
                                        });
                                        window.remove_window();
                                    })
                                    .into_any_element();
                            error_notice_row(&notice, now, dismiss_button, cx)
                        });
                    }
                    if backend_failed {
                        menu = menu
                            .separator()
                            .entry("Retry terminal backend", None, move |_, cx| {
                                let _ = retry.update(cx, |this, cx| this.retry_backend(false, cx));
                            })
                            .entry("Restart terminal backend", None, move |window, cx| {
                                let _ = restart.update(cx, |this, cx| {
                                    this.request_backend_restart(window, cx)
                                });
                            });
                    }
                    menu
                }))
            })
            .into_any_element()
    }

    pub(super) fn chrome_end_controls(
        &self,
        on: chrome::Emit,
        notices: Vec<ErrorNotice>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let has_notices = !notices.is_empty();
        h_flex()
            .gap_1()
            .child(self.presentation_toggle(on.clone()))
            .when(has_notices, |controls| controls.child(self.error_menu(notices, cx)))
            .child(self.settings_button(on))
            .into_any_element()
    }

    fn presentation_toggle(&self, on: chrome::Emit) -> AnyElement {
        let use_sidebar = on.clone();
        let use_tabs = on;
        SegmentedControl::new(
            "Session list presentation",
            [
                SegmentedControlOption::new(
                    "presentation-sidebar",
                    "Sidebar",
                    self.mode == Mode::Sidebar,
                    move |_, window, cx| use_sidebar(Action::SwitchToSidebar, window, cx),
                ),
                SegmentedControlOption::new(
                    "presentation-tabs",
                    "Tabbed",
                    self.mode == Mode::Tabs,
                    move |_, window, cx| use_tabs(Action::SwitchToTabs, window, cx),
                ),
            ],
        )
        .into_any_element()
    }

    pub(super) fn settings_button(&self, on: chrome::Emit) -> AnyElement {
        IconButton::new("open-settings", IconName::Settings)
            .icon_size(IconSize::Small)
            .aria_label("Settings")
            .tooltip(Tooltip::text("Settings"))
            .on_click(move |_, window, cx| on(Action::OpenSettings, window, cx))
            .into_any_element()
    }

    pub(super) fn workspace_title_bar(
        &self,
        controls: Option<(AnyElement, AnyElement)>,
        window: &Window,
        cx: &App,
    ) -> AnyElement {
        if !cfg!(target_os = "macos") {
            return self.title_bar.clone().into_any_element();
        }

        let colors = cx.theme().colors();
        let window_active = window.is_window_active();
        let mut overlays = Vec::with_capacity(4);
        overlays.push(
            div()
                .absolute()
                .top_0()
                .right_0()
                .bottom(if self.mode == Mode::Tabs { px(0.) } else { px(1.) })
                .left_0()
                .bg(colors.panel_background)
                .into_any_element(),
        );
        if self.mode == Mode::Sidebar {
            overlays.push(
                div()
                    .absolute()
                    .left_0()
                    .bottom_0()
                    .w(px(self.sidebar_width - 1.))
                    .h(px(1.))
                    .bg(colors.panel_background)
                    .into_any_element(),
            );
        }
        if let Some((space_switcher, view_menu)) = controls {
            overlays.push(
                h_flex()
                    .absolute()
                    // Clear the native macOS traffic-light cluster.
                    .left(px(78.))
                    .top_0()
                    .h(px(crate::title_bar::HEIGHT))
                    .max_w(px(200.))
                    .when(!window_active, |controls| controls.opacity(0.65))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(space_switcher)
                    .into_any_element(),
            );
            overlays.push(
                h_flex()
                    .absolute()
                    .right(px(6.))
                    .top_0()
                    .h(px(crate::title_bar::HEIGHT))
                    .when(!window_active, |controls| controls.opacity(0.65))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(view_menu)
                    .into_any_element(),
            );
        }

        div()
            .id("workspace-title-bar-with-controls")
            .relative()
            .w_full()
            .h(px(crate::title_bar::HEIGHT))
            .flex_none()
            .child(self.title_bar.clone())
            .children(overlays)
            .into_any_element()
    }

    pub(super) fn new_item_button(&self, cx: &Context<Self>) -> AnyElement {
        let weak = cx.weak_entity();
        let button = chrome::new_item_button("new-item")
            .icon_color(Color::Muted)
            .aria_label("New terminal session")
            .tooltip(Tooltip::text("New terminal session"))
            .on_click(move |_, window, cx| {
                let _ = weak.update(cx, |this, cx| this.act(Action::New, window, cx));
            })
            .into_any_element();
        chrome::new_item_drag_handle(
            "new-item",
            self.active.as_ref().map(Entity::entity_id),
            chrome::NewItemKind::Terminal,
            button,
        )
    }

    pub(super) fn new_plugin_pane_button(&self, cx: &Context<Self>) -> AnyElement {
        let weak = cx.weak_entity();
        let button = chrome::new_plugin_pane_button_with_color(
            "new-plugin-pane",
            IconSize::Small,
            Color::Muted,
        )
        .on_click(move |_, window, cx| {
            let _ = weak.update(cx, |this, cx| {
                this.act(Action::NewPluginPane, window, cx);
            });
        })
        .into_any_element();
        chrome::new_item_drag_handle(
            "new-plugin-pane",
            self.active.as_ref().map(Entity::entity_id),
            chrome::NewItemKind::Plugin,
            button,
        )
    }
}
