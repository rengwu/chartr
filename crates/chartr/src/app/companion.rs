//! Resolve remote operations against the live window, never the saved workspace file.
use super::*;
use chartr_companion::Operation;
use futures::StreamExt;
use serde_json::{Value, json};
use std::sync::atomic::Ordering;

pub(super) struct MobileLease {
    owner: std::sync::Arc<std::sync::atomic::AtomicBool>,
    expires: std::time::Instant,
    terminal: Entity<terminal::Terminal>,
    desktop_bounds: terminal::TerminalBounds,
    pub columns: u16,
    pub rows: u16,
}

impl WorkspaceWindow {
    pub(super) fn bind_companion(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_millis(250)).await;
                if this
                    .update_in(cx, |this, window, cx| this.expire_mobile_leases(window, cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        let (sender, mut receiver) =
            futures::channel::mpsc::channel::<crate::companion_plugin::Call>(16);
        let bridge = crate::companion_plugin::Bridge(sender);
        self.companion_bridge = Some(bridge.clone());
        cx.set_global(bridge);
        cx.spawn_in(window, async move |this, cx| {
            while let Some(call) = receiver.next().await {
                if !call.alive.load(Ordering::Acquire) || std::time::Instant::now() > call.deadline
                {
                    let _ = call.reply.send(Err("Request expired or sharing stopped.".into()));
                    continue;
                }
                let history = if let Operation::History { space, session, snapshot: None, .. } =
                    &call.operation
                {
                    let task = this.update_in(cx, |this, window, cx| {
                        let space = this
                            .spaces
                            .iter()
                            .find(|s| s.read(cx).key() == *space)
                            .ok_or("Space is no longer open.")?
                            .read(cx);
                        let item = space
                            .all_item_ids()
                            .into_iter()
                            .filter_map(|id| space.item(id)?.as_session())
                            .find(|s| s.session.id().0 == *session)
                            .ok_or("Session is no longer open.")?;
                        if item.session.ended().is_some() {
                            return Err("Terminal attachment ended.");
                        }
                        let terminal = item.session.terminal();
                        let client = space.companion_client();
                        let pane = item.session.id().clone();
                        let columns = terminal.update(cx, |t, cx| {
                            t.sync(window, cx);
                            t.last_content().columns
                        });
                        Ok(cx.background_executor().spawn(async move {
                            client.read_scrollback(&pane, columns).map_err(|e| e.to_string())
                        }))
                    });
                    match task {
                        Ok(Ok(task)) => task.await.map(Some),
                        Ok(Err(error)) => Err(error.to_owned()),
                        Err(_) => Err("Workspace closed.".into()),
                    }
                } else {
                    Ok(None)
                };
                let history = match history {
                    Ok(history) => history,
                    Err(error) => {
                        let _ = call.reply.send(Err(error));
                        continue;
                    }
                };
                if !call.alive.load(Ordering::Acquire) || std::time::Instant::now() > call.deadline
                {
                    let _ = call.reply.send(Err("Request expired or sharing stopped.".into()));
                    continue;
                }
                let result = this
                    .update_in(cx, |this, window, cx| {
                        this.companion_operation(call.operation, history, call.alive, window, cx)
                    })
                    .unwrap_or_else(|_| Err("Workspace closed.".into()));
                let _ = call.reply.send(result);
            }
        })
        .detach();
    }

    fn companion_operation(
        &mut self,
        operation: Operation,
        history: Option<String>,
        owner: std::sync::Arc<std::sync::atomic::AtomicBool>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<Value, String> {
        self.expire_mobile_leases(window, cx);
        if let Operation::List {} = operation {
            let spaces: Vec<_> = self.spaces.iter().map(|space| {
                let space = space.read(cx);
                let sessions: Vec<_> = space.all_item_ids().into_iter().filter_map(|id| {
                    let item = space.item(id)?.as_session()?;
                    let session = &item.session;
                    Some(json!({"id":session.id().0,"title":session.title(),"cwd":session.info.cwd,"status":format!("{:?}",session.info.status),"ended":session.ended().is_some()}))
                }).collect();
                json!({"id":space.key(),"name":space.name(),"path":space.path(),"error":space.problem(),"sessions":sessions})
            }).collect();
            return Ok(json!({"version":1,"spaces":spaces}));
        }
        let key = match &operation {
            Operation::Screen { space, .. }
            | Operation::Watch { space, .. }
            | Operation::Release { space, .. }
            | Operation::History { space, .. }
            | Operation::Input { space, .. }
            | Operation::Paste { space, .. }
            | Operation::Submit { space, .. }
            | Operation::Focus { space, .. }
            | Operation::Create { space } => space,
            Operation::List {} => unreachable!(),
        };
        let space = self
            .spaces
            .iter()
            .find(|s| s.read(cx).key() == *key)
            .cloned()
            .ok_or("Space is no longer open.")?;
        if let Operation::Create { .. } = operation {
            space.update(cx, |space, cx| space.start_session(cx));
            return Ok(json!({"accepted":true}));
        }
        if let Operation::Focus { session: None, .. } = &operation {
            self.activate(space, window, cx);
            return Ok(json!(true));
        }
        let id = match &operation {
            Operation::Screen { session, .. }
            | Operation::Watch { session, .. }
            | Operation::Release { session, .. }
            | Operation::History { session, .. }
            | Operation::Input { session, .. }
            | Operation::Paste { session, .. } => session,
            Operation::Submit { session, .. } => session,
            Operation::Focus { session: Some(session), .. } => session,
            _ => unreachable!(),
        };
        let (terminal, ended) = {
            let space = space.read(cx);
            let session = space
                .all_item_ids()
                .into_iter()
                .filter_map(|id| space.item(id)?.as_session())
                .find(|s| s.session.id().0 == *id)
                .ok_or("Session is no longer open.")?;
            (session.session.terminal(), session.session.ended().is_some())
        };
        if ended {
            return Err("Terminal attachment ended. Reattach it in Chartr.".into());
        }
        if matches!(
            operation,
            Operation::Input { .. } | Operation::Paste { .. } | Operation::Submit { .. }
        ) && self
            .companion_leases
            .get(id)
            .is_some_and(|lease| !std::sync::Arc::ptr_eq(&lease.owner, &owner))
        {
            return Err("This terminal is being controlled by another mobile device.".into());
        }
        if let Operation::Release { session, .. } = &operation {
            if self
                .companion_leases
                .get(session)
                .is_some_and(|lease| std::sync::Arc::ptr_eq(&lease.owner, &owner))
            {
                self.release_mobile(session, window, cx);
            }
            return Ok(json!(true));
        }
        if let Operation::Watch { session, columns, rows, .. } = &operation {
            if !(2..=400).contains(columns) || !(1..=240).contains(rows) {
                return Err("Invalid mobile terminal dimensions.".into());
            }
            if self
                .companion_leases
                .get(session)
                .is_some_and(|lease| !std::sync::Arc::ptr_eq(&lease.owner, &owner))
            {
                return Err("This terminal is being viewed on another mobile device.".into());
            }
            let first = !self.companion_leases.contains_key(session);
            let lease =
                self.companion_leases.entry(session.clone()).or_insert_with(|| MobileLease {
                    owner,
                    expires: std::time::Instant::now(),
                    terminal: terminal.clone(),
                    desktop_bounds: terminal.read(cx).last_content().terminal_bounds,
                    columns: *columns,
                    rows: *rows,
                });
            lease.expires = std::time::Instant::now() + std::time::Duration::from_secs(10);
            lease.columns = *columns;
            lease.rows = *rows;
            terminal.update(cx, |t, cx| {
                t.set_size(terminal::TerminalBounds::new(
                    px(18.),
                    px(9.),
                    gpui::Bounds {
                        origin: gpui::point(px(0.), px(0.)),
                        size: gpui::size(px(*columns as f32 * 9.), px(*rows as f32 * 18.)),
                    },
                ));
                t.scroll_to_bottom();
                t.sync(window, cx);
            });
            if first {
                window.focus(&self.focus, cx);
            }
            cx.notify();
        }
        match operation {
            Operation::History { session, snapshot, offset, known, .. } =>
                self.companion_history.read(&session, snapshot.as_deref(), offset, known.as_deref(),
                    || history.unwrap_or_default()),
            Operation::Screen{..} | Operation::Watch{..} => terminal.update(cx, |terminal,cx| {
                terminal.sync(window,cx);
                let content = terminal.last_content();
                let cells: Vec<_> = content.cells.iter().filter(|cell| !cell.is_wide_char_spacer()).map(|cell| {
                    let mut text = cell.character().to_string();
                    if let Some(extra) = cell.zerowidth() { text.extend(extra); }
                    let foreground = color(cell.foreground(),cx);
                    let background = color(cell.background(),cx);
                    let (foreground,background) = if cell.is_inverse() {(background,foreground)} else {(foreground,background)};
                    json!([cell.point.line + content.display_offset as i32, cell.point.column, text, foreground, background, cell.is_bold(), cell.has_underline()])
                }).collect();
                Ok(json!({"columns":content.columns,"rows":content.screen_lines,"cells":cells,"cursor":[content.cursor.point.line + content.display_offset as i32,content.cursor.point.column],"show_cursor":content.mode.contains(terminal::Modes::SHOW_CURSOR),"app_cursor":content.mode.contains(terminal::Modes::APP_CURSOR)}))
            }),
            Operation::Input{data,..} => { terminal.update(cx, |t,_| t.input(data.into_bytes())); Ok(json!(true)) },
            Operation::Paste{data,..} => { terminal.update(cx, |t,_| t.paste(&data)); Ok(json!(true)) },
            Operation::Submit{data,..} => { terminal.update(cx, |t,_| { t.paste(&data); t.input(vec![b'\r']); }); Ok(json!(true)) },
            Operation::Focus{session:Some(session),..} => {
                space.update(cx, |s,cx| s.activate_session(&chartr_herdr::PaneId(session),cx));
                self.activate(space,window,cx); Ok(json!(true))
            },
            _ => unreachable!(),
        }
    }

    fn release_mobile(&mut self, session: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(lease) = self.companion_leases.remove(session) {
            lease.terminal.update(cx, |terminal, cx| {
                terminal.set_size(lease.desktop_bounds);
                terminal.sync(window, cx);
            });
            cx.notify();
        }
    }

    fn expire_mobile_leases(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let expired: Vec<_> = self
            .companion_leases
            .iter()
            .filter(|(_, lease)| {
                !lease.owner.load(Ordering::Acquire) || lease.expires <= std::time::Instant::now()
            })
            .map(|(session, _)| session.clone())
            .collect();
        for session in expired {
            self.release_mobile(&session, window, cx);
        }
    }
}
fn color(color: terminal::Color, cx: &App) -> String {
    let color = match color {
        terminal::Color::Spec(rgb) => terminal::rgba_color(rgb.r, rgb.g, rgb.b),
        terminal::Color::Named(name) => terminal::get_color_at_index(name as usize, cx.theme()),
        terminal::Color::Indexed(index) => terminal::get_color_at_index(index as usize, cx.theme()),
    };
    let rgba: gpui::Rgba = color.into();
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgba.r * 255.0) as u8,
        (rgba.g * 255.0) as u8,
        (rgba.b * 255.0) as u8
    )
}
