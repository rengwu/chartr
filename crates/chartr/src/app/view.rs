//! Window layout and semantic action dispatch.

use super::*;

const WORKSPACE_INSET: f32 = 4.;
const WORKSPACE_RADIUS: f32 = 12.;

impl Render for WorkspaceWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ui_font = Fonts::setup_ui(window, cx);
        self.restore_plugins_once(window, cx);
        self.sync_conversation_spaces(cx);
        if self.mode == Mode::Inbox {
            self.sync_inbox_terminal(cx);
        }
        let entries = self.entries(cx);
        let now = cx.background_executor().now();
        if self.space_sorter.tick(now, window.rem_size(), cx.reduce_motion()) {
            window.request_animation_frame();
        }
        let (chrome_visibility, mode_animating) =
            self.mode_transition.advance(self.mode, now, cx.reduce_motion());
        let (sidebar_width, sidebar_animating) = self.sidebar.advance(now, cx.reduce_motion());
        let (sidebar_position, sidebar_switching) = self.mode_transition.sidebar_position(now);
        if mode_animating || sidebar_animating || sidebar_switching {
            window.request_animation_frame();
        }
        // The pane keeps sliding, but each terminal reflows only once, after
        // both the chrome transition and any Inbox width expansion settle.
        let terminal_views: Vec<_> = self
            .spaces
            .iter()
            .flat_map(|space| {
                let space = space.read(cx);
                space.all_item_ids().into_iter().filter_map(move |id| {
                    space.item(id).and_then(crate::item::Item::as_session)?.terminal_view()
                })
            })
            .collect();
        for view in terminal_views {
            view.update(cx, |view, cx| {
                view.set_resize_paused(mode_animating || sidebar_animating, cx);
            });
        }
        let error_notices = self.error_notices(cx);
        let mut sidebar_spaces = self.sidebar_spaces(cx);
        self.space_sorter.arrange(&mut sidebar_spaces, |space| space.id);
        let chrome_entries: &[Entry] = &entries;
        let new_item = self.new_item_button(cx);
        let new_plugin_pane = self.new_plugin_pane_button(cx);
        let (background, text, workspace_background, chrome_background) = {
            let colors = cx.theme().colors();
            (
                colors.background,
                colors.text,
                colors.editor_background,
                colors.background.blend(colors.panel_background),
            )
        };

        let on_action =
            cx.listener(|this, action: &Action, window, cx| this.act(action.clone(), window, cx));
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));
        let title_controls = cfg!(target_os = "macos").then(|| {
            let space_switcher_width = if chrome_visibility.tabs > 0. {
                window_chrome::chrome_controls_width(
                    self.visible_space_switcher(chrome_visibility.tabs, window, cx),
                    window,
                    cx,
                )
                .min(px(window_chrome::SPACE_SWITCHER_MAX_WIDTH))
                    + window.rem_size() * 0.25
            } else {
                px(0.)
            };
            let (end_controls, show_brand) = self.title_bar_end_controls(
                emit.clone(),
                error_notices.clone(),
                window.viewport_size().width
                    - px(window_chrome::TITLE_CONTROLS_LEFT + window_chrome::TITLE_CONTROLS_RIGHT)
                    - space_switcher_width,
                window,
                cx,
            );
            (
                self.visible_space_switcher(chrome_visibility.tabs, window, cx),
                end_controls,
                show_brand,
            )
        });
        let (title_bar, title_bar_foreground) =
            self.workspace_title_bar(title_controls, chrome_visibility, window, cx);
        let (app_bar, app_bar_foreground) = cfg!(target_os = "linux")
            .then(|| {
                self.linux_app_bar(
                    emit.clone(),
                    error_notices.clone(),
                    chrome_visibility,
                    window,
                    cx,
                )
            })
            .unzip();

        let has_pane_headers = self.mode != Mode::Inbox
            && self.active.as_ref().is_some_and(|space| {
                space.read(cx).workspace_tabs().active_tab().is_some_and(|tab| tab.is_grouped())
            });
        let workspace = v_flex()
            .id("mode-workspace")
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            // Tabs expose the left window edge; reveal its gutter with the strip.
            .pl(px(WORKSPACE_INSET * chrome_visibility.tabs))
            .pr(px(WORKSPACE_INSET))
            .pb(px(WORKSPACE_INSET))
            .bg(chrome_background)
            .child(
                v_flex()
                    .relative()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .p_px()
                    .rounded(px(WORKSPACE_RADIUS))
                    .overflow_hidden()
                    .bg(workspace_background)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            // Collapse the first tab row's top edge onto the frame.
                            .when(has_pane_headers, |content| content.mt(px(-1.)))
                            .child(if self.mode == Mode::Inbox {
                                self.conversations.clone().into_any_element()
                            } else {
                                self.workspace_pane(window, cx)
                            })
                            .when(self.mode == Mode::Inbox, |content| {
                                content.children(self.terminal_search_overlay(cx))
                            }),
                    )
                    .child(workspace_frame(chrome_background, cx.theme().colors().border)),
            );

        let tab_height = chrome::tabs::height(cx);
        // Keep one layout and one workspace subtree throughout the transition.
        // Fixed-size surfaces slide as their layout slots shrink, so labels never
        // squash. Tabs overflow upward beneath the title bar gradient and controls.
        let tab_slot = div()
            .id("mode-tab-slot")
            .relative()
            .w_full()
            .h(tab_height * chrome_visibility.tabs)
            .flex_none()
            .when(chrome_visibility.tabs > 0., |slot| {
                let controls = (!cfg!(any(target_os = "macos", target_os = "linux"))).then(|| {
                    (
                        self.visible_space_switcher(1., window, cx),
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
                        // Leave a visible trail after the quick initial slide,
                        // then dissolve it completely before removing the strip.
                        .opacity(chrome_visibility.tabs.sqrt())
                        .child(chrome::tabs::render(
                            chrome_entries,
                            controls,
                            new_item,
                            new_plugin_pane,
                            emit.clone(),
                            window,
                            cx,
                        ))
                        .when(self.mode != Mode::Tabs, |strip| {
                            // The outgoing strip is visual only. Cover its full
                            // moving bounds to block clicks, drags, hover and scroll
                            // immediately, while title-bar controls stay above it.
                            strip.child(div().absolute().inset_0().occlude())
                        }),
                )
            });
        let sidebar_slot = div()
            .id("mode-sidebar-slot")
            .relative()
            .w(px(sidebar_width * chrome_visibility.sidebar))
            .h_full()
            .flex_none()
            // The settled sidebar's resize handle extends into the workspace.
            .when(chrome_visibility.sidebar < 1., |slot| slot.overflow_hidden())
            .when(chrome_visibility.sidebar > 0., |slot| {
                let controls = (!cfg!(any(target_os = "macos", target_os = "linux"))).then(|| {
                    (
                        gpui::Empty.into_any_element(),
                        self.chrome_end_controls(emit.clone(), error_notices.clone(), cx),
                    )
                });
                // Both pages keep their full width while sliding through the
                // same clipped viewport. Spaces is left of Chats; controls and
                // the resize handle stay fixed outside this moving content.
                let sidebar_page_stride = sidebar_width + 32.;
                let contents = div()
                    .relative()
                    .size_full()
                    .overflow_hidden()
                    .when(sidebar_position < 1., |pages| {
                        pages.child(
                            div()
                                .id("sidebar-spaces-page")
                                .absolute()
                                .top_0()
                                .left(px(-sidebar_page_stride * sidebar_position))
                                .w(px(sidebar_width))
                                .h_full()
                                .child(chrome::sidebar::render(
                                    &sidebar_spaces,
                                    emit.clone(),
                                    &self.space_sorter,
                                    window,
                                    cx,
                                ))
                                .when(self.mode != Mode::Sidebar, |page| {
                                    page.child(div().absolute().inset_0().occlude())
                                }),
                        )
                    })
                    .when(sidebar_position > 0., |pages| {
                        pages.child(
                            div()
                                .id("sidebar-chats-page")
                                .absolute()
                                .top_0()
                                .left(px(sidebar_page_stride * (1. - sidebar_position)))
                                .w(px(sidebar_width))
                                .h_full()
                                .child(
                                    self.conversations
                                        .update(cx, |inbox, cx| inbox.render_sidebar(window, cx)),
                                )
                                .when(self.mode != Mode::Inbox, |page| {
                                    page.child(div().absolute().inset_0().occlude())
                                }),
                        )
                    })
                    .into_any_element();
                slot.child(
                    div()
                        .absolute()
                        .left(px(sidebar_width * (chrome_visibility.sidebar - 1.)))
                        .top_0()
                        .w(px(sidebar_width))
                        .h_full()
                        .child(chrome::sidebar_pane::render(sidebar_width, controls, contents, cx)),
                )
            });
        let body = v_flex()
            .w_full()
            .flex_1()
            .min_h_0()
            .child(tab_slot)
            .child(h_flex().w_full().flex_1().min_h_0().child(sidebar_slot).child(workspace));

        if self.mode_focus_pending {
            self.mode_focus_pending = false;
            self.focus_active_terminal(window, cx);
        }

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
                "chartr"
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
                    this.sidebar.resize(event.event.position.x / px(1.), this.mode);
                    cx.notify();
                },
            ))
            .on_drag_move::<chrome::DraggedSpace>(cx.listener(
                |this, event: &DragMoveEvent<chrome::DraggedSpace>, window, cx| {
                    let dragged = event.drag(cx).0;
                    let order = this.spaces.iter().map(|space| space.entity_id()).collect();
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
            .on_action(cx.listener(|this, _: &actions::workspace::CloseAllSessions, window, cx| {
                this.request_close_all_sessions(window, cx)
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
            .on_action(cx.listener(|this, _: &actions::workspace::RenameTerminal, window, cx| {
                this.rename_active_terminal(window, cx)
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
            .on_action(cx.listener(|this, _: &actions::workspace::ToggleStatusBar, _, cx| {
                this.toggle_status_bar(cx);
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
            .children(app_bar)
            .when(self.state_restore_problem.is_some(), |view| {
                view.child(
                    div().w_full().flex_none().child(
                        Banner::new().severity(Severity::Warning).child(
                            Label::new(
                                "Workspace couldn’t be restored. Layout changes won’t be saved this time.",
                            )
                            .size(UI_LABEL_DEFAULT),
                        ),
                    ),
                )
            })
            .child(body)
            .when(self.settings.resolved().show_status_bar, |view| view.child(self.status_bar(cx)))
            .child(title_bar_foreground)
            .children(app_bar_foreground)
            .children(rename)
    }
}

fn workspace_frame(background: gpui::Hsla, border: gpui::Hsla) -> impl IntoElement {
    gpui::canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let radius =
                px(WORKSPACE_RADIUS).min(bounds.size.width / 2.).min(bounds.size.height / 2.);
            // GPUI overflow masks are rectangular. Paint the outside of the
            // rounded frame over child backgrounds before drawing its outline.
            // The expanded border's inner edge is exactly the desired curve.
            window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                window.paint_quad(gpui::quad(
                    bounds.dilate(radius),
                    radius * 2.,
                    gpui::transparent_black(),
                    radius,
                    background,
                    gpui::BorderStyle::default(),
                ));
                window.paint_quad(
                    gpui::outline(bounds, border, gpui::BorderStyle::default())
                        .corner_radii(radius),
                );
            });
        },
    )
    .absolute()
    .inset_0()
}
