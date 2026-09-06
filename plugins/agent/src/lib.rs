//! Chartr's bundled native agent launcher.
//!
//! The pane is an ordinary GPUI view. Agent definitions live in the plugin's
//! private data directory, while terminal creation is delegated to the owning
//! Chartr space through the native plugin instance context.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use ui::{
    Button, ButtonStyle, Color, ColumnWidthConfig, Icon, IconButton, IconName, IconPosition,
    IconSize, Label, Table, TintColor, prelude::*,
};
use zeddy_plugin::{
    Host, InstanceContext, PaneKey, Plugin, PluginObject, Registrar, TerminalLauncher, gpui,
    gpui::{
        Anchor, AnyElement, App, Context, Entity, Focusable, IntoElement, MouseButton, Render,
        SharedString, Window, div, px, relative,
    },
};

use crate::{
    components::{ContextMenu, PopupMenu, form_picker, form_row, input_field},
    fonts::{UI_LABEL_DEFAULT, UI_LABEL_LARGE, UI_LABEL_SMALL},
    text_input::TextInput,
};

const STORAGE_FILE: &str = "agents.json";

pub struct AgentPlugin {
    registry: Entity<AgentRegistry>,
}

#[derive(Default)]
struct SharedRegistries(std::collections::HashMap<PathBuf, gpui::WeakEntity<AgentRegistry>>);
impl gpui::Global for SharedRegistries {}

/// Construct the plugin object linked into Chartr.
pub fn bundled(host: Host, cx: &mut App) -> Box<dyn PluginObject> {
    Box::new(AgentPlugin::new(host, cx))
}

impl Plugin for AgentPlugin {
    const ID: &'static str = "com.chartr.agent";

    fn new(host: Host, cx: &mut App) -> Self {
        let path = host.data_dir.join(STORAGE_FILE);
        let existing = cx
            .default_global::<SharedRegistries>()
            .0
            .get(&path)
            .and_then(gpui::WeakEntity::upgrade);
        let registry = existing.unwrap_or_else(|| {
            let registry = cx.new(|_| AgentRegistry::load(path.clone()));
            cx.default_global::<SharedRegistries>().0.insert(path, registry.downgrade());
            registry
        });
        Self { registry }
    }

    fn activate(&mut self, registrar: &mut Registrar, _: &mut App) {
        registrar.add_pane("main", "Agent").add_settings();
    }

    fn services(&self) -> Vec<zeddy_plugin::services::ServiceExport> {
        use zeddy_plugin::services::{Agents, ServiceExport};
        let listing = self.registry.downgrade();
        let preparing = listing.clone();
        vec![ServiceExport::new(Agents::new(
            move |cx| {
                let registry = listing.upgrade().ok_or("Agent is unavailable.")?;
                let registry = registry.read(cx);
                if let Some(error) = &registry.problem {
                    return Err(error.clone());
                }
                Ok(registry.agents.iter().map(|agent| agent.name.clone()).collect())
            },
            move |name, prompt, cx| {
                let registry = preparing.upgrade().ok_or("Agent is unavailable.")?;
                let registry = registry.read(cx);
                if let Some(error) = &registry.problem {
                    return Err(error.clone());
                }
                let agent = registry
                    .agents
                    .iter()
                    .find(|agent| agent.name == name)
                    .ok_or("The selected agent is no longer registered.")?;
                opening_input(agent, prompt)
            },
        ))]
    }

    fn settings(&mut self, _: &mut Window, cx: &mut App) -> Option<gpui::AnyView> {
        let context = InstanceContext {
            instance_id: 0,
            space: String::new(),
            space_name: String::new(),
            project_dir: None,
            bound_session: None,
            terminal: TerminalLauncher::new(|_, _| {}),
            services: Default::default(),
            plugin_settings: zeddy_plugin::services::PluginSettings::new(|_, _, _| {}),
        };
        Some(
            cx.new(|cx| {
                let mut view = AgentView::new(self.registry.clone(), context, cx);
                view.settings_only = true;
                view
            })
            .into(),
        )
    }

    fn view(
        &mut self,
        _: &PaneKey,
        context: &InstanceContext,
        _: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyView {
        let registry = self.registry.clone();
        let context = context.clone();
        cx.new(|cx| AgentView::new(registry, context, cx)).into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AgentRecord {
    name: String,
    adapter: String,
    args: Vec<String>,
    #[serde(default)]
    env: Vec<String>,
    #[serde(default = "default_delivery")]
    delivery: String,
}

fn default_delivery() -> String {
    "default".to_owned()
}

#[derive(Serialize)]
struct RegistryFile<'a> {
    version: u32,
    agents: &'a [AgentRecord],
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StoredRegistry {
    Versioned {
        #[allow(dead_code)]
        version: Option<u32>,
        agents: Vec<AgentRecord>,
    },
    Legacy(Vec<AgentRecord>),
}

struct AgentRegistry {
    path: PathBuf,
    agents: Vec<AgentRecord>,
    problem: Option<String>,
}

impl AgentRegistry {
    fn load(path: PathBuf) -> Self {
        let loaded = std::fs::read_to_string(&path);
        match loaded {
            Ok(encoded) => match serde_json::from_str::<StoredRegistry>(&encoded) {
                Ok(StoredRegistry::Versioned { agents, .. })
                | Ok(StoredRegistry::Legacy(agents)) => {
                    let agents = agents.into_iter().filter(valid_stored_agent).collect();
                    Self { path, agents, problem: None }
                }
                Err(error) => Self {
                    path,
                    agents: Vec::new(),
                    problem: Some(format!("Could not read registered agents: {error}")),
                },
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Self { path, agents: Vec::new(), problem: None }
            }
            Err(error) => Self {
                path,
                agents: Vec::new(),
                problem: Some(format!("Could not read registered agents: {error}")),
            },
        }
    }

    fn replace(&mut self, agents: Vec<AgentRecord>, cx: &mut Context<Self>) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("creating the Agent data directory: {error}"))?;
        }
        let encoded = serde_json::to_string_pretty(&RegistryFile { version: 1, agents: &agents })
            .map_err(|error| format!("encoding registered agents: {error}"))?;
        std::fs::write(&self.path, format!("{encoded}\n"))
            .map_err(|error| format!("saving registered agents: {error}"))?;
        self.agents = agents;
        self.problem = None;
        cx.notify();
        Ok(())
    }
}

