//! Plugin installation and configuration controls.

use super::*;

impl SettingsWindow {
    fn uninstall_plugin(
        &mut self,
        plugin: String,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.plugin_operation.is_some() {
            return;
        }
        let Some(origin) = self.original.upgrade() else {
            return;
        };
        self.plugin_operation = Some(format!("Confirm removal of {name}…"));
        self.problem = None;
        cx.notify();
        let dependents = origin.read(cx).settings_plugin_dependents(&plugin);
        let mut detail = "Its panes will close and its package and pending update will be removed. Plugin data and preferences are kept. Reinstalling leaves it disabled until you enable it.".to_owned();
        if !dependents.is_empty() {
            detail.push_str(&format!(
                "\n\nThis also disables these dependent plugins: {}.",
                dependents.join(", ")
            ));
        }
        let confirmation = window.prompt(
            gpui::PromptLevel::Warning,
            &format!("Uninstall {name}?"),
            Some(&detail),
            &["Uninstall", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if confirmation.await != Ok(0) {
                let _ = this.update_in(cx, |this, _, cx| {
                    this.plugin_operation = None;
                    cx.notify();
                });
                return;
            }
            let removal = this.update_in(cx, |this, _, cx| {
                if this.plugin_settings.as_ref().is_some_and(|(id, _)| id == &plugin) {
                    this.plugin_settings = None;
                }
                if this.plugin_information.as_ref() == Some(&plugin) {
                    this.plugin_information = None;
                }
                this.plugin_operation = Some(format!("Uninstalling {name}…"));
                cx.notify();
                origin.update(cx, |origin, cx| origin.settings_uninstall_plugin(plugin, cx))
            });
            let result = match removal {
                Ok(Ok(task)) => task.await,
                Ok(Err(error)) => Err(error),
                Err(_) => return,
            };
            let _ = this.update_in(cx, |this, _, cx| {
                this.plugin_operation = None;
                this.problem = result.err();
                this.plugin_restart_required =
                    crate::plugin_installer::has_pending(&crate::app::plugin_paths());
                cx.notify();
            });
        })
        .detach();
    }

    fn request_plugin_enabled(
        &mut self,
        plugin: String,
        name: String,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.plugin_operation.is_some() {
            return;
        }
        let dependents = self
            .original
            .upgrade()
            .map(|origin| origin.read(cx).settings_plugin_dependents(&plugin))
            .unwrap_or_default();
        if enabled || dependents.is_empty() {
            self.set_plugin_enabled(plugin, enabled, cx);
            return;
        }
        self.plugin_operation = Some(format!("Confirm disabling {name}…"));
        cx.notify();
        let confirmation = window.prompt(
            gpui::PromptLevel::Warning,
            &format!("Disable {name}?"),
            Some(&format!("This also disables these dependent plugins and closes their panes: {}. You can enable them again after their prerequisites are enabled.", dependents.join(", "))),
            &["Disable", "Cancel"], cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let confirmed = confirmation.await == Ok(0);
            let _ = this.update_in(cx, |this, _, cx| {
                this.plugin_operation = None;
                if confirmed {
                    this.set_plugin_enabled(plugin, false, cx);
                }
                cx.notify();
            });
        })
        .detach();
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

