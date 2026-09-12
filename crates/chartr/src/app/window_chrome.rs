//! Space switcher, problem notices, and window controls.

use super::*;

pub(super) const TITLE_CONTROLS_LEFT: f32 = 78.;
pub(super) const TITLE_CONTROLS_RIGHT: f32 = 6.;
pub(super) const SPACE_SWITCHER_MAX_WIDTH: f32 = 200.;

// Measure a separate tree so the displayed controls retain their normal layout
// lifecycle. Max-content measurement includes the current font, scale and notices.
fn chrome_controls_width(content: AnyElement, window: &mut Window, cx: &mut App) -> Pixels {
    div()
        .id("chrome-controls-size-probe")
        .font(theme::theme_settings(cx).ui_font(cx).clone())
        .child(content)
        .into_any_element()
        .layout_as_root(
            gpui::size(gpui::AvailableSpace::MaxContent, gpui::AvailableSpace::MaxContent),
            window,
            cx,
        )
        .width
}

impl WorkspaceWindow {
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
                    if !spaces.is_empty() {
                        menu = menu.separator();
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
                                    this.activate(target.clone(), window, cx);
                                    cx.notify();
                                });
                            },
                        );
                    }

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
                                    this.activate(target.clone(), window, cx);
                                    cx.notify();
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
        visibility: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if visibility <= 0. {
            gpui::Empty.into_any_element()
        } else {
            // Keep the picker mounted throughout its exit, including reversals.
            // The shared mode transition also handles reduced motion.
            div().opacity(visibility).child(self.space_switcher(window, cx)).into_any_element()
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
                source: "chartr".to_owned(),
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
        available_width: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let build = |show_picker, cx: &Context<Self>| {
            h_flex()
                .flex_none()
                .gap_1()
                .when(show_picker, |controls| controls.child(self.presentation_toggle(on.clone())))
                .when(!notices.is_empty(), |controls| {
                    controls.child(self.error_menu(notices.clone(), cx))
                })
                .child(self.settings_button(on.clone()))
                .into_any_element()
        };
        let show_picker = self.settings.resolved().show_view_mode_picker
            && chrome_controls_width(build(true, cx), window, cx) <= available_width;
        build(show_picker, cx)
    }

    fn presentation_toggle(&self, on: chrome::Emit) -> AnyElement {
        let use_sidebar = on.clone();
        let use_tabs = on.clone();
        let use_conversations = on;
        SegmentedControl::new(
            "Session list presentation",
            [
                SegmentedControlOption::new(
                    "presentation-tabs",
                    "Tabs",
                    self.mode == Mode::Tabs,
                    move |_, window, cx| use_tabs(Action::SwitchToTabs, window, cx),
                ),
                SegmentedControlOption::new(
                    "presentation-sidebar",
                    "Spaces",
                    self.mode == Mode::Sidebar,
                    move |_, window, cx| use_sidebar(Action::SwitchToSidebar, window, cx),
                ),
                SegmentedControlOption::new(
                    "presentation-conversations",
                    "Chats",
                    self.mode == Mode::Inbox,
                    move |_, window, cx| {
                        use_conversations(Action::SwitchToConversations, window, cx)
                    },
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
        visibility: crate::mode::ChromeVisibility,
        window: &Window,
        cx: &App,
    ) -> (AnyElement, AnyElement) {
        if !cfg!(target_os = "macos") {
            return (self.title_bar.clone().into_any_element(), gpui::Empty.into_any_element());
        }

        let background = cx.theme().colors().panel_background;
        let window_active = window.is_window_active();
        let title_bar = div()
            .id("workspace-title-bar-background")
            .relative()
            .w_full()
            .h(px(crate::title_bar::HEIGHT))
            // Pull the tab strip into the title bar's spare bottom spacing.
            .mb(px(-4. * visibility.tabs))
            .flex_none()
            .child(self.title_bar.clone())
            .child(div().absolute().inset_0().bg(background))
            .into_any_element();

        // Paint this after the body: tabs sit beneath the stationary gradient,
        // then title-bar controls sit above both. The canvas has no hitbox and
        // only covers the upper row, leaving the rest of the tab strip crisp.
        let mut foreground = div()
            .id("workspace-title-bar-with-controls")
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h(px(crate::title_bar::HEIGHT))
            .when(visibility.tabs > 0. && visibility.tabs < 1., |bar| {
                bar.child(
                    gpui::canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| {
                            window.paint_quad(gpui::fill(
                                bounds,
                                gpui::linear_gradient(
                                    180.,
                                    gpui::linear_color_stop(background, 0.),
                                    gpui::linear_color_stop(background.opacity(0.), 1.),
                                ),
                            ));
                        },
                    )
                    .absolute()
                    .inset_0(),
                )
            });
        if let Some((space_switcher, view_menu)) = controls {
            foreground = foreground
                .child(
                    h_flex()
                        .absolute()
                        // Clear the native macOS traffic-light cluster.
                        .left(px(TITLE_CONTROLS_LEFT))
                        .top_0()
                        .h(px(crate::title_bar::HEIGHT))
                        .max_w(px(SPACE_SWITCHER_MAX_WIDTH))
                        .when(!window_active, |controls| controls.opacity(0.65))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(space_switcher),
                )
                .child(
                    h_flex()
                        .absolute()
                        .right(px(TITLE_CONTROLS_RIGHT))
                        .top_0()
                        .h(px(crate::title_bar::HEIGHT))
                        .when(!window_active, |controls| controls.opacity(0.65))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(view_menu),
                );
        }

        (title_bar, foreground.into_any_element())
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

#[cfg(test)]
mod responsive_controls_tests {
    use super::*;
    use gpui::TestAppContext;

    struct Harness {
        width: Pixels,
        rem_size: Pixels,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(self.rem_size);
            let controls = |picker| {
                h_flex()
                    .flex_none()
                    .gap_1()
                    .when(picker, |row| {
                        row.child(div().debug_selector(|| "VIEW_PICKER".into()).child(
                            SegmentedControl::new(
                                "Session list presentation",
                                ["Tabs", "Spaces", "Chats"].map(|label| {
                                    SegmentedControlOption::new(
                                        label,
                                        label,
                                        label == "Tabs",
                                        |_, _, _| {},
                                    )
                                }),
                            ),
                        ))
                    })
                    .child(
                        IconButton::new("settings", IconName::Settings).icon_size(IconSize::Small),
                    )
                    .into_any_element()
            };
            let width = chrome_controls_width(controls(true), window, cx);
            div().w(self.width).child(controls(width <= self.width))
        }
    }

    #[gpui::test]
    fn picker_hides_and_returns_as_available_width_and_scale_change(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness { width: px(500.), rem_size: px(14.) });
        for (width, rem_size, visible) in [
            (500., 14., true),
            (80., 14., false),
            (500., 14., true),
            (500., 42., false),
            (500., 14., true),
        ] {
            view.update(cx, |view, cx| {
                view.width = px(width);
                view.rem_size = px(rem_size);
                cx.notify();
            });
            cx.run_until_parked();
            assert_eq!(cx.debug_bounds("VIEW_PICKER").is_some(), visible);
        }
    }
}
