//! Window layout and semantic action dispatch.

use super::*;

impl Render for Zeddy {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ui_font = Fonts::setup_ui(window, cx);
        self.restore_plugins_once(window, cx);
        let entries = self.entries(cx);
        let now = cx.background_executor().now();
        if self.space_sorter.tick(now, window.rem_size(), cx.reduce_motion()) {
            window.request_animation_frame();
        }
        let (chrome_visibility, mode_animating) =
            self.mode_transition.advance(self.mode, now, cx.reduce_motion());
        if mode_animating {
            window.request_animation_frame();
        }
        let error_notices = self.error_notices(cx);
        let mut sidebar_spaces = self.sidebar_spaces(cx);
        self.space_sorter.arrange(&mut sidebar_spaces, |space| space.id);
        let chrome_entries: &[Entry] = &entries;
        let new_item = self.new_item_button(cx);
        let new_plugin_pane = self.new_plugin_pane_button(cx);
        let (background, text, workspace_background) = {
            let colors = cx.theme().colors();
            (colors.background, colors.text, colors.editor_background)
        };

        let on_action =
            cx.listener(|this, action: &Action, window, cx| this.act(action.clone(), window, cx));
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));
        let title_controls = cfg!(target_os = "macos").then(|| {
            (
                self.visible_space_switcher(window, cx),
                self.chrome_end_controls(emit.clone(), error_notices.clone(), cx),
            )
        });
        let title_bar = self.workspace_title_bar(title_controls, chrome_visibility, window, cx);

        let workspace = v_flex()
            .id("mode-workspace")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_hidden()
            .bg(workspace_background)
            .child(self.workspace_pane(window, cx));

        let tab_height = chrome::tabs::height(cx);
        let reveal_edge_height = px(14.);
        let reveal_edge_strength =
            (tab_height * (1. - chrome_visibility.tabs) / reveal_edge_height).clamp(0., 1.);
        let reveal_edge_color = cx.theme().colors().panel_background;
        // Keep one layout and one workspace subtree throughout the transition.
        // Fixed-size surfaces slide within shrinking slots, so labels never squash.
        let tab_slot = div()
            .id("mode-tab-slot")
            .relative()
            .w_full()
            .h(tab_height * chrome_visibility.tabs)
            .flex_none()
            .overflow_hidden()
            .when(chrome_visibility.tabs > 0., |slot| {
                let controls = (!cfg!(target_os = "macos")).then(|| {
                    (
                        self.visible_space_switcher(window, cx),
                        self.chrome_end_controls(emit.clone(), error_notices.clone(), cx),
                    )
                });
                slot.child(
                    div()
                        .absolute()
                        .top(tab_height * (chrome_visibility.tabs - 1.))
                        .left_0()
                        .w_full()
                        .h(tab_height)
                        .child(chrome::tabs::render(
                            chrome_entries,
                            controls,
                            new_item,
                            new_plugin_pane,
                            emit.clone(),
                            window,
                            cx,
                        )),
                )
            })
            .when(chrome_visibility.tabs > 0. && reveal_edge_strength > 0., |slot| {
                // A short, passive veil softens the actual clipping edge beneath
                // the title bar. Let it recede as the strip settles into view so
                // the resting tabs stay crisp, without fading the whole surface.
                slot.child(
                    gpui::canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| {
                            window.paint_quad(gpui::fill(
                                bounds,
                                gpui::linear_gradient(
                                    180.,
                                    gpui::linear_color_stop(
                                        reveal_edge_color.opacity(reveal_edge_strength),
                                        0.,
                                    ),
                                    gpui::linear_color_stop(reveal_edge_color.opacity(0.), 1.),
                                ),
                            ));
                        },
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(reveal_edge_height),
                )
            });
        let sidebar_slot = div()
            .id("mode-sidebar-slot")
            .relative()
            .w(px(self.sidebar_width * chrome_visibility.sidebar))
            .h_full()
            .flex_none()
            // The settled sidebar's resize handle extends into the workspace.
            .when(chrome_visibility.sidebar < 1., |slot| slot.overflow_hidden())
            .when(chrome_visibility.sidebar > 0., |slot| {
                let controls = (!cfg!(target_os = "macos")).then(|| {
                    (
                        self.visible_space_switcher(window, cx),
                        self.chrome_end_controls(emit.clone(), error_notices.clone(), cx),
                    )
                });
                slot.child(
                    div()
                        .absolute()
                        .left(px(self.sidebar_width * (chrome_visibility.sidebar - 1.)))
                        .top_0()
                        .w(px(self.sidebar_width))
                        .h_full()
                        .child(chrome::sidebar::render(
                            &sidebar_spaces,
                            controls,
                            emit.clone(),
                            &self.space_sorter,
                            self.sidebar_width,
                            window,
                            cx,
                        )),
                )
            });
        let body = v_flex()
            .w_full()
            .flex_1()
            .min_h_0()
            .child(tab_slot)
            .child(h_flex().w_full().flex_1().min_h_0().child(sidebar_slot).child(workspace));

        // Native child webviews sit above their parent window's GPUI scene. Rename dialogs use a
        // window-sized native popup when possible; these in-window overlays are the platform
        // fallback only.
        let rename = self.rename_overlay(cx);

        div()
            .relative()
            .track_focus(&self.focus)
            .key_context(if self.rename_space.is_some() {
                "RenameSpace"
            } else if self.rename_group.is_some() {
                "RenameGroup"
            } else {
                "Chartr"
            })
            .size_full()
            .flex()
            .flex_col()
            .font(ui_font)
            .text_size(UI_TEXT_DEFAULT)
            .bg(background)
            .text_color(text)
            .on_drag_move::<chrome::DraggedSidebar>(cx.listener(
                |this, event: &DragMoveEvent<chrome::DraggedSidebar>, _, cx| {
                    this.sidebar_width = (event.event.position.x / px(1.))
                        .clamp(chrome::sidebar::MIN_WIDTH, chrome::sidebar::MAX_WIDTH);
                    cx.notify();
                },
            ))
            .on_drag_move::<chrome::DraggedSpace>(cx.listener(
                |this, event: &DragMoveEvent<chrome::DraggedSpace>, window, cx| {
                    let dragged = event.drag(cx).0;
                    let order = this
                        .spaces
                        .iter()
                        .filter(|space| space.read(cx).kind() != SpaceKind::AdHoc)
                        .map(|space| space.entity_id())
                        .collect();
                    if this.space_sorter.drag_move(
                        dragged,
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
                    this.finish_space_drag(event.position.y, window, cx)
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    this.finish_space_drag(event.position.y, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &actions::pane::CloseActiveItem, _, cx| {
                this.close_active_item(cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::CloseAllItems, window, cx| {
                this.request_close_active_pane(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveLeft, _, cx| {
                this.move_active_to_pane(SplitDirection::Left, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveRight, _, cx| {
                this.move_active_to_pane(SplitDirection::Right, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveUp, _, cx| {
                this.move_active_to_pane(SplitDirection::Up, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::MoveDown, _, cx| {
                this.move_active_to_pane(SplitDirection::Down, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::JoinIntoNext, _, cx| {
                this.join_active_into_next(cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::ActivatePaneLeft, window, cx| {
                this.activate_pane_in_direction(SplitDirection::Left, window, cx)
            }))
            .on_action(cx.listener(
                |this, _: &actions::workspace::ActivatePaneRight, window, cx| {
                    this.activate_pane_in_direction(SplitDirection::Right, window, cx)
                },
            ))
            .on_action(cx.listener(|this, _: &actions::workspace::ActivatePaneUp, window, cx| {
                this.activate_pane_in_direction(SplitDirection::Up, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::ActivatePaneDown, window, cx| {
                this.activate_pane_in_direction(SplitDirection::Down, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewTerminal, window, cx| {
                this.act(Action::New, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewTerminalPane, window, cx| {
                this.new_adaptive_pane(true, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewSurface, window, cx| {
                this.act(Action::NewPluginPane, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewSurfacePane, window, cx| {
                this.new_adaptive_pane(false, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::Ungroup, window, cx| {
                this.ungroup_current(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::SidebarMode, _, cx| {
                this.settings_set_mode(Mode::Sidebar, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::TabbedMode, _, cx| {
                this.settings_set_mode(Mode::Tabs, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::CycleViewMode, _, cx| {
                this.settings_set_mode(
                    match this.mode {
                        Mode::Tabs => Mode::Sidebar,
                        Mode::Sidebar => Mode::Tabs,
                    },
                    cx,
                )
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewSpace, window, cx| {
                this.act(Action::NewSpace, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::CloseSpace, window, cx| {
                if let Some(space) = this.active.as_ref() {
                    this.request_close_space(space.entity_id(), window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::ZoomIn, _, cx| {
                this.adjust_zoom(false, 1., cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::ZoomOut, _, cx| {
                this.adjust_zoom(false, -1., cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::TerminalZoomIn, _, cx| {
                this.adjust_zoom(true, 1., cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::TerminalZoomOut, _, cx| {
                this.adjust_zoom(true, -1., cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewFreeTerminal, window, cx| {
                this.new_free_item(true, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::workspace::NewFreeSurface, window, cx| {
                this.new_free_item(false, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::settings::Open, window, cx| {
                this.open_settings(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::command_palette::Toggle, window, cx| {
                this.toggle_command_palette(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::terminal_search::Toggle, window, cx| {
                this.toggle_terminal_search(window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::terminal_search::Next, _, cx| {
                this.navigate_terminal_search(true, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::terminal_search::Previous, _, cx| {
                this.navigate_terminal_search(false, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::terminal_search::Close, window, cx| {
                this.close_terminal_search(window, cx)
            }))
            .on_key_down(cx.listener(|this, event, window, cx| this.on_key(event, window, cx)))
            .child(title_bar)
            .child(body)
            .children(rename)
    }
}