    fn set_plugin_unsafe(
        &mut self,
        plugin: String,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
                    self.open_plugin_settings(plugin.clone(), window, cx);
                }
                self.problem = None;
            }
            Err(error) => self.problem = Some(error),
        }
        cx.notify();
    }

    pub(super) fn open_plugin_settings(
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
        self.plugin_information = None;
        self.plugin_settings = Some((plugin, view));
        self.problem = None;
        cx.notify();
    }

    pub(super) fn plugin_configuration_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (id, view) = self.plugin_settings.clone().expect("configuration is open");
        let descriptor = self.original.upgrade().and_then(|origin| {
            origin.read(cx).settings_plugins().0.into_iter().find(|p| p.manifest.id == id)
        });
        let back = cx.listener(|this, _, _, cx| {
            this.plugin_settings = None;
            cx.notify();
        });
        let mut page = v_flex().w_full().gap_3().child(
            h_flex()
                .child(settings_button("plugin-settings-back", "Back to plugins").on_click(back)),
        );
        let Some(descriptor) = descriptor else {
            return page
                .child(Label::new("This plugin is no longer installed."))
                .into_any_element();
        };
        let manifest = descriptor.manifest;
        let declarative_settings = manifest.settings.is_some();
        if let Some(error) = descriptor.prerequisite_error {
            page = page.child(Label::new(error).size(UI_LABEL_SMALL).color(Color::Error));
        }
        if !descriptor.enabled {
            page = page.child(
                Label::new("Enable this plugin to open its own settings.").color(Color::Muted),
            );
        }
        for dependency in manifest.dependencies {
            let provider = dependency.plugin;
            page = page.child(setting_field(
                dependency.feature,
                format!("Requires {provider}."),
                settings_button(format!("configure-provider-{provider}"), "Configure provider")
                    .disabled(self.plugin_operation.is_some())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_plugin_settings(provider.clone(), window, cx)
                    })),
            ));
        }
        if manifest.kind == zeddy_plugin::manifest::Kind::Web {
            let unsafe_filesystem = self.settings(cx).plugin(&id).unsafe_filesystem;
            let weak = cx.weak_entity();
            let plugin = id.clone();
            page = page.child(setting_field("Unsafe filesystem access",
                "Allow the project-file API to read and write outside the project, overriding its declared file permissions.",
                Switch::new(format!("plugin-unsafe-{id}"), unsafe_filesystem.into())
                    .disabled(self.plugin_operation.is_some())
                    .aria_label("Unsafe filesystem access")
                    .on_click(move |state, window, cx| {
                        let _ = weak.update(cx, |this, cx| this.set_plugin_unsafe(plugin.clone(), state.selected(), window, cx));
                    })));
        }
        page.when_some(view.filter(|_| descriptor.enabled), |page, view| {
            // Native plugin management views need room for their own dialogs.
            // Declarative forms flow in the ordinary Settings page scroll.
            page.child(
                div()
                    .w_full()
                    .when(!declarative_settings, |panel| {
                        panel.min_h((window.viewport_size().height - px(140.)).max(px(320.)))
                    })
                    .flex_none()
                    .child(view),
            )
        })
        .into_any_element()
    }

    fn open_plugin_information(&mut self, plugin: String, cx: &mut Context<Self>) {
        self.plugin_settings = None;
        self.plugin_information = Some(plugin);
        self.problem = None;
        cx.notify();
    }

    pub(super) fn plugin_information_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let id = self.plugin_information.clone().expect("plugin information is open");
        let mut page = v_flex()
            .w_full()
            .gap_3()
            .child(h_flex().child(
                settings_button("plugin-information-back", "Back to plugins").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.plugin_information = None;
                        cx.notify();
                    }),
                ),
            ))
            .child(Label::new("Plugin Information").size(UI_LABEL_LARGE));
        let (descriptors, rejected) = self
            .original
            .upgrade()
            .map(|origin| origin.read(cx).settings_plugins())
            .unwrap_or_default();
        let name;
        if let Some(descriptor) = descriptors.into_iter().find(|p| p.manifest.id == id) {
            let manifest = descriptor.manifest;
            name = manifest.name.clone();
            let source = descriptor
                .installation
                .map(|installation| {
                    let commit = installation
                        .commit
                        .map(|commit| format!(" ({commit})"))
                        .unwrap_or_default();
                    format!("{}{}", installation.source, commit)
                })
                .unwrap_or_else(|| {
                    if descriptor.bundled {
                        "Bundled with Chartr".into()
                    } else {
                        "Source not recorded".into()
                    }
                });
            let access = match manifest.kind {
                zeddy_plugin::manifest::Kind::Native => "Runs as fully trusted native code.".into(),
                zeddy_plugin::manifest::Kind::Hosted => format!(
                    "Uses Chartr's built-in {} surface.",
                    manifest.surface.as_deref().unwrap_or("hosted")
                ),
                zeddy_plugin::manifest::Kind::Web => {
                    format!("Declared host access: {}.", manifest.permissions.summary())
                }
            };
            let details = v_flex()
                .w_full()
                .gap_2()
                .p_3()
                .rounded_md()
                .bg(cx.theme().colors().surface_background)
                .child(Label::new(manifest.name.clone()).size(UI_LABEL_LARGE))
                .child(
                    Label::new(format!("{} · Version {} · {}", id, manifest.version, source))
                        .size(UI_LABEL_SMALL)
                        .color(Color::Muted),
                )
                .child(Label::new(access).size(UI_LABEL_SMALL).color(Color::Muted));
            page = page.child(details);
        } else if let Some(rejected) = rejected.into_iter().find(|plugin| {
            let paths = crate::app::plugin_paths();
            plugin.dir.file_name().and_then(|name| name.to_str()) == Some(id.as_str())
                && (plugin.dir.parent() == Some(paths.installed.as_path())
                    || plugin.dir.parent() == Some(paths.bundled.as_path()))
        }) {
            name = id.clone();
            page = page
                .child(Label::new(name.clone()).size(UI_LABEL_LARGE))
                .child(Label::new(rejected.why).size(UI_LABEL_SMALL).color(Color::Error));
        } else {
            return page
                .child(Label::new("This plugin is no longer installed."))
                .into_any_element();
        }
        page.child(
            h_flex().child(
                settings_button(format!("plugin-uninstall-{id}"), "Uninstall")
                    .start_icon(Icon::new(IconName::Trash))
                    .disabled(self.original.upgrade().is_none() || self.plugin_operation.is_some())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.uninstall_plugin(id.clone(), name.clone(), window, cx)
                    })),
            ),
        )
        .into_any_element()
    }

    fn pick_plugin_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.plugin_operation.is_some() {
            return;
        }
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Install Plugin".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let outcome = chosen.await;
            let _ = this.update_in(cx, |this, window, cx| match outcome {
                Ok(Ok(Some(paths))) if !paths.is_empty() => this.begin_plugin_install(
                    crate::plugin_installer::Source::Local(paths[0].clone()),
                    window,
                    cx,
                ),
                Ok(Ok(Some(_))) | Ok(Ok(None)) => {}
                Ok(Err(error)) => {
                    this.problem = Some(format!("choosing a plugin folder: {error}"));
                    cx.notify();
                }
                Err(_) => {
                    this.problem = Some("the plugin folder picker closed unexpectedly".into());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn install_from_git(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let url = self.git_url_input.read(cx).text().trim().to_owned();
        if url.is_empty() {
            self.problem = Some("Enter a Git repository URL.".into());
            cx.notify();
            return;
        }
        self.begin_plugin_install(crate::plugin_installer::Source::Git(url), window, cx);
    }

    fn begin_plugin_install(
        &mut self,
        source: crate::plugin_installer::Source,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.plugin_operation.is_some() {
            return;
        }
        let source_label = source.label();
        self.problem = None;
        self.plugin_operation = Some(format!("Inspecting {source_label}…"));
        cx.notify();

        let paths = crate::app::plugin_paths();
        let executor = cx.background_executor().clone();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.plugin_cancel = Some(cancel.clone());
        let prepare = executor.spawn(async move {
            crate::plugin_installer::prepare_cancellable(source, &paths, &cancel)
        });
        cx.spawn_in(window, async move |this, cx| {
            let prepared = match prepare.await {
                Ok(prepared) => prepared,
                Err(error) => {
                    let _ = this.update_in(cx, |this, _, cx| {
                        this.plugin_operation = None;
                        this.plugin_cancel = None;
                        this.problem = Some(format!("{error:#}"));
                        cx.notify();
                    });
                    return;
                }
            };
            let name = prepared.manifest.name.clone();
            let detail = prepared.trust_detail();
            let confirmation = this.update_in(cx, |this, window, cx| {
                let cancelled = this
                    .plugin_cancel
                    .take()
                    .is_some_and(|cancel| cancel.load(std::sync::atomic::Ordering::Relaxed));
                if cancelled {
                    this.plugin_operation = None;
                    cx.notify();
                    return None;
                }
                this.plugin_operation = Some(format!("Confirm installation of {name}…"));
                cx.notify();
                Some(window.prompt(
                    gpui::PromptLevel::Info,
                    &format!("Install {name}?"),
                    Some(&detail),
                    &["Install", "Cancel"],
                    cx,
                ))
            });
            let Ok(Some(confirmation)) = confirmation else {
                return;
            };
            if confirmation.await != Ok(0) {
                let _ = this.update_in(cx, |this, _, cx| {
                    this.plugin_operation = None;
                    cx.notify();
                });
                return;
            }

            let _ = this.update_in(cx, |this, _, cx| {
                this.plugin_operation = Some(format!("Installing {name}…"));
                this.problem = None;
                cx.notify();
            });
            let paths = crate::app::plugin_paths();
            let install =
                executor.spawn(async move { crate::plugin_installer::install(prepared, &paths) });
            let installed = match install.await {
                Ok(installed) => installed,
                Err(error) => {
                    let _ = this.update_in(cx, |this, _, cx| {
                        this.plugin_operation = None;
                        this.plugin_cancel = None;
                        this.problem = Some(format!("{error:#}"));
                        cx.notify();
                    });
                    return;
                }
            };
            let restart = this.update_in(cx, |this, window, cx| {
                this.plugin_operation = None;
                if let Err(error) = crate::settings::update_global(cx, |content| {
                    content.plugins.entry(installed.id.clone()).or_default().uninstalled = Some(false);
                }) {
                    this.problem = Some(error.to_string());
                    cx.notify();
                    return window.prompt(gpui::PromptLevel::Warning, "Plugin installed", Some("The package was installed, but its preferences could not be saved. Restart Chartr to load it."), &["Restart", "Later"], cx);
                }
                this.plugin_restart_required = true;
                this.git_install_open = false;
                this.git_url_input.update(cx, |input, cx| input.clear(cx));
                this.problem = None;
                cx.notify();
                window.prompt(
                    gpui::PromptLevel::Info,
                    &format!("{} installed", installed.name),
                    Some("Restart Chartr to enable the plugin."),
                    &["Restart", "Later"],
                    cx,
                )
            });
            if let Ok(restart) = restart
                && restart.await == Ok(0)
            {
                let _ = cx.update(|_, cx| cx.restart());
            }
        })
        .detach();
    }

    pub(super) fn plugins_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let origin_available = self.original.upgrade().is_some();
        let busy = self.plugin_operation.is_some();
        let show_git = self.git_install_open;
        let open_git = cx.listener(|this, _, window, cx| {
            this.git_install_open = true;
            this.problem = None;
            window.focus(&this.git_url_input.focus_handle(cx), cx);
            cx.notify();
        });
        let install_git = cx.listener(|this, _, window, cx| {
            this.install_from_git(window, cx);
        });
        let cancel_git = cx.listener(|this, _, _, cx| {
            this.git_install_open = false;
            this.problem = None;
            cx.notify();
        });
        let pick_folder = cx.listener(|this, _, window, cx| {
            this.pick_plugin_folder(window, cx);
        });
        let restart = cx.listener(|_, _, _, cx| cx.restart());
        let colors = cx.theme().colors();
        let install_actions = h_flex()
            .gap_2()
            .child(
                settings_button("install-plugin-git", "Install from Git…")
                    .disabled(busy)
                    .on_click(open_git),
            )
            .child(
                settings_button("install-plugin-folder", "Install from Folder…")
                    .disabled(busy)
                    .on_click(pick_folder),
            );
        let git_form = v_flex()
            .gap_2()
            .p_3()
            .border_1()
            .border_color(colors.border_variant)
            .rounded_md()
            .child(Label::new("Git repository URL").size(UI_LABEL_DEFAULT))
            .child(input_field("plugin-git-url", self.git_url_input.clone(), cx))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        settings_button("confirm-install-plugin-git", "Install")
                            .on_click(install_git),
                    )
                    .child(
                        settings_button("cancel-install-plugin-git", "Cancel").on_click(cancel_git),
                    ),
            );
        let (descriptors, rejected) = self
            .original
            .upgrade()
            .map(|origin| origin.read(cx).settings_plugins())
            .unwrap_or_default();
        let mut fields = Vec::new();
        for descriptor in descriptors {
            let manifest = descriptor.manifest;
            let name = manifest.name.clone();
            let id = manifest.id.clone();
            let enabled_id = id.clone();
            let enabled_name = name.clone();
            let enabled_setting = cx.weak_entity();
            let enabled_control =
                Switch::new(format!("plugin-enabled-{id}"), descriptor.enabled.into())
                    .disabled(
                        !origin_available
                            || busy
                            || (!descriptor.enabled && descriptor.prerequisite_error.is_some()),
                    )
                    .tab_index(0isize)
                    .aria_label(format!("Enable {name}"))
                    .aria_description(descriptor.prerequisite_error.clone().unwrap_or_default())
                    .on_click(move |state, window, cx| {
                        let _ = enabled_setting.update(cx, |this, cx| {
                            this.request_plugin_enabled(
                                enabled_id.clone(),
                                enabled_name.clone(),
                                state.selected(),
                                window,
                                cx,
                            )
                        });
                    });
            let information_id = id.clone();
            let settings_id = id.clone();
            // Keep configuration and package information in separate destinations.
            let controls = h_flex()
                .flex_none()
                .gap_2()
                .child(
                    IconButton::new(format!("plugin-information-{id}"), IconName::Info)
                        .aria_label(format!("Information about {name}"))
                        .tooltip(Tooltip::text("Plugin Information"))
                        .disabled(!origin_available || busy)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.open_plugin_information(information_id.clone(), cx)
                        })),
                )
                .child(
                    IconButton::new(format!("plugin-settings-{id}"), IconName::Settings)
                        .aria_label(format!("Configure {name}"))
                        .tooltip(Tooltip::text(format!("Configure {name}")))
                        .disabled(!origin_available || busy)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open_plugin_settings(settings_id.clone(), window, cx)
                        })),
                )
                .child(enabled_control);
            fields.push(
                v_flex()
                    .id(format!("plugin-row-{id}"))
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .py(px(SETTINGS_FIELD_VERTICAL_PADDING))
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .gap_3()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Label::new(name).size(UI_LABEL_DEFAULT)),
                            )
                            .child(controls),
                    )
                    .child(
                        Label::new(if manifest.description.trim().is_empty() {
                            format!("{} plugin.", manifest.name)
                        } else {
                            manifest.description
                        })
                        .size(UI_LABEL_SMALL)
                        .color(Color::Muted),
                    )
                    .when_some(descriptor.prerequisite_error, |row, error| {
                        row.child(Label::new(error).size(UI_LABEL_SMALL).color(Color::Error))
                    })
                    .into_any_element(),
            );
        }
        let has_fields = !fields.is_empty();
        let paths = crate::app::plugin_paths();
        let rejected: Vec<_> = rejected
            .into_iter()
            .map(|rejected| {
                let id = rejected.dir.file_name().and_then(|name| name.to_str()).map(str::to_owned);
                let removable = rejected.dir.parent() == Some(paths.installed.as_path())
                    || rejected.dir.parent() == Some(paths.bundled.as_path());
                let name = id.clone().unwrap_or_else(|| "Invalid plugin".into());
                v_flex()
                    .w_full()
                    .gap_2()
                    .py(px(SETTINGS_FIELD_VERTICAL_PADDING))
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .gap_3()
                            .child(Label::new(name.clone()).size(UI_LABEL_DEFAULT))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .when_some(id.filter(|_| removable), |row, id| {
                                        row.child(
                                            IconButton::new(
                                                format!("plugin-information-rejected-{id}"),
                                                IconName::Info,
                                            )
                                            .aria_label(format!("Information about {id}"))
                                            .tooltip(Tooltip::text("Plugin Information"))
                                            .disabled(!origin_available || busy)
                                            .on_click(
                                                cx.listener(move |this, _, _, cx| {
                                                    this.open_plugin_information(id.clone(), cx)
                                                }),
                                            ),
                                        )
                                    })
                                    .child(
                                        Switch::new(
                                            format!("plugin-rejected-{name}"),
                                            false.into(),
                                        )
                                        .disabled(true)
                                        .aria_label(format!("Enable {name}")),
                                    ),
                            ),
                    )
                    .child(Label::new(rejected.why).size(UI_LABEL_SMALL).color(Color::Error))
            })
            .collect();
        let _ = window;
        v_flex()
            .gap_4()
            .child(install_actions)
            .when(show_git, |view| view.child(git_form))
            .when_some(self.plugin_operation.clone(), |view, status| {
                view.child(Banner::new().child(
                    h_flex().gap_3().child(Label::new(status).size(UI_LABEL_DEFAULT)).when(
                        self.plugin_cancel.is_some(),
                        |row| {
                            row.child(settings_button("cancel-plugin-install", "Cancel").on_click(
                                cx.listener(|this, _, _, cx| {
                                    if let Some(cancel) = &this.plugin_cancel {
                                        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                                        this.plugin_operation =
                                            Some("Cancelling installation…".into());
                                        cx.notify();
                                    }
                                }),
                            ))
                        },
                    ),
                ))
            })
            .when(self.plugin_restart_required, |view| {
                view.child(
                    Banner::new().child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .gap_3()
                            .child(Label::new("Restart Chartr to enable installed plugins."))
                            .child(
                                settings_button("restart-after-plugin-install", "Restart")
                                    .disabled(busy)
                                    .on_click(restart),
                            ),
                    ),
                )
            })
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
