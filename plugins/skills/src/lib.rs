//! Bundled Skills plugin: an empty pane and an ordered source-management page.
mod sources;

use crate::{
    components::{
        ContextMenu, ListSorter, PopupMenu, form_button, form_picker, form_row, input_field,
    },
    fonts::{UI_LABEL_DEFAULT, UI_LABEL_LARGE, UI_LABEL_SMALL},
    text_input::TextInput,
};
use sources::{Kind, Operation, Source, State, Store};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use ui::{
    Button, ButtonStyle, Color, Icon, IconButton, IconName, IconPosition, IconSize, Label, Switch,
    TintColor, prelude::*,
};
use zeddy_plugin::{
    Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar, gpui,
    gpui::{
        Anchor, AnyElement, App, Context, Entity, Focusable, IntoElement, MouseButton, Render,
        SharedString, Window, div, px, relative,
    },
    services::PluginSettings,
};

pub struct SkillsPlugin {
    registry: Entity<Registry>,
}

#[derive(Default)]
struct SharedRegistries(std::collections::HashMap<PathBuf, gpui::WeakEntity<Registry>>);
impl gpui::Global for SharedRegistries {}

pub fn bundled(host: Host, cx: &mut App) -> Box<dyn PluginObject> {
    Box::new(SkillsPlugin::new(host, cx))
}

impl Plugin for SkillsPlugin {
    const ID: &'static str = "com.chartr.skills";
    fn new(host: Host, cx: &mut App) -> Self {
        let existing = cx
            .default_global::<SharedRegistries>()
            .0
            .get(&host.data_dir)
            .and_then(gpui::WeakEntity::upgrade);
        let registry = existing.unwrap_or_else(|| {
            let registry = cx.new(|_| Registry::load(host.data_dir.clone()));
            cx.default_global::<SharedRegistries>().0.insert(host.data_dir, registry.downgrade());
            registry
        });
        Self { registry }
    }
    fn activate(&mut self, registrar: &mut Registrar, _: &mut App) {
        registrar.add_pane("main", "Skills").add_settings();
    }
    fn services(&self) -> Vec<zeddy_plugin::services::ServiceExport> {
        use zeddy_plugin::services::{ServiceExport, Skills};
        let registry = self.registry.downgrade();
        vec![ServiceExport::new(Skills::new(move |cx| {
            let Some(registry) = registry.upgrade() else {
                return gpui::Task::ready(Err("Skills is unavailable.".into()));
            };
            let registry = registry.read(cx);
            if registry.load_failed {
                return gpui::Task::ready(Err(registry.problem.clone().unwrap_or_default()));
            }
            if registry.busy.is_some() {
                return gpui::Task::ready(Err(
                    "Skill sources are being updated. Try again shortly.".into(),
                ));
            }
            let store = registry.store.clone();
            cx.background_executor()
                .spawn(async move { store.catalog().map_err(|error| format!("{error:#}")) })
        }))]
    }
    fn view(
        &mut self,
        _: &PaneKey,
        context: &InstanceContext,
        _: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyView {
        cx.new(|_| SkillsPane { settings: context.plugin_settings.clone() }).into()
    }
    fn settings(&mut self, _: &mut Window, cx: &mut App) -> Option<gpui::AnyView> {
        Some(cx.new(|cx| SkillsView::new(self.registry.clone(), cx)).into())
    }
}

struct SkillsPane {
    settings: PluginSettings,
}

impl Render for SkillsPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.settings.clone();
        let menu = PopupMenu::new("skills-pane-menu")
            .trigger(
                IconButton::new("skills-pane-menu-trigger", IconName::ChevronDown)
                    .icon_size(IconSize::Small)
                    .aria_label("Skills pane menu"),
            )
            .anchor(Anchor::TopRight)
            .menu(move |window, cx| {
                let settings = settings.clone();
                Some(ContextMenu::build_popup(window, cx, move |menu| {
                    menu.entry("Skill source settings", None, move |window, cx| {
                        settings.open(Some(SkillsPlugin::ID), window, cx);
                    })
                }))
            });
        div()
            .id("skills-pane")
            .size_full()
            .relative()
            .bg(cx.theme().colors().editor_background)
            .child(div().absolute().top_3().right_3().child(menu))
            .into_any_element()
    }
}

struct Registry {
    store: Store,
    states: Vec<State>,
    problem: Option<String>,
    load_failed: bool,
    busy: Option<String>,
    cancel: Option<Arc<AtomicBool>>,
}

impl Registry {
    fn load(root: PathBuf) -> Self {
        let (store, problem) = match Store::load(root.clone()) {
            Ok(store) => (store, None),
            Err(error) => (Store { root, sources: Vec::new() }, Some(format!("{error:#}"))),
        };
        Self {
            store,
            states: Vec::new(),
            load_failed: problem.is_some(),
            problem,
            busy: None,
            cancel: None,
        }
    }

