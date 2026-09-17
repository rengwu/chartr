//! Saved prompts: one shared library, a settings page, and a read service.
mod dialog;
mod store;

use chartr_plugin::ui as plugin_ui;
use std::{collections::HashMap, path::PathBuf};

use chartr_plugin::{
    Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar, RenderSettings, SettingsPage,
    SettingsView,
    services::{Prompts, SavedPrompt, ServiceExport},
};
use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, Focusable, KeyBinding, Render, WeakEntity,
    Window, div, px,
};
use ui::{Color, Icon, IconName, prelude::*};

use crate::{components::input_field, text_input::TextInput};
use dialog::PromptDialog;
use store::Store;

/// Reuse the pinned editor's platform editing keys. Project/workspace actions
/// are not part of this plain text field.
pub fn init(cx: &mut App) {
    let bindings = ::settings::KeymapFile::load_asset_allow_partial_failure(
        ::settings::DEFAULT_KEYMAP_PATH,
        cx,
    )
    .expect("the pinned editor keymap must remain loadable");
    cx.bind_keys(bindings.into_iter().filter(|binding| {
        let Some(name) = binding.action().name().strip_prefix("editor::") else { return false };
        name.starts_with("Move")
            || name.starts_with("Select")
            || name.starts_with("Delete")
            || matches!(name, "Backspace" | "Copy" | "Cut" | "Paste" | "Undo" | "Redo" | "Newline")
    }));
    cx.bind_keys([KeyBinding::new("enter", editor::actions::Newline, Some("Prompts > Editor"))]);
}

pub struct PromptsPlugin {
    registry: Entity<Registry>,
}

#[derive(Default)]
struct SharedRegistries(HashMap<PathBuf, WeakEntity<Registry>>);
impl gpui::Global for SharedRegistries {}

pub fn bundled(host: Host, cx: &mut App) -> Box<dyn PluginObject> {
    Box::new(PromptsPlugin::new(host, cx))
}

impl Plugin for PromptsPlugin {
    const ID: &'static str = "com.chartr.prompts";

    fn new(host: Host, cx: &mut App) -> Self {
        let path = host.data_dir.join("prompts.json");
        let existing =
            cx.default_global::<SharedRegistries>().0.get(&path).and_then(WeakEntity::upgrade);
        let registry = existing.unwrap_or_else(|| {
            let registry = cx.new(|_| Registry::load(path.clone()));
            cx.default_global::<SharedRegistries>().0.insert(path, registry.downgrade());
            registry
        });
        Self { registry }
    }

    fn activate(&mut self, registrar: &mut Registrar, _: &mut App) {
        registrar.add_settings();
    }

    fn services(&self) -> Vec<ServiceExport> {
        let registry = self.registry.downgrade();
        let templates = registry.clone();
        vec![
            ServiceExport::new(chartr_plugin::services::PromptTemplates::new(move |_, cx| {
                let result = templates
                    .upgrade()
                    .ok_or_else(|| "Saved Prompts is unavailable.".to_owned())
                    .and_then(|registry| {
                        registry
                            .read(cx)
                            .store
                            .as_ref()
                            .map(|store| store.prompts().to_vec())
                            .map_err(Clone::clone)
                    });
                gpui::Task::ready(result)
            })),
            ServiceExport::new(Prompts::new(move |cx| {
                let registry = registry.upgrade().ok_or("Prompts is unavailable.")?;
                let registry = registry.read(cx);
                registry.store.as_ref().map(|store| store.prompts().to_vec()).map_err(Clone::clone)
            })),
        ]
    }

    fn view(
        &mut self,
        _: &PaneKey,
        _: &InstanceContext,
        _: &mut Window,
        _: &mut App,
    ) -> gpui::AnyView {
        unreachable!("Saved Prompts contributes settings only")
    }

    fn settings(&mut self, window: &mut Window, cx: &mut App) -> Option<SettingsView> {
        let view = cx.new(|cx| PromptsView::new(self.registry.clone(), window, cx));
        Some(SettingsView::new(view, cx))
    }
}

