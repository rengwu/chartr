//! Plugin launcher, instance creation, restoration, and cloning.

use super::*;
use ui::{ScrollAxes, Scrollbars, WithScrollbar};

const LAUNCHER_CARD_WIDTH_REMS: f32 = 15.5;

#[derive(IntoElement)]
struct LauncherSurface {
    id: gpui::SharedString,
    content: AnyElement,
}

impl gpui::RenderOnce for LauncherSurface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let scroll = window.use_keyed_state(self.id.clone(), cx, |_, _| gpui::ScrollHandle::new());
        let scroll = scroll.read(cx).clone();
        let [thumb, hovered_thumb, active_thumb] =
            crate::components::scrollbar_thumb_colors(cx.theme().colors());
        v_flex()
            .relative()
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .id(self.id.clone())
                    .size_full()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(
                        v_flex()
                            .w_full()
                            // Three capped cards, two gaps, and the outer padding.
                            .max_w(rems(LAUNCHER_CARD_WIDTH_REMS * 3. + 1. + 2.5))
                            .flex_none()
                            .mx_auto()
                            .my_auto()
                            .p_5()
                            .child(self.content),
                    ),
            )
            .custom_scrollbars(
                Scrollbars::always_visible(ScrollAxes::Vertical)
                    .id(format!("{}-scrollbar", self.id))
                    .thumb_colors(thumb, hovered_thumb, active_thumb)
                    .tracked_scroll_handle(&scroll)
                    .notify_content(),
                window,
                cx,
            )
    }
}

fn launcher_cards(cards: Vec<AnyElement>) -> gpui::Div {
    let columns = match cards.len() {
        0 | 1 => 1.,
        2..=4 => 2.,
        _ => 3.,
    };
    h_flex()
        .w_full()
        .max_w(rems(LAUNCHER_CARD_WIDTH_REMS * columns + 0.5 * (columns - 1.)))
        .mx_auto()
        .flex_wrap()
        .justify_start()
        .items_start()
        .content_start()
        .gap_2()
        .children(cards.into_iter().map(|card| {
            div().w(rems(LAUNCHER_CARD_WIDTH_REMS)).max_w(relative(1.)).flex_none().child(card)
        }))
}

#[cfg(test)]
mod launcher_layout_tests {
    use super::*;

    fn test_cards(count: usize) -> Vec<AnyElement> {
        (0..count)
            .map(|index| {
                div()
                    .w_full()
                    .h(rems(5.5))
                    .debug_selector(move || format!("CARD-{index}"))
                    .into_any_element()
            })
            .collect()
    }