    fn run(
        &mut self,
        operation: Operation,
        cx: &mut Context<Self>,
    ) -> Option<gpui::Task<Result<(), String>>> {
        if self.busy.is_some() {
            return None;
        }
        if self.load_failed && !matches!(operation, Operation::Scan) {
            return None;
        }
        let label = match &operation {
            Operation::Scan => "Reading skill sources…",
            Operation::Save { .. } => "Saving skill source…",
            Operation::Remove(_) => "Removing skill source…",
            Operation::Enable(..) | Operation::Move { .. } => "Saving source order and selection…",
            Operation::Refresh(_) => "Refreshing remote source…",
        };
        self.busy = Some(label.into());
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        let mut store = self.store.clone();
        let reload = self.load_failed;
        let work = cx.background_executor().spawn(async move {
            let result = (|| -> anyhow::Result<_> {
                if reload {
                    store = Store::load(store.root.clone())?;
                }
                store.apply(operation, &cancel)?;
                let states = store.states();
                Ok((store, states))
            })();
            result.map_err(|error| format!("{error:#}"))
        });
        cx.notify();
        Some(cx.spawn(async move |this, cx| {
            let result = work.await;
            let status = result.as_ref().map(|_| ()).map_err(Clone::clone);
            let _ = this.update(cx, |this, cx| {
                this.busy = None;
                this.cancel = None;
                match result {
                    Ok((store, states)) => {
                        this.store = store;
                        this.states = states;
                        this.problem = None;
                        this.load_failed = false;
                    }
                    Err(error) => this.problem = Some(error),
                }
                cx.notify();
            });
            status
        }))
    }
}

struct SkillsView {
    registry: Entity<Registry>,
    editor_open: bool,
    editing: Option<String>,
    deleting: Option<String>,
    name: Entity<TextInput>,
    path: Entity<TextInput>,
    url: Entity<TextInput>,
    git_ref: Entity<TextInput>,
    kind: Kind,
    form_error: Option<String>,
    sorter: ListSorter<String>,
    pending_order: Option<Vec<String>>,
    focus: gpui::FocusHandle,
}

#[derive(Clone)]
struct DraggedSource {
    name: String,
    owner: gpui::EntityId,
}
impl Render for DraggedSource {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

impl SkillsView {
    fn new(registry: Entity<Registry>, cx: &mut Context<Self>) -> Self {
        cx.observe(&registry, |this, registry, cx| {
            if registry.read(cx).busy.is_some() && this.pending_order.is_none() {
                this.sorter.cancel();
            }
            cx.notify();
        })
        .detach();
        let this = Self {
            registry,
            sorter: ListSorter::new(gpui::Rems(0.)),
            pending_order: None,
            focus: cx.focus_handle(),
            editor_open: false,
            editing: None,
            deleting: None,
            kind: Kind::Local,
            form_error: None,
            name: cx.new(|cx| TextInput::new("my skills", cx)),
            path: cx.new(|cx| TextInput::new("~/skills", cx)),
            url: cx.new(|cx| TextInput::new("https://github.com/someone/skills.git", cx)),
            git_ref: cx.new(|cx| TextInput::new("The default branch", cx)),
        };
        this.run(Operation::Scan, cx);
        this
    }

    fn run(&self, operation: Operation, cx: &mut Context<Self>) {
        if let Some(task) = self.registry.update(cx, |registry, cx| registry.run(operation, cx)) {
            task.detach();
        }
    }

