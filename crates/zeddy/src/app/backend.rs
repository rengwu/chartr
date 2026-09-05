//! Backend connection, supervision, recovery, and shutdown.

use super::*;

impl Zeddy {
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

    /// Supervise the private daemon on the same discipline as Chartr-rs: the
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
                    executor.spawn(async move { probe.answers() }).await
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
                            Err(error) => this.problem = Some(error.to_string()),
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
        self.drop_dead_terminals(cx);
        self.backend_ready_since = None;

        if self.backend_restart_spent {
            let problem = "The terminal backend failed again before it held for 60 seconds. Chartr stopped automatic recovery to expose the crash loop.".to_owned();
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
                        "Chartr could not recover the terminal backend: {error}"
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
        infos: Vec<zeddy_herdr::control::Session>,
        cx: &mut Context<Self>,
    ) {
        let paths_by_workspace: HashMap<WorkspaceId, PathBuf> = infos
            .iter()
            .filter_map(|info| info.cwd.clone().map(|cwd| (info.workspace.clone(), cwd)))
            .collect();
        let mut by_space: HashMap<EntityId, Vec<zeddy_herdr::control::Session>> = HashMap::new();

        for info in infos {
            let path =
                info.cwd.clone().or_else(|| paths_by_workspace.get(&info.workspace).cloned());
            let Some(path) = path else {
                continue;
            };
            let target = self.spaces.iter().find(|space| {
                let space = space.read(cx);
                spaces::same_path(space.path(), &path)
            });
            if let Some(target) = target {
                by_space.entry(target.entity_id()).or_default().push(info);
            }
        }

        for space in &self.spaces {
            let infos = by_space.remove(&space.entity_id()).unwrap_or_default();
            space.update(cx, |space, cx| space.adopt(infos, cx));
        }
    }
}