fn valid_stored_agent(agent: &AgentRecord) -> bool {
    valid_agent_name(&agent.name)
        && !agent.adapter.trim().is_empty()
        && agent.env.iter().all(|entry| environment_entry(entry).is_ok())
        && resolved_delivery(&agent.adapter, &agent.delivery).is_ok()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DeliveryMode {
    #[default]
    Default,
    Argument,
    Typed,
    Flag,
}

impl DeliveryMode {
    const ALL: [Self; 4] = [Self::Default, Self::Argument, Self::Typed, Self::Flag];

    fn from_stored(value: &str) -> Self {
        match value {
            "argv" => Self::Argument,
            "type" => Self::Typed,
            value if value.starts_with('-') => Self::Flag,
            _ => Self::Default,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Default => "The adapter's default",
            Self::Argument => "Trailing argument (argv)",
            Self::Typed => "Type into the TUI",
            Self::Flag => "A named flag",
        }
    }
}

struct AgentView {
    settings_only: bool,
    registry: Entity<AgentRegistry>,
    terminal: TerminalLauncher,
    space_name: String,
    branch: String,
    plugin_settings: zeddy_plugin::services::PluginSettings,
    selected: String,
    prompt: Entity<TextInput>,
    name: Entity<TextInput>,
    adapter: Entity<TextInput>,
    args: Entity<TextInput>,
    env: Entity<TextInput>,
    prompt_flag: Entity<TextInput>,
    delivery: DeliveryMode,
    editor_open: bool,
    editing: Option<String>,
    deleting: Option<String>,
    form_error: Option<String>,
    notice: Option<(String, bool)>,
}

impl AgentView {
    fn new(
        registry: Entity<AgentRegistry>,
        context: InstanceContext,
        cx: &mut Context<Self>,
    ) -> Self {
        let selected =
            registry.read(cx).agents.first().map(|agent| agent.name.clone()).unwrap_or_default();
        let branch = context
            .project_dir
            .as_deref()
            .and_then(git_branch)
            .unwrap_or_else(|| "No Git branch".to_owned());
        let prompt = cx.new(|cx| TextInput::new("Do anything", cx));
        let name = cx.new(|cx| TextInput::new("codex-sol-xhigh", cx));
        let adapter = cx.new(|cx| TextInput::new("codex", cx));
        let args = cx.new(|cx| TextInput::new("--model gpt-5 --sandbox workspace-write", cx));
        let env = cx.new(|cx| TextInput::new("AGENT_CONFIG_DIR=~/.agent-work", cx));
        let prompt_flag = cx.new(|cx| TextInput::new("--prompt", cx));
        cx.observe(&registry, |this, registry, cx| {
            let agents = &registry.read(cx).agents;
            if !agents.iter().any(|agent| agent.name == this.selected) {
                this.selected = agents.first().map(|agent| agent.name.clone()).unwrap_or_default();
            }
            cx.notify();
        })
        .detach();
        Self {
            registry,
            settings_only: false,
            terminal: context.terminal,
            space_name: context.space_name,
            branch,
            plugin_settings: context.plugin_settings,
            selected,
            prompt,
            name,
            adapter,
            args,
            env,
            prompt_flag,
            delivery: DeliveryMode::Default,
            editor_open: false,
            editing: None,
            deleting: None,
            form_error: None,
            notice: None,
        }
    }

    fn open_settings(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.plugin_settings.open(Some(AgentPlugin::ID), window, cx);
    }

    fn open_editor(
        &mut self,
        agent: Option<AgentRecord>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing = agent.as_ref().map(|agent| agent.name.clone());
        self.delivery = agent
            .as_ref()
            .map(|agent| DeliveryMode::from_stored(&agent.delivery))
            .unwrap_or_default();
        self.name.update(cx, |input, cx| {
            input.set_text(agent.as_ref().map(|agent| agent.name.as_str()).unwrap_or(""), false, cx)
        });
        self.adapter.update(cx, |input, cx| {
            input.set_text(
                agent.as_ref().map(|agent| agent.adapter.as_str()).unwrap_or(""),
                false,
                cx,
            )
        });
        self.args.update(cx, |input, cx| {
            input.set_text(
                agent.as_ref().map(|agent| format_args(&agent.args)).unwrap_or_default(),
                false,
                cx,
            )
        });
        self.env.update(cx, |input, cx| {
            input.set_text(
                agent.as_ref().map(|agent| format_args(&agent.env)).unwrap_or_default(),
                false,
                cx,
            )
        });
        self.prompt_flag.update(cx, |input, cx| {
            input.set_text(
                agent
                    .as_ref()
                    .filter(|agent| agent.delivery.starts_with('-'))
                    .map(|agent| agent.delivery.as_str())
                    .unwrap_or(""),
                false,
                cx,
            )
        });
        self.editor_open = true;
        self.form_error = None;
        window.focus(&self.name.focus_handle(cx), cx);
        cx.notify();
    }

    fn edit_named(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let agent = self.registry.read(cx).agents.iter().find(|agent| agent.name == name).cloned();
        if let Some(agent) = agent {
            self.open_editor(Some(agent), window, cx);
        }
    }

    fn close_editor(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dismiss_editor(cx);
    }

    fn dismiss_editor(&mut self, cx: &mut Context<Self>) {
        self.editor_open = false;
        self.editing = None;
        self.form_error = None;
        cx.notify();
    }

    fn save_editor(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).text().trim().to_owned();
        let adapter = self.adapter.read(cx).text().trim().to_owned();
        let args_text = self.args.read(cx).text().trim().to_owned();
        let env_text = self.env.read(cx).text().trim().to_owned();
        let prompt_flag = self.prompt_flag.read(cx).text().trim().to_owned();
        let args = parse_args(&args_text);
        let env = parse_args(&env_text);

        let error =
            if !valid_agent_name(&name) {
                Some("Name may contain only letters, numbers, hyphens, and underscores.".to_owned())
            } else if adapter.is_empty() {
                Some("Adapter is required.".to_owned())
            } else if let Some(invalid) = env.iter().find(|entry| environment_entry(entry).is_err())
            {
                Some(format!("Environment entry “{invalid}” must be KEY=VALUE."))
            } else if self.delivery == DeliveryMode::Flag
                && (!prompt_flag.starts_with('-')
                    || prompt_flag.chars().any(char::is_whitespace)
                    || prompt_flag.len() == 1)
            {
                Some("Prompt flag must start with a hyphen and contain no spaces.".to_owned())
            } else if self.registry.read(cx).agents.iter().any(|agent| {
                agent.name == name && Some(agent.name.as_str()) != self.editing.as_deref()
            }) {
                Some(format!("An agent named “{name}” is already registered."))
            } else {
                None
            };
        if let Some(error) = error {
            self.form_error = Some(error);
            cx.notify();
            return;
        }

        let delivery = match self.delivery {
            DeliveryMode::Default => "default".to_owned(),
            DeliveryMode::Argument => "argv".to_owned(),
            DeliveryMode::Typed => "type".to_owned(),
            DeliveryMode::Flag => prompt_flag,
        };
        let record = AgentRecord { name: name.clone(), adapter, args, env, delivery };
        let mut agents = self.registry.read(cx).agents.clone();
        if let Some(index) =
            agents.iter().position(|agent| Some(agent.name.as_str()) == self.editing.as_deref())
        {
            agents[index] = record;
        } else {
            agents.push(record);
        }
        agents.sort_by(|left, right| left.name.cmp(&right.name));
        match self.registry.update(cx, |registry, cx| registry.replace(agents, cx)) {
            Ok(()) => {
                let updated = self.editing.is_some();
                self.selected = name;
                self.editor_open = false;
                self.editing = None;
                self.form_error = None;
                self.notice = Some((
                    if updated { "Agent updated." } else { "Agent registered." }.to_owned(),
                    false,
                ));
            }
            Err(error) => self.form_error = Some(format!("Could not save the agent: {error}")),
        }
        cx.notify();
    }

    fn ask_delete(&mut self, name: String, cx: &mut Context<Self>) {
        self.deleting = Some(name);
        self.notice = None;
        cx.notify();
    }

    fn cancel_delete(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.deleting = None;
        cx.notify();
    }

    fn confirm_delete(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(name) = self.deleting.clone() else {
            return;
        };
        let agents = self
            .registry
            .read(cx)
            .agents
            .iter()
            .filter(|agent| agent.name != name)
            .cloned()
            .collect();
        match self.registry.update(cx, |registry, cx| registry.replace(agents, cx)) {
            Ok(()) => {
                self.deleting = None;
                self.notice = Some(("Agent deleted.".to_owned(), false));
            }
            Err(error) => {
                self.deleting = None;
                self.notice = Some((format!("Could not delete the agent: {error}"), true));
            }
        }
        cx.notify();
    }

    fn launch(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let agent =
            self.registry.read(cx).agents.iter().find(|agent| agent.name == self.selected).cloned();
        let Some(agent) = agent else {
            return;
        };
        let prompt = self.prompt.read(cx).text().to_owned();
        match opening_input(&agent, &prompt) {
            Ok(input) => {
                self.terminal.launch(input, cx);
                self.prompt.update(cx, |input, cx| input.clear(cx));
                self.notice = None;
            }
            Err(error) => {
                self.notice = Some((format!("Could not launch the session: {error}"), true))
            }
        }
        cx.notify();
    }

    fn launcher(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let agents = self.registry.read(cx).agents.clone();
        let has_agents = !agents.is_empty();
        let register = cx.listener(|this, _, window, cx| this.open_settings(window, cx));
        let launch = cx.listener(Self::launch);
        let weak = cx.weak_entity();
        let pane_menu = PopupMenu::new("agent-pane-menu")
            .trigger(
                IconButton::new("agent-pane-menu-trigger", IconName::ChevronDown)
                    .icon_size(IconSize::Small)
                    .aria_label("Agent pane menu"),
            )
            .anchor(Anchor::TopRight)
            .menu(move |window, cx| {
                let weak = weak.clone();
                Some(ContextMenu::build_popup(window, cx, move |menu| {
                    menu.entry("Manage agents", None, move |window, cx| {
                        let _ = weak.update(cx, |this, cx| this.open_settings(window, cx));
                    })
                }))
            });

        let picker: AnyElement = if has_agents {
            let selected = self.selected.clone();
            let selected_icon = agents
                .iter()
                .find(|agent| agent.name == selected)
                .map(|agent| inferred_agent_icon(&agent.name, &agent.adapter))
                .unwrap_or(GENERIC_AGENT_ICON);
            let options: Vec<_> = agents
                .iter()
                .map(|agent| {
                    (
                        agent.name.clone(),
                        inferred_agent_icon(&agent.name, &agent.adapter),
                        agent.name == selected,
                    )
                })
                .collect();
            let weak = cx.weak_entity();
            PopupMenu::new("registered-agent-picker")
                .trigger(
                    Button::new("registered-agent-picker-trigger", selected)
                        .style(ButtonStyle::Outlined)
                        .start_icon(Icon::from_path(selected_icon).size(IconSize::Small))
                        .end_icon(Icon::new(IconName::ChevronDown).size(IconSize::XSmall)),
                )
                .anchor(Anchor::TopLeft)
                .menu(move |window, cx| {
                    let weak = weak.clone();
                    let options = options.clone();
                    Some(ContextMenu::build_popup(window, cx, move |mut menu| {
                        for (name, icon, selected) in options {
                            let choose = weak.clone();
                            let value = name.clone();
                            menu = menu.toggleable_entry_with_icon_path(
                                name,
                                icon,
                                selected,
                                move |_, cx| {
                                    let _ = choose.update(cx, |this, cx| {
                                        this.selected = value.clone();
                                        cx.notify();
                                    });
                                },
                            );
                        }
                        menu
                    }))
                })
                .into_any_element()
        } else {
            Button::new("registered-agent-picker-empty", "No registered agents")
                .style(ButtonStyle::Outlined)
                .disabled(true)
                .into_any_element()
        };

        let registry_problem = self.registry.read(cx).problem.clone();
        let notice = self.notice.clone();
        let context = h_flex()
            .w_full()
            .min_h(px(24.))
            .px_1()
            .gap_5()
            .child(context_item(IconName::FolderOpen, self.space_name.clone()))
            .child(context_item(IconName::GitBranch, self.branch.clone()));
        let composer = v_flex()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .overflow_hidden()
            .child(
                h_flex()
                    .h(px(48.))
                    .px_3()
                    .track_focus(&self.prompt.focus_handle(cx))
                    .child(self.prompt.clone()),
            )
            .child(
                h_flex().w_full().justify_between().gap_3().px_3().pb_3().child(picker).child(
                    Button::new("launch-agent-session", "Launch session")
                        .style(ButtonStyle::Filled)
                        .disabled(!has_agents)
                        .on_click(launch),
                ),
            );

        v_flex()
            .id("agent-launcher-page")
            .size_full()
            .relative()
            .items_center()
            .bg(cx.theme().colors().editor_background)
            .child(div().absolute().top_3().right_3().child(pane_menu))
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(780.))
                    .h_full()
                    .px_6()
                    .pt_8()
                    .pb_6()
                    .gap_5()
                    .child(
                        v_flex()
                            .flex_1()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .pb_2()
                            .child(
                                Label::new(if has_agents {
                                    "Launch a new session in this space with the selected agent"
                                } else {
                                    "Welcome! Let's get you started."
                                })
                                .size(UI_LABEL_LARGE)
                                .color(Color::Muted),
                            )
                            .when(!has_agents, |hero| {
                                hero.child(
                                    div().debug_selector(|| "REGISTER_FIRST_AGENT".into()).child(
                                        Button::new(
                                            "register-first-agent",
                                            "Register your first agent",
                                        )
                                        .style(ButtonStyle::Outlined)
                                        .start_icon(Icon::new(IconName::Plus))
                                        .on_click(register),
                                    ),
                                )
                            }),
                    )
                    .when_some(registry_problem, |page, problem| {
                        page.child(notice_banner(problem, true, cx))
                    })
                    .when_some(notice, |page, (message, error)| {
                        page.child(notice_banner(message, error, cx))
                    })
                    .child(v_flex().w_full().gap_2().child(context).child(composer)),
            )
            .into_any_element()
    }

    fn management(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let agents = self.registry.read(cx).agents.clone();
        let add = cx.listener(|this, _, window, cx| this.open_editor(None, window, cx));
        let table = agents
            .iter()
            .enumerate()
            .fold(
                Table::new(3)
                    .striped()
                    .width_config(ColumnWidthConfig::explicit(vec![
                        relative(0.34),
                        relative(0.51),
                        relative(0.15),
                    ]))
                    .header(vec!["Name", "Adapter", ""]),
                |table, (index, agent)| {
                    let name = agent.name.clone();
                    let edit_name = name.clone();
                    let edit = cx.listener(move |this, _, window, cx| {
                        this.edit_named(&edit_name, window, cx)
                    });
                    let delete_name = name.clone();
                    let delete =
                        cx.listener(move |this, _, _, cx| this.ask_delete(delete_name.clone(), cx));
                    table.row(vec![
                        Label::new(name).truncate().into_any_element(),
                        Label::new(agent.adapter.clone())
                            .color(Color::Muted)
                            .truncate()
                            .into_any_element(),
                        h_flex()
                            .w_full()
                            .justify_end()
                            .gap_1()
                            .child(Button::new(("edit-agent", index), "Edit").on_click(edit))
                            .child(
                                IconButton::new(("delete-agent", index), IconName::Trash)
                                    .icon_size(IconSize::Small)
                                    .aria_label(format!("Delete {}", agent.name))
                                    .on_click(delete),
                            )
                            .into_any_element(),
                    ])
                },
            )
            .empty_table_callback(|_, _| {
                Label::new("No registered agents.").color(Color::Muted).into_any_element()
            });
        let problem = self.registry.read(cx).problem.clone();
        let notice = self.notice.clone();

        v_flex()
            .id("agent-management-page")
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .overflow_y_scroll()
            .child(
                v_flex()
                    .w_full()
                    .gap_5()
                    .child(
                        v_flex()
                            .w_full()
                            .gap_2()
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .gap_4()
                                    .child(Label::new("Agent management").size(UI_LABEL_LARGE))
                                    .child(
                                        Button::new("new-agent", "New agent")
                                            .style(ButtonStyle::Outlined)
                                            .start_icon(Icon::new(IconName::Plus))
                                            .on_click(add),
                                    ),
                            )
                            .child(
                                div().w_full().child(
                                    Label::new(
                                        "Configure the command, flags, and prompt delivery for each agent.",
                                    )
                                    .size(UI_LABEL_DEFAULT)
                                    .color(Color::Muted),
                                ),
                            ),
                    )
                    .when_some(problem, |page, problem| {
                        page.child(notice_banner(problem, true, cx))
                    })
                    .when_some(notice, |page, (message, error)| {
                        page.child(notice_banner(message, error, cx))
                    })
                    .child(table),
            )
            .into_any_element()
    }

    fn editor_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.editor_open {
            return None;
        }
        let title = if self.editing.is_some() { "Edit agent" } else { "Register new agent" };
        let close_scrim = cx.weak_entity();
        let close_button = cx.listener(Self::close_editor);
        let cancel = cx.listener(Self::close_editor);
        let save = cx.listener(Self::save_editor);
        let delivery = self.delivery;
        let weak = cx.weak_entity();
        let delivery_picker = PopupMenu::new("agent-delivery-picker")
            .trigger(form_picker("agent-delivery-trigger", delivery.label()))
            .anchor(Anchor::TopLeft)
            .menu(move |window, cx| {
                let weak = weak.clone();
                Some(ContextMenu::build_popup(window, cx, move |mut menu| {
                    for option in DeliveryMode::ALL {
                        let select = weak.clone();
                        menu = menu.toggleable_entry(
                            option.label(),
                            option == delivery,
                            IconPosition::End,
                            None,
                            move |_, cx| {
                                let _ = select.update(cx, |this, cx| {
                                    this.delivery = option;
                                    this.form_error = None;
                                    cx.notify();
                                });
                            },
                        );
                    }
                    menu
                }))
            })
            .into_any_element();
        let error = self.form_error.clone();

        Some(
            div()
                .id("agent-editor-scrim")
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
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let _ = close_scrim.update(cx, |this, cx| this.dismiss_editor(cx));
                })
                .child(
                    v_flex()
                        .id("agent-editor-dialog")
                        .w(px(620.))
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
                                .child(Label::new(title).size(UI_LABEL_LARGE))
                                .child(
                                    IconButton::new("close-agent-editor", IconName::Close)
                                        .aria_label("Close dialog")
                                        .on_click(close_button),
                                ),
                        )
                        .child(
                            v_flex()
                                .id("agent-editor-fields")
                                .w_full()
                                .p_4()
                                .gap_3()
                                .overflow_y_scroll()
                                .child(form_row(
                                    "Name",
                                    None,
                                    input_field("agent-name", self.name.clone(), cx),
                                ))
                                .child(form_row(
                                    "Adapter",
                                    Some("For example: claude, codex, opencode, or any agent CLI on your PATH."),
                                    input_field("agent-adapter", self.adapter.clone(), cx),
                                ))
                                .child(form_row("Prompt", None, delivery_picker))
                                .when(delivery == DeliveryMode::Flag, |form| {
                                    form.child(form_row(
                                        "Prompt flag",
                                        None,
                                        input_field("agent-prompt-flag", self.prompt_flag.clone(), cx),
                                    ))
                                })
                                .child(form_row(
                                    "Args",
                                    Some("Optional. Quotes group an argument; values are passed through without shell expansion."),
                                    input_field("agent-args", self.args.clone(), cx),
                                ))
                                .child(form_row(
                                    "Environment",
                                    Some("Optional KEY=VALUE entries, set before the adapter runs."),
                                    input_field("agent-environment", self.env.clone(), cx),
                                ))
                                .when_some(error, |form, error| {
                                    form.child(notice_banner(error, true, cx))
                                }),
                        )
                        .child(
                            h_flex()
                                .w_full()
                                .justify_end()
                                .gap_2()
                                .px_4()
                                .py_3()
                                .border_t_1()
                                .border_color(cx.theme().colors().border)
                                .child(Button::new("cancel-agent-editor", "Cancel").on_click(cancel))
                                .child(
                                    Button::new("save-agent", "Save")
                                        .style(ButtonStyle::Filled)
                                        .on_click(save),
                                ),
                        ),
                )
                .into_any_element(),
        )
    }

    fn delete_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let name = self.deleting.clone()?;
        let cancel_scrim = cx.weak_entity();
        let cancel = cx.listener(Self::cancel_delete);
        let confirm = cx.listener(Self::confirm_delete);
        Some(
            div()
                .id("delete-agent-scrim")
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .flex()
                .items_start()
                .justify_center()
                .pt(px(96.))
                .bg(gpui::black().opacity(0.35))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let _ = cancel_scrim.update(cx, |this, cx| {
                        this.deleting = None;
                        cx.notify();
                    });
                })
                .child(
                    v_flex()
                        .id("delete-agent-dialog")
                        .w(px(440.))
                        .max_w(relative(0.9))
                        .p_4()
                        .gap_3()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().colors().border)
                        .bg(cx.theme().colors().elevated_surface_background)
                        .shadow_lg()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(Label::new("Delete agent?").size(UI_LABEL_LARGE))
                        .child(
                            Label::new(format!(
                                "Delete “{name}”? This does not close sessions already running with it."
                            ))
                            .size(UI_LABEL_DEFAULT),
                        )
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_2()
                                .child(Button::new("cancel-agent-delete", "Cancel").on_click(cancel))
                                .child(
                                    Button::new("confirm-agent-delete", "Delete")
                                        .style(ButtonStyle::Tinted(TintColor::Error))
                                        .on_click(confirm),
                                ),
                        ),
                )
                .into_any_element(),
        )
    }
}

