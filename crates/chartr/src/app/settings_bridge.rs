//! Settings window integration and workspace-scoped settings operations.

use super::*;

impl WorkspaceWindow {
    pub(super) fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(palette) = self.command_palette_window.take() {
            let _ = palette.update(cx, |_, window, _| window.remove_window());
        }
        let Some(original_window) = window.window_handle().downcast::<Self>() else {
            return;
        };
        crate::settings_window::open(original_window, cx.weak_entity(), cx);
        cx.notify();
    }

    fn close_plugin_instances(&mut self, plugin: &str, cx: &mut Context<Self>) {
        for space in &self.spaces {
            let ids = space.read(cx).plugin_item_ids(plugin);
            space.update(cx, |space, _| space.finish_bulk_close(&ids));
        }
    }

    pub(crate) fn settings_backend_label(&self) -> String {
        match &self.backend {
            Backend::Ready => "Connected".to_owned(),
            Backend::Starting => "Starting".to_owned(),
            Backend::Recovering(detail) | Backend::Failed(detail) => detail.clone(),
        }
    }

    pub(crate) fn settings_mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn settings_set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        self.mode = mode;
        cx.notify();
    }

    pub(crate) fn settings_show_space_picker(&self) -> bool {
        self.show_space_picker
    }

    pub(crate) fn settings_set_show_space_picker(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_space_picker = show;
        cx.notify();
    }

    pub(crate) fn settings_retry_backend(&mut self, cx: &mut Context<Self>) {
        self.retry_backend(false, cx);
    }

    pub(crate) fn settings_restart_backend(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.request_backend_restart(window, cx);
    }

    /// Apply global plugin changes to every workspace catalog and close stale views.
    pub(super) fn sync_plugin_settings(
        &mut self,
        previous: &crate::settings::ResolvedSettings,
        cx: &mut Context<Self>,
    ) {
        let current = self.settings.resolved().clone();
        let ids: Vec<_> =
            self.catalog.loaded.keys().chain(self.catalog.disabled.keys()).cloned().collect();
        for id in &ids {
            let old = previous.plugin(id);
            let new = current.plugin(id);
            if !new.enabled || (new.uninstalled && !old.uninstalled) {
                for dependent in self.catalog.dependents(id) {
                    self.close_plugin_instances(&dependent, cx);
                }
                self.close_plugin_instances(id, cx);
                self.catalog.disable(id);
            } else if new.unsafe_filesystem != old.unsafe_filesystem {
                self.close_plugin_instances(id, cx);
            }
            if new.uninstalled && !old.uninstalled {
                self.catalog.disabled.remove(id);
            }
        }
        self.catalog.rejected.retain(|rejected| {
            let Some(id) = rejected.dir.file_name().and_then(|name| name.to_str()) else {
                return true;
            };
            !current.plugin(id).uninstalled || previous.plugin(id).uninstalled
        });
        if let Some(bridge) = &self.companion_bridge {
            cx.set_global(bridge.clone());
        }
        self.catalog.enable_requested(
            &plugin_paths(),
            |id| current.plugin(id).enabled && !previous.plugin(id).enabled,
            cx,
        );
    }

    pub(crate) fn settings_plugins(
        &self,
    ) -> (Vec<SettingsPluginDescriptor>, Vec<SettingsPluginRejection>) {
        let mut descriptors: Vec<_> = self
            .catalog
            .loaded
            .values()
            .map(|loaded| SettingsPluginDescriptor {
                manifest: loaded.manifest.clone(),
                installation: loaded.installation.clone(),
                prerequisite_error: self.catalog.prerequisite_error(&loaded.manifest.id),
                enabled: true,
                bundled: loaded.dir == plugin_paths().bundled.join(&loaded.manifest.id),
            })
            .chain(self.catalog.disabled.values().map(|disabled| SettingsPluginDescriptor {
                manifest: disabled.manifest.clone(),
                installation: disabled.installation.clone(),
                prerequisite_error: self.catalog.prerequisite_error(&disabled.manifest.id),
                enabled: false,
                bundled: disabled.dir == plugin_paths().bundled.join(&disabled.manifest.id),
            }))
            .collect();
        descriptors.sort_by(|left, right| left.manifest.name.cmp(&right.manifest.name));
        let rejected = self
            .catalog
            .rejected
            .iter()
            .map(|rejected| SettingsPluginRejection {
                dir: rejected.dir.clone(),
                why: rejected.why.clone(),
            })
            .collect();
        (descriptors, rejected)
    }

    pub(crate) fn settings_uninstall_plugin(
        &mut self,
        plugin: String,
        cx: &mut Context<Self>,
    ) -> Result<gpui::Task<Result<(), String>>, String> {
        let paths = plugin_paths();
        let installed = paths.installed.join(&plugin);
        let bundled = paths.bundled.join(&plugin);
        let directory = self
            .catalog
            .get(&plugin)
            .map(|loaded| &loaded.dir)
            .or_else(|| self.catalog.disabled.get(&plugin).map(|disabled| &disabled.dir))
            .or_else(|| {
                self.catalog
                    .rejected
                    .iter()
                    .find(|rejected| rejected.dir == installed || rejected.dir == bundled)
                    .map(|rejected| &rejected.dir)
            });
        if directory != Some(&installed) && directory != Some(&bundled) {
            return Err("That plugin is no longer installed.".into());
        }
        let was_uninstalled = self.settings.resolved().plugin(&plugin).uninstalled;
        self.settings_set_plugin_enabled(plugin.clone(), false, cx)?;
        crate::settings::update_global(cx, |content| {
            content.plugins.entry(plugin.clone()).or_default().uninstalled = Some(true);
        })
        .map_err(|error| error.to_string())?;
        // Do not leave an Enable control for a package being removed.
        let disabled = self.catalog.disabled.remove(&plugin);
        let rejected = self
            .catalog
            .rejected
            .iter()
            .position(|rejected| rejected.dir == installed || rejected.dir == bundled)
            .map(|index| self.catalog.rejected.remove(index));
        let id = plugin.clone();
        let remove = cx.background_executor().spawn(async move {
            crate::plugin_installer::uninstall(&id, &paths).map_err(|error| format!("{error:#}"))
        });
        // Finish the catalog update even if the Settings window closes meanwhile.
        Ok(cx.spawn(async move |this, cx| {
            let result = remove.await;
            let _ = this.update(cx, |this, cx| {
                if result.is_err() {
                    if let Err(error) = crate::settings::update_global(cx, |content| {
                        content.plugins.entry(plugin.clone()).or_default().uninstalled =
                            Some(was_uninstalled);
                    }) {
                        this.problem =
                            Some(format!("Could not restore plugin preferences: {error}"));
                    }
                    if let Some(disabled) = disabled {
                        this.catalog.disabled.insert(plugin, disabled);
                    }
                    if let Some(rejected) = rejected {
                        this.catalog.rejected.push(rejected);
                    }
                }
                cx.notify();
            });
            result
        }))
    }

    pub(crate) fn settings_set_plugin_enabled(
        &mut self,
        plugin: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let was_enabled = self.catalog.get(&plugin).is_some();
        if enabled {
            if let Some(bridge) = &self.companion_bridge {
                cx.set_global(bridge.clone());
            }
            self.catalog.enable(&plugin_paths(), &plugin, cx).map_err(|error| error.to_string())?;
        }
        let mut affected = if enabled { Vec::new() } else { self.catalog.dependents(&plugin) };
        affected.push(plugin.clone());
        let result = crate::settings::update_global(cx, |content| {
            for id in &affected {
                content.plugins.entry(id.clone()).or_default().enabled = Some(enabled);
            }
        });
        match result {
            Ok(_) => {
                if !enabled {
                    for id in &affected {
                        self.close_plugin_instances(id, cx);
                    }
                    self.catalog.disable(&plugin);
                }
                self.problem = None;
                cx.notify();
                Ok(())
            }
            Err(error) => {
                if enabled && !was_enabled {
                    self.catalog.disable(&plugin);
                }
                Err(error.to_string())
            }
        }
    }

    pub(crate) fn settings_plugin_dependents(&self, plugin: &str) -> Vec<String> {
        self.catalog
            .dependents(plugin)
            .into_iter()
            .filter_map(|id| self.catalog.manifest(&id).map(|manifest| manifest.name.clone()))
            .collect()
    }

    pub(crate) fn settings_set_plugin_unsafe(
        &mut self,
        plugin: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        crate::settings::update_global(cx, |content| {
            content
                .plugins
                .entry(plugin.clone())
                .or_insert_with(PluginSettingsContent::default)
                .unsafe_filesystem = Some(enabled);
        })
        .map_err(|error| error.to_string())?;
        // Brokers are instance-owned. Destroying every instance is the
        // revocation boundary; reopening constructs one with the new grant.
        self.close_plugin_instances(&plugin, cx);
        self.problem = None;
        cx.notify();
        Ok(())
    }

    pub(crate) fn settings_plugin_view(
        &mut self,
        plugin: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyView> {
        let source = self.catalog.get_mut(plugin)?.settings(window, cx)?;
        Some(match source {
            SettingsSource::Native(view) => view,
            SettingsSource::Declarative(schema) => {
                crate::plugin_settings::view(schema, plugin_paths().data.join(plugin), cx)
            }
        })
    }

    pub(crate) fn settings_set_free_sessions_directory(
        &mut self,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        if let Some(space) =
            self.spaces.iter().find(|space| space.read(cx).kind() == SpaceKind::AdHoc)
        {
            space.update(cx, |space, _| space.set_path(path));
        }
        cx.notify();
    }
}