    fn finish_drag(&mut self, y: gpui::Pixels, window: &mut Window, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        let Some((name, target)) =
            self.sorter.drop_at(y, window.rem_size(), now, cx.reduce_motion())
        else {
            return;
        };
        let sources = &self.registry.read(cx).store.sources;
        let Some(before) = sources.get(target).map(|source| source.name.clone()) else {
            self.sorter.cancel();
            return;
        };
        let mut order: Vec<_> = sources.iter().map(|source| source.name.clone()).collect();
        self.sorter.arrange(&mut order, Clone::clone);
        self.pending_order = Some(order);
        let task = self
            .registry
            .update(cx, |registry, cx| registry.run(Operation::Move { name, before }, cx));
        let Some(task) = task else {
            self.pending_order = None;
            self.sorter.cancel();
            cx.notify();
            return;
        };
        self.sorter.accept_drop(now, cx.reduce_motion());
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.pending_order = None;
                if result.is_err() {
                    this.sorter.cancel();
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn open_editor(&mut self, source: Option<Source>, window: &mut Window, cx: &mut Context<Self>) {
        if self.registry.read(cx).busy.is_some() {
            return;
        }
        self.editing = source.as_ref().map(|source| source.name.clone());
        self.kind = source.as_ref().map(|source| source.kind).unwrap_or_default();
        self.name.update(cx, |input, cx| {
            input.set_text(source.as_ref().map(|s| s.name.as_str()).unwrap_or(""), false, cx)
        });
        self.path.update(cx, |input, cx| {
            input.set_text(
                source
                    .as_ref()
                    .filter(|s| s.kind == Kind::Local)
                    .map(|s| s.path.display().to_string())
                    .unwrap_or_default(),
                false,
                cx,
            )
        });
        self.url.update(cx, |input, cx| {
            input.set_text(source.as_ref().map(|s| s.url.as_str()).unwrap_or(""), false, cx)
        });
        self.git_ref.update(cx, |input, cx| {
            input.set_text(source.as_ref().map(|s| s.git_ref.as_str()).unwrap_or(""), false, cx)
        });
        self.form_error = None;
        self.editor_open = true;
        window.focus(&self.name.focus_handle(cx), cx);
        cx.notify();
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = &self.registry.read(cx).cancel {
            cancel.store(true, Ordering::Relaxed);
            return;
        }
        self.editor_open = false;
        self.editing = None;
        self.deleting = None;
        self.form_error = None;
        cx.notify();
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.blur();
        let source = Source {
            name: self.name.read(cx).text().to_owned(),
            kind: self.kind,
            path: PathBuf::from(self.path.read(cx).text()),
            url: self.url.read(cx).text().to_owned(),
            git_ref: self.git_ref.read(cx).text().to_owned(),
            commit: String::new(),
            enabled: true,
        };
        self.submit(Operation::Save { source, editing: self.editing.clone() }, cx);
    }

    fn submit(&mut self, operation: Operation, cx: &mut Context<Self>) {
        let Some(task) = self.registry.update(cx, |registry, cx| registry.run(operation, cx))
        else {
            return;
        };
        self.form_error = None;
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.editor_open = false;
                        this.editing = None;
                        this.deleting = None;
                    }
                    Err(error) => this.form_error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn pick_folder(&mut self, _: &gpui::ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Register skill source".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = chosen.await;
            let _ = this.update_in(cx, |this, _, cx| {
                if !this.editor_open || this.registry.read(cx).busy.is_some() {
                    return;
                }
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.first() {
                            this.path.update(cx, |input, cx| {
                                input.set_text(path.display().to_string(), false, cx)
                            });
                        }
                    }
                    Ok(Err(error)) => this.form_error = Some(error.to_string()),
                    _ => {}
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn management(&self, cx: &mut Context<Self>) -> AnyElement {
        let registry = self.registry.read(cx);
        let busy = registry.busy.is_some();
        let blocked = busy || registry.load_failed;
        let sources = registry.store.sources.clone();
        let states = registry.states.clone();
        let problem = registry.problem.clone();
        // A save banner would move the whole table just as the dropped row
        // settles. Keep its geometry steady during the optimistic commit.
        let status = self.pending_order.is_none().then(|| registry.busy.clone()).flatten();
        let now = cx.background_executor().now();
        let mut rows: Vec<_> = sources
            .iter()
            .enumerate()
            .map(|(index, source)| (index, source, states.get(index).cloned()))
            .collect();
        if let Some(order) = &self.pending_order {
            rows.sort_by_key(|(_, source, _)| order.iter().position(|name| name == &source.name));
        } else {
            self.sorter.arrange(&mut rows, |(_, source, _)| source.name.clone());
        }
        let mut drawn = Vec::new();
        for (position, (index, source, state)) in rows.iter().enumerate() {
            let index = *index;
            let unavailable = state.as_ref().is_some_and(|state| state.unavailable);
            let state = state.clone().unwrap_or_default();
            let name = source.name.clone();
            let drag = DraggedSource { name: name.clone(), owner: cx.entity_id() };
            let enabled_registry = self.registry.clone();
            let toggle_name = name.clone();
            let detail = if source.kind == Kind::Local {
                source.path.display().to_string()
            } else {
                source.url.clone()
            };
            let name_cell = h_flex()
                .id(format!("skill-source-name-{name}"))
                .gap_2()
                .w_full()
                .min_w_0()
                .child(
                    div().on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()).child(
                        Switch::new(format!("skill-source-enabled-{name}"), source.enabled.into())
                            .disabled(blocked)
                            .aria_label(format!("Enable {name}"))
                            .on_click(move |state, _, cx| {
                                if let Some(task) = enabled_registry.update(cx, |registry, cx| {
                                    registry.run(
                                        Operation::Enable(toggle_name.clone(), state.selected()),
                                        cx,
                                    )
                                }) {
                                    task.detach();
                                }
                            }),
                    ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(Label::new(name.clone()).truncate())
                        .child(
                            Label::new(detail).size(UI_LABEL_SMALL).color(Color::Muted).truncate(),
                        )
                        .when(source.kind == Kind::Git, |view| {
                            view.child(
                                Label::new(format!(
                                    "{} · {}",
                                    source.git_ref,
                                    source.commit.chars().take(12).collect::<String>()
                                ))
                                .size(UI_LABEL_SMALL)
                                .color(Color::Muted),
                            )
                        }),
                );
            let edit_source = (*source).clone();
            let delete_name = name.clone();
            let refresh_name = name.clone();
            let mut actions = h_flex()
                .w_full()
                .justify_end()
                .gap_1()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    Button::new(("edit-skill-source", index), "Edit").disabled(blocked).on_click(
                        cx.listener(move |this, _, window, cx| {
                            this.open_editor(Some(edit_source.clone()), window, cx)
                        }),
                    ),
                );
            if source.kind == Kind::Git {
                actions = actions.child(
                    IconButton::new(("refresh-skill-source", index), IconName::RotateCw)
                        .icon_size(IconSize::Small)
                        .aria_label(format!("Refresh {name}"))
                        .disabled(blocked)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.run(Operation::Refresh(refresh_name.clone()), cx)
                        })),
                );
            }
            // Keyboard-accessible order controls complement dragging a source row.
            for (offset, icon, label) in
                [(-1isize, IconName::ArrowUp, "Move up"), (1, IconName::ArrowDown, "Move down")]
            {
                let adjacent = position
                    .checked_add_signed(offset)
                    .and_then(|i| rows.get(i))
                    .map(|(_, source, _)| source.name.clone());
                let move_name = name.clone();
                actions = actions.child(
                    IconButton::new(format!("source-order-{index}-{offset}"), icon)
                        .icon_size(IconSize::Small)
                        .aria_label(format!("{label}: {name}"))
                        .disabled(blocked || adjacent.is_none())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(before) = &adjacent {
                                this.run(
                                    Operation::Move {
                                        name: move_name.clone(),
                                        before: before.clone(),
                                    },
                                    cx,
                                );
                            }
                        })),
                );
            }
            actions = actions.child(
                IconButton::new(("delete-skill-source", index), IconName::Trash)
                    .icon_size(IconSize::Small)
                    .aria_label(format!("Delete {name}"))
                    .disabled(blocked)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.deleting = Some(delete_name.clone());
                        this.form_error = None;
                        cx.notify();
                    })),
            );
            let count = if unavailable {
                "Unavailable".into()
            } else if states.get(index).is_none() {
                "…".into()
            } else {
                state.skills.len().to_string()
            };
            let shadowed = state.skills.iter().filter(|skill| skill.shadowed).count();
            let cells = vec![
                name_cell.into_any_element(),
                Label::new(if source.kind == Kind::Local { "local" } else { "remote" })
                    .color(Color::Muted)
                    .into_any_element(),
                v_flex()
                    .child(Label::new(count))
                    .when(shadowed > 0, |view| {
                        view.child(
                            Label::new(format!("{shadowed} shadowed"))
                                .size(UI_LABEL_SMALL)
                                .color(Color::Muted),
                        )
                    })
                    .into_any_element(),
                actions.into_any_element(),
            ];
            let held = self.sorter.holds(name.clone());
            let offset = self.sorter.offset_of(name.clone(), now, cx.reduce_motion());
            let colors = cx.theme().colors();
            let row = source_columns(cells)
                .id(format!("skill-source-row-{name}"))
                .debug_selector({
                    let name = name.clone();
                    move || format!("SOURCE_ROW-{name}")
                })
                .relative()
                .py_1()
                .border_1()
                .border_color(gpui::transparent_black())
                .bg(if position % 2 == 1 {
                    colors.element_background
                } else {
                    colors.editor_background
                })
                .when(held, |row| row.border_color(colors.drop_target_border).shadow_md())
                .when(offset != px(0.), |row| row.top(offset))
                .when(!blocked && sources.len() > 1, |row| {
                    row.when(!cx.has_active_drag(), |row| row.cursor_grab())
                        .when(cx.has_active_drag(), |row| row.cursor_grabbing())
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                                window.focus(&this.focus, cx);
                                this.sorter.press(event.position.y)
                            }),
                        )
                        .on_drag(drag, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
                });
            drawn.push(
                div()
                    .id(format!("skill-source-slot-{name}"))
                    .debug_selector({
                        let name = name.clone();
                        move || format!("SOURCE_SLOT-{name}")
                    })
                    .relative()
                    .w_full()
                    .flex_none()
                    .child(if held {
                        gpui::deferred(row).into_any_element()
                    } else {
                        row.into_any_element()
                    }),
            );
        }
        let header = source_columns(
            ["Name", "Type", "Skills", "Actions"]
                .into_iter()
                .map(|name| Label::new(name).into_any_element())
                .collect(),
        )
        .pb_2()
        .border_b_1()
        .border_color(cx.theme().colors().border);
        let table = v_flex().w_full().flex_1().min_h_0().child(header).child(
            v_flex()
                .id("skill-source-rows")
                .w_full()
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(self.sorter.scroll_handle())
                .children(drawn)
                .when(sources.is_empty(), |view| {
                    view.child(Label::new("No registered skill sources.").color(Color::Muted))
                }),
        );
        let warnings: Vec<_> = sources
            .iter()
            .zip(&states)
            .flat_map(|(source, state)| {
                state.warnings.iter().map(|warning| format!("{}: {warning}", source.name))
            })
            .collect();
        v_flex().id("skill-source-settings").size_full().min_h_0().bg(cx.theme().colors().editor_background)
            .child(v_flex().w_full().h_full().min_h_0().gap_5()
                .child(h_flex().w_full().justify_between().gap_3()
                    .child(Label::new("Skill sources").size(UI_LABEL_LARGE))
                    .child(h_flex().gap_2()
                        .child(Button::new("rescan-skill-sources", "Rescan").disabled(busy).on_click(cx.listener(|this, _, _, cx| this.run(Operation::Scan, cx))))
                        .child(Button::new("new-skill-source", "New source").style(ButtonStyle::Outlined).start_icon(Icon::new(IconName::Plus)).disabled(blocked).on_click(cx.listener(|this, _, window, cx| this.open_editor(None, window, cx))))))
                .child(Label::new("Register folders and Git repositories containing skills. Earlier enabled sources take precedence for duplicate skill names. Drag rows or use the arrows to change their order.").color(Color::Muted))
                .when_some(problem, |view, problem| view.child(notice_banner(problem, true, cx)))
                .when_some(status, |view, status| view.child(h_flex().gap_2().child(Label::new(status).color(Color::Muted)).child(Button::new("cancel-skill-source-operation", "Cancel").on_click(cx.listener(|this, _, _, cx| { if let Some(cancel) = &this.registry.read(cx).cancel { cancel.store(true, Ordering::Relaxed); } })))))
                .children(warnings.into_iter().map(|warning| notice_banner(warning, false, cx)))
                .child(table))
            .into_any_element()
    }

    fn editor(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.editor_open {
            return None;
        }
        let busy = self.registry.read(cx).busy.is_some();
        let kind = self.kind;
        let weak = cx.weak_entity();
        let picker = PopupMenu::new("skill-source-kind")
            .trigger(form_picker("skill-source-kind-trigger", kind.label()).disabled(busy))
            .anchor(Anchor::TopLeft)
            .menu(move |window, cx| {
                let weak = weak.clone();
                Some(ContextMenu::build_popup(window, cx, move |mut menu| {
                    for option in [Kind::Local, Kind::Git] {
                        let weak = weak.clone();
                        menu = menu.toggleable_entry(
                            option.label(),
                            option == kind,
                            IconPosition::End,
                            None,
                            move |_, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    this.kind = option;
                                    this.form_error = None;
                                    cx.notify();
                                });
                            },
                        );
                    }
                    menu
                }))
            });
        let mut fields = v_flex()
            .id("skill-source-form-fields")
            .relative()
            .w_full()
            .p_4()
            .gap_3()
            .overflow_y_scroll()
            .child(form_row("Name", None, input_field("skill-source-name", self.name.clone(), cx)))
            .child(form_row("Kind", None, picker.into_any_element()));
        match kind {
            Kind::Local => {
                let folder = h_flex()
                    .w_full()
                    .gap_2()
                    .child(div().flex_1().min_w_0().child(input_field(
                        "skill-source-path",
                        self.path.clone(),
                        cx,
                    )))
                    .child(
                        form_button("pick-skill-source-folder", "Select folder")
                            .start_icon(Icon::new(IconName::FolderOpen))
                            .disabled(busy)
                            .on_click(cx.listener(Self::pick_folder)),
                    );
                fields = fields.child(form_row("Path", Some("Your folder, edited by you — Chartr only reads it. Use an absolute path, or ~/ for your home directory."), folder.into_any_element()));
            }
            Kind::Git => {
                fields = fields
                    .child(form_row("URL", None, input_field("skill-source-url", self.url.clone(), cx)))
                    .child(form_row("Ref", Some("Optional branch or tag; leave blank for the default branch. Chartr keeps its own checkout. Register repositories you trust; no further approval is required for refreshes."), input_field("skill-source-ref", self.git_ref.clone(), cx)));
            }
        }
        let fields = fields
            .when_some(self.form_error.clone(), |form, error| {
                form.child(notice_banner(error, true, cx))
            })
            .when(busy, |form| {
                form.child(Label::new("Saving source…").color(Color::Muted)).child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .occlude()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
                )
            });
        let content = v_flex()
            .id("skill-source-editor")
            .w(px(660.))
            .max_w(relative(0.92))
            .max_h(relative(0.9))
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().elevated_surface_background)
            .shadow_lg()
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(cx.theme().colors().border)
                    .child(
                        Label::new(if self.editing.is_some() {
                            "Edit skill source"
                        } else {
                            "Register a source"
                        })
                        .size(UI_LABEL_LARGE),
                    )
                    .child(
                        IconButton::new("close-skill-source-editor", IconName::Close)
                            .aria_label("Close dialog")
                            .on_click(cx.listener(|this, _, _, cx| this.dismiss(cx))),
                    ),
            )
            .child(fields)
            .child(
                h_flex()
                    .w_full()
                    .justify_end()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .border_t_1()
                    .border_color(cx.theme().colors().border)
                    .child(
                        Button::new("cancel-skill-source-editor", "Cancel")
                            .on_click(cx.listener(|this, _, _, cx| this.dismiss(cx))),
                    )
                    .child(
                        Button::new(
                            "save-skill-source",
                            if self.editing.is_some() { "Save" } else { "Register" },
                        )
                        .style(ButtonStyle::Filled)
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            );
        Some(self.overlay("skill-source-editor-scrim", content.into_any_element(), cx))
    }

    fn deletion(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let name = self.deleting.clone()?;
        let busy = self.registry.read(cx).busy.is_some();
        let remote = self
            .registry
            .read(cx)
            .store
            .sources
            .iter()
            .any(|source| source.name == name && source.kind == Kind::Git);
        let detail = if remote {
            "Its managed checkout will also be removed. The remote repository is untouched."
        } else {
            "Only the registration will be removed. The folder and its files are untouched."
        };
        let content = v_flex()
            .id("delete-skill-source-dialog")
            .w(px(460.))
            .max_w(relative(0.9))
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().elevated_surface_background)
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(Label::new("Delete skill source?").size(UI_LABEL_LARGE))
            .child(Label::new(format!("Delete “{name}”? {detail}")))
            .when_some(self.form_error.clone(), |view, error| {
                view.child(notice_banner(error, true, cx))
            })
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("cancel-delete-skill-source", "Cancel")
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.dismiss(cx))),
                    )
                    .child(
                        Button::new("confirm-delete-skill-source", "Delete")
                            .style(ButtonStyle::Tinted(TintColor::Error))
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.submit(Operation::Remove(name.clone()), cx)
                            })),
                    ),
            );
        Some(self.overlay("delete-skill-source-scrim", content.into_any_element(), cx))
    }

    fn overlay(&self, id: &'static str, content: AnyElement, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id(id)
            .absolute()
            .top_0()
            .right_0()
            .bottom_0()
            .left_0()
            .flex()
            .items_start()
            .justify_center()
            .pt_8()
            .bg(gpui::black().opacity(0.35))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.dismiss(cx)))
            .child(content)
            .into_any_element()
    }
}

