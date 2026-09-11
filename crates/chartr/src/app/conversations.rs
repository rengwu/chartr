use super::*;
use crate::conversations::Event;
use chartr_conversations::{NativeSession, Observation, Provider, Status};

impl WorkspaceWindow {
    pub(super) fn sync_conversation_spaces(&mut self, cx: &mut Context<Self>) {
        let choices = self
            .spaces
            .iter()
            .map(|space| {
                let space = space.read(cx);
                crate::conversations::SpaceChoice {
                    key: space.key(),
                    name: space.name().into(),
                    path: (space.kind() == SpaceKind::Registered).then(|| space.path().clone()),
                }
            })
            .collect();
        let active = self.active.as_ref().map(|space| space.read(cx).key());
        self.conversations.update(cx, |view, cx| {
            if view.set_spaces(choices, self.conversation_all_spaces, active) {
                cx.notify();
            }
        });
    }

    pub(super) fn observe_conversations(
        &mut self,
        infos: &[chartr_herdr::control::Session],
        cx: &mut Context<Self>,
    ) {
        self.conversations.update(cx, |view, _| {
            view.set_terminal_client(self.client.clone());
            view.set_agent_services(self.catalog.services.clone());
        });
        if let Some(client) = self.client.clone()
            && self.conversations.update(cx, |view, _| view.needs_integration_check())
        {
            let executor = cx.background_executor().clone();
            cx.spawn(async move |this, cx| {
                let result =
                    executor
                        .spawn(async move {
                            client.agent_integrations().map_err(|error| error.to_string())
                        })
                        .await;
                let _ = this.update(cx, |this, cx| {
                    this.conversations.update(cx, |view, cx| view.integrations_checked(result, cx));
                });
            })
            .detach();
        }
        let observations = infos
            .iter()
            .filter_map(|session| {
                let provider = Provider::detect(session.agent.as_deref()?)?;
                let native = session
                    .agent_session
                    .as_ref()
                    .filter(|identity| Provider::detect(&identity.agent) == Some(provider))
                    .filter(|identity| identity.kind == "id")
                    .map(|identity| NativeSession { id: identity.value.clone(), path: None });
                let owner =
                    self.spaces
                        .iter()
                        .find(|space| space.read(cx).session_access(&session.id).is_some())
                        .or_else(|| {
                            self.spaces.iter().find(|space| {
                                session.cwd.as_ref().is_some_and(|cwd| {
                                    spaces::same_path(space.read(cx).path(), cwd)
                                })
                            })
                        })
                        .map(|space| {
                            let space = space.read(cx);
                            chartr_conversations::SpaceIdentity {
                                key: space.key(),
                                name: space.name().into(),
                            }
                        });
                Some(Observation {
                    space: owner,
                    runtime: session.id.0.clone(),
                    terminal: session.terminal.0.clone(),
                    provider,
                    native,
                    cwd: session.cwd.clone(),
                    title: session.conversation_title.clone(),
                    status: match session.status {
                        chartr_herdr::control::SessionStatus::Working => Status::Working,
                        chartr_herdr::control::SessionStatus::Blocked => Status::Waiting,
                        chartr_herdr::control::SessionStatus::Idle
                        | chartr_herdr::control::SessionStatus::Done => Status::Idle,
                        chartr_herdr::control::SessionStatus::Unknown => Status::Unknown,
                    },
                    pid: session.foreground_pid,
                })
            })
            .collect();
        self.conversations.update(cx, |view, cx| view.observe(observations, cx));
    }

