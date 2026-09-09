//! Saved prompts: one shared library, a table surface, and a read service.
mod store;

use std::{collections::HashMap, path::PathBuf};

use chartr_plugin::{
    Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar,
    services::{Prompts, SavedPrompt, ServiceExport},
};
use editor::Editor;
use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, Focusable, KeyBinding, Render, WeakEntity,
    Window, div, px,
};
use ui::{Button, ButtonStyle, Color, Icon, IconName, Label, prelude::*};

use crate::{
    components::{form_button, input_field},
    fonts::{UI_LABEL_LARGE, UI_LABEL_SMALL},
    text_input::TextInput,
};
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
        registrar.add_pane("main", "Saved Prompts");
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
        cx: &mut App,
    ) -> gpui::AnyView {
        cx.new(|cx| PromptsView::new(self.registry.clone(), cx)).into()
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

struct Draft {
    original: Option<SavedPrompt>,
    title: Entity<TextInput>,
    body: Entity<Editor>,
}

struct PromptsView {
    registry: Entity<Registry>,
    search: Entity<TextInput>,
    draft: Option<Draft>,
    deleting: Option<SavedPrompt>,
    error: Option<String>,
    copied: Option<String>,
}

impl PromptsView {
    fn new(registry: Entity<Registry>, cx: &mut Context<Self>) -> Self {
        cx.observe(&registry, |_, _, cx| cx.notify()).detach();
        let search = cx.new(|cx| TextInput::new("Search prompts…", cx));
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        Self { registry, search, draft: None, deleting: None, error: None, copied: None }
    }

    fn edit(&mut self, original: Option<SavedPrompt>, window: &mut Window, cx: &mut Context<Self>) {
        let title = cx.new(|cx| {
            let mut input = TextInput::new("A short title", cx);
            input.set_text(
                original.as_ref().map(|p| p.title.clone()).unwrap_or_default(),
                false,
                cx,
            );
            input
        });
        let body = cx.new(|cx| {
            let mut input = Editor::auto_height(10, 24, window, cx);
            input.set_soft_wrap();
            input.set_autoindent(false);
            input.set_use_autoclose(false);
            input.set_show_wrap_guides(false, cx);
            input.set_show_indent_guides(false, cx);
            input.set_placeholder_text("Write the prompt you want to reuse…", window, cx);
            input.set_text(
                original.as_ref().map(|p| p.prompt.clone()).unwrap_or_default(),
                window,
                cx,
            );
            input
        });
        window.focus(&title.focus_handle(cx), cx);
        self.draft = Some(Draft { original, title, body });
        self.deleting = None;
        self.error = None;
        self.copied = None;
        cx.notify();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = &self.draft else { return };
        let title = draft.title.read(cx).text().to_owned();
        let body = draft.body.read(cx).text(cx);
        let original = draft.original.clone();
        let result = self.registry.update(cx, |registry, cx| {
            registry.modify(|store| store.save(original.as_ref(), title, body), cx)
        });
        match result {
            Ok(()) => {
                self.draft = None;
                self.error = None;
            }
            Err(error) => self.error = Some(error),
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
                let confirming = self.deleting.as_ref().is_some_and(|p| p.id == prompt.id);
                let actions = if confirming {
                    h_flex()
                        .gap_1()
                        .child(Button::new(("confirm-delete-prompt", index), "Delete?").on_click(
                            cx.listener(|this, _, _, cx| {
                                let Some(original) = this.deleting.clone() else { return };
                                let result = this.registry.update(cx, |registry, cx| {
                                    registry.modify(|store| store.delete(&original), cx)
                                });
                                this.error = result.err();
                                this.deleting = None;
                                cx.notify();
                            }),
                        ))
                        .child(Button::new(("cancel-delete-prompt", index), "Cancel").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.deleting = None;
                                cx.notify();
                            }),
                        ))
                } else {
                    h_flex()
                        .gap_1()
                        .child(
                            Button::new(
                                ("copy-prompt", index),
                                if self.copied.as_ref() == Some(&prompt.id) {
                                    "Copied"
                                } else {
                                    "Copy"
                                },
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        copying.prompt.clone(),
                                    ));
                                    this.copied = Some(copying.id.clone());
                                    cx.notify();
                                },
                            )),
                        )
                        .child(Button::new(("edit-prompt", index), "Edit").on_click(cx.listener(
                            move |this, _, window, cx| this.edit(Some(editing.clone()), window, cx),
                        )))
                        .child(Button::new(("delete-prompt", index), "Delete").on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.deleting = Some(deleting.clone());
                                this.error = None;
                                cx.notify();
                            }),
                        ))
                };
                let preview = prompt.prompt.split_whitespace().collect::<Vec<_>>().join(" ");
                columns([
                    Label::new(prompt.title).truncate().into_any_element(),
                    Label::new(preview).color(Color::Muted).truncate().into_any_element(),
                    actions.into_any_element(),
                ])
                .id(("prompt-row", index))
                .py_2()
                .flex_none()
                .border_b_1()
                .border_color(cx.theme().colors().border_variant)
                .into_any_element()
            })
            .collect::<Vec<_>>();
        v_flex()
            .size_full()
            .min_h_0()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .gap_2()
                    .child(Label::new("Saved Prompts").size(UI_LABEL_LARGE))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("reload-prompts", "Reload").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.registry.update(cx, |registry, cx| registry.reload(cx));
                                    this.deleting = None;
                                    this.error = None;
                                    this.copied = None;
                                    cx.notify();
                                },
                            )))
                            .child(
                                form_button("new-prompt", "New prompt")
                                    .disabled(blocked)
                                    .start_icon(Icon::new(IconName::Plus))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.edit(None, window, cx)
                                    })),
                            ),
                    ),
            )
            .child(Label::new("Your reusable prompts, shared across spaces.").color(Color::Muted))
            .when_some(registry.store.as_ref().err().cloned(), |view, error| {
                view.child(
                    Label::new(format!("{error} Fix the library file, then Reload."))
                        .color(Color::Error),
                )
            })
            .when_some(self.error.clone(), |view, error| {
                view.child(Label::new(error).color(Color::Error))
            })
            .child(input_field("search-prompts", self.search.clone(), cx))
            .child(
                v_flex().id("prompt-table").flex_1().min_h_0().overflow_x_scroll().child(
                    v_flex()
                        .min_w(px(560.))
                        .w_full()
                        .h_full()
                        .min_h_0()
                        .child(
                            columns(["Name", "Content", "Actions"].map(|label| {
                                Label::new(label)
                                    .size(UI_LABEL_SMALL)
                                    .color(Color::Muted)
                                    .into_any_element()
                            }))
                            .pb_2()
                            .border_b_1()
                            .border_color(cx.theme().colors().border),
                        )
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
                                        div().py_4().child(Label::new(empty).color(Color::Muted)),
                                    )
                                }),
                        ),
                ),
            )
            .into_any_element()
    }

    fn editor(&self, draft: &Draft, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .min_h_0()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .gap_2()
                    .child(
                        Label::new(if draft.original.is_some() {
                            "Edit prompt"
                        } else {
                            "New prompt"
                        })
                        .size(UI_LABEL_LARGE),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("cancel-prompt-edit", "Cancel").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.draft = None;
                                    this.error = None;
                                    cx.notify();
                                }),
                            ))
                            .child(
                                form_button("save-prompt", "Save prompt")
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                            ),
                    ),
            )
            .when_some(self.error.clone(), |view, error| {
                view.child(Label::new(error).color(Color::Error))
            })
            .child(
                v_flex()
                    .id("prompt-fields")
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .gap_3()
                    .overflow_y_scroll()
                    .child(Label::new("Title"))
                    .child(input_field("prompt-title", draft.title.clone(), cx))
                    .child(
                        Label::new("For your reference. Not included in the prompt.")
                            .size(UI_LABEL_SMALL)
                            .color(Color::Muted),
                    )
                    .child(Label::new("Prompt"))
                    .child(
                        div()
                            .w_full()
                            .flex_none()
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .child(draft.body.clone()),
                    )
                    .child(
                        Label::new("Only this text is copied or used as the prompt.")
                            .size(UI_LABEL_SMALL)
                            .color(Color::Muted),
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
        .child(div().w(px(180.)).flex_none().child(actions))
}

impl Render for PromptsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("prompts-pane")
            .key_context("Prompts")
            .size_full()
            .min_h_0()
            .p_4()
            .bg(cx.theme().colors().editor_background)
            .child(if let Some(draft) = &self.draft {
                self.editor(draft, cx)
            } else {
                self.table(cx)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_plugin::services::{PROMPTS_SERVICE, Services};

    #[gpui::test]
    fn multiline_editing_saves_only_body_and_service_sees_live_renames(
        cx: &mut gpui::TestAppContext,
    ) {
        let root = tempfile::tempdir().unwrap();
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            init(cx);
        });
        let plugin = cx.update(|cx| {
            PromptsPlugin::new(
                Host { data_dir: root.path().into(), plugin_dir: root.path().into() },
                cx,
            )
        });
        let registry = plugin.registry.clone();
        let services = Services::default();
        services.publish(PROMPTS_SERVICE, Plugin::services(&plugin));
        let service = services.get::<Prompts>(PROMPTS_SERVICE).unwrap();
        let (view, cx) = cx.add_window_view(|_, cx| PromptsView::new(registry.clone(), cx));
        view.update_in(cx, |view, window, cx| view.edit(None, window, cx));
        cx.simulate_input("Review code");
        let body = cx.read_entity(&view, |view, _| view.draft.as_ref().unwrap().body.clone());
        cx.update(|window, cx| window.focus(&body.focus_handle(cx), cx));
        cx.simulate_input("First line");
        cx.simulate_keystrokes("enter");
        cx.simulate_input("Second line 🦀");
        view.update(cx, |view, cx| view.save(cx));
        let saved = cx.update(|_, cx| service.list(cx).unwrap().remove(0));
        assert_eq!(saved.title, "Review code");
        assert_eq!(saved.prompt, "First line\nSecond line 🦀");
        assert!(cx.read_entity(&view, |view, _| view.draft.is_none()));

        view.update_in(cx, |view, window, cx| view.edit(Some(saved.clone()), window, cx));
        view.update(cx, |view, cx| {
            view.draft
                .as_ref()
                .unwrap()
                .title
                .update(cx, |input, cx| input.set_text("Renamed", false, cx));
            view.save(cx);
        });
        let renamed = cx.update(|_, cx| service.resolve(&saved.id, cx).unwrap());
        assert_eq!(renamed.title, "Renamed");
        assert_eq!(renamed.prompt, saved.prompt);
        let reloaded = Store::load(root.path().join("prompts.json")).unwrap();
        assert_eq!(reloaded.prompts(), &[renamed]);
    }
}
