//! Plugin launcher, instance creation, restoration, and cloning.

use super::*;

impl Zeddy {
    /// Construct every plugin instance through the same permission and host wiring.
    fn build_plugin_view(
        &mut self,
        space: &Entity<Space>,
        key: &zeddy_plugin::PaneKey,
        item: crate::workspace::ItemId,
        bound_session: Option<&zeddy_herdr::PaneId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<PluginView> {
        let project =
            (space.read(cx).kind() == SpaceKind::Registered).then(|| space.read(cx).path().clone());
        let instance = InstanceContext {
            instance_id: item.get(),
            space: space.read(cx).key(),
            space_name: space.read(cx).name().to_owned(),
            project_dir: project.clone(),
            bound_session: bound_session.map(|session| session.0.clone()),
            terminal: Self::plugin_terminal_launcher(space.clone()),
        };
        let session_access =
            bound_session.and_then(|session| space.read(cx).session_access(session));
        let on_focus = Some(Self::web_plugin_focus_handler(space.clone(), cx));
        let on_title_change = Some(Self::browser_title_handler(space.clone(), item));
        let unsafe_filesystem = self.settings.resolved().plugin(&key.plugin).unsafe_filesystem;
        let loaded = self.catalog.get_mut(&key.plugin)?;
        let permissions = loaded.permissions().clone();
        let package = loaded.dir.clone();
        let data = plugin_paths().data.join(&key.plugin);
        Some(match loaded.pane(key)? {
            PaneSource::Native(plugin) => PluginView::new(plugin.view(key, &instance, window, cx)),
            PaneSource::Hosted(HostedSurface::Browser) => {
                crate::browser_plugin::view(data, &instance, on_focus, on_title_change, window, cx)
            }
            PaneSource::Web(entry) => crate::web_plugin::pane(
                crate::web_plugin::Document { package, entry: entry.to_path_buf() },
                FileBroker::new(project, data, permissions.project_files, unsafe_filesystem),
                permissions,
                session_access,
                on_focus,
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
                let version = plugin.manifest.version.clone();
                let icon_path =
                    gpui::SharedString::from(plugin.icon_path().to_string_lossy().into_owned());
                let surface = pane.title.clone();
                let kind = match plugin.kind() {
                    zeddy_plugin::manifest::Kind::Native => "Native",
                    zeddy_plugin::manifest::Kind::Hosted => "Hosted",
                    zeddy_plugin::manifest::Kind::Web => "Web",
                };
                let surface_label = (surface != name).then(|| surface.clone());
                let key = pane.key.clone();
                let open = weak.clone();
                Some(
                    ButtonLike::new(("plugin-launcher-card", index))
                        .style(ButtonStyle::Outlined)
                        .size(ButtonSize::None)
                        .full_width()
                        .height(px(88.).into())
                        .tab_index(0isize)
                        .aria_label(format!("Open {surface} from {name}"))
                        .on_click(move |_, window, cx| {
                            let _ = open.update(cx, |this, cx| {
                                this.open_plugin_from_launcher(launcher, key.clone(), window, cx)
                            });
                        })
                        .child(
                            h_flex()
                                .size_full()
                                .text_left()
                                .items_center()
                                .gap_3()
                                .px_3()
                                .child(
                                    div()
                                        .size(px(36.))
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(cx.theme().colors().border_variant)
                                        .bg(cx.theme().colors().editor_background)
                                        .child(
                                            Icon::from_external_svg(icon_path)
                                                .size(IconSize::Medium),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .min_w_0()
                                        .flex_1()
                                        .items_start()
                                        .gap_0p5()
                                        .child(
                                            Label::new(name)
                                                .size(UI_LABEL_LARGE)
                                                .weight(gpui::FontWeight::SEMIBOLD)
                                                .truncate(),
                                        )
                                        .when_some(surface_label, |details, surface| {
                                            details.child(
                                                Label::new(surface)
                                                    .size(UI_LABEL_DEFAULT)
                                                    .color(Color::Muted)
                                                    .truncate(),
                                            )
                                        })
                                        .child(
                                            Label::new(format!("{kind} plugin  ·  v{version}"))
                                                .size(UI_LABEL_SMALL)
                                                .color(Color::Muted),
                                        ),
                                )
                                .child(
                                    Icon::new(IconName::ChevronRight)
                                        .size(IconSize::Small)
                                        .color(Color::Muted),
                                ),
                        )
                        .into_any_element(),
                )
            })
            .collect();

        let has_cards = !cards.is_empty();
        div()
            .id(format!("plugin-launcher-{}", launcher.get()))
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(760.))
                    .mx_auto()
                    .px_8()
                    .pt_8()
                    .pb_8()
                    .gap_5()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(px(40.))
                                    .flex_none()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(cx.theme().colors().border_variant)
                                    .bg(cx.theme().colors().element_background)
                                    .child(
                                        Icon::from_path(crate::assets::PLUGIN_LAUNCHER_ICON_PATH)
                                            .size(IconSize::Medium),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(
                                        Label::new("Open a plugin")
                                            .size(UI_LABEL_LARGE)
                                            .weight(gpui::FontWeight::SEMIBOLD),
                                    )
                                    .child(
                                        Label::new("Choose a tool to open in this pane.")
                                            .size(UI_LABEL_DEFAULT)
                                            .color(Color::Muted),
                                    ),
                            ),
                    )
                    .when(has_cards, |launcher| {
                        launcher.child(div().w_full().grid().grid_cols(2).gap_2().children(cards))
                    })
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
                                            Icon::from_path(
                                                crate::assets::PLUGIN_LAUNCHER_ICON_PATH,
                                            )
                                            .size(IconSize::Medium)
                                            .color(Color::Muted),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .gap_0p5()
                                        .child(
                                            Label::new("No plugins available")
                                                .size(UI_LABEL_DEFAULT)
                                                .weight(gpui::FontWeight::SEMIBOLD),
                                        )
                                        .child(
                                            Label::new(
                                                "Enable a plugin in Settings to see it here.",
                                            )
                                            .size(UI_LABEL_SMALL)
                                            .color(Color::Muted),
                                        ),
                                ),
                        )
                    }),
            )
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

    fn browser_title_handler(
        space: Entity<Space>,
        item: crate::workspace::ItemId,
    ) -> crate::browser_plugin::TitleHandler {
        Rc::new(move |title, cx| {
            space.update(cx, |space, cx| {
                if space.set_plugin_title(item, title) {
                    cx.notify();
                }
            });
        })
    }

    fn plugin_terminal_launcher(space: Entity<Space>) -> zeddy_plugin::TerminalLauncher {
        zeddy_plugin::TerminalLauncher::new(move |input, cx| {
            space.update(cx, |space, cx| space.start_session_with_input(input, cx));
        })
    }

    fn open_plugin_from_launcher(
        &mut self,
        launcher: crate::workspace::ItemId,
        key: zeddy_plugin::PaneKey,
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
                let key = zeddy_plugin::PaneKey { plugin: plugin.clone(), key: pane.clone() };
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
                let bound = bound_session.clone().map(zeddy_herdr::PaneId);
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
