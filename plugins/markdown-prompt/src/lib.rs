//! A project-scoped composer. Templates remain references until preview/apply.
mod document;
mod rich_input;
mod sync;
use crate::{components::input_field, text_input::TextInput};
use chartr_plugin::ui as plugin_ui;
use chartr_plugin::ui::action as form_button;
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
        cx.new(|cx| {
            let mut view = Composer::new(context.clone(), path, window, cx);
            view.sync = Some(self.sync.downgrade());
            cx.observe(&self.sync, |_, _, cx| cx.notify()).detach();
            view
        })
        .into()
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
struct Composer {
    context: InstanceContext,
    path: PathBuf,
    doc: Document,
    filename: Entity<TextInput>,
    editor: Entity<Editor>,
    chips_dirty: bool,
    templates_dirty: bool,
    templates: Vec<TemplateChip>,
    bodies: Bodies,
    warnings: Vec<String>,
    error: Option<String>,
    status: Option<String>,
    preview: Option<String>,
    busy: bool,
    invalid_config: bool,
    saved_bytes: Option<Vec<u8>>,
    sync: Option<gpui::WeakEntity<sync::Manager>>,
}
impl Composer {
    fn new(
        context: InstanceContext,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let loaded = document::load(&path);
        let error = loaded.as_ref().err().cloned();
        let invalid_config = error.is_some();
        let doc = loaded.unwrap_or_default();
        let filename = cx.new(|cx| {
            let mut input = TextInput::new("CHARTR.md", cx);
            input.set_text(doc.filename.clone(), false, cx);
            input
        });
        cx.subscribe(&filename, |this, _, _: &crate::text_input::InputEvent, cx| {
            this.preview = None;
            this.status = None;
            cx.notify();
        })
        .detach();
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
                this.preview = None;
                this.status = None;
                cx.notify();
            }
        })
        .detach();
        let changes = PromptTemplates::changes(cx);
        cx.observe(&changes, |this, _, cx| {
            this.templates_dirty = true;
            cx.notify();
        })
        .detach();
        let mut this = Self {
            context,
            path,
            doc,
            filename,
            editor,
            chips_dirty: true,
            templates_dirty: false,
            templates: vec![],
            bodies: Bodies::new(),
            warnings: vec![],
            error,
            status: None,
            preview: None,
            busy: false,
            invalid_config,
            saved_bytes,
            sync: None,
        };
        this.refresh(false, window, cx);
        this
    }
    fn snapshot(&self, cx: &App) -> Document {
        let mut doc = self.doc.clone();
        doc.filename = self.filename.read(cx).text().trim().to_owned();
        doc.parts = rich_input::decode(&self.editor.read(cx).text(cx));
        doc
    }
    fn insert_at_cursor(
        &mut self,
        chip: TemplateChip,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        window.focus(&self.editor.focus_handle(cx), cx);
        self.editor.update(cx, |editor, cx| rich_input::insert(editor, &chip, window, cx));
        self.chips_dirty = true;
        self.preview = None;
        self.status = None;
        cx.notify();
    }
    fn refresh(&mut self, apply: bool, _: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.invalid_config {
            return;
        }
        if apply && !self.doc.enabled {
            return;
        }
        let providers = self.context.services.all::<PromptTemplates>();
        let project = self.context.project_dir.clone();
        let requests: Vec<_> = providers
            .into_iter()
            .map(|(id, provider)| (id, provider.list(project.clone(), cx)))
            .collect();
        let doc = self.snapshot(cx);
        let expected = self.saved_bytes.clone();
        self.busy = true;
        self.error = None;
        if apply {
            self.status = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let mut templates = Vec::new(); let mut bodies = Bodies::new(); let mut warnings = Vec::new();
            for (provider, task) in requests {
                match task.await {
                    Ok(items) => {
                        let mut ids = std::collections::HashSet::new();
                        if items.iter().any(|item| item.id.is_empty() || item.title.trim().is_empty() || item.prompt.len() > 1024 * 1024 || !ids.insert(item.id.clone())) {
                            warnings.push(format!("{provider}: invalid or duplicate template IDs")); continue;
                        }
                        for item in items {
                            bodies.insert((provider.clone(), item.id.clone()), item.prompt);
                            templates.push(TemplateChip { provider: provider.clone(), id: item.id, title: item.title, origin: None });
                        }
                    },
                    Err(error) => warnings.push(format!("{provider}: {error}")),
                }
            }
            let prepared = this.update(cx, |this, cx| {
                // A provider disabled during a scan cannot supply stale content.
                let enabled: std::collections::HashSet<_> = this.context.services.all::<PromptTemplates>().into_iter().map(|(id, _)| id).collect();
                bodies.retain(|(provider, _), _| enabled.contains(provider));
                templates.retain(|t| enabled.contains(&t.provider));
                let composed = document::compose(&doc.parts, &bodies);
                this.templates = templates; this.chips_dirty = true; this.bodies = bodies; this.warnings = warnings;
                this.preview = composed.as_ref().ok().cloned();
                this.error = composed.as_ref().err().cloned();
                if !apply || composed.is_err() { this.busy = false; }
                cx.notify();
                composed
            });
            let Ok(Ok(body)) = prepared else { return };
            if !apply { return; }
            let Some(root) = project else {
                let _ = this.update(cx, |this, cx| { this.error = Some("Open a folder space to write Markdown.".into()); this.busy = false; cx.notify(); }); return;
            };
            let Ok(path) = this.read_with(cx, |this, _| this.path.clone()) else { return };
            let result = cx.background_executor().spawn(async move {
                std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
                let lock = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(path.with_extension("lock")).map_err(|e| e.to_string())?;
                lock.lock().map_err(|e| e.to_string())?;
                if std::fs::read(&path).ok() != expected { return Err("This composition changed in another pane. Reopen Markdown Prompt before saving or applying.".into()); }
                let mut doc = doc;
                // Ownership receipts come from disk, never a stale pane.
                doc.managed_files = sync::receipts(&path, &document::load(&path)?)?;
                let written = document::apply(&root, &doc, &body)?;
                if !doc.append { doc.managed_files.insert(doc.filename.clone(), body); }
                document::save(&path, &doc).map_err(|e| format!("File written, but configuration could not be saved: {e}"))?;
                sync::activate(&path, &root, &doc).map_err(|e| format!("File applied, but automatic sync could not be enabled: {e}"))?;
                Ok::<_, String>((written, doc))
            }).await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok((path, doc)) => { this.saved_bytes = serde_json::to_vec_pretty(&doc).ok(); this.doc = doc; PromptTemplates::changed(cx); this.status = Some(format!("Applied to {} · automatic sync is on", path.display())); },
                    Err(error) => this.error = Some(error),
                }
                cx.notify();
            });
        }).detach();
    }
    fn save_draft(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.invalid_config {
            return;
        }
        let doc = self.snapshot(cx);
        let path = self.path.clone();
        let expected = self.saved_bytes.clone();
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move {
                std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
                let lock = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(path.with_extension("lock")).map_err(|e| e.to_string())?;
                lock.lock().map_err(|e| e.to_string())?;
                if std::fs::read(&path).ok() != expected { return Err("This composition changed in another pane. Reopen Markdown Prompt before saving or applying.".into()); }
                let mut doc = doc;
                doc.managed_files = sync::receipts(&path, &document::load(&path)?)?;
                document::save(&path, &doc)?; sync::set_enabled(&path, doc.enabled)?; Ok::<_, String>(doc)
            }).await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result { Ok(doc) => { this.saved_bytes = serde_json::to_vec_pretty(&doc).ok(); this.doc = doc; PromptTemplates::changed(cx); this.status = Some("Draft saved.".into()); this.error = None; }, Err(e) => this.error = Some(e) }
                cx.notify();
            });
        }).detach();
        cx.notify();
    }
}
impl Render for Composer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.templates_dirty && !self.busy {
            self.templates_dirty = false;
            self.refresh(false, window, cx);
        }
        let blocked = self.busy || self.invalid_config;
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
        plugin_ui::pane_surface("markdown-prompt", cx).key_context("Prompts").size_full().overflow_y_scroll().p_4().gap_3()
            .child(plugin_ui::PageHeader::new("Markdown Prompt"))
            .child(h_flex().gap_2().child(Switch::new("markdown-enabled", self.doc.enabled.into()).disabled(blocked).on_click(cx.listener(|this, state: &ui::ToggleState, _, cx| { this.doc.enabled = state.selected(); this.status = None; cx.notify(); }))).child(plugin_ui::label("Enabled")))
            .child(h_flex().justify_between().child(plugin_ui::label("Templates")).child(plugin_ui::action("refresh-templates", "Refresh / preview").disabled(blocked).on_click(cx.listener(|this, _, window, cx| this.refresh(false, window, cx)))))
            .child(h_flex().gap_2().flex_wrap().children(templates))
            .when(self.templates.is_empty(), |view| view.child(plugin_ui::label("No templates available. Enable Saved Prompts, Skills or another provider.").color(Color::Muted)))
            .children(self.warnings.iter().map(|warning| plugin_ui::notice(warning.clone(), true)))
            .child(plugin_ui::label("Constructed Markdown contents"))
            .child(plugin_ui::label("Type freely. Click or drag a template into the text. Select a chip to move, copy or delete it.").color(Color::Muted))
            .child(plugin_ui::outlined_content(cx).id("composition").w_full()
                .on_drag_move(cx.listener(|this, event: &gpui::DragMoveEvent<TemplateChip>, window, cx| {
                    if !this.busy && event.bounds.contains(&event.event.position) {
                        window.focus(&this.editor.focus_handle(cx), cx);
                        this.editor.update(cx, |editor, cx| rich_input::place_caret(editor, event.event.position, window, cx));
                    }
                }))
                .on_drop(cx.listener(|this, chip: &TemplateChip, window, cx| { cx.stop_propagation(); this.insert_at_cursor(chip.clone(), window, cx); }))
                .child(self.editor.clone()))
            .child(h_flex().gap_2().child(plugin_ui::label("Mode"))
                .child(plugin_ui::action("append-mode", if self.doc.append { "● Append" } else { "Append" }).disabled(blocked).on_click(cx.listener(|this, _, _, cx| { this.doc.append = true; this.status = None; cx.notify(); })))
                .child(plugin_ui::action("new-file-mode", if !self.doc.append { "● New file" } else { "New file" }).disabled(blocked).on_click(cx.listener(|this, _, _, cx| { this.doc.append = false; this.status = None; cx.notify(); }))))
            .child(plugin_ui::label(if self.doc.append { "Update a marked section in an existing file; surrounding content is preserved." } else { "Create and maintain an owned file; external edits are reported before replacement." }).color(Color::Muted))
            .child(plugin_ui::label("Filename (relative to this folder)"))
            .when(self.context.project_dir.is_none(), |view| view.child(plugin_ui::label("Free sessions has no destination folder. Open Markdown Prompt inside a folder space to apply this file.").color(Color::Muted)))
            .child(input_field("markdown-filename", self.filename.clone(), cx))
            .when_some(self.context.project_dir.clone(), |view, root| view.child(plugin_ui::label(root.display().to_string()).color(Color::Muted)))
            .when_some(self.preview.clone(), |view, text| view.child(plugin_ui::label("Expanded preview")).child(plugin_ui::outlined_content(cx).child(plugin_ui::label(text))))
            .when_some(self.error.clone(), |view, error| view.child(plugin_ui::notice(error, true)))
            .when_some(self.sync.as_ref().and_then(|sync| sync.upgrade()).and_then(|sync| sync.read(cx).problem(&self.path)), |view, error| view.child(plugin_ui::notice(format!("Automatic sync: {error}"), true)))
            .when_some(self.status.clone(), |view, status| view.child(plugin_ui::label(status)))
            .child(h_flex().justify_end().gap_2()
                .child(plugin_ui::action("save-markdown-draft", "Save draft").disabled(blocked).on_click(cx.listener(|this, _, _, cx| this.save_draft(cx))))
                .child(form_button("apply-markdown", if self.busy { "Working…" } else { "Apply" }).disabled(blocked || !self.doc.enabled || self.context.project_dir.is_none()).on_click(cx.listener(|this, _, window, cx| this.refresh(true, window, cx)))))
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
        let (view, cx) = cx
            .add_window_view(|window, cx| Composer::new(context.clone(), path.clone(), window, cx));
        cx.run_until_parked();
        view.update_in(cx, |this, window, cx| {
            let chip = this.templates[0].clone();
            this.insert_at_cursor(chip, window, cx);
            this.refresh(true, window, cx);
        });
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "first");
        publish("second\nline", "Renamed");
        view.update_in(cx, |this, window, cx| this.refresh(true, window, cx));
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "second\nline");
        let saved = document::load(&path).unwrap();
        assert!(
            saved
                .parts
                .iter()
                .any(|part| matches!(part, Part::Template { id, .. } if id == "stable"))
        );
        assert_eq!(saved.managed_files["CHARTR.md"], "second\nline");
        assert_eq!(
            cx.read_entity(&view, |this, _| this.preview.clone()),
            Some("second\nline".into())
        );
        services.remove("example.provider");
        view.update_in(cx, |this, window, cx| this.refresh(true, window, cx));
        cx.run_until_parked();
        assert!(cx.read_entity(&view, |this, _| {
            this.error.as_ref().is_some_and(|e| e.contains("unavailable"))
        }));
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "second\nline");
        publish("third", "Renamed");
        std::fs::write(root.path().join("CHARTR.md"), "User edits").unwrap();
        view.update_in(cx, |this, window, cx| this.refresh(true, window, cx));
        cx.run_until_parked();
        assert!(cx.read_entity(&view, |this, _| {
            this.error.as_ref().is_some_and(|e| e.contains("edited outside"))
        }));
        assert_eq!(std::fs::read_to_string(root.path().join("CHARTR.md")).unwrap(), "User edits");
    }
}
