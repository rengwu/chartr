//! Backend connection, supervision, recovery, and shutdown.

use super::*;

impl WorkspaceWindow {
    /// Apply the explicit exit policy before the window releases its spaces.
    pub fn apply_exit_policy(&mut self, cx: &mut Context<Self>) {
        if !self.settings.resolved().terminate_sessions_on_exit {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        for space in &self.spaces {
            let ids = space.read(cx).all_item_ids();
            let targets = space.read(cx).close_targets(&ids);
            let closed: Vec<_> = targets
                .into_iter()
                .filter_map(|(item, backend)| match backend {
                    None => Some(item),
                    Some(backend) if client.close_session(&backend).is_ok() => Some(item),
                    Some(_) => None,
                })
                .collect();
            space.update(cx, |space, _| space.finish_bulk_close(&closed));
        }
    }

    /// Bring the private backend up and take one snapshot of every running
    /// session. Like Zed's project I/O, the blocking transport stays on the
    /// background executor and only owned answers return to GPUI.
    pub(super) fn connect(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    client.connect(BACKEND_TIMEOUT)?;
                    client.sessions(None)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => {
                    this.backend_became_ready();
                    this.distribute(infos, cx);
                    this.start_supervision(cx);
                    cx.notify();
                }
                Err(error) => {
                    let problem = error.to_string();
                    this.backend = Backend::Failed(problem.clone());
                    this.backend_ready_since = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn backend_became_ready(&mut self) {
        self.backend = Backend::Ready;
        self.backend_ready_since = Some(Instant::now());
    }

    /// Supervise the private daemon on the same discipline as chartr-rs: the
    /// socket is checked every two seconds, the first failure in an episode is
    /// given one clean replacement, and a replacement that cannot hold for a
    /// minute is exposed as a crash loop rather than restarted again.
    fn start_supervision(&mut self, cx: &mut Context<Self>) {
        if self.supervision_started {
            return;
        }
        self.supervision_started = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            loop {
                executor.timer(BACKEND_SUPERVISION).await;
                let check = this.update(cx, |this, _| {
                    matches!(this.backend, Backend::Ready).then(|| this.client.clone()).flatten()
                });
                let Ok(Some(client)) = check else {
                    continue;
                };
                let probe = client.clone();
                let snapshot = executor.spawn(async move { probe.sessions(None) }).await;
                let answers = if snapshot.is_ok() {
                    true
                } else {
                    let probe = client.clone();
                    executor.spawn(async move { probe.handshake().is_ok() }).await
                };
                if answers {
                    let _ = this.update(cx, |this, cx| {
                        if this.backend_restart_spent
                            && this
                                .backend_ready_since
                                .is_some_and(|since| since.elapsed() >= BACKEND_STEADY)
                        {
                            this.backend_restart_spent = false;
                        }
                        match snapshot {
                            Ok(infos) => this.distribute(infos, cx),
                            Err(error) => {
                                this.conversations.update(cx, |view, cx| view.disconnected(cx));
                                this.problem = Some(error.to_string());
                            }
                        }
                        cx.notify();
                    });
                    continue;
                }
                let _ = this.update(cx, |this, cx| this.backend_died(client, cx));
            }
        })
        .detach();
    }

    pub(super) fn backend_died(&mut self, client: Client, cx: &mut Context<Self>) {
        if !matches!(self.backend, Backend::Ready) {
            return;
        }
        if self.backend_ready_since.is_some_and(|since| since.elapsed() >= BACKEND_STEADY) {
            self.backend_restart_spent = false;
        }
        self.conversations.update(cx, |view, cx| view.disconnected(cx));
        self.drop_dead_terminals(cx);
        self.backend_ready_since = None;

        if self.backend_restart_spent {
            let problem = "The terminal backend failed again before it held for 60 seconds. chartr stopped automatic recovery to expose the crash loop.".to_owned();
            self.backend = Backend::Failed(problem);
            let executor = cx.background_executor().clone();
            executor.spawn(async move { client.clear_saved_shape() }).detach();
            cx.notify();
            return;
        }

        self.backend_restart_spent = true;
        self.backend = Backend::Recovering(
            "The terminal backend stopped answering. Starting one clean replacement…".to_owned(),
        );
        cx.notify();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    client.restart()?;
                    client.reconnect(BACKEND_TIMEOUT)?;
                    client.sessions(None)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => {
                    this.backend_became_ready();
                    this.distribute(infos, cx);
                    cx.notify();
                }
                Err(error) => {
                    this.backend = Backend::Failed(format!(
                        "chartr could not recover the terminal backend: {error}"
                    ));
                    this.backend_ready_since = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn drop_dead_terminals(&mut self, cx: &mut Context<Self>) {
        for space in &self.spaces {
            space.update(cx, |space, _| space.drop_dead_sessions());
        }
    }

    pub(super) fn retry_backend(&mut self, clean_restart: bool, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        if clean_restart {
            self.conversations.update(cx, |view, cx| view.disconnected(cx));
            self.drop_dead_terminals(cx);
        }
        self.backend = if clean_restart {
            Backend::Recovering("Restarting the terminal backend…".to_owned())
        } else {
            Backend::Starting
        };
        self.backend_ready_since = None;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = executor
                .spawn(async move {
                    if clean_restart {
                        client.restart()?;
                        client.reconnect(BACKEND_TIMEOUT)?;
                    } else {
                        client.connect(BACKEND_TIMEOUT)?;
                    }
                    client.sessions(None)
                })
                .await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(infos) => {
                    this.backend_restart_spent = false;
                    this.backend_became_ready();
                    this.distribute(infos, cx);
                    this.start_supervision(cx);
                    cx.notify();
                }
                Err(error) => {
                    this.backend = Backend::Failed(error.to_string());
                    this.backend_ready_since = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn request_backend_restart(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let terminal_count: usize = self
            .spaces
            .iter()
            .map(|space| {
                let ids = space.read(cx).all_item_ids();
                space
                    .read(cx)
                    .close_targets(&ids)
                    .iter()
                    .filter(|(_, backend)| backend.is_some())
                    .count()
            })
            .sum();
        if terminal_count <= 1 {
            self.retry_backend(true, cx);
            return;
        }
        let prompt = window.prompt(
            gpui::PromptLevel::Critical,
            "Restart the terminal backend?",
            Some(&format!(
                "Restarting is destructive: all {terminal_count} underlying sessions will be terminated."
            )),
            &["Restart and Terminate", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if prompt.await == Ok(0) {
                let _ = this.update(cx, |this, cx| this.retry_backend(true, cx));
            }
        })
        .detach();
    }

    /// Partition a single backend snapshot by workspace path, then let each
    /// space attach its own sessions.
    pub(super) fn distribute(
        &mut self,
        infos: Vec<chartr_herdr::control::Session>,
        cx: &mut Context<Self>,
    ) {
        self.observe_conversations(&infos, cx);
        let owners: Vec<_> = self
            .spaces
            .iter()
            .map(|space| {
                let space = space.read(cx);
                (space.path().clone(), space.owned_session_ids())
            })
            .collect();
        for (space, infos) in self.spaces.iter().zip(partition_sessions(infos, &owners)) {
            space.update(cx, |space, cx| space.adopt(infos, cx));
        }
    }
}

/// Stable live/saved identities take precedence over cwd, including while an
/// attachment is being restored. Cwd only discovers otherwise unknown sessions.
fn partition_sessions(
    infos: Vec<chartr_herdr::control::Session>,
    owners: &[(PathBuf, HashSet<chartr_herdr::PaneId>)],
) -> Vec<Vec<chartr_herdr::control::Session>> {
    let known_owner = |info: &chartr_herdr::control::Session| {
        owners.iter().position(|(_, sessions)| sessions.contains(&info.id))
    };
    let mut workspaces: HashMap<WorkspaceId, Option<usize>> = HashMap::new();
    for info in &infos {
        if let Some(owner) = known_owner(info) {
            workspaces
                .entry(info.workspace.clone())
                .and_modify(|existing| {
                    if *existing != Some(owner) {
                        *existing = None;
                    }
                })
                .or_insert(Some(owner));
        }
    }
    let mut partitions = vec![Vec::new(); owners.len()];
    for info in infos {
        let owner = known_owner(&info)
            .or_else(|| workspaces.get(&info.workspace).copied().flatten())
            .or_else(|| {
                info.cwd.as_ref().and_then(|cwd| {
                    owners.iter().position(|(path, _)| spaces::same_path(path, cwd))
                })
            });
        if let Some(owner) = owner {
            partitions[owner].push(info);
        }
    }
    partitions
}

#[cfg(test)]
mod tests {
    use super::*;
    use chartr_herdr::{
        PaneId, TerminalId,
        control::{Session, SessionStatus},
    };

    fn session(id: &str, workspace: &str, cwd: Option<&str>) -> Session {
        Session {
            id: PaneId(id.into()),
            terminal: TerminalId(id.into()),
            workspace: WorkspaceId(workspace.into()),
            label: id.into(),
            running: None,
            status: SessionStatus::Unknown,
            agent: None,
            agent_session: None,
            conversation_title: None,
            foreground_pid: None,
            cwd: cwd.map(PathBuf::from),
        }
    }

    #[test]
    fn live_and_restored_sessions_stay_in_their_space_after_cd() {
        let owners = vec![
            (
                PathBuf::from("/project/a"),
                HashSet::from([PaneId("live".into()), PaneId("saved".into())]),
            ),
            (PathBuf::from("/project/b"), HashSet::new()),
        ];
        let partitions = partition_sessions(
            vec![
                session("live", "w1", Some("/project/b")),
                session("saved", "w1", Some("/project/a/subdir")),
                session("new-sibling", "w1", None),
                session("new-project", "w2", Some("/project/b")),
            ],
            &owners,
        );
        assert_eq!(
            partitions[0].iter().map(|s| s.id.0.as_str()).collect::<Vec<_>>(),
            ["live", "saved", "new-sibling"]
        );
        assert_eq!(partitions[1][0].id.0, "new-project");
    }

    #[test]
    fn explicit_ownership_wins_even_when_a_backend_workspace_spans_spaces() {
        let owners = vec![
            (PathBuf::from("/a"), HashSet::from([PaneId("one".into())])),
            (PathBuf::from("/b"), HashSet::from([PaneId("two".into())])),
        ];
        let infos = vec![
            session("one", "w", Some("/b")),
            session("two", "w", Some("/a")),
            session("new", "w", Some("/b")),
        ];
        for infos in [infos.clone(), infos.into_iter().rev().collect()] {
            let partitions = partition_sessions(infos, &owners);
            assert_eq!(partitions[0].len(), 1);
            assert_eq!(partitions[0][0].id.0, "one");
            assert_eq!(partitions[1].len(), 2);
            assert!(partitions[1].iter().any(|s| s.id.0 == "two"));
            assert!(partitions[1].iter().any(|s| s.id.0 == "new"));
        }
    }
}