    struct LauncherGrid {
        count: usize,
    }
    impl Render for LauncherGrid {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            v_flex().w_full().child(launcher_cards(test_cards(self.count)))
        }
    }

    #[gpui::test]
    fn launcher_cards_keep_incomplete_rows_on_the_grid_and_respect_the_size_cap(
        cx: &mut gpui::TestAppContext,
    ) {
        let (view, cx) = cx.add_window_view(|_, _| LauncherGrid { count: 5 });
        for (width, columns) in [(665., 3), (600., 2), (441., 2), (400., 1), (217., 1), (160., 1)] {
            cx.simulate_resize(gpui::size(px(width), px(600.)));
            cx.run_until_parked();
            let first = cx.debug_bounds("CARD-0").unwrap();
            assert_eq!(first.size.width, px(width.min(217.)));
            assert_eq!(first.size.height, px(77.));
            for (index, selector) in
                ["CARD-0", "CARD-1", "CARD-2", "CARD-3", "CARD-4"].into_iter().enumerate()
            {
                let card = cx.debug_bounds(selector).unwrap();
                assert_eq!(card.size, first.size);
                assert!(card.left() >= px(0.) && card.right() <= px(width));
                assert_eq!(
                    card.left(),
                    (first.size.width + px(7.)) * (index % columns) as f32,
                    "each wrapped row must keep the same columns at {width}px",
                );
                assert_eq!(card.top(), px((index / columns) as f32 * 84.));
            }
        }
        cx.simulate_resize(gpui::size(px(700.), px(600.)));
        for count in 1..=6 {
            view.update(cx, |view, cx| {
                view.count = count;
                cx.notify();
            });
            cx.run_until_parked();
            let columns = match count {
                1 => 1,
                2..=4 => 2,
                _ => 3,
            };
            let grid_width = 217. * columns as f32 + 7. * (columns - 1) as f32;
            let grid_left = (700. - grid_width) / 2.;
            for (index, selector) in ["CARD-0", "CARD-1", "CARD-2", "CARD-3", "CARD-4", "CARD-5"]
                .into_iter()
                .take(count)
                .enumerate()
            {
                let card = cx.debug_bounds(selector).unwrap();
                assert_eq!(card.size, gpui::size(px(217.), px(77.)));
                assert_eq!(card.top(), px((index / columns) as f32 * 84.));
                assert_eq!(card.left(), px(grid_left + (index % columns) as f32 * 224.));
            }
        }
    }

    struct LauncherPane;
    impl Render for LauncherPane {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            LauncherSurface {
                id: "test-launcher-pane".into(),
                content: launcher_cards(test_cards(4)).into_any_element(),
            }
        }
    }

    fn scrollbar_thumb(cx: &mut gpui::VisualTestContext) -> Option<gpui::Point<Pixels>> {
        cx.update(|window, cx| {
            let [color, _, _] = crate::components::scrollbar_thumb_colors(cx.theme().colors());
            let scale = window.scale_factor();
            window.painted_quads().into_iter().find_map(|quad| {
                (quad.background == color.into()).then(|| {
                    let center = quad.bounds.center();
                    gpui::point(px(center.x.as_f32() / scale), px(center.y.as_f32() / scale))
                })
            })
        })
    }

    #[gpui::test]
    fn short_launcher_panes_scroll_without_losing_the_first_or_last_card(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
        });
        let (_, cx) = cx.add_window_view(|_, _| LauncherPane);
        cx.simulate_resize(gpui::size(px(300.), px(600.)));
        cx.run_until_parked();
        let first = cx.debug_bounds("CARD-0").unwrap();
        let last = cx.debug_bounds("CARD-3").unwrap();
        assert_eq!(first.left(), last.left(), "wrapped cards stay on the first grid column");
        assert_eq!(first.top(), px(600.) - last.bottom(), "center vertically when content fits");
        assert!(scrollbar_thumb(cx).is_none());

        cx.simulate_resize(gpui::size(px(300.), px(200.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("CARD-0").unwrap().top() >= px(0.));
        let thumb = scrollbar_thumb(cx).expect("overflow must show the shared scrollbar");
        let bottom = gpui::point(thumb.x, px(195.));
        cx.simulate_mouse_down(thumb, gpui::MouseButton::Left, gpui::Modifiers::none());
        cx.simulate_mouse_move(bottom, gpui::MouseButton::Left, gpui::Modifiers::none());
        cx.simulate_mouse_up(bottom, gpui::MouseButton::Left, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.debug_bounds("CARD-0").unwrap().top() < px(0.));
        assert!(cx.debug_bounds("CARD-3").unwrap().bottom() <= px(200.));

        cx.simulate_resize(gpui::size(px(300.), px(600.)));
        cx.run_until_parked();
        assert_eq!(cx.debug_bounds("CARD-0").unwrap(), first);
        assert_eq!(cx.debug_bounds("CARD-3").unwrap(), last);
        assert!(scrollbar_thumb(cx).is_none());
    }
}