struct Registry {
    path: PathBuf,
    store: Result<Store, String>,
}

impl Registry {
    fn load(path: PathBuf) -> Self {
        Self { store: Store::load(path.clone()), path }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        self.store = Store::load(self.path.clone());
        chartr_plugin::services::PromptTemplates::changed(cx);
        cx.notify();
    }

    fn modify(
        &mut self,
        operation: impl FnOnce(&mut Store) -> Result<(), String>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let result = match &mut self.store {
            Ok(store) => operation(store),
            Err(error) => Err(error.clone()),
        };
        if result.is_ok() {
            chartr_plugin::services::PromptTemplates::changed(cx);
        }
        cx.notify();
        result
    }
}

struct PromptsView {
    registry: Entity<Registry>,
    search: Entity<TextInput>,
    dialog: Option<gpui::WindowHandle<PromptDialog>>,
    error: Option<String>,
    copied: Option<String>,
}

impl PromptsView {
    fn new(registry: Entity<Registry>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe(&registry, |_, _, cx| cx.notify()).detach();
        let search = cx.new(|cx| TextInput::new("Search prompts…", cx));
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        cx.observe_window_bounds(window, |this, window, cx| {
            if let Some(dialog) = this.dialog {
                let size = window.viewport_size();
                let _ = dialog.update(cx, |_, window, _| window.resize(size));
            }
        })
        .detach();
        cx.on_release(|this, cx| {
            if let Some(dialog) = this.dialog.take() {
                let _ = dialog.update(cx, |_, window, _| window.remove_window());
            }
        })
        .detach();
        Self { registry, search, dialog: None, error: None, copied: None }
    }

    fn open_dialog(
        &mut self,
        original: Option<SavedPrompt>,
        deleting: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(dialog) = self.dialog {
            let _ = dialog.update(cx, |_, window, _| window.activate_window());
            return;
        }
        let registry = self.registry.clone();
        let parent = window.window_handle();
        let owner = cx.weak_entity();
        let opened = crate::components::open_native_modal(window, cx, move |window, cx| {
            let view = cx.new(|cx| {
                cx.on_release(move |_, cx| {
                    let _ = parent.update(cx, |_, window, cx| {
                        let _ = owner.update(cx, |owner, cx| {
                            owner.dialog = None;
                            window.focus(&owner.search.focus_handle(cx), cx);
                            cx.notify();
                        });
                    });
                })
                .detach();
                PromptDialog::new(registry, original, deleting, window, cx)
            });
            window.focus(&view.read(cx).initial_focus(cx), cx);
            view
        });
        match opened {
            Ok(dialog) => {
                self.dialog = Some(dialog);
                self.error = None;
                self.copied = None;
            }
            Err(error) => self.error = Some(format!("Could not open prompt dialog: {error}")),
        }
        cx.notify();
    }

