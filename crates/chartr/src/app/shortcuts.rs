//! Workspace commands shared by keyboard shortcuts and the command palette.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct MeasuredPane {
    pub space: EntityId,
    pub tab: WorkspaceTabId,
    pub pane: LayoutPaneId,
    pub size: gpui::Size<gpui::Pixels>,
}

fn split_direction(size: gpui::Size<gpui::Pixels>) -> SplitDirection {
    if size.width > size.height { SplitDirection::Right } else { SplitDirection::Down }
}

impl WorkspaceWindow {
    pub(super) fn new_adaptive_pane(
        &mut self,
        terminal: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.mode == Mode::Conversations {
            self.settings_set_mode(self.terminal_mode, cx);
        }
        if terminal && !matches!(self.backend, Backend::Ready) {
            return;
        }
        let Some(space) = self.active.clone() else { return };
        let target = space
            .read(cx)
            .active_tab_id()
            .zip(space.read(cx).active_layout().map(Workspace::active_pane));
        if let Some((tab, pane)) = target {
            let size = self
                .active_pane_size
                .get()
                .filter(|measured| {
                    measured.space == space.entity_id()
                        && measured.tab == tab
                        && measured.pane == pane
                })
                .map(|measured| measured.size)
                .unwrap_or_else(|| window.viewport_size());
            let direction = split_direction(size);
            space.update(cx, |space, cx| {
                if terminal {
                    space.start_session_dropped(tab, pane, Some(direction), cx);
                } else {
                    space.open_plugin_launcher_dropped(tab, pane, Some(direction), cx);
                }
            });
        } else {
            space.update(cx, |space, cx| {
                if terminal {
                    space.start_session(cx);
                } else {
                    space.open_plugin_launcher(cx);
                }
            });
        }
        cx.notify();
    }

    pub(super) fn ungroup_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Mode::Conversations {
            return;
        }
        if let Some(space) = self.active.clone()
            && let Some(tab) = space.read(cx).active_tab_id()
            && space.read(cx).workspace_tabs().tab(tab).is_some_and(|tab| tab.is_grouped())
        {
            self.act(Action::UngroupPane { space: space.entity_id(), tab }, window, cx);
        }
    }

    pub(super) fn new_free_item(
        &mut self,
        terminal: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(space) =
            self.spaces.iter().find(|space| space.read(cx).kind() == SpaceKind::AdHoc)
        {
            let space = space.entity_id();
            self.act(
                if terminal {
                    Action::NewInSpace { space }
                } else {
                    Action::NewPluginPaneInSpace { space }
                },
                window,
                cx,
            );
        }
    }

    pub(super) fn adjust_zoom(&mut self, terminal: bool, delta: f32, cx: &mut Context<Self>) {
        let current = cx.global::<SettingsStore>().resolved();
        let size = if terminal {
            (current.terminal_font_size + delta).clamp(8., 72.)
        } else {
            (current.ui_font_size + delta).clamp(8., 32.)
        };
        match crate::settings::update_global(cx, |content| {
            if terminal {
                content.terminal.get_or_insert_default().font_size = Some(size);
            } else {
                content.appearance.get_or_insert_default().ui_font_size = Some(size);
            }
        }) {
            Ok(settings) => {
                crate::fonts::install(&settings, cx);
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_splits_follow_the_panes_longer_dimension() {
        assert_eq!(split_direction(gpui::size(px(900.), px(400.))), SplitDirection::Right);
        assert_eq!(split_direction(gpui::size(px(400.), px(900.))), SplitDirection::Down);
        assert_eq!(split_direction(gpui::size(px(400.), px(400.))), SplitDirection::Down);
    }
}