    pub(super) fn conversation_event(
        &mut self,
        event: &Event,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            Event::SelectionChanged => self.schedule_persistence(cx),
            Event::ShowTerminal(runtime) => {
                let pane = chartr_herdr::PaneId(runtime.clone());
                let target = self
                    .spaces
                    .iter()
                    .find(|space| space.read(cx).session_access(&pane).is_some())
                    .cloned();
                if let Some(space) = target {
                    space.update(cx, |space, cx| {
                        space.activate_session(&pane, cx);
                    });
                    self.active = Some(space);
                    let mode = if self.terminal_mode == Mode::Conversations {
                        Mode::Sidebar
                    } else {
                        self.terminal_mode
                    };
                    self.settings_set_mode(mode, cx);
                    self.focus_active_terminal(window, cx);
                } else {
                    self.conversations.update(cx, |view, cx| {
                        view.report(Err("The original terminal is no longer available.".into()), cx)
                    });
                }
            }
            Event::EnableIntegration(provider) => {
                let provider = *provider;
                let Some(client) = self.client.clone() else {
                    self.conversations.update(cx, |view, cx| {
                        view.integration_installed(
                            provider,
                            Err("Terminal service is unavailable.".into()),
                            cx,
                        )
                    });
                    return;
                };
                let executor = cx.background_executor().clone();
                cx.spawn(async move |this, cx| {
                    let result = executor
                        .spawn(async move {
                            client
                                .install_agent_integration(provider.slug())
                                .map_err(|e| e.to_string())
                        })
                        .await;
                    let _ = this.update(cx, |this, cx| {
                        this.conversations.update(cx, |view, cx| {
                            view.integration_installed(provider, result, cx)
                        });
                        this.refresh(cx);
                    });
                })
                .detach();
            }
            Event::LaunchAgent { name, prompt, space } => {
                self.new_agent_conversation(name.clone(), prompt.clone(), space.clone(), window, cx)
            }
            Event::ManageAgents => {
                if let Some(handle) = window.window_handle().downcast::<Self>() {
                    crate::settings_window::open_plugin(
                        handle,
                        cx.weak_entity(),
                        Some("com.chartr.agent".into()),
                        cx,
                    );
                }
            }
        }
        cx.notify();
    }

    fn new_agent_conversation(
        &mut self,
        name: String,
        prompt: String,
        space_key: String,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let preparation = (|| {
            let client = self
                .client
                .clone()
                .filter(|_| matches!(self.backend, Backend::Ready))
                .ok_or("Wait for the terminal service to connect.")?;
            let space = self
                .spaces
                .iter()
                .find(|space| space.read(cx).key() == space_key)
                .cloned()
                .ok_or("The selected space is no longer available. Choose a space again.")?;
            let launch =
                prepare_registered_conversation(&self.catalog.services, &name, &prompt, cx)?;
            Ok::<_, String>((client, space, launch))
        })();
        let (client, space, launch) = match preparation {
            Ok(prepared) => prepared,
            Err(error) => {
                self.conversations.update(cx, |view, cx| view.report(Err(error), cx));
                return;
            }
        };
        let executor = cx.background_executor().clone();
        let cwd = space.read(cx).path().clone();
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let install_client = client.clone();
            // Install/verify known providers before launch so SessionStart is not missed.
            let installed = executor
                .spawn(async move {
                    if let Some(provider) = launch.integration {
                        install_client.install_agent_integration(&provider).map_err(|e| e.to_string())?;
                    }
                    Ok::<_, String>(())
                })
                .await;
            let result = async {
                installed?;
                this.update(cx, |this, _| {
                    if this.spaces.contains(&space) { Ok(()) }
                    else { Err("The selected space was removed before launch.".to_owned()) }
                }).map_err(|e| e.to_string())??;
                this.update(cx, |this, cx| {
                    prepare_registered_conversation(&this.catalog.services, &name, &prompt, cx)
                }).map_err(|e| e.to_string())??;
                let terminal =
                    space.update(cx, |space, cx| space.prepare_plugin_session(cx)).await?;
                cx.update_window(window_handle, |_, window, cx| {
                    space.update(cx, |space, cx| {
                        space.size_conversation_terminal(&chartr_herdr::PaneId(terminal.id.clone()), window, cx)
                    });
                }).map_err(|e| e.to_string())?;
                let launch = this.update(cx, |this, cx| {
                    this.conversations.update(cx, |view, cx| {
                        view.launch_allocated(&name, &terminal.id, cx)
                    });
                    // Resolve again after async startup: disabled/deleted profiles must not run.
                    prepare_registered_conversation(&this.catalog.services, &name, &prompt, cx)
                }).map_err(|e| e.to_string())??;
                terminal.send(&launch.input)?;
                if let Some(options) = launch.opencode {
                    let pane = chartr_herdr::PaneId(terminal.id.clone());
                    executor.spawn(async move {
                        initialize_registered_opencode(&client, &pane, &cwd, options)
                            .map_err(|e| format!("{e}. Open its terminal to continue; your opening message is retained."))
                    }).await?;
                }
                this.update(cx, |this, cx| {
                    this.conversations.update(cx, |view, cx| {
                        view.launch_started(&name, &terminal.id, cx)
                    });
                }).map_err(|e| e.to_string())?;
                Ok::<_, String>(())
            }
            .await;
            let _ = this.update(cx, |this, cx| {
                this.conversations.update(cx, |view, cx| view.report(result, cx));
                this.refresh(cx);
            });
        })
        .detach();
    }
}