impl WorkspaceWindow {
    /// Construct every plugin instance through the same permission and host wiring.
    fn build_plugin_view(
        &mut self,
        space: &Entity<Space>,
        key: &chartr_plugin::PaneKey,
        item: crate::workspace::ItemId,
        bound_session: Option<&chartr_herdr::PaneId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<PluginView> {
        let project =
            (space.read(cx).kind() == SpaceKind::Registered).then(|| space.read(cx).path().clone());
        let origin = cx.weak_entity();
        let instance = InstanceContext {
            instance_id: item.get(),
            space: space.read(cx).key(),
            space_name: space.read(cx).name().to_owned(),
            project_dir: project.clone(),
            bound_session: bound_session.map(|session| session.0.clone()),
            terminal: Self::plugin_terminal_launcher(space.clone(), cx),
            services: self.catalog.services.clone(),
            plugin_settings: chartr_plugin::services::PluginSettings::new(
                move |plugin, window, cx| {
                    if let Some(handle) = window.window_handle().downcast::<Self>() {
                        crate::settings_window::open_plugin(
                            handle,
                            origin.clone(),
                            plugin.map(str::to_owned),
                            cx,
                        );
                    }
                },
            ),
        };
        let session_access =
            bound_session.and_then(|session| space.read(cx).session_access(session));
        let on_focus = Some(Self::web_plugin_focus_handler(space.clone(), cx));
        let title_space = space.downgrade();
        let on_title: crate::native_plugin::TitleHandler = Rc::new(move |title, cx| {
            let _ = title_space.update(cx, |space, cx| {
                if space.set_plugin_title(item, title) {
                    cx.notify();
                }
            });
        });
        let unsafe_filesystem = self.settings.resolved().plugin(&key.plugin).unsafe_filesystem;
        let loaded = self.catalog.get_mut(&key.plugin)?;
        let permissions = loaded.permissions().clone();
        let package = loaded.dir.clone();
        let data = plugin_paths().data.join(&key.plugin);
        Some(match loaded.pane(key)? {
            PaneSource::Native(plugin) => PluginView::new(plugin.view(key, &instance, window, cx)),
            PaneSource::Embedded(library) => crate::native_plugin::pane(
                package,
                library.to_path_buf(),
                data,
                &instance,
                on_focus,
                Some(on_title),
                window,
                cx,
            ),
            PaneSource::Web(entry) => crate::web_plugin::pane(
                crate::web_plugin::Document { package, entry: entry.to_path_buf() },
                FileBroker::new(project, data, permissions.project_files, unsafe_filesystem),
                permissions,
                session_access,
                on_focus,
                instance,
                window,
                cx,
            ),
        })
    }

    pub(super) fn plugin_launcher(
        &self,
        launcher: crate::workspace::ItemId,
        weak: &gpui::WeakEntity<Self>,
        cx: &App,
    ) -> AnyElement {
        let cards: Vec<_> = self
            .catalog
            .panes()
            .into_iter()
            .enumerate()
            .filter_map(|(index, pane)| {
                let plugin = self.catalog.get(&pane.key.plugin)?;
                let name = plugin.manifest.name.clone();
                let description = if plugin.manifest.description.trim().is_empty() {
                    format!("Open {name} in your workspace.")
                } else {
                    plugin.manifest.description.clone()
                };
                let icon_path =
                    gpui::SharedString::from(plugin.icon_path().to_string_lossy().into_owned());
                let surface = pane.title.clone();
                let key = pane.key.clone();
                let open = weak.clone();
                Some(
                    ButtonLike::new(("plugin-launcher-card", index))
                        .style(ButtonStyle::OutlinedGhost)
                        .size(ButtonSize::None)
                        .full_width()
                        .height(rems(5.5).into())
                        .tab_index(0isize)
                        .aria_label(format!("Open {surface} from {name}"))
                        .aria_description(description.clone())
                        .on_click(move |_, window, cx| {
                            let _ = open.update(cx, |this, cx| {
                                this.open_plugin_from_launcher(launcher, key.clone(), window, cx)
                            });
                        })
                        .child(
                            v_flex()
                                .size_full()
                                .min_w_0()
                                .text_left()
                                .items_start()
                                .gap_1()
                                .px_3()
                                .py_2p5()
                                .child(
                                    h_flex()
                                        .w_full()
                                        .h(rems(1.5))
                                        .flex_none()
                                        .gap_2p5()
                                        .line_height(relative(1.5))
                                        .child(
                                            h_flex()
                                                .w(rems(1.25))
                                                .flex_none()
                                                .justify_center()
                                                .child(
                                                    Icon::from_external_svg(icon_path)
                                                        .size(IconSize::Medium)
                                                        .color(Color::Muted),
                                                ),
                                        )
                                        .child(
                                            div().flex_1().min_w_0().child(
                                                Label::new(surface)
                                                    .size(UI_LABEL_DEFAULT)
                                                    .weight(gpui::FontWeight::MEDIUM)
                                                    .truncate(),
                                            ),
                                        ),
                                )
                                .child(
                                    div().w_full().line_height(relative(1.35)).child(
                                        Label::new(description)
                                            .size(UI_LABEL_DEFAULT)
                                            .color(Color::Muted)
                                            .line_clamp(2),
                                    ),
                                ),
                        )
                        .into_any_element(),
                )
            })
            .collect();

        let has_cards = !cards.is_empty();
        LauncherSurface {
            id: format!("plugin-launcher-{}", launcher.get()).into(),
            content: v_flex()
                .when(has_cards, |launcher| launcher.child(launcher_cards(cards)))
                .when(!has_cards, |launcher| {
                    launcher.child(
                        h_flex()
                            .w_full()
                            .h(px(88.))
                            .px_3()
                            .gap_3()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .bg(cx.theme().colors().element_background)
                            .child(
                                div()
                                    .size(px(36.))
                                    .flex_none()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .bg(cx.theme().colors().editor_background)
                                    .child(
                                        Icon::from_path(crate::assets::PLUGIN_LAUNCHER_ICON_PATH)
                                            .size(IconSize::Medium)
                                            .color(Color::Muted),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(
                                        Label::new("No surfaces available")
                                            .size(UI_LABEL_DEFAULT)
                                            .weight(gpui::FontWeight::SEMIBOLD),
                                    )
                                    .child(
                                        Label::new("Enable a plugin in Settings to see it here.")
                                            .size(UI_LABEL_SMALL)
                                            .color(Color::Muted),
                                    ),
                            ),
                    )
                })
                .into_any_element(),
        }
        .into_any_element()
    }

    fn web_plugin_focus_handler(
        space: Entity<Space>,
        cx: &Context<Self>,
    ) -> crate::web_plugin::FocusHandler {
        let weak = cx.weak_entity();
        Rc::new(move |view, cx| {
            let space = space.clone();
            let _ = weak.update(cx, |this, cx| {
                if space.update(cx, |space, _| space.activate_plugin_view(view)) {
                    this.active = Some(space);
                    cx.notify();
                }
            });
        })
    }

    fn plugin_terminal_launcher(
        space: Entity<Space>,
        cx: &Context<Self>,
    ) -> chartr_plugin::TerminalLauncher {
        let prepare_space = space.downgrade();
        let launch_space = space.downgrade();
        let focus_space = space.downgrade();
        let owner = cx.weak_entity();
        chartr_plugin::TerminalLauncher::new(move |input, cx| {
            if let Some(space) = launch_space.upgrade() {
                space.update(cx, |space, cx| space.start_session_with_input(input, cx));
            }
        })
        .with_prepare(move |cx| {
            let Some(space) = prepare_space.upgrade() else {
                return gpui::Task::ready(Err("The owning space was closed.".into()));
            };
            space.update(cx, |space, cx| space.prepare_plugin_session(cx))
        })
        .with_focus(move |session, window, cx| {
            let Some(space) = focus_space.upgrade() else {
                return false;
            };
            owner
                .update(cx, |this, cx| {
                    if !space.update(cx, |space, cx| {
                        space.activate_session(&chartr_herdr::PaneId(session.into()), cx)
                    }) {
                        return false;
                    }
                    this.activate(space, window, cx);
                    this.focus_active_terminal(window, cx);
                    true
                })
                .unwrap_or(false)
        })
    }

    fn open_plugin_from_launcher(
        &mut self,
        launcher: crate::workspace::ItemId,
        key: chartr_plugin::PaneKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title =
            self.catalog.panes().iter().find(|pane| pane.key == key).map(|pane| pane.title.clone());
        let Some(title) = title else {
            self.problem = Some("That plugin contribution is no longer available.".to_owned());
            cx.notify();
            return;
        };
        let (capabilities, icon_path) = match self.catalog.get(&key.plugin) {
            Some(plugin) => (
                plugin.capabilities().clone(),
                gpui::SharedString::from(plugin.icon_path().to_string_lossy().into_owned()),
            ),
            None => {
                self.problem = Some("That plugin is no longer loaded.".to_owned());
                cx.notify();
                return;
            }
        };
        let Some(space) = self.active.clone() else {
            return;
        };
        if !space.read(cx).is_plugin_launcher(launcher) {
            return;
        }
        let bound_session = if capabilities.session_binding {
            let Some(session) = space.read(cx).plugin_launcher_bound_session(launcher) else {
                self.problem =
                    Some("Open this launcher from a terminal to use that plugin.".to_owned());
                cx.notify();
                return;
            };
            Some(session)
        } else {
            None
        };
        if capabilities.multiplicity == Multiplicity::PerSpace
            && space.update(cx, |space, _| space.activate_plugin(&key))
        {
            space.update(cx, |space, _| space.finish_bulk_close(&[launcher]));
            self.problem = None;
            cx.notify();
            return;
        }
        let Some(view) =
            self.build_plugin_view(&space, &key, launcher, bound_session.as_ref(), window, cx)
        else {
            self.problem = Some("That pane is no longer contributed.".to_owned());
            cx.notify();
            return;
        };
        space.update(cx, |space, cx| {
            space.replace_plugin_launcher(
                launcher,
                PluginItem {
                    contribution: key,
                    title,
                    icon_path,
                    view,
                    bound_session,
                    can_clone: capabilities.cloneable,
                    restorable: capabilities.restorable,
                },
                cx,
            );
        });
        self.problem = None;
        cx.notify();
    }

    pub(super) fn restore_plugins_once(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.plugins_restored {
            return;
        }
        if matches!(self.backend, Backend::Starting | Backend::Recovering(_)) {
            return;
        }
        self.plugins_restored = true;
        let mut failures = Vec::new();
        for space in self.spaces.clone() {
            let records = space.update(cx, |space, _| space.take_restoring_plugins());
            for record in &records {
                let crate::persistence::PersistedItem::Plugin {
                    plugin, pane, bound_session, ..
                } = record
                else {
                    continue;
                };
                // Companion is excluded from this build; Prompts is settings-only.
                // Let placeholder cleanup retire saved tabs without a restore error.
                if matches!(plugin.as_str(), "com.chartr.companion" | "com.chartr.prompts")
                    && pane == "main"
                {
                    continue;
                }
                let key = chartr_plugin::PaneKey { plugin: plugin.clone(), key: pane.clone() };
                let descriptor = self.catalog.get(plugin).and_then(|loaded| {
                    loaded.panes.iter().find(|candidate| candidate.key == key).map(|candidate| {
                        (
                            candidate.title.clone(),
                            loaded.capabilities().clone(),
                            gpui::SharedString::from(
                                loaded.icon_path().to_string_lossy().into_owned(),
                            ),
                        )
                    })
                });
                let Some((title, capabilities, icon_path)) = descriptor else {
                    failures.push(format!("{plugin}:{pane} is unavailable"));
                    continue;
                };
                if !capabilities.restorable {
                    failures.push(format!("{plugin}:{pane} does not support restoration"));
                    continue;
                }
                let bound = bound_session.clone().map(chartr_herdr::PaneId);
                if bound
                    .as_ref()
                    .is_some_and(|session| space.read(cx).session_access(session).is_none())
                {
                    failures.push(format!("{plugin}:{pane} lost its bound session"));
                    continue;
                }
                let Some(item_id) = space
                    .read(cx)
                    .workspace_tabs()
                    .item_ids()
                    .find(|item| item.get() == record.item_id())
                else {
                    failures.push(format!("{plugin}:{pane} had no saved layout item"));
                    continue;
                };
                let Some(view) =
                    self.build_plugin_view(&space, &key, item_id, bound.as_ref(), window, cx)
                else {
                    failures.push(format!("{plugin}:{pane} is no longer contributed"));
                    continue;
                };
                let item = PluginItem {
                    contribution: key,
                    title,
                    icon_path,
                    view,
                    bound_session: bound,
                    can_clone: capabilities.cloneable,
                    restorable: true,
                };
                if !space.update(cx, |space, cx| space.restore_plugin(record, item, cx)) {
                    failures.push(format!("{plugin}:{pane} had no saved layout item"));
                }
            }
            space.update(cx, |space, _| space.remove_plugin_placeholders(&records));
        }
        if !failures.is_empty() {
            self.problem =
                Some(format!("Some plugin items could not be restored: {}.", failures.join(", ")));
        }
        self.schedule_persistence(cx);
    }

    pub(super) fn clone_plugin_drop(
        &mut self,
        space: Entity<Space>,
        source_item: crate::workspace::ItemId,
        target_tab: WorkspaceTabId,
        target: LayoutPaneId,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some((key, title, bound_session)) = space.read(cx).cloneable_plugin(source_item) else {
            return false;
        };
        let Some(loaded) = self.catalog.get(&key.plugin) else {
            return false;
        };
        let capabilities = loaded.capabilities().clone();
        let icon_path = gpui::SharedString::from(loaded.icon_path().to_string_lossy().into_owned());
        let Some(destination) =
            space.update(cx, |space, _| space.prepare_drop_destination(target_tab, target))
        else {
            return true;
        };
        let item = space.update(cx, |space, _| space.reserve_plugin_item());
        let Some(view) =
            self.build_plugin_view(&space, &key, item, bound_session.as_ref(), window, cx)
        else {
            return false;
        };
        space.update(cx, |space, cx| {
            space.open_plugin_in_at(
                item,
                PluginItem {
                    contribution: key,
                    title,
                    icon_path,
                    view,
                    bound_session,
                    can_clone: capabilities.cloneable,
                    restorable: capabilities.restorable,
                },
                target_tab,
                destination,
                index,
                cx,
            );
        });
        true
    }
}