impl Render for AgentView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = if self.settings_only { self.management(cx) } else { self.launcher(cx) };
        div()
            .size_full()
            .relative()
            .key_context("ChartrAgentPlugin")
            .child(page)
            .children(self.editor_overlay(cx))
            .children(self.delete_overlay(cx))
    }
}

fn context_item(icon: IconName, text: impl Into<SharedString>) -> AnyElement {
    h_flex()
        .min_w_0()
        .gap_1()
        .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(Label::new(text).size(UI_LABEL_SMALL).color(Color::Muted).truncate())
        .into_any_element()
}

const GENERIC_AGENT_ICON: &str = "icons/agent_ai_programming.svg";

/// Pick one of the bundled Hugeicons glyphs from a registered display name,
/// falling back to the executable when the name is intentionally generic.
fn inferred_agent_icon(name: &str, adapter: &str) -> &'static str {
    known_agent_icon(name).or_else(|| known_agent_icon(adapter)).unwrap_or(GENERIC_AGENT_ICON)
}

fn known_agent_icon(value: &str) -> Option<&'static str> {
    let value = value.to_ascii_lowercase();
    let terms: Vec<_> = value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .collect();
    let has = |aliases: &[&str]| terms.iter().any(|term| aliases.contains(term));
    let compact: String = value.chars().filter(char::is_ascii_alphanumeric).collect();

    if has(&["claude", "anthropic"]) {
        Some("icons/agent_claude.svg")
    } else if has(&["codex", "openai", "chatgpt"]) {
        Some("icons/agent_chat_gpt.svg")
    } else if has(&["grok", "xai"]) {
        Some("icons/agent_grok.svg")
    } else if compact.contains("opencode") {
        Some(GENERIC_AGENT_ICON)
    } else if compact.contains("deepseek") {
        Some("icons/agent_deepseek.svg")
    } else if has(&["gemini"]) {
        Some("icons/agent_google_gemini.svg")
    } else if has(&["mistral"]) {
        Some("icons/agent_mistral.svg")
    } else if has(&["qwen"]) {
        Some("icons/agent_qwen.svg")
    } else if has(&["copilot"]) {
        Some("icons/agent_copilot.svg")
    } else if has(&["pi"]) {
        Some("icons/agent_pi.svg")
    } else {
        None
    }
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