fn prepare_registered_conversation(
    services: &chartr_plugin::services::Services,
    name: &str,
    prompt: &str,
    cx: &App,
) -> Result<chartr_plugin::services::ConversationLaunch, String> {
    use chartr_plugin::services::{AGENT_SERVICE, Agents, ConversationAgents, ConversationLaunch};
    if let Some(service) = services.get::<ConversationAgents>(AGENT_SERVICE) {
        return service.prepare(name, prompt, cx);
    }
    let service =
        services.get::<Agents>(AGENT_SERVICE).ok_or("Enable Agent and register an agent first.")?;
    Ok(ConversationLaunch {
        input: service.prepare(name, prompt, cx)?,
        integration: None,
        opencode: None,
    })
}

/// Bootstrap only the terminal we just allocated. Native IDs are never guessed
/// from a provider's most recent transcript or another pane in the same folder.
fn initialize_registered_opencode(
    client: &chartr_herdr::control::Client,
    pane: &chartr_herdr::PaneId,
    cwd: &std::path::Path,
    options: chartr_plugin::services::OpenCodeConversation,
) -> anyhow::Result<()> {
    use chartr_conversations::{OpenCode, endpoints_for_process};
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(25);
    let (pid, adapter) = loop {
        if let Some(pid) = client.foreground_pid(pane)? {
            if let Some(adapter) = endpoints_for_process(pid)?
                .iter()
                .filter_map(|endpoint| OpenCode::new(endpoint, cwd).ok())
                .find(|adapter| adapter.health().is_ok())
            {
                break (pid, adapter);
            }
        }
        anyhow::ensure!(Instant::now() < deadline, "OpenCode's local connection is not ready");
        std::thread::sleep(Duration::from_millis(150));
    };
    let native = if options.reuse {
        loop {
            if let Some(native) = client
                .sessions(None)?
                .into_iter()
                .find(|session| &session.id == pane)
                .and_then(|session| session.agent_session)
                .filter(|native| native.agent == "opencode" && native.kind == "id")
            {
                break native.value;
            }
            anyhow::ensure!(
                Instant::now() < deadline,
                "The saved OpenCode session has not connected"
            );
            std::thread::sleep(Duration::from_millis(150));
        }
    } else {
        loop {
            let screen = client.read_visible(pane)?;
            if screen.contains("tab agents") || screen.contains("Ask anything") {
                break;
            }
            anyhow::ensure!(Instant::now() < deadline, "OpenCode needs setup in its terminal");
            std::thread::sleep(Duration::from_millis(150));
        }
        let native = adapter.create()?;
        let title = adapter.read(&native)?.0;
        adapter.select(&native)?;
        let selection_deadline = Instant::now() + Duration::from_secs(5);
        while !client.read_visible(pane)?.contains(&title) {
            anyhow::ensure!(
                Instant::now() < selection_deadline,
                "OpenCode did not select the new conversation"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        anyhow::ensure!(
            client.foreground_pid(pane)? == Some(pid),
            "The original OpenCode process changed"
        );
        client.report_opencode_session(pane, &native)?;
        native
    };
    anyhow::ensure!(
        client.foreground_pid(pane)? == Some(pid),
        "The original OpenCode process changed"
    );
    adapter.ready(&native)?;
    adapter.send_with_options(
        &native,
        &OpenCode::message_id(),
        &options.prompt,
        options.model.as_deref(),
        options.agent.as_deref(),
    )?;
    Ok(())
}
