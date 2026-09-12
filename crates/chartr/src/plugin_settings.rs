//! Portable plugin settings use native controls and the existing private JSON data file.
//! No plugin HTML or JavaScript is constructed in the Settings window.

use crate::components::{ContextMenu, FORM_CONTROL_SIZE, PopupMenu, form_picker, form_row};
use chartr_plugin::{
    ProjectAccess,
    settings::{SettingsControl, SettingsField, SettingsSchema},
};
use chartr_plugin_host::{BrokerError, FileBroker};
use gpui::{Anchor, AnyElement, AnyView, App, AppContext, Context, Render, Window};
use serde_json::{Map, Value};
use std::{io::Read, path::PathBuf};
use ui::{Color, IconPosition, Switch, prelude::*};

const MAX_SETTINGS_BYTES: u64 = 1024 * 1024;

pub fn view(schema: SettingsSchema, data: PathBuf, cx: &mut App) -> AnyView {
    cx.new(|_| {
        let store =
            Store { schema, broker: FileBroker::new(None, data, ProjectAccess::None, false) };
        let mut view =
            PluginSettings { store, values: Map::new(), problem: None, unreadable: false };
        view.reload();
        view
    })
    .into()
}

struct Store {
    schema: SettingsSchema,
    broker: FileBroker,
}

fn default_value(field: &SettingsField) -> Value {
    match &field.control {
        SettingsControl::Select { default, .. } => Value::String(default.clone()),
        SettingsControl::Toggle { default } => Value::Bool(*default),
    }
}

fn accepts(field: &SettingsField, value: &Value) -> bool {
    match &field.control {
        SettingsControl::Select { options, .. } => {
            options.iter().any(|option| value.as_str() == Some(&option.value))
        }
        SettingsControl::Toggle { .. } => value.is_boolean(),
    }
}

impl Store {
    fn read(&self) -> Result<Map<String, Value>, String> {
        let mut values = match self.broker.open_data(&self.schema.file) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(MAX_SETTINGS_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() as u64 > MAX_SETTINGS_BYTES {
                    return Err("Plugin settings exceed 1 MiB.".into());
                }
                serde_json::from_slice::<Map<String, Value>>(&bytes)
                    .map_err(|e| format!("Could not read {}: {e}", self.schema.file.display()))?
            }
            Err(BrokerError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Map::new()
            }
            Err(error) => return Err(error.to_string()),
        };
        for field in &self.schema.fields {
            let value = values.entry(field.key.clone()).or_insert_with(|| default_value(field));
            if !accepts(field, value) {
                return Err(format!(
                    "The saved value for '{}' is not supported. Correct {} and reload.",
                    field.label,
                    self.schema.file.display()
                ));
            }
        }
        Ok(values)
    }

    fn set(&self, key: &str, value: Value) -> Result<Map<String, Value>, String> {
        let field = self
            .schema
            .fields
            .iter()
            .find(|field| field.key == key)
            .ok_or_else(|| "Unknown setting.".to_owned())?;
        if !accepts(field, &value) {
            return Err(format!("Invalid value for '{}'.", field.label));
        }
        // Re-read before each update so other keys written by the pane are preserved.
        // A malformed or unreadable document must never be replaced with defaults.
        let mut values = self.read()?;
        values.insert(key.to_owned(), value);
        let bytes = serde_json::to_vec_pretty(&values).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_SETTINGS_BYTES {
            return Err("Plugin settings exceed 1 MiB.".into());
        }
        self.broker.write_data(&self.schema.file, &bytes).map_err(|e| e.to_string())?;
        Ok(values)
    }
}

struct PluginSettings {
    store: Store,
    values: Map<String, Value>,
    problem: Option<String>,
    unreadable: bool,
}

impl PluginSettings {
    fn reload(&mut self) {
        match self.store.read() {
            Ok(values) => {
                self.values = values;
                self.problem = None;
                self.unreadable = false;
            }
            Err(error) => {
                self.problem = Some(error);
                self.unreadable = true;
            }
        }
    }

    fn set(&mut self, key: &str, value: Value, cx: &mut Context<Self>) {
        match self.store.set(key, value) {
            Ok(values) => {
                self.values = values;
                self.problem = None;
            }
            Err(error) => self.problem = Some(error),
        }
        cx.notify();
    }

