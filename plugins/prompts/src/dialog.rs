//! Prompt tasks live above the Settings window while the library stays in place.

use super::*;
use editor::Editor;
use gpui::{FocusHandle, KeyDownEvent, Window};
use ui::{ButtonStyle, TintColor};

gpui::actions!(chartr_prompt_dialog, [Confirm]);

pub(super) fn init_keybindings(cx: &mut App) {
    let shortcut = if cfg!(target_os = "macos") { "cmd-enter" } else { "ctrl-enter" };
    cx.bind_keys([
        KeyBinding::new(shortcut, Confirm, Some("PromptDialog")),
        // Match at editor depth, after its platform bindings, so Ctrl-Enter
        // confirms the dialog before the auto-height editor consumes it.
        KeyBinding::new(shortcut, Confirm, Some("PromptDialog > Editor")),
    ]);
}

struct Draft {
    original: Option<SavedPrompt>,
    title: Entity<TextInput>,
    body: Entity<Editor>,
}

enum Task {
    Edit(Draft),
    Delete(SavedPrompt),
}

pub(super) struct PromptDialog {
    registry: Entity<Registry>,
    task: Task,
    error: Option<String>,
    focus: FocusHandle,
    cancel_focus: FocusHandle,
    confirm_focus: FocusHandle,
}

impl PromptDialog {
    pub(super) fn new(
        registry: Entity<Registry>,
        original: Option<SavedPrompt>,
        deleting: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let task = if deleting {
            Task::Delete(original.expect("deletion requires the selected prompt"))
        } else {
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
            Task::Edit(Draft { original, title, body })
        };
        Self {
            registry,
            task,
            error: None,
            focus: cx.focus_handle(),
            cancel_focus: cx.focus_handle(),
            confirm_focus: cx.focus_handle(),
        }
    }

    pub(super) fn initial_focus(&self, cx: &App) -> FocusHandle {
        match &self.task {
            Task::Edit(draft) => draft.title.focus_handle(cx),
            Task::Delete(_) => self.cancel_focus.clone(),
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let result = match &self.task {
            Task::Edit(draft) => {
                let title = draft.title.read(cx).text().to_owned();
                let body = draft.body.read(cx).text(cx);
                self.registry.update(cx, |registry, cx| {
                    registry.modify(|store| store.save(draft.original.as_ref(), title, body), cx)
                })
            }
            Task::Delete(original) => self
                .registry
                .update(cx, |registry, cx| registry.modify(|store| store.delete(original), cx)),
        };
        match result {
            Ok(()) => window.remove_window(),
            Err(error) => {
                self.error = Some(error);
                cx.notify();
            }
        }
    }

    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        match key.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                window.remove_window();
            }
            "tab" => {
                cx.stop_propagation();
                let mut fields = Vec::new();
                if let Task::Edit(draft) = &self.task {
                    fields.extend([draft.title.focus_handle(cx), draft.body.focus_handle(cx)]);
                }
                fields.extend([self.cancel_focus.clone(), self.confirm_focus.clone()]);
                let current = fields.iter().position(|focus| focus.is_focused(window)).unwrap_or(0);
                let offset = if key.modifiers.shift { fields.len() - 1 } else { 1 };
                window.focus(&fields[(current + offset) % fields.len()], cx);
            }
            "enter" | "space" if self.cancel_focus.is_focused(window) => {
                cx.stop_propagation();
                window.remove_window();
            }
            "enter" | "space" if self.confirm_focus.is_focused(window) => {
                cx.stop_propagation();
                self.submit(window, cx);
            }
            _ => {}
        }
    }
}