fn valid_agent_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

// Shell-like field editing without shell interpretation: whitespace splits,
// quotes group, and only quote/backslash escapes inside double quotes.
fn parse_args(text: &str) -> Vec<String> {
    let mut output = Vec::new();
    let mut current = String::new();
    let mut started = false;
    let mut quote = None;
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if let Some(active) = quote {
            if active == '"' && character == '\\' && matches!(characters.peek(), Some('"' | '\\')) {
                current.push(characters.next().expect("peeked character"));
            } else if character == active {
                quote = None;
            } else {
                current.push(character);
            }
        } else if matches!(character, '"' | '\'') {
            quote = Some(character);
            started = true;
        } else if character.is_whitespace() {
            if started {
                output.push(std::mem::take(&mut current));
            }
            started = false;
        } else {
            current.push(character);
            started = true;
        }
    }
    if started {
        output.push(current);
    }
    output
}

fn format_args(arguments: &[String]) -> String {
    arguments
        .iter()
        .map(|argument| {
            if !argument.is_empty()
                && !argument.chars().any(|character| {
                    character.is_whitespace() || matches!(character, '"' | '\'' | '\\')
                })
            {
                argument.clone()
            } else {
                format!("\"{}\"", argument.replace('\\', "\\\\").replace('"', "\\\""))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Turn one registered-agent launch into exactly what the new shell receives.
/// Every word is quoted independently so fields stay data rather than shell
/// fragments.
fn opening_input(agent: &AgentRecord, prompt: &str) -> Result<Vec<u8>, String> {
    let program = agent.adapter.trim();
    if program.is_empty() {
        return Err("an agent launch needs an adapter".to_owned());
    }
    reject_nul("adapter", program)?;
    for argument in &agent.args {
        reject_nul("argument", argument)?;
    }
    reject_nul("prompt", prompt)?;

    let delivery = resolved_delivery(program, &agent.delivery)?;
    let mut line = String::new();
    for entry in &agent.env {
        let (name, value) = environment_entry(entry)?;
        line.push_str(name);
        line.push('=');
        line.push_str(&shell_quoted(&expand_home(value)));
        line.push(' ');
    }
    line.push_str(&shell_quoted(program));

    if let PromptDelivery::Flag(flag) = &delivery
        && !prompt.is_empty()
    {
        line.push(' ');
        line.push_str(&shell_quoted(flag));
        line.push(' ');
        line.push_str(&shell_quoted(prompt));
    }
    for argument in &agent.args {
        line.push(' ');
        line.push_str(&shell_quoted(argument));
    }
    if delivery == PromptDelivery::Argument && !prompt.is_empty() {
        line.push(' ');
        line.push_str(&shell_quoted(prompt));
    }

    let mut input = line.into_bytes();
    input.push(b'\r');
    if delivery == PromptDelivery::Typed && !prompt.is_empty() {
        input.extend_from_slice(prompt.as_bytes());
        input.push(b'\r');
    }
    Ok(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PromptDelivery {
    Argument,
    Typed,
    Flag(String),
}

fn resolved_delivery(program: &str, configured: &str) -> Result<PromptDelivery, String> {
    let configured = configured.trim();
    match configured {
        "" | "default" => {
            let name =
                Path::new(program).file_name().and_then(|name| name.to_str()).unwrap_or(program);
            Ok(if matches!(name, "claude" | "codex") {
                PromptDelivery::Argument
            } else {
                PromptDelivery::Typed
            })
        }
        "argv" => Ok(PromptDelivery::Argument),
        "type" => Ok(PromptDelivery::Typed),
        flag if flag.starts_with('-')
            && flag.len() > 1
            && !flag.chars().any(char::is_whitespace) =>
        {
            Ok(PromptDelivery::Flag(flag.to_owned()))
        }
        _ => {
            Err("prompt delivery must be default, argv, type, or a flag such as --prompt"
                .to_owned())
        }
    }
}

fn environment_entry(entry: &str) -> Result<(&str, &str), String> {
    reject_nul("environment entry", entry)?;
    let Some((name, value)) = entry.split_once('=') else {
        return Err(format!("environment entry `{entry}` must be KEY=VALUE"));
    };
    let mut characters = name.chars();
    if !characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        || !characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
    {
        return Err(format!("`{name}` is not a valid environment variable name"));
    }
    Ok((name, value))
}

fn reject_nul(label: &str, value: &str) -> Result<(), String> {
    if value.contains('\0') { Err(format!("the {label} contains a NUL byte")) } else { Ok(()) }
}

fn expand_home(value: &str) -> Cow<'_, str> {
    let Some(rest) = value.strip_prefix("~/") else {
        return Cow::Borrowed(value);
    };
    let Some(home) = std::env::var_os("HOME") else {
        return Cow::Borrowed(value);
    };
    Cow::Owned(PathBuf::from(home).join(rest).to_string_lossy().into_owned())
}

fn shell_quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// Read the current branch from Git's HEAD. Linked worktrees use a `.git`
/// pointer file, so resolve that form as well as the ordinary directory.
fn git_branch(project: &Path) -> Option<String> {
    let mut looking = Some(project);
    while let Some(folder) = looking {
        let dot_git = folder.join(".git");
        let directory = if dot_git.is_dir() {
            Some(dot_git)
        } else if dot_git.is_file() {
            let pointer = std::fs::read_to_string(&dot_git).ok()?;
            let named =
                pointer.lines().find_map(|line| line.strip_prefix("gitdir:").map(str::trim))?;
            let named = PathBuf::from(named);
            Some(if named.is_absolute() { named } else { folder.join(named) })
        } else {
            None
        };
        if let Some(directory) = directory {
            let head = std::fs::read_to_string(directory.join("HEAD")).ok()?;
            let named = head.trim().strip_prefix("ref:")?.trim();
            return named.strip_prefix("refs/heads/").map(str::to_owned);
        }
        looking = folder.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn provider_registries_are_shared_across_window_catalogs(cx: &mut gpui::TestAppContext) {
        let root = tempfile::tempdir().unwrap();
        let host = Host { data_dir: root.path().into(), plugin_dir: root.path().into() };
        cx.update(|cx| {
            let first = AgentPlugin::new(host.clone(), cx);
            let second = AgentPlugin::new(host, cx);
            assert_eq!(first.registry, second.registry);
        });
    }

    #[gpui::test]
    fn setup_shortcuts_open_host_settings_and_keep_the_launcher(cx: &mut gpui::TestAppContext) {
        use gpui::Modifiers;
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
        let root = tempfile::tempdir().unwrap();
        let registry = cx.new(|_| AgentRegistry::load(root.path().join(STORAGE_FILE)));
        let opened = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let requests = opened.clone();
        let context = InstanceContext {
            instance_id: 1,
            space: String::new(),
            space_name: "Free sessions".into(),
            project_dir: None,
            bound_session: None,
            terminal: TerminalLauncher::new(|_, _| {}),
            services: Default::default(),
            plugin_settings: zeddy_plugin::services::PluginSettings::new(
                move |plugin, window, _| {
                    requests.borrow_mut().push((plugin.map(str::to_owned), window.window_handle()));
                },
            ),
        };
        let (view, cx) = cx.add_window_view(|_, cx| AgentView::new(registry, context, cx));
        cx.run_until_parked();
        let register = cx.debug_bounds("REGISTER_FIRST_AGENT").unwrap();
        cx.simulate_click(register.center(), Modifiers::none());
        cx.run_until_parked();
        let trigger = cx.debug_bounds("ICON-ChevronDown").unwrap();
        cx.simulate_click(trigger.center(), Modifiers::none());
        let popup = cx.windows().into_iter().find(|window| *window != cx.window_handle()).unwrap();
        let mut popup = gpui::VisualTestContext::from_window(popup, cx);
        popup.run_until_parked();
        let item = popup.debug_bounds("MENU_ITEM-Manage agents").unwrap();
        popup.simulate_click(item.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(&*opened.borrow(), &vec![(Some(AgentPlugin::ID.into()), cx.window_handle()); 2],);
        view.read_with(cx, |view, _| {
            assert!(!view.settings_only);
            assert!(!view.editor_open);
        });
    }

    fn record(adapter: &str, args: &[&str], env: &[&str], delivery: &str) -> AgentRecord {
        AgentRecord {
            name: "test".to_owned(),
            adapter: adapter.to_owned(),
            args: args.iter().map(|value| (*value).to_owned()).collect(),
            env: env.iter().map(|value| (*value).to_owned()).collect(),
            delivery: delivery.to_owned(),
        }
    }

    #[test]
    fn argument_fields_round_trip_without_shell_interpretation() {
        let values = vec!["--model".to_owned(), "a model".to_owned(), "it's-safe".to_owned()];
        assert_eq!(parse_args(&format_args(&values)), values);
    }

    #[test]
    fn arguments_are_optional() {
        let agent = record("codex", &[], &[], "argv");
        assert!(valid_stored_agent(&agent));
        assert_eq!(
            String::from_utf8(opening_input(&agent, "inspect this").unwrap()).unwrap(),
            "'codex' 'inspect this'\r"
        );
    }

    #[test]
    fn argv_launches_quote_every_shell_word() {
        let agent = record(
            "codex",
            &["--model", "a model", "it's-safe"],
            &["AGENT_HOME=/agent data"],
            "argv",
        );
        let input =
            String::from_utf8(opening_input(&agent, "fix the 'quoted' test").unwrap()).unwrap();
        assert!(input.starts_with("AGENT_HOME='/agent data' "));
        assert!(input.contains("'codex' '--model' 'a model' 'it'\\''s-safe'"));
        assert!(input.ends_with("'fix the '\\''quoted'\\'' test'\r"));
    }

    #[test]
    fn typed_delivery_puts_the_prompt_after_the_command() {
        let agent = record("opencode", &["--fast"], &[], "type");
        assert_eq!(
            String::from_utf8(opening_input(&agent, "inspect this").unwrap()).unwrap(),
            "'opencode' '--fast'\rinspect this\r"
        );
    }

    #[test]
    fn a_named_prompt_flag_precedes_registered_arguments() {
        let agent = record("agent-cli", &["--model", "large"], &[], "--prompt");
        assert_eq!(
            String::from_utf8(opening_input(&agent, "inspect this").unwrap()).unwrap(),
            "'agent-cli' '--prompt' 'inspect this' '--model' 'large'\r"
        );
    }

    #[test]
    fn invalid_environment_names_are_rejected() {
        let agent = record("codex", &["--help"], &["NOT-A-NAME=value"], "argv");
        assert!(opening_input(&agent, "").unwrap_err().contains("valid environment"));
    }

    #[test]
    fn branch_detection_does_not_need_to_invoke_git() {
        let scratch = tempfile::tempdir().unwrap();
        let project = scratch.path().join("project");
        std::fs::create_dir_all(project.join(".git")).unwrap();
        std::fs::write(project.join(".git/HEAD"), "ref: refs/heads/feature/agents\n").unwrap();
        assert_eq!(git_branch(&project).as_deref(), Some("feature/agents"));
    }

    #[test]
    fn agent_icons_are_inferred_from_names_then_adapters() {
        assert_eq!(inferred_agent_icon("codex-sol-xhigh", "custom"), "icons/agent_chat_gpt.svg");
        assert_eq!(inferred_agent_icon("opus", "/usr/local/bin/claude"), "icons/agent_claude.svg");
        assert_eq!(inferred_agent_icon("open-code", "custom"), GENERIC_AGENT_ICON);
        assert_eq!(inferred_agent_icon("pi", "custom"), "icons/agent_pi.svg");
        assert_eq!(inferred_agent_icon("grok-fast", "custom"), "icons/agent_grok.svg");
    }

    #[test]
    fn unknown_agents_get_the_generic_programming_icon() {
        assert_eq!(inferred_agent_icon("my-agent", "agent-cli"), GENERIC_AGENT_ICON);
        assert_eq!(inferred_agent_icon("copilot", "pi"), "icons/agent_copilot.svg");
    }
}