    fn field(&self, field: &SettingsField, cx: &mut Context<Self>) -> AnyElement {
        let value = self.values.get(&field.key).cloned().unwrap_or_else(|| default_value(field));
        let control = match &field.control {
            SettingsControl::Select { options, .. } => {
                let label = options
                    .iter()
                    .find(|option| value.as_str() == Some(&option.value))
                    .map(|option| option.label.clone())
                    .unwrap_or_default();
                let options = options.clone();
                let key = field.key.clone();
                let weak = cx.weak_entity();
                PopupMenu::new(format!("plugin-setting-{}", field.key))
                    .trigger(
                        form_picker(format!("plugin-setting-{}-trigger", field.key), label)
                            .disabled(self.unreadable),
                    )
                    .anchor(Anchor::TopLeft)
                    .menu(move |window, cx| {
                        Some(ContextMenu::build_popup(window, cx, |mut menu| {
                            for option in &options {
                                let weak = weak.clone();
                                let key = key.clone();
                                let selected = value.as_str() == Some(&option.value);
                                let value = Value::String(option.value.clone());
                                menu = menu.toggleable_entry(
                                    option.label.clone(),
                                    selected,
                                    IconPosition::End,
                                    None,
                                    move |_, cx| {
                                        let _ = weak.update(cx, |this, cx| {
                                            this.set(&key, value.clone(), cx)
                                        });
                                    },
                                );
                            }
                            menu
                        }))
                    })
                    .into_any_element()
            }
            SettingsControl::Toggle { .. } => {
                let key = field.key.clone();
                h_flex()
                    .min_h(FORM_CONTROL_SIZE.rems())
                    .child(
                        Switch::new(
                            format!("plugin-setting-{}", field.key),
                            value.as_bool().unwrap_or(false).into(),
                        )
                        .aria_label(field.label.clone())
                        .disabled(self.unreadable)
                        .on_click(cx.listener(
                            move |this, state: &ui::ToggleState, _, cx| {
                                this.set(&key, Value::Bool(state.selected()), cx);
                            },
                        )),
                    )
                    .into_any_element()
            }
        };
        let key = field.key.clone();
        let control = gpui::div()
            .debug_selector(move || format!("PLUGIN_SETTING-{key}"))
            .child(control)
            .into_any_element();
        form_row(field.label.clone(), field.description.as_deref(), control)
    }
}

impl chartr_plugin::RenderSettings for PluginSettings {
    fn render_settings(
        &mut self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> chartr_plugin::SettingsPage {
        let fields: Vec<_> =
            self.store.schema.fields.iter().map(|field| self.field(field, cx)).collect();
        chartr_plugin::SettingsPage::flow("native-plugin-settings")
            .children(fields)
            .child(
                chartr_plugin::ui::caption("Changes are saved for this plugin.")
                    .color(Color::Muted),
            )
            .when_some(self.problem.clone(), |page, problem| {
                page.child(chartr_plugin::ui::notice(problem, true).action(
                    chartr_plugin::ui::action("reload-plugin-settings", "Reload").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.reload();
                            cx.notify();
                        }),
                    ),
                ))
            })
    }
}
impl Render for PluginSettings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        chartr_plugin::RenderSettings::render_settings(self, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock_store(data: &std::path::Path) -> Store {
        let manifest = chartr_plugin::Manifest::parse(include_str!(
            "../../../examples/plugins/clock/chartr-plugin.toml"
        ))
        .unwrap();
        Store {
            schema: manifest.settings.unwrap(),
            broker: FileBroker::new(None, data.into(), ProjectAccess::None, false),
        }
    }

    #[test]
    fn clock_settings_preserve_existing_values_and_pane_owned_keys() {
        let data = tempfile::tempdir().unwrap();
        let store = clock_store(data.path());
        assert_eq!(store.read().unwrap()["format"], "24");
        assert!(!data.path().join("settings.json").exists());
        std::fs::write(data.path().join("settings.json"), r#"{"format":"12","timezone":"UTC"}"#)
            .unwrap();
        assert_eq!(store.read().unwrap()["format"], "12");
        let saved = store.set("format", Value::String("24".into())).unwrap();
        assert_eq!(saved["timezone"], "UTC");
        assert_eq!(clock_store(data.path()).read().unwrap()["format"], "24");
    }

    #[test]
    fn malformed_or_unsupported_saved_values_are_never_overwritten() {
        let data = tempfile::tempdir().unwrap();
        let store = clock_store(data.path());
        let file = data.path().join("settings.json");
        for content in ["{broken", "[]", r#"{"format":true}"#, r#"{"format":"unknown"}"#] {
            std::fs::write(&file, content).unwrap();
            assert!(store.set("format", "12".into()).is_err());
            assert_eq!(std::fs::read_to_string(&file).unwrap(), content);
        }
        std::fs::write(&file, r#"{"format":"24"}"#).unwrap();
        assert!(store.set("format", "unsupported".into()).is_err());
        assert!(store.set("undeclared", true.into()).is_err());
        assert_eq!(store.read().unwrap()["format"], "24");
    }

    #[test]
    fn native_settings_cannot_follow_a_link_outside_plugin_data() {
        let data = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(outside.path(), r#"{"format":"24"}"#).unwrap();
        std::os::unix::fs::symlink(outside.path(), data.path().join("settings.json")).unwrap();
        assert!(clock_store(data.path()).set("format", "12".into()).is_err());
        assert_eq!(std::fs::read_to_string(outside.path()).unwrap(), r#"{"format":"24"}"#);
    }

    #[gpui::test]
    fn clock_settings_render_and_save_through_a_native_picker(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
        let data = tempfile::tempdir().unwrap();
        let store = clock_store(data.path());
        let (_, cx) = cx.add_window_view(|_, _| PluginSettings {
            values: store.read().unwrap(),
            store,
            problem: None,
            unreadable: false,
        });
        cx.run_until_parked();
        let trigger = cx.debug_bounds("PLUGIN_SETTING-format").unwrap();
        cx.simulate_click(trigger.center(), gpui::Modifiers::none());
        let popup = cx.windows().into_iter().find(|window| *window != cx.window_handle()).unwrap();
        let mut popup = gpui::VisualTestContext::from_window(popup, cx);
        popup.run_until_parked();
        let choice = popup.debug_bounds("MENU_ITEM-12-hour").unwrap();
        popup.simulate_click(choice.center(), gpui::Modifiers::none());
        cx.run_until_parked();
        assert_eq!(clock_store(data.path()).read().unwrap()["format"], "12");
        assert_eq!(cx.windows().len(), 1);
    }
}