    fn table(&self, cx: &mut Context<Self>) -> AnyElement {
        let registry = self.registry.read(cx);
        let blocked = registry.store.is_err();
        let prompts =
            registry.store.as_ref().map(|store| store.prompts().to_vec()).unwrap_or_default();
        let query = self.search.read(cx).text().trim().to_lowercase();
        let filtered: Vec<_> = prompts
            .iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&query) || p.prompt.to_lowercase().contains(&query)
            })
            .collect();
        let empty = if prompts.is_empty() {
            "No saved prompts yet. Add a prompt to start your library."
        } else {
            "No prompts match your search."
        };
        let rows = filtered
            .iter()
            .enumerate()
            .map(|(index, prompt)| {
                let prompt = (*prompt).clone();
                let editing = prompt.clone();
                let copying = prompt.clone();
                let deleting = prompt.clone();
                let copied = self.copied.as_ref() == Some(&prompt.id);
                let copy_label = if copied { "Copied prompt" } else { "Copy prompt" };
                let actions = h_flex()
                    .gap_1()
                    .child(
                        plugin_ui::icon_action(
                            ("copy-prompt", index),
                            if copied { IconName::Check } else { IconName::Copy },
                        )
                        .aria_label(format!("{copy_label}: {}", prompt.title))
                        .tooltip(copy_label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                copying.prompt.clone(),
                            ));
                            this.copied = Some(copying.id.clone());
                            cx.notify();
                        })),
                    )
                    .child(
                        plugin_ui::icon_action(("edit-prompt", index), IconName::Pencil)
                            .aria_label(format!("Edit prompt: {}", prompt.title))
                            .tooltip("Edit prompt")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_dialog(Some(editing.clone()), false, window, cx)
                            })),
                    )
                    .child(
                        plugin_ui::icon_action(("delete-prompt", index), IconName::Trash)
                            .aria_label(format!("Delete prompt: {}", prompt.title))
                            .tooltip("Delete prompt")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_dialog(Some(deleting.clone()), true, window, cx)
                            })),
                    );
                let preview = prompt.prompt.split_whitespace().collect::<Vec<_>>().join(" ");
                plugin_ui::separated_row(
                    columns([
                        plugin_ui::label(prompt.title).truncate().into_any_element(),
                        plugin_ui::label(preview).color(Color::Muted).truncate().into_any_element(),
                        actions.into_any_element(),
                    ]),
                    cx,
                )
                .id(("prompt-row", index))
                .py_2()
                .flex_none()
                .into_any_element()
            })
            .collect::<Vec<_>>();
        v_flex()
            .size_full()
            .min_h_0()
            .gap_3()
            .child(
                plugin_ui::PageHeader::new("Saved Prompts")
                    .description("Your reusable prompts, shared across spaces.")
                    .action(plugin_ui::action("reload-prompts", "Reload").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.registry.update(cx, |registry, cx| registry.reload(cx));
                            this.error = None;
                            this.copied = None;
                            cx.notify();
                        },
                    )))
                    .action(
                        plugin_ui::action("new-prompt", "New prompt")
                            .disabled(blocked)
                            .start_icon(Icon::new(IconName::Plus))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_dialog(None, false, window, cx)
                            })),
                    ),
            )
            .when_some(registry.store.as_ref().err().cloned(), |view, error| {
                view.child(
                    plugin_ui::label(format!("{error} Fix the library file, then Reload."))
                        .color(Color::Error),
                )
            })
            .when_some(self.error.clone(), |view, error| view.child(plugin_ui::notice(error, true)))
            .child(input_field("search-prompts", self.search.clone(), cx))
            .child(
                v_flex().id("prompt-table").flex_1().min_h_0().overflow_x_scroll().child(
                    v_flex()
                        .min_w(px(560.))
                        .w_full()
                        .h_full()
                        .min_h_0()
                        .child(plugin_ui::table_header(
                            columns(["Name", "Content", "Actions"].map(|label| {
                                plugin_ui::caption(label).color(Color::Muted).into_any_element()
                            })),
                            cx,
                        ))
                        .child(
                            v_flex()
                                .id("prompt-rows")
                                .w_full()
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .children(rows)
                                .when(filtered.is_empty() && !blocked, |view| {
                                    view.child(
                                        div()
                                            .py_4()
                                            .child(plugin_ui::label(empty).color(Color::Muted)),
                                    )
                                }),
                        ),
                ),
            )
            .into_any_element()
    }
}

fn columns([title, prompt, actions]: [AnyElement; 3]) -> gpui::Div {
    h_flex()
        .w_full()
        .gap_3()
        .child(div().w(px(150.)).flex_none().overflow_hidden().child(title))
        .child(div().flex_1().min_w_0().overflow_hidden().child(prompt))
        .child(div().w(px(100.)).flex_none().child(actions))
}

impl RenderSettings for PromptsView {
    fn render_settings(&mut self, _: &mut Window, cx: &mut Context<Self>) -> SettingsPage {
        SettingsPage::fill("prompts-settings")
            .child(v_flex().key_context("Prompts").size_full().min_h_0().child(self.table(cx)))
    }
}

impl Render for PromptsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_settings(window, cx)
    }
}
