//! A project-scoped autosaving composer with optional live Markdown output.
mod document;
mod rich_input;
mod sync;
use crate::{
    components::{SegmentedControl, SegmentedControlOption, input_field},
    text_input::TextInput,
};
use chartr_plugin::ui as plugin_ui;
use chartr_plugin::{
    Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar, services::PromptTemplates,
};
#[cfg(test)]
use document::Part;
use document::{Bodies, Document};
use editor::Editor;
use gpui::{App, Context, Entity, Focusable, Render, Window, div};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
    time::Duration,
};
use ui::{Color, Switch, prelude::*};

pub struct MarkdownPromptPlugin {
    data: PathBuf,
    sync: Entity<sync::Manager>,
}
pub fn bundled(host: Host, cx: &mut App) -> Box<dyn PluginObject> {
    Box::new(MarkdownPromptPlugin::new(host, cx))
}
impl Plugin for MarkdownPromptPlugin {
    const ID: &'static str = "com.chartr.markdown-prompt";
    fn new(host: Host, cx: &mut App) -> Self {
        Self { sync: cx.new(|_| sync::Manager::new(host.data_dir.clone())), data: host.data_dir }
    }
    fn connect_services(&mut self, services: chartr_plugin::services::Services, cx: &mut App) {
        self.sync.update(cx, |manager, cx| manager.connect(services, cx));
    }
    fn background_status(&self, cx: &App) -> Option<chartr_plugin::BackgroundStatus> {
        Some(self.sync.read(cx).status())
    }
    fn activate(&mut self, registrar: &mut Registrar, _: &mut App) {
        registrar.add_pane("main", "Markdown Prompt");
    }
    fn view(
        &mut self,
        _: &PaneKey,
        context: &InstanceContext,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyView {
        let mut hash = DefaultHasher::new();
        context.project_dir.hash(&mut hash);
        if context.project_dir.is_none() {
            context.space.hash(&mut hash);
        }
        let path = self.data.join(format!("{:016x}.json", hash.finish()));
        cx.new(|cx| Composer::new(context.clone(), path, self.sync.clone(), window, cx)).into()
    }
}

#[derive(Clone)]
struct TemplateChip {
    provider: String,
    id: String,
    title: String,
    origin: Option<(gpui::EntityId, std::ops::Range<usize>)>,
}
impl Render for TemplateChip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        plugin_ui::template_chip(false, cx).p_2().child(plugin_ui::label(self.title.clone()))
    }
}
struct ModeDraft {
    append: bool,
    create_if_missing: bool,
    filename: Entity<TextInput>,
}

