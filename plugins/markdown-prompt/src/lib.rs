//! A project-scoped composer that applies Markdown only on explicit Save.
mod document;
mod persistence;
mod rich_input;
use crate::{components::input_field, text_input::TextInput};
use chartr_plugin::ui as plugin_ui;
use chartr_plugin::{
    Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar, services::PromptTemplates,
};
use document::{Bodies, Document, Part};
use editor::Editor;
use gpui::{App, Context, Entity, Focusable, Render, Window, div};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use ui::{Color, ScrollAxes, Scrollbars, WithScrollbar, prelude::*};

pub struct MarkdownPromptPlugin {
    data: PathBuf,
}
pub fn bundled(host: Host, cx: &mut App) -> Box<dyn PluginObject> {
    Box::new(MarkdownPromptPlugin::new(host, cx))
}
impl Plugin for MarkdownPromptPlugin {
    const ID: &'static str = "com.chartr.markdown-prompt";
    fn new(host: Host, _: &mut App) -> Self {
        Self { data: host.data_dir }
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
        cx.new(|cx| Composer::new(context.clone(), path, window, cx)).into()
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
        plugin_ui::template_chip(false, cx)
            .child(plugin_ui::label(self.title.clone()).color(Color::Muted))
    }
}
struct ApplyDraft {
    filename: Entity<TextInput>,
}

