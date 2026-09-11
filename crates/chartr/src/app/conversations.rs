use super::*;
use crate::conversations::Event;
use chartr_conversations::{NativeSession, Observation, Provider, Status};

impl WorkspaceWindow {
    pub(super) fn sync_inbox_terminal(&mut self, cx: &mut Context<Self>) {
        let inbox = self.conversations.read(cx);
        let target = inbox.selected_runtime().and_then(|runtime| {
            let pane = chartr_herdr::PaneId(runtime.to_owned());
            let item = self.spaces.iter().find_map(|space| space.read(cx).session_item(&pane))?;
            if let Some(row) = inbox.selected_row()
                && !matches_session(row, &item.session.info)
            {
                return Some((
                    None,
                    Some("Session ended. This conversation remains in Inbox.".to_owned()),
                ));
            }
            if item.session.ended().is_some() {
                return Some((
                    None,
                    Some("Session ended. This conversation remains in Inbox.".to_owned()),
                ));
            }
            if let Some(lease) = self.companion_leases.get(runtime) {
                return Some((
                    None,
                    Some(format!("Viewing on mobile · {} × {}", lease.columns, lease.rows)),
                ));
            }
            Some((item.terminal_view(), None))
        });
        let (view, notice) = target.unwrap_or_default();
        self.conversations.update(cx, |inbox, cx| inbox.set_terminal(view, notice, cx));
    }

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
            Event::SelectionChanged => {
                if self.terminal_search_open {
                    self.close_terminal_search(window, cx);
                }
                self.schedule_persistence(cx);
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
            Event::LaunchAgent { name, space } => {
                self.new_agent_conversation(name.clone(), space.clone(), window, cx)
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
            let launch = prepare_registered_conversation(&self.catalog.services, &name, cx)?;
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
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let install_client = client.clone();
            // Install/verify known providers before launch so SessionStart is not missed.
            let installed = executor
                .spawn(async move {
                    if let Some(provider) = launch.integration {
                        install_client
                            .install_agent_integration(&provider)
                            .map_err(|e| e.to_string())?;
                    }
                    Ok::<_, String>(())
                })
                .await;
            let result = async {
                installed?;
                this.update(cx, |this, _| {
                    if this.spaces.contains(&space) {
                        Ok(())
                    } else {
                        Err("The selected space was removed before launch.".to_owned())
                    }
                })
                .map_err(|e| e.to_string())??;
                this.update(cx, |this, cx| {
                    prepare_registered_conversation(&this.catalog.services, &name, cx)
                })
                .map_err(|e| e.to_string())??;
                let terminal =
                    space.update(cx, |space, cx| space.prepare_plugin_session(cx)).await?;
                cx.update_window(window_handle, |_, window, cx| {
                    space.update(cx, |space, cx| {
                        space.size_conversation_terminal(
                            &chartr_herdr::PaneId(terminal.id.clone()),
                            window,
                            cx,
                        )
                    });
                })
                .map_err(|e| e.to_string())?;
                let launch = this
                    .update(cx, |this, cx| {
                        this.conversations
                            .update(cx, |view, cx| view.launch_allocated(&name, &terminal.id, cx));
                        // Resolve again after async startup: disabled/deleted profiles must not run.
                        prepare_registered_conversation(&this.catalog.services, &name, cx)
                    })
                    .map_err(|e| e.to_string())??;
                terminal.send(&launch.input)?;
                this.update(cx, |this, cx| {
                    this.conversations
                        .update(cx, |view, cx| view.launch_started(&name, &terminal.id, cx));
                })
                .map_err(|e| e.to_string())?;
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
    cx: &App,
) -> Result<chartr_plugin::services::InboxLaunch, String> {
    use chartr_plugin::services::{AGENT_SERVICE, Agents, InboxAgents, InboxLaunch};
    if let Some(service) = services.get::<InboxAgents>(AGENT_SERVICE) {
        return service.prepare(name, cx);
    }
    let service =
        services.get::<Agents>(AGENT_SERVICE).ok_or("Enable Agent and register an agent first.")?;
    Ok(InboxLaunch { input: service.prepare(name, "", cx)?, integration: None })
}

/// Reused panes must never expose another native conversation through an old row.
fn matches_session(
    row: &chartr_conversations::Conversation,
    session: &chartr_herdr::control::Session,
) -> bool {
    row.runtime.as_deref() == Some(&session.id.0)
        && row.terminal.as_deref() == Some(&session.terminal.0)
        && session.agent.as_deref().and_then(Provider::detect) == Some(row.provider)
        && row.native.as_ref().is_none_or(|native| {
            session.agent_session.as_ref().is_some_and(|identity| {
                identity.kind == "id"
                    && identity.value == native.id
                    && Provider::detect(&identity.agent) == Some(row.provider)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> chartr_herdr::control::Session {
        chartr_herdr::control::Session {
            id: chartr_herdr::PaneId("pane".into()),
            terminal: chartr_herdr::TerminalId("pty".into()),
            workspace: chartr_herdr::WorkspaceId("workspace".into()),
            label: "agent".into(),
            running: None,
            status: chartr_herdr::control::SessionStatus::Idle,
            agent: Some("codex".into()),
            agent_session: Some(chartr_herdr::protocol::AgentSession {
                source: "integration".into(),
                agent: "codex".into(),
                kind: "id".into(),
                value: "native-a".into(),
            }),
            conversation_title: None,
            foreground_pid: None,
            cwd: None,
        }
    }

    #[test]
    fn inbox_terminal_binding_rejects_reused_panes_and_changed_native_sessions() {
        let mut row: chartr_conversations::Conversation =
            serde_json::from_value(serde_json::json!({
                "id":"a", "provider":"codex", "native":{"id":"native-a", "path":null},
                "title":"Task", "custom_title":null, "cwd":null, "updated":1,
                "draft":"", "archived":false, "messages":[]
            }))
            .unwrap();
        row.runtime = Some("pane".into());
        row.terminal = Some("pty".into());
        assert!(matches_session(&row, &session()));
        let mut changed = session();
        changed.agent_session.as_mut().unwrap().value = "native-b".into();
        assert!(!matches_session(&row, &changed));
        changed = session();
        changed.terminal.0 = "replacement-pty".into();
        assert!(!matches_session(&row, &changed));
        changed = session();
        changed.agent = None;
        assert!(!matches_session(&row, &changed));
        changed = session();
        changed.agent_session = None;
        assert!(!matches_session(&row, &changed));
        row.native = None;
        assert!(
            matches_session(&row, &changed),
            "Detected agents can use their terminal before integration"
        );
        row.runtime = None;
        assert!(
            !matches_session(&row, &session()),
            "Ended history cannot fall back to an active pane"
        );
    }
}
