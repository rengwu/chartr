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
        let confirmation = window.prompt(
            gpui::PromptLevel::Warning,
            &format!("Uninstall {name}?"),
            Some("Its panes will close and its installed copy and pending update will be removed. Plugin data and preferences are kept. Reinstalling leaves it disabled until you enable it."),
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
        if let Some(view) = view {
            self.plugin_settings = Some((plugin, view));
            self.problem = None;
        }
        cx.notify();
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
        let git_focus = self.git_url_input.focus_handle(cx);
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
            .child(
                h_flex()
                    .h(ButtonSize::Default.rems())
                    .px_2()
                    .border_1()
                    .border_color(colors.border_variant)
                    .bg(colors.surface_background)
                    .track_focus(&git_focus)
                    .in_focus(|field| field.border_color(colors.border_focused))
                    .child(self.git_url_input.clone()),
            )
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
        let settings = self.settings(cx);
        let enabled_plugins: std::collections::HashSet<_> = descriptors
            .iter()
            .filter(|descriptor| descriptor.enabled)
            .map(|descriptor| descriptor.manifest.id.clone())
            .collect();
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
                zeddy_plugin::manifest::Kind::Hosted => format!(
                    "Identifier: {id}. Uses Chartr's built-in {} surface.",
                    manifest.surface.as_deref().unwrap_or("hosted")
                ),
                zeddy_plugin::manifest::Kind::Web => format!(
                    "Identifier: {id}. Declared host access: {}.",
                    manifest.permissions.summary()
                ),
            };

            let enabled_name = format!("{name} — Enabled");
            let source = match &descriptor.installation {
                Some(installation) => {
                    let commit = installation
                        .commit
                        .as_ref()
                        .map(|commit| format!(" Commit: {commit}."))
                        .unwrap_or_default();
                    format!("Source: {}.{commit}", installation.source)
                }
                None if descriptor.removable => {
                    "Source: not recorded for this installation.".into()
                }
                None => "Bundled with Chartr.".into(),
            };
            let enabled_description = format!("{access} Version: {}. {source}", manifest.version);
            let enabled_id = id.clone();
            let enabled_setting = cx.weak_entity();
            let enabled_control = Switch::new(format!("plugin-enabled-{id}"), enabled.into())
                .disabled(!origin_available || busy)
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

            for dependency in &manifest.dependencies {
                let available = enabled_plugins.contains(&dependency.plugin);
                let provider = dependency.plugin.clone();
                fields.push(setting_field(
                    format!("{name} — {}", dependency.feature),
                    format!(
                        "Requires {}. {}",
                        provider,
                        if available {
                            "Provider enabled; configure it to finish setup."
                        } else {
                            "Provider is missing or disabled. Other features remain available."
                        }
                    ),
                    settings_button(
                        format!("dependency-{id}-{provider}"),
                        if available { "Configure" } else { "Unavailable" },
                    )
                    .disabled(!available || busy)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_plugin_settings(provider.clone(), window, cx)
                    })),
                ));
            }

            if descriptor.removable {
                let remove_id = id.clone();
                let remove_name = name.clone();
                fields.push(setting_field(
                    format!("{name} — Installation"),
                    "Remove this package and its pending update. Keeps plugin data and preferences.",
                    settings_button(format!("plugin-uninstall-{id}"), "Uninstall…")
                        .disabled(!origin_available || busy)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.uninstall_plugin(remove_id.clone(), remove_name.clone(), window, cx)
                        })),
                ));
            }

            if has_settings {
                let settings_id = id.clone();
                fields.push(setting_field(
                    format!("{name} — Configuration"),
                    "Open this plugin's own settings.",
                    settings_button(format!("plugin-settings-{id}"), "Configure")
                        .disabled(!origin_available || busy)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open_plugin_settings(settings_id.clone(), window, cx)
                        })),
                ));
            }

            if is_web {
                let unsafe_name = format!("{name} — Unsafe filesystem access");
                let unsafe_description = "Allow the project-file API to read and write outside the project, overriding its declared file permissions.";
                let unsafe_id = id.clone();
                let unsafe_setting = cx.weak_entity();
                let unsafe_control =
                    Switch::new(format!("plugin-unsafe-{id}"), unsafe_filesystem.into())
                        .disabled(!origin_available || busy)
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
        let installed_root = crate::app::plugin_paths().installed;
        let rejected: Vec<_> = rejected
            .into_iter()
            .map(|rejected| {
                let id = rejected.dir.file_name().and_then(|name| name.to_str()).map(str::to_owned);
                let removable = rejected.dir.parent() == Some(installed_root.as_path());
                let row = h_flex()
                    .gap_3()
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(format!("{}: {}", rejected.dir.display(), rejected.why))
                                .size(UI_LABEL_SMALL),
                        ),
                    )
                    .when_some(id.filter(|_| removable), |row, id| {
                        row.child(
                            settings_button(
                                format!("plugin-uninstall-rejected-{id}"),
                                "Uninstall…",
                            )
                            .disabled(!origin_available || busy)
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.uninstall_plugin(id.clone(), id.clone(), window, cx)
                                },
                            )),
                        )
                    });
                Banner::new().severity(Severity::Error).child(row)
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