impl Render for PromptDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let font = crate::fonts::Fonts::setup_ui(window, cx);
        let (title, confirm_label, deleting) = match &self.task {
            Task::Edit(draft) => (
                if draft.original.is_some() { "Edit prompt" } else { "New prompt" },
                "Save prompt",
                false,
            ),
            Task::Delete(_) => ("Delete prompt?", "Delete", true),
        };
        let mut body = plugin_ui::dialog_body()
            .id("prompt-dialog-body")
            .debug_selector(|| "PROMPT_DIALOG_BODY".into())
            .overflow_y_scroll();
        body = match &self.task {
            Task::Edit(draft) => body
                .child(plugin_ui::label("Title"))
                .child(input_field("prompt-title", draft.title.clone(), cx))
                .child(
                    plugin_ui::caption("For your reference. Not included in the prompt.")
                        .color(Color::Muted),
                )
                .child(plugin_ui::label("Prompt"))
                .child(
                    plugin_ui::outlined_content(cx).w_full().flex_none().child(draft.body.clone()),
                )
                .child(
                    plugin_ui::caption("Only this text is copied or used as the prompt.")
                        .color(Color::Muted),
                ),
            Task::Delete(prompt) => body.child(plugin_ui::label(format!(
                "Delete “{}” from Saved Prompts? This cannot be undone.",
                prompt.title
            ))),
        };
        let dialog = plugin_ui::DialogSurface::new("prompt-dialog")
            .debug_selector("PROMPT_DIALOG")
            .when(deleting, |dialog| dialog.compact())
            .height_limit((window.viewport_size().height - px(64.)).max(px(0.)))
            .child(plugin_ui::dialog_header(title, div(), cx))
            .child(body.when_some(self.error.clone(), |body, error| {
                body.child(plugin_ui::notice(error, true))
            }))
            .child(
                plugin_ui::dialog_actions(cx)
                    .debug_selector(|| "PROMPT_DIALOG_ACTIONS".into())
                    .child(
                        plugin_ui::form_button("cancel-prompt-dialog", "Cancel")
                            .track_focus(&self.cancel_focus)
                            .on_click(|_, window, _| window.remove_window()),
                    )
                    .child(
                        plugin_ui::form_button("confirm-prompt-dialog", confirm_label)
                            .style(if deleting {
                                ButtonStyle::Tinted(TintColor::Error)
                            } else {
                                ButtonStyle::Filled
                            })
                            .track_focus(&self.confirm_focus)
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
            );
        div()
            .id("prompt-modal")
            .role(gpui::Role::Dialog)
            .aria_label(title)
            .key_context("Prompts PromptDialog")
            .track_focus(&self.focus)
            .size_full()
            .font(font)
            .text_size(crate::fonts::UI_TEXT_DEFAULT)
            .text_color(cx.theme().colors().text)
            .on_action(cx.listener(|this, _: &Confirm, window, cx| this.submit(window, cx)))
            .capture_key_down(cx.listener(Self::on_key))
            .child(
                plugin_ui::ModalOverlay::new("prompt-dialog-scrim", |_, window, _| {
                    window.remove_window()
                })
                .child(dialog),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_plugin::services::{PROMPTS_SERVICE, Services};
    use gpui::{TestAppContext, VisualTestContext, size};

    fn init_test(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            init(cx);
        });
    }

    fn modal(view: &Entity<PromptsView>, cx: &VisualTestContext) -> VisualTestContext {
        let handle = cx.read_entity(view, |view, _| view.dialog.unwrap());
        VisualTestContext::from_window(handle.into(), cx)
    }

    fn save_shortcut() -> &'static str {
        if cfg!(target_os = "macos") { "cmd-enter" } else { "ctrl-enter" }
    }

    #[gpui::test]
    fn multiline_modal_keeps_actions_visible_and_publishes_saved_edits(cx: &mut TestAppContext) {
        init_test(cx);
        let root = tempfile::tempdir().unwrap();
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
        let (view, cx) =
            cx.add_window_view(|window, cx| PromptsView::new(registry.clone(), window, cx));
        cx.simulate_resize(size(px(720.), px(420.)));
        view.update_in(cx, |view, window, cx| view.open_dialog(None, false, window, cx));
        let mut dialog = modal(&view, cx);
        dialog.run_until_parked();
        let bounds = dialog.debug_bounds("PROMPT_DIALOG").unwrap();
        let actions = dialog.debug_bounds("PROMPT_DIALOG_ACTIONS").unwrap();
        assert!(bounds.left() >= px(0.) && bounds.right() <= px(720.));
        assert!(bounds.bottom() <= px(420.));
        assert!(actions.size.height > px(0.) && actions.bottom() <= bounds.bottom());
        dialog.simulate_keystrokes(save_shortcut());
        dialog.run_until_parked();
        assert!(cx.read_entity(&view, |view, _| view.dialog.is_some()));
        assert!(cx.update(|_, cx| service.list(cx).unwrap().is_empty()));
        dialog.simulate_input("Review code");
        dialog.simulate_keystrokes("tab");
        dialog.simulate_keystrokes(save_shortcut());
        dialog.run_until_parked();
        let handle = cx.read_entity(&view, |view, _| view.dialog.unwrap());
        cx.update(|_, cx| {
            let dialog = handle.read(cx).unwrap();
            assert_eq!(dialog.error.as_deref(), Some("Write some prompt text before saving."));
            let Task::Edit(draft) = &dialog.task else { unreachable!() };
            assert_eq!(draft.body.read(cx).text(cx), "");
        });
        dialog.simulate_input("First line");
        dialog.simulate_keystrokes("enter");
        dialog.simulate_input("Second line 🦀");
        dialog.simulate_keystrokes(save_shortcut());
        cx.run_until_parked();
        let saved = cx.update(|_, cx| service.list(cx).unwrap().remove(0));
        assert_eq!(saved.title, "Review code");
        assert_eq!(saved.prompt, "First line\nSecond line 🦀");
        assert!(cx.read_entity(&view, |view, _| view.dialog.is_none()));
        cx.update(|window, cx| assert!(view.read(cx).search.focus_handle(cx).is_focused(window)));

        view.update_in(cx, |view, window, cx| {
            view.open_dialog(Some(saved.clone()), false, window, cx)
        });
        let mut dialog = modal(&view, cx);
        dialog.simulate_keystrokes(if cfg!(target_os = "macos") { "cmd-a" } else { "ctrl-a" });
        dialog.simulate_input("Renamed");
        dialog.simulate_keystrokes(save_shortcut());
        cx.run_until_parked();
        let renamed = cx.update(|_, cx| service.resolve(&saved.id, cx).unwrap());
        assert_eq!(renamed.title, "Renamed");
        assert_eq!(renamed.prompt, saved.prompt);
        assert_eq!(Store::load(root.path().join("prompts.json")).unwrap().prompts(), &[renamed]);
    }

    #[gpui::test]
    fn confirm_binding_preserves_newlines_outside_prompt_dialog(cx: &mut TestAppContext) {
        struct Composer(Entity<Editor>);
        impl Render for Composer {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().key_context("Prompts").child(self.0.clone())
            }
        }

        init_test(cx);
        // Markdown Prompt shares the Prompts context but must not confirm a dialog.
        let (composer, cx) = cx.add_window_view(|window, cx| {
            Composer(cx.new(|cx| Editor::auto_height(4, 20, window, cx)))
        });
        let editor = cx.read_entity(&composer, |composer, _| composer.0.clone());
        cx.update(|window, cx| window.focus(&editor.focus_handle(cx), cx));
        cx.simulate_input("First line");
        // The platform editor uses Ctrl-Enter for a newline on both platforms.
        cx.simulate_keystrokes("ctrl-enter");
        cx.simulate_input("Second line");
        assert_eq!(
            cx.read_entity(&editor, |editor, cx| editor.text(cx)),
            "First line\nSecond line"
        );
    }

    #[gpui::test]
    fn delete_modal_requires_confirmation_and_keeps_conflicts_open(cx: &mut TestAppContext) {
        init_test(cx);
        let root = tempfile::tempdir().unwrap();
        let registry = cx.new(|_| Registry::load(root.path().join("prompts.json")));
        let original = registry.update(cx, |registry, cx| {
            registry
                .modify(|store| store.save(None, "Review".into(), "Original".into()), cx)
                .unwrap();
            registry.store.as_ref().unwrap().prompts()[0].clone()
        });
        let (view, cx) =
            cx.add_window_view(|window, cx| PromptsView::new(registry.clone(), window, cx));
        view.update_in(cx, |view, window, cx| {
            view.open_dialog(Some(original.clone()), true, window, cx)
        });
        let mut dialog = modal(&view, cx);
        // Delete opens on Cancel, so an accidental Enter cannot delete the prompt.
        dialog.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert!(cx.read_entity(&view, |view, _| view.dialog.is_none()));
        assert_eq!(cx.read_entity(&registry, |r, _| r.store.as_ref().unwrap().prompts().len()), 1);

        view.update_in(cx, |view, window, cx| {
            view.open_dialog(Some(original.clone()), true, window, cx)
        });
        let mut dialog = modal(&view, cx);
        registry.update(cx, |registry, cx| {
            registry
                .modify(
                    |store| store.save(Some(&original), "Updated".into(), "New body".into()),
                    cx,
                )
                .unwrap();
        });
        dialog.simulate_keystrokes("tab");
        dialog.simulate_keystrokes("enter");
        cx.run_until_parked();
        let handle = cx.read_entity(&view, |view, _| view.dialog.unwrap());
        cx.update(|_, cx| {
            assert!(handle.read(cx).unwrap().error.as_deref().unwrap().contains("changed"));
        });
        assert_eq!(cx.read_entity(&registry, |r, _| r.store.as_ref().unwrap().prompts().len()), 1);
        dialog.simulate_keystrokes("escape");
        cx.run_until_parked();

        let updated =
            cx.read_entity(&registry, |r, _| r.store.as_ref().unwrap().prompts()[0].clone());
        view.update_in(cx, |view, window, cx| view.open_dialog(Some(updated), true, window, cx));
        let mut dialog = modal(&view, cx);
        dialog.simulate_keystrokes("tab");
        dialog.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert!(cx.read_entity(&view, |view, _| view.dialog.is_none()));
        assert!(cx.read_entity(&registry, |r, _| r.store.as_ref().unwrap().prompts().is_empty()));
    }
}