struct Composer {
    context: InstanceContext,
    path: PathBuf,
    doc: Document,
    apply_draft: Option<ApplyDraft>,
    editor: Entity<Editor>,
    chips_dirty: bool,
    templates_dirty: bool,
    templates: Vec<TemplateChip>,
    templates_scroll: gpui::ScrollHandle,
    bodies: Bodies,
    warnings: Vec<String>,
    error: Option<String>,
    preview: Option<String>,
    preview_focus: gpui::FocusHandle,
    busy: bool,
    invalid_config: bool,
    saved_bytes: Option<Vec<u8>>,
    saving: bool,
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
        let saved_bytes = std::fs::read(&path).ok();
        let editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(5, 22, window, cx);
            editor.set_mode(editor::EditorMode::Full {
                scale_ui_elements_with_buffer_font_size: false,
                show_active_line_background: false,
                sizing_behavior: editor::SizingBehavior::ExcludeOverscrollMargin,
            });
            editor.set_show_gutter(false, cx);
            editor.set_soft_wrap();
            editor.set_autoindent(false);
            editor.set_use_autoclose(false);
            editor.set_show_wrap_guides(false, cx);
            editor.set_show_indent_guides(false, cx);
            editor.set_text(rich_input::encode(&doc.parts), window, cx);
            editor
        });
        cx.subscribe(&editor, |this, _, event: &editor::EditorEvent, cx| {
            if matches!(event, editor::EditorEvent::BufferEdited) {
                this.chips_dirty = true;
                this.mark_edited(cx);
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
            apply_draft: None,
            editor,
            chips_dirty: true,
            templates_dirty: false,
            templates: vec![],
            templates_scroll: gpui::ScrollHandle::new(),
            bodies: Bodies::new(),
            warnings: vec![],
            error,
            preview: None,
            preview_focus: cx.focus_handle(),
            busy: false,
            invalid_config,
            saved_bytes,
            saving: false,
        };
        this.refresh(window, cx);
        this
    }
    fn snapshot(&self, cx: &App) -> Document {
        let mut doc = self.doc.clone();
        doc.parts = rich_input::decode(&self.editor.read(cx).text(cx));
        doc
    }
    fn has_changes(&self, cx: &App) -> bool {
        self.editor.read(cx).text(cx) != rich_input::encode(&self.doc.parts)
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.invalid_config || self.saving || !self.has_changes(cx) {
            return;
        }
        let saved = rich_input::encode(&self.doc.parts);
        self.editor.update(cx, |editor, cx| {
            editor.finalize_last_transaction(cx);
            editor.set_text(saved, window, cx);
            editor.finalize_last_transaction(cx);
        });
        self.chips_dirty = true;
        self.preview = None;
        self.error = None;
        window.focus(&self.editor.focus_handle(cx), cx);
        cx.notify();
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
        self.mark_edited(cx);
    }
    fn show_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.invalid_config || self.saving {
            return;
        }
        let filename = cx.new(|cx| {
            let mut input = TextInput::new("CHARTR.md", cx);
            input.set_text(self.doc.filename.clone(), false, cx);
            input
        });
        window.focus(&filename.focus_handle(cx), cx);
        self.preview = None;
        self.error = None;
        self.apply_draft = Some(ApplyDraft { filename });
        cx.notify();
    }

    fn cancel_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        self.apply_draft = None;
        window.focus(&self.editor.focus_handle(cx), cx);
        cx.notify();
    }

    fn save_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving || self.invalid_config {
            return;
        }
        let Some(draft) = self.apply_draft.as_ref() else { return };
        let mut doc = self.snapshot(cx);
        doc.filename = draft.filename.read(cx).text().trim().to_owned();
        if let Err(error) = document::validate_filename(&doc.filename) {
            self.error = Some(error);
            cx.notify();
            return;
        }
        let Some(project) = self.context.project_dir.clone() else {
            self.error = Some("Open a folder space to apply Markdown changes.".into());
            cx.notify();
            return;
        };
        let services = self.context.services.clone();
        let mut providers: Vec<_> = doc
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Template { provider, .. } => Some(provider.clone()),
                _ => None,
            })
            .collect();
        providers.sort();
        providers.dedup();
        // Resolve templates for this Save, even if the palette has not refreshed yet.
        let requests: Vec<_> = providers
            .into_iter()
            .map(|id| {
                let task = services
                    .get::<PromptTemplates>(&id)
                    .map(|provider| provider.list(Some(project.clone()), cx));
                (id, task)
            })
            .collect();
        let path = self.path.clone();
        let expected = self.saved_bytes.clone();
        self.saving = true;
        self.error = None;
        let keep_alive = cx.entity();
        cx.spawn_in(window, async move |this, cx| {
            let _keep_alive = keep_alive;
            let result = async {
                let mut bodies = Bodies::new();
                for (id, task) in requests {
                    let items = task
                        .ok_or_else(|| format!("Template provider {id} is unavailable."))?
                        .await?;
                    let mut ids = std::collections::HashSet::new();
                    for item in items {
                        if item.id.is_empty()
                            || !ids.insert(item.id.clone())
                            || item.title.trim().is_empty()
                            || item.prompt.len() > 1024 * 1024
                        {
                            return Err(format!("{id} returned invalid templates."));
                        }
                        bodies.insert((id.clone(), item.id), item.prompt);
                    }
                    if services.get::<PromptTemplates>(&id).is_none() {
                        return Err(format!("Template provider {id} is unavailable."));
                    }
                }
                let body = document::compose(&doc.parts, &bodies)?;
                cx.background_executor()
                    .spawn(async move {
                        persistence::save(&path, &project, doc, &body, expected.as_deref())
                    })
                    .await
            }
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.saving = false;
                match result {
                    Ok(doc) => {
                        this.saved_bytes = serde_json::to_vec_pretty(&doc).ok();
                        this.doc = doc;
                        this.apply_draft = None;
                        window.focus(&this.editor.focus_handle(cx), cx);
                    }
                    Err(error) => {
                        this.error = Some(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
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

    fn mark_edited(&mut self, cx: &mut Context<Self>) {
        if self.invalid_config {
            return;
        }
        self.preview = None;
        if self.apply_draft.is_none() {
            self.error = None;
        }
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
        let ui_font = theme::theme_settings(cx).ui_font(cx).clone();
        self.editor.update(cx, |editor, _| {
            editor.set_text_style_refinement(gpui::TextStyleRefinement {
                font_family: Some(ui_font.family),
                font_features: Some(ui_font.features),
                font_weight: Some(ui_font.weight),
                font_size: Some(plugin_ui::UI_TEXT_DEFAULT.into()),
                ..Default::default()
            });
        });
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
                plugin_ui::template_chip(false, cx)
                    .id(("template", index))
                    .max_w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .tooltip(ui::Tooltip::text(chip.provider.clone()))
                    .when(!blocked, |item| {
                        item.cursor_pointer()
                            .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.insert_at_cursor(click.clone(), window, cx)
                            }))
                    })
                    .child(plugin_ui::label(chip.title.clone()).truncate().color(if blocked {
                        Color::Disabled
                    } else {
                        Color::Muted
                    }))
            })
            .collect();
        let [thumb, hovered_thumb, active_thumb] =
            crate::components::scrollbar_thumb_colors(cx.theme().colors());
        let templates_panel = v_flex()
            .w(gpui::relative(0.24))
            .max_w(gpui::px(170.))
            .min_w(gpui::px(120.))
            .min_h_0()
            .gap_2()
            .child(
                h_flex()
                    .h_8()
                    .flex_shrink_0()
                    .justify_between()
                    .child(plugin_ui::label("Templates"))
                    .child(
                        plugin_ui::icon_action("refresh-templates", ui::IconName::RotateCw)
                            .aria_label("Refresh templates")
                            .disabled(blocked || self.busy)
                            .on_click(cx.listener(|this, _, window, cx| this.refresh(window, cx))),
                    ),
            )
            .child(
                plugin_ui::outlined_content(cx)
                    .id("template-list")
                    .debug_selector(|| "MARKDOWN_TEMPLATE_LIST".into())
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .p_2()
                    .overflow_y_scroll()
                    .track_scroll(&self.templates_scroll)
                    .child(v_flex().items_start().gap_2().children(templates))
                    .when(self.templates.is_empty(), |view| {
                        view.child(plugin_ui::label("No templates").color(Color::Muted))
                    })
                    .children(
                        self.warnings
                            .iter()
                            .map(|warning| plugin_ui::notice(warning.clone(), true)),
                    )
                    .custom_scrollbars(
                        Scrollbars::always_visible(ScrollAxes::Vertical)
                            .id("markdown-template-scrollbar")
                            .thumb_colors(thumb, hovered_thumb, active_thumb)
                            .with_track_along(
                                ScrollAxes::Vertical,
                                cx.theme().colors().editor_background,
                            )
                            .tracked_scroll_handle(&self.templates_scroll)
                            .notify_content(),
                        window,
                        cx,
                    ),
            );
        let contents_panel = v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .gap_2()
            .child(
                h_flex()
                    .h_8()
                    .flex_shrink_0()
                    .justify_between()
                    .child(plugin_ui::label("Constructed Prompt"))
                    .child(
                        plugin_ui::action("preview-markdown", "Preview")
                            .disabled(blocked || self.busy)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.show_preview(window, cx)),
                            ),
                    ),
            )
            .child(
                plugin_ui::outlined_content(cx)
                    .id("composition")
                    .debug_selector(|| "MARKDOWN_COMPOSITION".into())
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_hidden()
                    .on_drag_move(cx.listener(
                        |this, event: &gpui::DragMoveEvent<TemplateChip>, window, cx| {
                            if !this.invalid_config && event.bounds.contains(&event.event.position)
                            {
                                window.focus(&this.editor.focus_handle(cx), cx);
                                this.editor.update(cx, |editor, cx| {
                                    rich_input::place_caret(
                                        editor,
                                        event.event.position,
                                        window,
                                        cx,
                                    )
                                });
                            }
                        },
                    ))
                    .on_drop(cx.listener(|this, chip: &TemplateChip, window, cx| {
                        cx.stop_propagation();
                        this.insert_at_cursor(chip.clone(), window, cx);
                    }))
                    .child(self.editor.clone()),
            );
        let page = plugin_ui::pane_surface("markdown-prompt", cx)
            .key_context("Prompts")
            .overflow_hidden()
            .pt_4()
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .gap_4()
                    .px_4()
                    .pb_4()
                    .child(templates_panel)
                    .child(contents_panel),
            )
            .when_some(self.error.clone().filter(|_| self.apply_draft.is_none()), |view, error| {
                view.child(div().px_4().pb_2().child(plugin_ui::notice(error, true)))
            })
            .child(
                plugin_ui::dialog_actions(cx)
                    .flex_shrink_0()
                    .px_4()
                    .pb_3()
                    .debug_selector(|| "MARKDOWN_FOOTER".into())
                    .child(
                        plugin_ui::action("reset-markdown", "Reset")
                            .disabled(blocked || self.saving || !self.has_changes(cx))
                            .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
                    )
                    .child(
                        plugin_ui::action(
                            "edit-markdown-apply",
                            if self.saving { "Applying…" } else { "Apply changes" },
                        )
                        .disabled(blocked || self.saving)
                        .on_click(cx.listener(|this, _, window, cx| this.show_apply(window, cx))),
                    ),
            );

        div()
            .size_full()
            .relative()
            .track_focus(&self.preview_focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    if this.apply_draft.is_some() {
                        this.cancel_apply(window, cx);
                        cx.stop_propagation();
                    } else if this.preview.is_some() {
                        this.dismiss_preview(window, cx);
                        cx.stop_propagation();
                    }
                }
            }))
            .child(page)
            .when_some(self.apply_draft.as_ref(), |view, draft| {
                view.child(
                    plugin_ui::ModalOverlay::new("markdown-apply-scrim", cx.listener(|this, _, window, cx| this.cancel_apply(window, cx)))
                        .child(plugin_ui::DialogSurface::new("markdown-apply-dialog")
                            .child(plugin_ui::dialog_header("Apply changes", div(), cx))
                            .child(plugin_ui::dialog_body().id("markdown-apply-contents").overflow_y_scroll()
                                .child(plugin_ui::label("Filename (relative to this folder)"))
                                .when(self.context.project_dir.is_none(), |view| view.child(plugin_ui::label("Open Markdown Prompt inside a folder space to apply changes.").color(Color::Muted)))
                                .child(input_field("markdown-filename", draft.filename.clone(), cx))
                                .when_some(self.error.clone(), |view, error| view.child(plugin_ui::notice(error, true)))
                                .when_some(self.context.project_dir.clone(), |view, root| view.child(plugin_ui::label(root.display().to_string()).color(Color::Muted)))
                            )
                            .child(plugin_ui::dialog_actions(cx)
                                .child(plugin_ui::action("cancel-markdown-apply", "Cancel").disabled(self.saving).on_click(cx.listener(|this, _, window, cx| this.cancel_apply(window, cx))))
                                .child(plugin_ui::action("save-markdown-apply", "Save").disabled(blocked || self.saving).on_click(cx.listener(|this, _, window, cx| this.save_apply(window, cx)))))
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
    use std::{fs, time::Duration};

    fn init(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            crate::prompts_plugin::init(cx);
        });
    }
    fn context(root: &std::path::Path, services: Services) -> InstanceContext {
        InstanceContext {
            instance_id: 1,
            space: "fixture".into(),
            space_name: "Fixture".into(),
            project_dir: Some(root.into()),
            bound_session: None,
            terminal: TerminalLauncher::new(|_, _| {}),
            services,
            plugin_settings: PluginSettings::new(|_, _, _| {}),
        }
    }

    #[gpui::test]
    fn edits_only_apply_on_save_and_dialog_errors_and_cancellation_preserve_files(
        cx: &mut gpui::TestAppContext,
    ) {
        init(cx);
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        let context = context(root.path(), Services::default());
        let (view, cx) = cx
            .add_window_view(|window, cx| Composer::new(context.clone(), path.clone(), window, cx));
        cx.run_until_parked();
        assert!(!cx.read_entity(&view, |this, cx| this.has_changes(cx)));
        let templates = cx.debug_bounds("MARKDOWN_TEMPLATE_LIST").unwrap();
        let composition = cx.debug_bounds("MARKDOWN_COMPOSITION").unwrap();
        let footer = cx.debug_bounds("MARKDOWN_FOOTER").unwrap();
        assert!(templates.right() < composition.left());
        assert_eq!(templates.top(), composition.top());
        assert_eq!(templates.bottom(), composition.bottom());
        assert!(composition.bottom() < footer.top());
        assert!(composition.size.height > gpui::px(100.));
        view.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("first", window, cx));
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(5));
        cx.run_until_parked();
        assert!(!path.exists());
        let output = root.path().join("CHARTR.md");
        assert!(!output.exists());
        view.update_in(cx, |this, window, cx| {
            this.show_apply(window, cx);
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        let first = document::appended("", "first").unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), first);
        assert!(cx.read_entity(&view, |this, _| this.apply_draft.is_none()));
        assert!(!cx.read_entity(&view, |this, cx| this.has_changes(cx)));
        view.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("discard this edit", window, cx));
            assert!(this.has_changes(cx));
            this.reset(window, cx);
            assert!(!this.has_changes(cx));
            assert_eq!(this.editor.read(cx).text(cx), "first");
        });
        cx.run_until_parked();
        assert_eq!(fs::read_to_string(&output).unwrap(), first);
        view.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("second", window, cx));
            this.show_apply(window, cx);
            this.apply_draft
                .as_ref()
                .unwrap()
                .filename
                .update(cx, |input, cx| input.set_text("invalid.txt", true, cx));
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        assert!(cx.read_entity(&view, |this, _| this.apply_draft.is_some()
            && this.error.as_ref().is_some_and(|e| e.contains(".md"))));
        assert_eq!(document::load(&path).unwrap().filename, "CHARTR.md");
        assert_eq!(fs::read_to_string(&output).unwrap(), first);
        cx.simulate_keystrokes("escape");
        assert!(cx.read_entity(&view, |this, _| this.apply_draft.is_none()));
        view.update_in(cx, |this, window, cx| this.show_apply(window, cx));
        cx.run_until_parked();
        cx.simulate_click(gpui::point(gpui::px(1.), gpui::px(1.)), gpui::Modifiers::none());
        assert!(cx.read_entity(&view, |this, _| this.apply_draft.is_none()));
        view.update_in(cx, |this, window, cx| {
            this.show_apply(window, cx);
            assert_eq!(this.apply_draft.as_ref().unwrap().filename.read(cx).text(), "CHARTR.md");
            this.apply_draft
                .as_ref()
                .unwrap()
                .filename
                .update(cx, |input, cx| input.set_text("AGENTS.md", true, cx));
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        assert!(!output.exists());
        let output = root.path().join("AGENTS.md");
        let second = document::appended("", "second").unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), second);
        view.update_in(cx, |_, window, _| window.remove_window());
        drop(view);
        let (restored, cx) = cx
            .add_window_view(|window, cx| Composer::new(context.clone(), path.clone(), window, cx));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(5));
        cx.run_until_parked();
        assert_eq!(fs::read_to_string(&output).unwrap(), second);
        assert_eq!(cx.read_entity(&restored, |this, cx| this.editor.read(cx).text(cx)), "second");
        restored.update_in(cx, |this, window, cx| {
            this.editor.update(cx, |editor, cx| editor.set_text("", window, cx));
            this.show_apply(window, cx);
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        assert!(!output.exists());
    }

    #[gpui::test]
    fn template_changes_wait_for_save_and_save_resolves_fresh_bodies(
        cx: &mut gpui::TestAppContext,
    ) {
        init(cx);
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        let services = Services::default();
        let publish = |body: &str| {
            let body = body.to_owned();
            services.publish(
                "example.provider",
                vec![ServiceExport::new(PromptTemplates::new(move |_, _| {
                    gpui::Task::ready(Ok(vec![SavedPrompt {
                        id: "stable".into(),
                        title: "Template".into(),
                        prompt: body.clone(),
                    }]))
                }))],
            );
        };
        publish("first");
        let context = context(root.path(), services.clone());
        let (view, cx) =
            cx.add_window_view(|window, cx| Composer::new(context, path.clone(), window, cx));
        cx.run_until_parked();
        view.update_in(cx, |this, window, cx| {
            this.insert_at_cursor(this.templates[0].clone(), window, cx);
            this.show_apply(window, cx);
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        let output = root.path().join("CHARTR.md");
        let first = document::appended("", "first").unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), first);
        publish("second");
        cx.update(|_, cx| PromptTemplates::changed(cx));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(5));
        cx.run_until_parked();
        assert_eq!(fs::read_to_string(&output).unwrap(), first);
        view.update_in(cx, |this, window, cx| this.show_preview(window, cx));
        assert_eq!(cx.read_entity(&view, |this, _| this.preview.clone()), Some("second".into()));
        view.update_in(cx, |this, window, cx| this.dismiss_preview(window, cx));
        // No notification: Save must still use the current provider body.
        publish("third");
        view.update_in(cx, |this, window, cx| {
            this.show_apply(window, cx);
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        let third = document::appended("", "third").unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), third);
        services.remove("example.provider");
        view.update_in(cx, |this, window, cx| {
            this.show_apply(window, cx);
            this.save_apply(window, cx);
        });
        cx.run_until_parked();
        assert!(
            cx.read_entity(&view, |this, _| this.error.is_some() && this.apply_draft.is_some())
        );
        assert_eq!(fs::read_to_string(&output).unwrap(), third);
    }
}