struct Composer {
    context: InstanceContext,
    path: PathBuf,
    doc: Document,
    mode_draft: Option<ModeDraft>,
    editor: Entity<Editor>,
    chips_dirty: bool,
    templates_dirty: bool,
    templates: Vec<TemplateChip>,
    bodies: Bodies,
    warnings: Vec<String>,
    error: Option<String>,
    status: Option<String>,
    preview: Option<String>,
    preview_focus: gpui::FocusHandle,
    busy: bool,
    invalid_config: bool,
    saved_bytes: Option<Vec<u8>>,
    sync: Entity<sync::Manager>,
    edit_revision: u64,
    save_ready: bool,
    saving: bool,
}
impl Composer {
    fn new(
        context: InstanceContext,
        path: PathBuf,
        sync: Entity<sync::Manager>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let loaded = document::load(&path);
        let error = loaded.as_ref().err().cloned();
        let invalid_config = error.is_some();
        let doc = loaded.unwrap_or_default();
        let saved_bytes = std::fs::read(&path).ok();
        let editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(5, 22, window, cx);
            editor.set_soft_wrap();
            editor.set_autoindent(false);
            editor.set_use_autoclose(false);
            editor.set_show_wrap_guides(false, cx);
            editor.set_show_indent_guides(false, cx);
            editor.set_placeholder_text("Write Markdown and drop templates anywhere…", window, cx);
            editor.set_text(rich_input::encode(&doc.parts), window, cx);
            editor
        });
        cx.subscribe(&editor, |this, _, event: &editor::EditorEvent, cx| {
            if matches!(event, editor::EditorEvent::BufferEdited) {
                this.chips_dirty = true;
                this.queue_save(false, cx);
            }
        })
        .detach();
        let changes = PromptTemplates::changes(cx);
        cx.observe(&changes, |this, _, cx| {
            this.templates_dirty = true;
            cx.notify();
        })
        .detach();
        cx.observe(&sync, |_, _, cx| cx.notify()).detach();
        let mut this = Self {
            context,
            path,
            doc,
            mode_draft: None,
            editor,
            chips_dirty: true,
            templates_dirty: false,
            templates: vec![],
            bodies: Bodies::new(),
            warnings: vec![],
            error,
            status: None,
            preview: None,
            preview_focus: cx.focus_handle(),
            busy: false,
            invalid_config,
            saved_bytes,
            sync,
            edit_revision: 0,
            save_ready: false,
            saving: false,
        };
        this.refresh(window, cx);
        // Opening a saved composition resumes its current Enabled state. A fresh
        // untouched editor never creates an empty project file just by opening.
        if this.saved_bytes.is_some() {
            this.queue_save(true, cx);
        }
        this
    }
    fn snapshot(&self, cx: &App) -> Document {
        let mut doc = self.doc.clone();
        doc.parts = rich_input::decode(&self.editor.read(cx).text(cx));
        doc
    }
    fn insert_at_cursor(
        &mut self,
        chip: TemplateChip,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.invalid_config {
            return;
        }
        window.focus(&self.editor.focus_handle(cx), cx);
        self.editor.update(cx, |editor, cx| rich_input::insert(editor, &chip, window, cx));
        self.chips_dirty = true;
        self.preview = None;
        self.queue_save(false, cx);
    }
    fn show_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.invalid_config {
            return;
        }
        let filename = cx.new(|cx| {
            let mut input = TextInput::new("CHARTR.md", cx);
            input.set_text(self.doc.filename.clone(), false, cx);
            input
        });
        window.focus(&filename.focus_handle(cx), cx);
        self.preview = None;
        self.mode_draft = Some(ModeDraft {
            append: self.doc.append,
            create_if_missing: self.doc.create_if_missing,
            filename,
        });
        cx.notify();
    }

    fn cancel_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.mode_draft = None;
        window.focus(&self.editor.focus_handle(cx), cx);
        cx.notify();
    }

    fn save_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.mode_draft.take() else { return };
        self.doc.append = draft.append;
        self.doc.create_if_missing = draft.create_if_missing;
        self.doc.filename = draft.filename.read(cx).text().trim().to_owned();
        self.queue_save(true, cx);
        window.focus(&self.editor.focus_handle(cx), cx);
    }

    fn show_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.invalid_config {
            return;
        }
        match document::compose(&self.snapshot(cx).parts, &self.bodies) {
            Ok(body) => {
                self.preview = Some(body);
                self.error = None;
                window.focus(&self.preview_focus, cx);
            }
            Err(error) => self.error = Some(error),
        }
        cx.notify();
    }

    fn dismiss_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.preview = None;
        window.focus(&self.editor.focus_handle(cx), cx);
        cx.notify();
    }

    fn queue_save(&mut self, immediate: bool, cx: &mut Context<Self>) {
        if self.invalid_config {
            return;
        }
        self.edit_revision += 1;
        self.save_ready = false;
        self.preview = None;
        self.error = None;
        self.status = Some("Saving…".into());
        self.sync.update(cx, |sync, cx| sync.pause_for_edits(&self.path, cx));
        if immediate {
            self.save_ready = true;
            self.flush_save(cx);
        } else {
            let revision = self.edit_revision;
            // Retain the editor until this save finishes, even if its pane closes.
            let keep_alive = cx.entity();
            let timer = cx.background_executor().timer(Duration::from_millis(500));
            cx.spawn(async move |this, cx| {
                let _keep_alive = keep_alive;
                timer.await;
                let _ = this.update(cx, |this, cx| {
                    if this.edit_revision == revision {
                        this.save_ready = true;
                        this.flush_save(cx);
                    }
                });
            })
            .detach();
        }
        cx.notify();
    }

    fn flush_save(&mut self, cx: &mut Context<Self>) {
        if self.saving || !self.save_ready || self.invalid_config {
            return;
        }
        self.save_ready = false;
        self.saving = true;
        let doc = self.snapshot(cx);
        let path = self.path.clone();
        let project = self.context.project_dir.clone();
        let expected = self.saved_bytes.clone();
        let revision = self.edit_revision;
        let keep_alive = cx.entity();
        cx.spawn(async move |this, cx| {
            let _keep_alive = keep_alive;
            let result =
                cx.background_executor()
                    .spawn(async move {
                        sync::persist(&path, project.as_deref(), doc, expected.as_deref())
                    })
                    .await;
            let _ = this.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok((doc, error)) => {
                        this.saved_bytes = serde_json::to_vec_pretty(&doc).ok();
                        // Controls and text may have changed while the save was in flight.
                        this.doc.managed_files = doc.managed_files;
                        if this.edit_revision == revision {
                            this.error = error;
                            this.status = None;
                        }
                    }
                    Err(error) => {
                        this.error = Some(error);
                        this.status = Some("Changes could not be saved".into());
                    }
                }
                if this.edit_revision == revision {
                    this.sync.update(cx, |sync, cx| sync.resume_after_save(&this.path, cx));
                }
                this.flush_save(cx);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn refresh(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.invalid_config {
            return;
        }
        let project = self.context.project_dir.clone();
        let requests: Vec<_> = self
            .context
            .services
            .all::<PromptTemplates>()
            .into_iter()
            .map(|(id, provider)| (id, provider.list(project.clone(), cx)))
            .collect();
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let mut templates = Vec::new();
            let mut bodies = Bodies::new();
            let mut warnings = Vec::new();
            for (provider, task) in requests {
                match task.await {
                    Ok(items) => {
                        let mut ids = std::collections::HashSet::new();
                        if items.iter().any(|item| {
                            item.id.is_empty()
                                || item.title.trim().is_empty()
                                || item.prompt.len() > 1024 * 1024
                                || !ids.insert(item.id.clone())
                        }) {
                            warnings.push(format!("{provider}: invalid or duplicate template IDs"));
                            continue;
                        }
                        for item in items {
                            bodies.insert((provider.clone(), item.id.clone()), item.prompt);
                            templates.push(TemplateChip {
                                provider: provider.clone(),
                                id: item.id,
                                title: item.title,
                                origin: None,
                            });
                        }
                    }
                    Err(error) => warnings.push(format!("{provider}: {error}")),
                }
            }
            let _ = this.update(cx, |this, cx| {
                let enabled: std::collections::HashSet<_> = this
                    .context
                    .services
                    .all::<PromptTemplates>()
                    .into_iter()
                    .map(|(id, _)| id)
                    .collect();
                bodies.retain(|(provider, _), _| enabled.contains(provider));
                templates.retain(|t| enabled.contains(&t.provider));
                this.templates = templates;
                this.chips_dirty = true;
                this.bodies = bodies;
                this.warnings = warnings;
                this.busy = false;
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
impl Render for Composer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.templates_dirty && !self.busy {
            self.templates_dirty = false;
            self.refresh(window, cx);
        }
        let blocked = self.invalid_config;
        if self.chips_dirty {
            self.chips_dirty = false;
            rich_input::decorate(&self.editor, &self.templates, window, cx);
        }
        let templates: Vec<_> = self
            .templates
            .iter()
            .enumerate()
            .map(|(index, chip)| {
                let drag = chip.clone();
                let click = chip.clone();
                div()
                    .id(("template", index))
                    .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
                    .child(
                        plugin_ui::action(
                            ("insert-template", index),
                            format!(
                                "{} · {}",
                                chip.title,
                                chip.provider.trim_start_matches("com.chartr.")
                            ),
                        )
                        .disabled(blocked)
                        .on_click(cx.listener(
                            move |this, _, window, cx| {
                                this.insert_at_cursor(click.clone(), window, cx)
                            },
                        )),
                    )
            })
            .collect();
        let sync = self.sync.read(cx);
        let status = self.status.clone().unwrap_or_else(|| {
            if self.invalid_config {
                "Configuration needs attention".into()
            } else if self.error.is_some()
                || (self.doc.enabled && sync.problem(&self.path).is_some())
            {
                "Draft saved · file needs attention".into()
            } else if !self.doc.enabled {
                "Draft saved · file updates paused".into()
            } else if sync.updated(&self.path) {
                "File updated · live sync on".into()
            } else if self.saved_bytes.is_some() {
                "Updating file…".into()
            } else {
                "Edits save automatically · live sync on".into()
            }
        });
        let page = plugin_ui::pane_surface("markdown-prompt", cx).key_context("Prompts").size_full().overflow_y_scroll().p_4().gap_3()
            .child(plugin_ui::PageHeader::new("Markdown Prompt"))
            .child(h_flex().gap_2().child(Switch::new("markdown-enabled", self.doc.enabled.into()).disabled(blocked).on_click(cx.listener(|this, state: &ui::ToggleState, _, cx| { this.doc.enabled = state.selected(); this.queue_save(true, cx); }))).child(plugin_ui::label("Enabled")).child(plugin_ui::label(status).color(Color::Muted)))
            .child(h_flex().justify_between().child(plugin_ui::label("Templates")).child(plugin_ui::action("refresh-templates", "Refresh").disabled(blocked).on_click(cx.listener(|this, _, window, cx| this.refresh(window, cx)))))
            .child(h_flex().gap_2().flex_wrap().children(templates))
            .when(self.templates.is_empty(), |view| view.child(plugin_ui::label("No templates available. Enable Saved Prompts, Skills or another provider.").color(Color::Muted)))
            .children(self.warnings.iter().map(|warning| plugin_ui::notice(warning.clone(), true)))
            .child(plugin_ui::label("Constructed Markdown contents"))
            .child(plugin_ui::label("Type freely. Click or drag a template into the text. Select a chip to move, copy or delete it.").color(Color::Muted))
            .child(plugin_ui::outlined_content(cx).id("composition").w_full()
                .on_drag_move(cx.listener(|this, event: &gpui::DragMoveEvent<TemplateChip>, window, cx| {
                    if !this.invalid_config && event.bounds.contains(&event.event.position) {
                        window.focus(&this.editor.focus_handle(cx), cx);
                        this.editor.update(cx, |editor, cx| rich_input::place_caret(editor, event.event.position, window, cx));
                    }
                }))
                .on_drop(cx.listener(|this, chip: &TemplateChip, window, cx| { cx.stop_propagation(); this.insert_at_cursor(chip.clone(), window, cx); }))
                .child(self.editor.clone()))
            .child(h_flex().child(plugin_ui::action("preview-markdown", "Preview").disabled(blocked).on_click(cx.listener(|this, _, window, cx| this.show_preview(window, cx)))))
            .child(h_flex().child(plugin_ui::action("edit-markdown-mode", "Mode").disabled(blocked).on_click(cx.listener(|this, _, window, cx| this.show_mode(window, cx)))))
            .when_some(self.error.clone(), |view, error| view.child(plugin_ui::notice(error, true)))
            .when_some(self.sync.read(cx).problem(&self.path), |view, error| view.child(plugin_ui::notice(format!("Automatic sync: {error}"), true)));
        div()
            .size_full()
            .relative()
            .track_focus(&self.preview_focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    if this.mode_draft.is_some() {
                        this.cancel_mode(window, cx);
                        cx.stop_propagation();
                    } else if this.preview.is_some() {
                        this.dismiss_preview(window, cx);
                        cx.stop_propagation();
                    }
                }
            }))
            .child(page)
            .when_some(self.mode_draft.as_ref(), |view, draft| {
                view.child(
                    plugin_ui::ModalOverlay::new("markdown-mode-scrim", cx.listener(|this, _, window, cx| this.cancel_mode(window, cx)))
                        .child(plugin_ui::DialogSurface::new("markdown-mode-dialog")
                            .child(plugin_ui::dialog_header("Mode", div(), cx))
                            .child(plugin_ui::dialog_body().id("markdown-mode-contents").overflow_y_scroll()
                                .child(h_flex().gap_2().child(plugin_ui::label("Mode"))
                                    .child(SegmentedControl::new("Markdown output mode", [
                                        SegmentedControlOption::new("append-mode", "Append", draft.append, cx.listener(|this, _, _, cx| { if let Some(draft) = this.mode_draft.as_mut() { draft.append = true; } cx.notify(); })),
                                        SegmentedControlOption::new("new-file-mode", "New file", !draft.append, cx.listener(|this, _, _, cx| { if let Some(draft) = this.mode_draft.as_mut() { draft.append = false; } cx.notify(); })),
                                    ]).disabled(blocked)))
                                .child(plugin_ui::label(if draft.append { "Update a marked section in the named file; surrounding content is preserved." } else { "Create and maintain an owned file; external edits are reported before replacement." }).color(Color::Muted))
                                .when(draft.append, |view| view.child(h_flex().gap_2()
                                    .child(Switch::new("markdown-create-if-missing", draft.create_if_missing.into()).disabled(blocked).on_click(cx.listener(|this, state: &ui::ToggleState, _, cx| {
                                        if let Some(draft) = this.mode_draft.as_mut() { draft.create_if_missing = state.selected(); }
                                        cx.notify();
                                    })))
                                    .child(plugin_ui::label("Create file if it doesn't exist"))))
                                .child(plugin_ui::label("Filename (relative to this folder)"))
                                .when(self.context.project_dir.is_none(), |view| view.child(plugin_ui::label("Free sessions has no destination folder. Open Markdown Prompt inside a folder space to enable file updates.").color(Color::Muted)))
                                .child(input_field("markdown-filename", draft.filename.clone(), cx))
                                .when_some(self.context.project_dir.clone(), |view, root| view.child(plugin_ui::label(root.display().to_string()).color(Color::Muted)))
                            )
                            .child(plugin_ui::dialog_actions(cx)
                                .child(plugin_ui::action("cancel-markdown-mode", "Cancel").on_click(cx.listener(|this, _, window, cx| this.cancel_mode(window, cx))))
                                .child(plugin_ui::action("save-markdown-mode", "Save").on_click(cx.listener(|this, _, window, cx| this.save_mode(window, cx)))))
                        ),
                )
            })
            .when_some(self.preview.clone(), |view, text| {
                view.child(
                    plugin_ui::ModalOverlay::new(
                        "markdown-preview-scrim",
                        cx.listener(|this, _, window, cx| this.dismiss_preview(window, cx)),
                    )
                    .child(
                        plugin_ui::DialogSurface::new("markdown-preview-dialog")
                            .child(plugin_ui::dialog_header(
                                "Expanded preview",
                                plugin_ui::action("close-markdown-preview", "Close").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.dismiss_preview(window, cx)
                                    }),
                                ),
                                cx,
                            ))
                            .child(
                                plugin_ui::dialog_body()
                                    .id("markdown-preview-contents")
                                    .overflow_y_scroll()
                                    .child(plugin_ui::label(text)),
                            ),
                    ),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_plugin::{
        TerminalLauncher,
        services::{PluginSettings, SavedPrompt, ServiceExport, Services},
    };

    #[gpui::test]
    fn autosave_debounces_edits_and_enabled_controls_file_updates(cx: &mut gpui::TestAppContext) {
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            crate::prompts_plugin::init(cx);
        });
        let services = Services::default();
        let manager = cx.new(|_| sync::Manager::new(data.path().into()));
        manager.update(cx, |manager, cx| manager.connect(services.clone(), cx));
        let context = InstanceContext {
            instance_id: 1,
            space: "fixture".into(),
            space_name: "Fixture".into(),
            project_dir: Some(root.path().into()),
            bound_session: None,
            terminal: TerminalLauncher::new(|_, _| {}),
            services,
            plugin_settings: PluginSettings::new(|_, _, _| {}),
        };
        let (view, cx) = cx.add_window_view(|window, cx| {
            Composer::new(context.clone(), path.clone(), manager.clone(), window, cx)
        });
        cx.run_until_parked();
        assert!(!path.exists());
        assert!(!root.path().join("CHARTR.md").exists());

        view.update_in(cx, |this, _, cx| {
            this.doc.enabled = false;
            this.queue_save(true, cx);
        });
        cx.run_until_parked();
        assert!(!document::load(&path).unwrap().enabled);
        view.update_in(cx, |this, window, cx| {
            this.show_mode(window, cx);
            let draft = this.mode_draft.as_mut().unwrap();
            draft.append = true;
            draft.create_if_missing = true;
            draft.filename.update(cx, |input, cx| input.set_text("AGENTS.md", true, cx));
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(1000));
        cx.run_until_parked();
        assert_eq!(document::load(&path).unwrap().filename, "CHARTR.md");
        assert!(!document::load(&path).unwrap().append);
        view.update_in(cx, |this, window, cx| this.save_mode(window, cx));
        cx.run_until_parked();
        let initial = std::fs::read(&path).unwrap();
        view.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("first", window, cx));
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(250));
        view.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("latest", window, cx));
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(250));
        cx.run_until_parked();
        assert_eq!(std::fs::read(&path).unwrap(), initial);
        cx.executor().advance_clock(Duration::from_millis(250));
        cx.run_until_parked();
        let draft = document::load(&path).unwrap();
        assert_eq!(draft.filename, "AGENTS.md");
        assert!(draft.append && draft.create_if_missing);
        assert_eq!(document::compose(&draft.parts, &Bodies::new()).unwrap(), "latest");
        let output = root.path().join("AGENTS.md");
        assert!(!output.exists());

        // A newer toggle while an autosave is in flight must win without writing.
        view.update_in(cx, |this, _, cx| {
            this.doc.enabled = true;
            this.queue_save(true, cx);
            this.doc.enabled = false;
            this.queue_save(true, cx);
        });
        cx.run_until_parked();
        assert!(!document::load(&path).unwrap().enabled);
        assert!(!output.exists());
        view.update_in(cx, |this, _, cx| {
            this.doc.enabled = true;
            this.queue_save(true, cx);
        });
        cx.run_until_parked();
        let expected = document::appended("", "latest").unwrap();
        assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);
        assert!(manager.read_with(cx, |manager, _| manager.updated(&path)));

        view.update_in(cx, |this, window, cx| {
            this.show_mode(window, cx);
            let draft = this.mode_draft.as_mut().unwrap();
            draft.append = false;
            draft.create_if_missing = false;
            draft.filename.update(cx, |input, cx| input.set_text("UNSAVED.md", true, cx));
            // An independent text autosave must use the committed mode and filename.
            this.editor.update(cx, |editor, cx| editor.set_text("live edit", window, cx));
        });
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        let expected = document::appended("", "live edit").unwrap();
        assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);

        assert!(!root.path().join("UNSAVED.md").exists());
        assert_eq!(document::load(&path).unwrap().filename, "AGENTS.md");
        view.update_in(cx, |this, window, cx| {
            this.cancel_mode(window, cx);
            this.show_mode(window, cx);
            let draft = this.mode_draft.as_ref().unwrap();
            assert!(draft.append && draft.create_if_missing);
            assert_eq!(draft.filename.read(cx).text(), "AGENTS.md");
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("escape");
        assert!(cx.read_entity(&view, |this, _| this.mode_draft.is_none()));
        view.update_in(cx, |this, window, cx| this.show_mode(window, cx));
        cx.run_until_parked();
        cx.simulate_click(gpui::point(gpui::px(1.), gpui::px(1.)), gpui::Modifiers::none());
        assert!(cx.read_entity(&view, |this, _| this.mode_draft.is_none()));
        assert_eq!(document::load(&path).unwrap().filename, "AGENTS.md");

        // Saving an invalid destination preserves the last good file and reports an error.
        view.update_in(cx, |this, window, cx| {
            this.show_mode(window, cx);
            this.mode_draft
                .as_ref()
                .unwrap()
                .filename
                .update(cx, |input, cx| input.set_text("../outside.md", true, cx));
            this.save_mode(window, cx);
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert_eq!(document::load(&path).unwrap().filename, "../outside.md");
        assert!(manager.read_with(cx, |manager, _| manager.problem(&path).is_some()));
        assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);

        view.update_in(cx, |this, window, cx| {
            this.doc.enabled = false;
            this.queue_save(true, cx);
            this.editor.update(cx, |editor, cx| editor.set_text("paused draft", window, cx));
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert!(!document::load(&path).unwrap().enabled);
        assert!(manager.read_with(cx, |manager, _| manager.problem(&path).is_none()));
        assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);

        view.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("saved after closing", window, cx));
        });
        cx.run_until_parked();
        view.update_in(cx, |_, window, _| window.remove_window());
        let weak = view.downgrade();
        drop(view);
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert_eq!(
            document::compose(&document::load(&path).unwrap().parts, &Bodies::new()).unwrap(),
            "saved after closing"
        );
        assert!(weak.upgrade().is_none());
        let (restored, cx) = cx.add_window_view(|window, cx| {
            Composer::new(context, path.clone(), manager.clone(), window, cx)
        });
        cx.run_until_parked();
        assert!(cx.read_entity(&restored, |this, cx| {
            !this.doc.enabled && this.editor.read(cx).text(cx) == "saved after closing"
        }));
        assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);
    }

    #[gpui::test]
    fn compose_live_templates_save_restore_and_maintain_owned_file(cx: &mut gpui::TestAppContext) {
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            crate::prompts_plugin::init(cx);
        });
        let services = Services::default();
        let publish = |body: &str, title: &str| {
            let body = body.to_owned();
            let title = title.to_owned();
            services.publish(
                "example.provider",
                vec![ServiceExport::new(PromptTemplates::new(move |_, _| {
                    gpui::Task::ready(Ok(vec![SavedPrompt {
                        id: "stable".into(),
                        title: title.clone(),
                        prompt: body.clone(),
                    }]))
                }))],
            );
        };
        publish("first", "Original title");
        let context = InstanceContext {
            instance_id: 1,
            space: "fixture".into(),
            space_name: "Fixture".into(),
            project_dir: Some(root.path().to_owned()),
            bound_session: None,
            terminal: TerminalLauncher::new(|_, _| {}),
            services: services.clone(),
            plugin_settings: PluginSettings::new(|_, _, _| {}),
        };
        let manager = cx.new(|_| sync::Manager::new(data.path().into()));
        manager.update(cx, |manager, cx| manager.connect(services.clone(), cx));
        let (view, cx) = cx.add_window_view(|window, cx| {
            Composer::new(context.clone(), path.clone(), manager.clone(), window, cx)
        });
        cx.run_until_parked();
        assert!(cx.read_entity(&view, |this, _| this.preview.is_none()));
        view.update_in(cx, |this, window, cx| {
            let chip = this.templates[0].clone();
            this.insert_at_cursor(chip, window, cx);
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "first");
        publish("second\nline", "Renamed");
        cx.update(|_, cx| PromptTemplates::changed(cx));
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "second\nline");
        let saved = document::load(&path).unwrap();
        assert!(
            saved
                .parts
                .iter()
                .any(|part| matches!(part, Part::Template { id, .. } if id == "stable"))
        );
        assert_eq!(sync::receipts(&path, &saved).unwrap()["CHARTR.md"], "second\nline");
        assert!(cx.read_entity(&view, |this, _| this.preview.is_none()));
        view.update_in(cx, |this, window, cx| this.show_preview(window, cx));
        assert_eq!(
            cx.read_entity(&view, |this, _| this.preview.clone()),
            Some("second\nline".into())
        );
        view.update_in(cx, |this, window, cx| this.dismiss_preview(window, cx));
        assert!(cx.read_entity(&view, |this, _| this.preview.is_none()));
        services.remove("example.provider");
        cx.update(|_, cx| PromptTemplates::changed(cx));
        cx.run_until_parked();
        assert!(manager.read_with(cx, |manager, _| {
            manager.problem(&path).is_some_and(|e| e.contains("disabled"))
        }));
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "second\nline");
        publish("third", "Renamed");
        std::fs::write(root.path().join("CHARTR.md"), "User edits").unwrap();
        cx.update(|_, cx| PromptTemplates::changed(cx));
        cx.run_until_parked();
        assert!(manager.read_with(cx, |manager, _| {
            manager.problem(&path).is_some_and(|e| e.contains("edited outside"))
        }));
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "User edits");
    }
}