impl Render for SkillsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.sorter.is_dragging() && !cx.has_active_drag() {
            self.sorter.cancel();
        }
        if self.sorter.tick(cx.background_executor().now(), window.rem_size(), cx.reduce_motion()) {
            window.request_animation_frame();
        }
        let page = self.management(cx);
        div()
            .size_full()
            .relative()
            .key_context("ChartrSkillsPlugin")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && this.sorter.is_dragging() {
                    cx.stop_active_drag(window);
                    this.sorter.cancel();
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .on_drag_move::<DraggedSource>(cx.listener(
                |this, event: &gpui::DragMoveEvent<DraggedSource>, window, cx| {
                    let dragged = event.drag(cx);
                    if dragged.owner != cx.entity_id() || this.registry.read(cx).busy.is_some() {
                        return;
                    }
                    let name = dragged.name.clone();
                    let order = this
                        .registry
                        .read(cx)
                        .store
                        .sources
                        .iter()
                        .map(|source| source.name.clone())
                        .collect();
                    if this.sorter.drag_move(
                        name,
                        order,
                        event.event.position,
                        window.rem_size(),
                        cx.background_executor().now(),
                        cx.reduce_motion(),
                    ) {
                        cx.notify();
                    }
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    this.finish_drag(event.position.y, window, cx)
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    this.finish_drag(event.position.y, window, cx)
                }),
            )
            .child(page)
            .children(self.editor(cx))
            .children(self.deletion(cx))
    }
}

