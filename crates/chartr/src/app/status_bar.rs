//! Persistent activity belongs to the workspace, independent of open plugin panes.
use super::*;
use chartr_plugin::BackgroundState;

impl WorkspaceWindow {
    pub(super) fn observe_background_status(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if this
                    .update(cx, |this, cx| {
                        let mut statuses: Vec<_> = this
                            .catalog
                            .loaded
                            .iter()
                            .filter_map(|(id, plugin)| {
                                plugin.background_status(cx).map(|status| (id.clone(), status))
                            })
                            .collect();
                        statuses.sort_by(|a, b| a.0.cmp(&b.0));
                        if statuses != this.background_statuses {
                            this.background_statuses = statuses;
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub(super) fn toggle_status_bar(&mut self, cx: &mut Context<Self>) {
        let show = !self.settings.resolved().show_status_bar;
        if let Err(error) = crate::settings::update_global(cx, |content| {
            content.general.get_or_insert_default().show_status_bar = Some(show);
        }) {
            self.problem = Some(error.to_string());
        }
        cx.notify();
    }

    pub(super) fn status_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let sessions = self
            .spaces
            .iter()
            .map(|space| {
                let space = space.read(cx);
                space
                    .all_item_ids()
                    .into_iter()
                    .filter(|id| {
                        space
                            .item(*id)
                            .and_then(|item| item.as_session())
                            .is_some_and(|session| session.session.ended().is_none())
                    })
                    .count()
            })
            .sum::<usize>();
        let (backend, color) = match &self.backend {
            Backend::Ready => ("Terminal service ready", Color::Muted),
            Backend::Starting => ("Terminal service starting…", Color::Warning),
            Backend::Recovering(_) => ("Terminal service reconnecting…", Color::Warning),
            Backend::Failed(_) => ("Terminal service unavailable", Color::Error),
        };
        let detail = self.settings_backend_label();
        let items = self.background_statuses.iter().enumerate().map(|(index, (id, status))| {
            let plugin = id.clone();
            let detail = status.detail.clone();
            let color = match status.state {
                BackgroundState::Idle => Color::Muted,
                BackgroundState::Running => Color::Success,
                BackgroundState::Error => Color::Error,
            };
            Button::new(("background-status", index), status.label.clone())
                .label_size(UI_LABEL_SMALL)
                .color(color)
                .tab_index(0isize)
                .aria_label(status.label.clone())
                .tooltip(Tooltip::text(detail))
                .on_click(cx.listener(move |_, _, window, cx| {
                    if let Some(owner) = window.window_handle().downcast::<Self>() {
                        crate::settings_window::open_plugin(
                            owner,
                            cx.weak_entity(),
                            Some(plugin.clone()),
                            cx,
                        );
                    }
                }))
        });
        h_flex()
            .id("workspace-status-bar")
            .w_full()
            .h(rems(1.75))
            .flex_none()
            .px_2()
            .gap_2()
            .border_t_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().panel_background)
            .child(
                Button::new("backend-status", backend)
                    .label_size(UI_LABEL_SMALL)
                    .color(color)
                    .tab_index(0isize)
                    .tooltip(Tooltip::text(detail))
                    .on_click(cx.listener(|this, _, window, cx| this.open_settings(window, cx))),
            )
            .child(
                Label::new(format!(
                    "{sessions} running session{}",
                    if sessions == 1 { "" } else { "s" }
                ))
                .size(UI_LABEL_SMALL)
                .color(Color::Muted),
            )
            .child(
                h_flex()
                    .id("background-status-items")
                    .flex_1()
                    .min_w_0()
                    .overflow_x_scroll()
                    .justify_end()
                    .gap_1()
                    .children(items),
            )
            .when(!self.companion_leases.is_empty(), |bar| {
                bar.child(
                    Label::new(format!("{} on mobile", self.companion_leases.len()))
                        .size(UI_LABEL_SMALL)
                        .color(Color::Muted),
                )
            })
            .child(
                IconButton::new("hide-status-bar", IconName::Close)
                    .icon_size(IconSize::Small)
                    .tab_index(0isize)
                    .aria_label("Hide status bar")
                    .tooltip(Tooltip::text(
                        "Hide status bar · Restore in Settings or the command palette",
                    ))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_status_bar(cx))),
            )
            .into_any_element()
    }
}
