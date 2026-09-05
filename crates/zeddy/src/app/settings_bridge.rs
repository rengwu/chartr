//! Settings window integration and workspace-scoped settings operations.

use super::*;

impl Zeddy {
    pub(super) fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_input.update(cx, |input, cx| input.clear(cx));
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

    pub(crate) fn settings_plugins(
        &self,
    ) -> (Vec<SettingsPluginDescriptor>, Vec<SettingsPluginRejection>) {
        let mut descriptors: Vec<_> = self
            .catalog
            .loaded
            .values()
            .map(|loaded| SettingsPluginDescriptor {
                manifest: loaded.manifest.clone(),
                enabled: true,
                has_settings: loaded.has_settings,
            })
            .chain(self.catalog.disabled.values().map(|disabled| SettingsPluginDescriptor {
                manifest: disabled.manifest.clone(),
                enabled: false,
                has_settings: false,
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

    pub(crate) fn settings_set_plugin_enabled(
        &mut self,
        plugin: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if enabled {
            self.catalog.enable(&plugin_paths(), &plugin, cx).map_err(|error| error.to_string())?;
        }
        let result = crate::settings::update_global(cx, |content| {
            content
                .plugins
                .entry(plugin.clone())
                .or_insert_with(PluginSettingsContent::default)
                .enabled = Some(enabled);
        });
        match result {
            Ok(_) => {
                if !enabled {
                    self.close_plugin_instances(&plugin, cx);
                    self.catalog.disable(&plugin);
                }
                self.problem = None;
                cx.notify();
                Ok(())
            }
            Err(error) => {
                if enabled {
                    self.catalog.disable(&plugin);
                }
                Err(error.to_string())
            }
        }
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
        let loaded = self.catalog.get(plugin)?;
        let permissions = loaded.permissions().clone();
        let unsafe_filesystem =
            cx.global::<SettingsStore>().resolved().plugin(plugin).unsafe_filesystem;
        let source = self.catalog.get_mut(plugin)?.settings(window, cx)?;
        Some(match source {
            SettingsSource::Native(view) => view,
            SettingsSource::Web(entry) => crate::web_plugin::view(
                entry,
                FileBroker::new(
                    None,
                    plugin_paths().data.join(plugin),
                    permissions.project_files,
                    unsafe_filesystem,
                ),
                permissions,
                None,
                None,
                window,
                cx,
            ),
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