// The header and every sortable row use the same column widths.
fn source_columns(cells: Vec<AnyElement>) -> gpui::Div {
    h_flex().w_full().flex_none().items_start().children(
        cells
            .into_iter()
            .zip([0.52, 0.1, 0.12, 0.26])
            .map(|(cell, width)| div().w(relative(width)).min_w_0().flex_none().px_1().child(cell)),
    )
}

fn notice_banner(message: impl Into<SharedString>, error: bool, cx: &App) -> AnyElement {
    let (foreground, background) = if error {
        (Color::Error, cx.theme().status().error.opacity(0.1))
    } else {
        (Color::Muted, cx.theme().colors().element_background)
    };
    div()
        .w_full()
        .px_3()
        .py_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(background)
        .child(Label::new(message).size(UI_LABEL_DEFAULT).color(foreground))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, TestAppContext};

    fn init(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
    }

    #[gpui::test]
    fn provider_registries_are_shared_across_window_catalogs(cx: &mut TestAppContext) {
        let root = tempfile::tempdir().unwrap();
        let host = Host { data_dir: root.path().into(), plugin_dir: root.path().into() };
        cx.update(|cx| {
            let first = SkillsPlugin::new(host.clone(), cx);
            let second = SkillsPlugin::new(host, cx);
            assert_eq!(first.registry, second.registry);
        });
    }

    #[gpui::test]
    fn pane_menu_opens_host_settings_in_the_owning_window(cx: &mut TestAppContext) {
        init(cx);
        let opened = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let requests = opened.clone();
        let settings = PluginSettings::new(move |plugin, window, _| {
            requests.borrow_mut().push((plugin.map(str::to_owned), window.window_handle()));
        });
        let (_, cx) = cx.add_window_view(|_, _| SkillsPane { settings });
        cx.run_until_parked();
        let trigger = cx.debug_bounds("ICON-ChevronDown").unwrap();
        cx.simulate_click(trigger.center(), Modifiers::none());
        let popup = cx.windows().into_iter().find(|window| *window != cx.window_handle()).unwrap();
        let mut popup = gpui::VisualTestContext::from_window(popup, cx);
        popup.run_until_parked();
        let item = popup.debug_bounds("MENU_ITEM-Skill source settings").unwrap();
        popup.simulate_click(item.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(&*opened.borrow(), &[(Some(SkillsPlugin::ID.into()), cx.window_handle())]);
    }

    #[gpui::test]
    fn settings_editor_and_delete_are_explicit(cx: &mut TestAppContext) {
        init(cx);
        let temp = tempfile::tempdir().unwrap();
        let registry = cx.new(|_| Registry::load(temp.path().join("data")));
        let (view, cx) = cx.add_window_view(|_, cx| SkillsView::new(registry.clone(), cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open_editor(None, window, cx);
                view.name.update(cx, |input, cx| input.set_text("Local", false, cx));
                view.path.update(cx, |input, cx| {
                    input.set_text(temp.path().display().to_string(), false, cx)
                });
                view.save(window, cx);
            })
        });
        cx.run_until_parked();
        assert_eq!(registry.read_with(cx, |registry, _| registry.store.sources.len()), 1);
        assert!(!view.read_with(cx, |view, _| view.editor_open));
        let original = registry.read_with(cx, |registry, _| registry.store.sources[0].clone());
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open_editor(Some(original), window, cx);
                assert_eq!(view.name.read(cx).text(), "Local");
                assert_eq!(view.editing.as_deref(), Some("Local"));
                view.name.update(cx, |input, cx| input.set_text("Renamed", false, cx));
                view.save(window, cx);
            })
        });
        cx.run_until_parked();
        assert_eq!(
            registry.read_with(cx, |registry, _| registry.store.sources[0].name.clone()),
            "Renamed"
        );
        cx.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.deleting = Some("Renamed".into());
                cx.notify();
            })
        });
        cx.run_until_parked();
        assert_eq!(
            registry.read_with(cx, |registry, _| registry.store.sources.len()),
            1,
            "asking for confirmation must not delete"
        );
        cx.update(|_, cx| view.update(cx, |view, cx| view.dismiss(cx)));
        assert_eq!(registry.read_with(cx, |registry, _| registry.store.sources.len()), 1);
        cx.update(|_, cx| {
            view.update(cx, |view, cx| view.submit(Operation::Remove("Renamed".into()), cx))
        });
        cx.run_until_parked();
        assert!(registry.read_with(cx, |registry, _| registry.store.sources.is_empty()));
        assert!(temp.path().is_dir());
    }

    fn drag_registry(root: &std::path::Path) -> Vec<Source> {
        let sources: Vec<_> = ["First", "Second", "Third"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| Source {
                name: name.into(),
                path: root.to_owned(),
                enabled: true,
                kind: if index == 1 { Kind::Git } else { Kind::Local },
                url: "https://example.invalid/skills.git".into(),
                git_ref: "main".into(),
                commit: "0123456789ab".into(),
            })
            .collect();
        std::fs::write(root.join("sources.json"), serde_json::to_vec(&sources).unwrap()).unwrap();
        sources
    }

    fn move_first_to_last(cx: &mut gpui::VisualTestContext) -> gpui::Point<gpui::Pixels> {
        let first = cx.debug_bounds("SOURCE_SLOT-First").unwrap();
        let last = cx.debug_bounds("SOURCE_SLOT-Third").unwrap();
        let start = gpui::point(first.left() + first.size.width * 0.4, first.center().y);
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(
            start + gpui::point(px(0.), px(6.)),
            Some(MouseButton::Left),
            Modifiers::none(),
        );
        cx.simulate_mouse_move(
            start + gpui::point(px(0.), px(12.)),
            Some(MouseButton::Left),
            Modifiers::none(),
        );
        let end = gpui::point(start.x, last.bottom() - px(2.));
        cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        end
    }

    #[gpui::test]
    fn source_drag_reorders_live_and_only_persists_on_release(cx: &mut TestAppContext) {
        init(cx);
        let temp = tempfile::tempdir().unwrap();
        let sources = drag_registry(temp.path());
        let registry = cx.new(|_| Registry::load(temp.path().to_owned()));
        let (view, cx) = cx.add_window_view(|_, cx| SkillsView::new(registry.clone(), cx));
        cx.run_until_parked();
        let first = cx.debug_bounds("SOURCE_SLOT-First").unwrap();
        let second = cx.debug_bounds("SOURCE_SLOT-Second").unwrap();
        assert!(second.size.height > first.size.height, "remote rows have an extra line");
        let end = move_first_to_last(cx);
        let carried = cx.debug_bounds("SOURCE_ROW-First").unwrap();
        assert!(
            f32::from(carried.top() - (end.y - first.size.height * 0.5)).abs() < 1.,
            "the held row must stay under its original grab point"
        );
        view.read_with(cx, |view, _| {
            let mut order = vec!["First".to_owned(), "Second".into(), "Third".into()];
            view.sorter.arrange(&mut order, Clone::clone);
            assert_eq!(order, ["Second", "Third", "First"]);
            assert!(view.sorter.holds("First".into()));
        });
        assert!(
            cx.debug_bounds("SOURCE_SLOT-Second").unwrap().top()
                < cx.debug_bounds("SOURCE_SLOT-First").unwrap().top()
        );
        assert_eq!(registry.read_with(cx, |registry, _| registry.store.sources.clone()), sources);
        assert_eq!(Store::load(temp.path().to_owned()).unwrap().sources, sources);
        // As in the sidebar, release uses Y even beyond the list's right edge.
        let release = gpui::point(first.right() + px(30.), end.y);
        cx.simulate_mouse_up(release, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            Store::load(temp.path().to_owned())
                .unwrap()
                .sources
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["Second", "Third", "First"]
        );
        assert!(view.read_with(cx, |view, _| view.pending_order.is_none()
            && !view.sorter.holds("First".into())));
    }

    #[gpui::test]
    fn source_drag_cancellation_and_failed_save_restore_the_original_order(
        cx: &mut TestAppContext,
    ) {
        init(cx);
        let temp = tempfile::tempdir().unwrap();
        let sources = drag_registry(temp.path());
        let registry = cx.new(|_| Registry::load(temp.path().to_owned()));
        let (view, cx) = cx.add_window_view(|_, cx| SkillsView::new(registry.clone(), cx));
        cx.run_until_parked();
        move_first_to_last(cx);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert!(view.read_with(cx, |view, _| !view.sorter.holds("First".into())));
        assert_eq!(Store::load(temp.path().to_owned()).unwrap().sources, sources);
        let end = move_first_to_last(cx);
        std::fs::remove_file(temp.path().join("sources.json")).unwrap();
        std::fs::create_dir(temp.path().join("sources.json")).unwrap();
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
        assert_eq!(registry.read_with(cx, |registry, _| registry.store.sources.clone()), sources);
        assert!(registry.read_with(cx, |registry, _| registry.problem.is_some()));
        assert!(view.read_with(cx, |view, _| view.pending_order.is_none()
            && !view.sorter.holds("First".into())));
        assert!(
            cx.debug_bounds("SOURCE_SLOT-First").unwrap().top()
                < cx.debug_bounds("SOURCE_SLOT-Second").unwrap().top()
        );
    }

    #[gpui::test]
    fn registry_serializes_mutations_across_views_and_preserves_corrupt_storage(
        cx: &mut TestAppContext,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("sources.json");
        std::fs::write(&file, "invalid").unwrap();
        let registry = cx.new(|_| Registry::load(temp.path().to_owned()));
        cx.update(|cx| {
            registry.update(cx, |registry, cx| {
                assert!(registry.run(Operation::Remove("anything".into()), cx).is_none());
                let task = registry.run(Operation::Scan, cx).unwrap();
                assert!(registry.run(Operation::Scan, cx).is_none());
                task.detach();
            })
        });
        cx.run_until_parked();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "invalid");
        assert!(registry.read_with(cx, |registry, _| registry.load_failed));
        std::fs::write(&file, "[]").unwrap();
        cx.update(|cx| {
            registry.update(cx, |registry, cx| registry.run(Operation::Scan, cx).unwrap().detach())
        });
        cx.run_until_parked();
        assert!(!registry.read_with(cx, |registry, _| registry.load_failed));
    }
}
