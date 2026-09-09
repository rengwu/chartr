//! Pane layout, headers, drag and drop, and resize rendering.

use super::*;
use crate::chrome::tab_sorter::{SortableTab, SortableTabList};
use crate::components::SortAxis;

impl WorkspaceWindow {
    fn pane_new_item_button(
        &self,
        tab_id: WorkspaceTabId,
        pane_id: LayoutPaneId,
        weak: &gpui::WeakEntity<Self>,
    ) -> AnyElement {
        let button_id = format!("new-item-pane-{}-{}", tab_id.get(), pane_id.get());
        let start = weak.clone();
        let button = chrome::new_item_button(button_id.clone())
            .tooltip(Tooltip::text("New terminal session"))
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                let _ = start.update(cx, |this, cx| {
                    if matches!(this.backend, Backend::Ready)
                        && let Some(space) = this.active.clone()
                    {
                        space.update(cx, |space, cx| space.start_session_in(tab_id, pane_id, cx));
                    }
                });
            })
            .into_any_element();
        chrome::new_item_drag_handle(
            button_id,
            self.active.as_ref().map(Entity::entity_id),
            chrome::NewItemKind::Terminal,
            button,
        )
    }

    fn pane_new_plugin_button(
        &self,
        tab_id: WorkspaceTabId,
        pane_id: LayoutPaneId,
        weak: &gpui::WeakEntity<Self>,
    ) -> AnyElement {
        let button_id = format!("new-plugin-pane-{}-{}", tab_id.get(), pane_id.get());
        let open = weak.clone();
        let button = chrome::new_plugin_pane_button(button_id.clone(), IconSize::Small)
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                let _ = open.update(cx, |this, cx| {
                    if let Some(space) = this.active.clone() {
                        space.update(cx, |space, cx| {
                            space.open_plugin_launcher_in(tab_id, pane_id, cx);
                        });
                    }
                });
            })
            .into_any_element();
        chrome::new_item_drag_handle(
            button_id,
            self.active.as_ref().map(Entity::entity_id),
            chrome::NewItemKind::Plugin,
            button,
        )
    }

    fn pane_new_item_cell(
        &self,
        tab_id: WorkspaceTabId,
        pane_id: LayoutPaneId,
        weak: &gpui::WeakEntity<Self>,
        cx: &App,
    ) -> AnyElement {
        chrome::new_item_cell(
            h_flex()
                .gap_px()
                .child(self.pane_new_item_button(tab_id, pane_id, weak))
                .child(self.pane_new_plugin_button(tab_id, pane_id, weak)),
            cx,
        )
    }

    fn handle_new_item_drop(
        &mut self,
        dragged: &chrome::DraggedNewItem,
        tab: WorkspaceTabId,
        pane: LayoutPaneId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(space) = self.active.clone().filter(|space| space.entity_id() == dragged.space)
        else {
            return;
        };
        let terminal_ready = matches!(self.backend, Backend::Ready);
        space.update(cx, |space, cx| {
            let direction = space
                .drag_target()
                .filter(|(target_tab, target_pane, _)| *target_tab == tab && *target_pane == pane)
                .and_then(|(_, _, direction)| direction);
            space.clear_drag_target();
            if space.workspace_tabs().workspace(tab).and_then(|layout| layout.pane(pane)).is_none()
            {
                return;
            }
            match dragged.kind {
                chrome::NewItemKind::Terminal if terminal_ready => {
                    space.start_session_dropped(tab, pane, direction, cx);
                }
                chrome::NewItemKind::Plugin => {
                    space.open_plugin_launcher_dropped(tab, pane, direction, cx);
                }
                _ => {}
            }
        });
        self.focus_active_terminal(window, cx);
        cx.notify();
    }

    /// Zed has one pane drop path shared by tab targets and the pane body.
    /// Body drops may consume the current edge split direction; tab-bar drops
    /// explicitly clear it and only reorder or move into the target pane.
    fn handle_item_drop(
        &mut self,
        dragged: &DraggedItem,
        target_tab: WorkspaceTabId,
        target: LayoutPaneId,
        index: usize,
        allow_split: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(space) = self.active.clone() else {
            return;
        };
        if dragged.grouped {
            space.update(cx, |space, _| {
                space.clear_drag_target();
            });
            cx.notify();
            return;
        }
        if space.read(cx).key() != dragged.space {
            space.update(cx, |space, _| {
                space.clear_drag_target();
            });
            cx.notify();
            return;
        }
        if !allow_split {
            space.update(cx, |space, _| space.set_drag_target(target_tab, target, None));
        }
        let clone = cfg!(target_os = "macos") && window.modifiers().alt
            || cfg!(not(target_os = "macos")) && window.modifiers().control;
        if clone
            && self.clone_plugin_drop(
                space.clone(),
                dragged.item,
                target_tab,
                target,
                Some(index),
                window,
                cx,
            )
        {
            cx.notify();
            return;
        }
        space.update(cx, |space, _| {
            space.drop_item(
                dragged.item,
                dragged.tab,
                dragged.pane,
                target_tab,
                target,
                Some(index),
            );
        });
        cx.notify();
    }

    pub(super) fn workspace_pane(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(space) = self.active.clone() else {
            return message("No space. Add a folder to begin.", cx).into_any_element();
        };
        let active = space.read(cx).active();
        let on_action =
            cx.listener(|this, action: &Action, window, cx| this.act(action.clone(), window, cx));
        let emit: chrome::Emit = Rc::new(move |action, window, cx| on_action(&action, window, cx));
        let space = space.read(cx);
        if active.is_some_and(|active| space.item(active).is_none()) {
            return message("That item is gone.", cx).into_any_element();
        }
        let weak = cx.weak_entity();
        let workspace = if let Some(tab) = space.workspace_tabs().active_tab() {
            let layout = &tab.layout;
            let show_pane_headers = tab.is_grouped();
            self.render_member(
                &space,
                tab.id,
                layout,
                &layout.center.root,
                show_pane_headers,
                &emit,
                &weak,
                &[],
                window,
                cx,
            )
        } else {
            message("No tabs. Create a new item to begin.", cx).into_any_element()
        };
        let terminal_search = self.terminal_search_overlay(cx);
        v_flex()
            .relative()
            .size_full()
            .min_h_0()
            .child(div().flex_1().min_h_0().child(workspace))
            .child(self.pane_drop_preview.overlay())
            .children(terminal_search)
            .into_any_element()
    }

    fn render_member(
        &self,
        space: &Space,
        tab_id: WorkspaceTabId,
        layout: &Workspace,
        member: &Member,
        show_pane_headers: bool,
        on: &chrome::Emit,
        weak: &gpui::WeakEntity<Self>,
        axis_path: &[usize],
        window: &mut Window,
        cx: &App,
    ) -> AnyElement {
        match member {
            Member::Pane { pane } => self.render_pane(
                space,
                tab_id,
                layout,
                *pane,
                show_pane_headers,
                on,
                weak,
                window,
                cx,
            ),
            Member::Axis(axis) => {
                let member_count = axis.members.len();
                let children: Vec<_> = axis
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, member)| {
                        let flex = axis.flexes.get(index).copied().unwrap_or(1.).max(0.01);
                        let mut child_path = axis_path.to_vec();
                        child_path.push(index);
                        let divider = DraggedPaneDivider {
                            axis_path: axis_path.to_vec(),
                            divider: index,
                            axis: axis.axis,
                        };
                        div()
                            .relative()
                            .flex_grow(flex)
                            .flex_basis(relative(0.))
                            .min_w_0()
                            .min_h_0()
                            .child(self.render_member(
                                space,
                                tab_id,
                                layout,
                                member,
                                show_pane_headers,
                                on,
                                weak,
                                &child_path,
                                window,
                                cx,
                            ))
                            .when(index + 1 < member_count, |child| {
                                child.child(pane_resize_handle(divider, axis.axis))
                            })
                    })
                    .collect();
                let resize = weak.clone();
                let current_path = axis_path.to_vec();
                let axis_direction = axis.axis;
                let horizontal = axis_direction == PaneAxisDirection::Horizontal;
                let container = if horizontal { h_flex().items_stretch() } else { v_flex() };
                container
                    .id(format!(
                        "pane-axis-{}-{current_path:?}",
                        if horizontal { "h" } else { "v" }
                    ))
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .gap_px()
                    .bg(cx.theme().colors().border)
                    .on_drag_move::<DraggedPaneDivider>(move |event, _, cx| {
                        let dragged = event.drag(cx).clone();
                        if dragged.axis_path != current_path || dragged.axis != axis_direction {
                            return;
                        }
                        let fraction = if horizontal {
                            (event.event.position.x - event.bounds.left()) / event.bounds.size.width
                        } else {
                            (event.event.position.y - event.bounds.top()) / event.bounds.size.height
                        };
                        let _ = resize.update(cx, |this, cx| {
                            if let Some(space) = this.active.clone() {
                                space.update(cx, |space, _| {
                                    space.resize_divider(
                                        tab_id,
                                        &dragged.axis_path,
                                        dragged.divider,
                                        fraction,
                                    )
                                });
                            }
                            cx.notify();
                        });
                    })
                    .children(children)
                    .into_any_element()
            }
        }
    }

    fn render_pane(
        &self,
        space: &Space,
        tab_id: WorkspaceTabId,
        layout: &Workspace,
        pane_id: LayoutPaneId,
        show_header: bool,
        on: &chrome::Emit,
        weak: &gpui::WeakEntity<Self>,
        _window: &mut Window,
        cx: &App,
    ) -> AnyElement {
        let Some(pane) = layout.pane(pane_id) else {
            return message("Pane layout is unavailable.", cx).into_any_element();
        };
        let active_pane = layout.active_pane() == pane_id;
        let measured_size = self.active_pane_size.clone();
        let measured_space = self.active.as_ref().map(Entity::entity_id);
        let header = show_header.then(|| {
            if pane.active().is_some() {
                self.pane_header(space, tab_id, layout, pane_id, on, weak, cx)
            } else {
                self.empty_pane_header(tab_id, pane_id, weak, cx)
            }
        });
        let content = pane
            .active()
            .and_then(|id| space.item(id).map(|item| (id, item)))
            .map(|(id, item)| match item {
                crate::item::Item::Session(item) => {
                    if let Some(lease) = self.companion_leases.get(&item.session.id().0) {
                        // Do not mount TerminalView while mobile owns geometry: its
                        // prepaint always resizes the PTY to the desktop pane.
                        let content = item.session.terminal().read(cx).get_content();
                        return v_flex()
                            .size_full()
                            .p_3()
                            .gap_2()
                            .overflow_hidden()
                            .child(
                                Label::new(format!(
                                    "Viewing on mobile · {} × {}",
                                    lease.columns, lease.rows
                                ))
                                .color(Color::Muted),
                            )
                            .child(div().font_family("monospace").text_size(px(13.)).child(content))
                            .into_any_element();
                    }
                    let Some(terminal_view) = item.terminal_view() else {
                        return message("Starting terminal…", cx).into_any_element();
                    };
                    let terminal = crate::terminal_host::element(
                        terminal_view,
                        cx.theme().colors().terminal_background,
                    );
                    let ended = item.session.ended();
                    let retrying = space.reattaching(id);
                    let retry = weak.clone();
                    v_flex()
                        .relative()
                        .size_full()
                        .child(terminal)
                        .when_some(ended, |view, ended| {
                            let detail = match &ended {
                                crate::session::Ended::Closed => {
                                    "Session ended. Close this tab when you are done reviewing it."
                                        .to_owned()
                                }
                            };
                            view.child(
                                div().absolute().left_2().right_2().bottom_2().child(
                                    Banner::new()
                                        .severity(Severity::Error)
                                        .child(Label::new(detail).size(UI_LABEL_DEFAULT))
                                        .action_slot(
                                            Button::new(
                                                format!("reattach-session-{}", id.get()),
                                                if retrying {
                                                    "Reattaching…"
                                                } else {
                                                    "Reattach"
                                                },
                                            )
                                            .disabled(retrying)
                                            .on_click(
                                                move |_, _, cx| {
                                                    let _ = retry.update(cx, |this, cx| {
                                                        if let Some(space) = this.active.clone() {
                                                            space.update(cx, |space, cx| {
                                                                space.reattach(id, cx)
                                                            });
                                                        }
                                                    });
                                                },
                                            ),
                                        ),
                                ),
                            )
                        })
                        .into_any_element()
                }
                crate::item::Item::Plugin(item) => item.view.clone_view().into_any_element(),
                crate::item::Item::PluginLauncher { .. } => self.plugin_launcher(id, weak, cx),
            })
            .unwrap_or_else(|| {
                empty_pane_message("Drop a tab here or create a new item.", cx).into_any_element()
            });

        let drag_move = weak.clone();
        let drop_item = weak.clone();
        let drag_new = weak.clone();
        let drop_new = weak.clone();
        let target_space = self.active.as_ref().expect("rendering the active space").entity_id();
        let focus_pane = weak.clone();
        let drop_group = format!("workspace-tab-{}-pane-drop-{}", tab_id.get(), pane_id.get());
        let drop_space = space.key();
        let drag_space = drop_space.clone();
        let pane_drop_index = pane
            .active()
            .and_then(|active| pane.items().iter().position(|item| *item == active))
            .unwrap_or(pane.items().len());
        let pane_is_empty = pane.active().is_none();
        let drop_direction = space
            .drag_target()
            .filter(|(tab, pane, _)| *tab == tab_id && *pane == pane_id)
            .and_then(|(_, _, direction)| direction);
        v_flex()
            .id(format!("workspace-tab-{}-pane-{}", tab_id.get(), pane_id.get()))
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(cx.theme().colors().editor_background)
            .when(active_pane, |view| {
                view.child(
                    gpui::canvas(
                        move |bounds, _, _| {
                            if let Some(space) = measured_space {
                                measured_size.set(Some(super::shortcuts::MeasuredPane {
                                    space,
                                    tab: tab_id,
                                    pane: pane_id,
                                    size: bounds.size,
                                }));
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
            })
            .when(pane.active().is_none() && active_pane, |pane| {
                pane.role(Role::Group).aria_label("Empty pane").tab_group().tab_index(0)
            })
            .on_any_mouse_down(move |_, window, cx| {
                let _ = focus_pane.update(cx, |this, cx| {
                    if let Some(space) = this.active.clone() {
                        space.update(cx, |space, _| space.activate_pane(tab_id, pane_id));
                    }
                    if pane_is_empty {
                        window.focus(&this.focus, cx);
                    }
                    cx.notify();
                });
            })
            .children(header)
            .child(
                div()
                    .flex_1()
                    .relative()
                    .min_h_0()
                    .min_w_0()
                    .group(drop_group.clone())
                    .on_drag_move::<DraggedItem>(move |event, _, cx| {
                        let Some(direction) = pane_drop_direction_for_drag(event) else {
                            // GPUI dispatches drag-move callbacks during capture even when the
                            // pointer is outside this element. Zed keeps split intent on each
                            // Pane entity; chartr's shared space state must therefore ignore
                            // callbacks from every pane except the one under the pointer.
                            return;
                        };
                        let dragged = event.drag(cx);
                        let accepted = !dragged.grouped && dragged.space == drag_space;
                        let _ = drag_move.update(cx, |this, cx| {
                            let changed = this.active.clone().is_some_and(|space| {
                                space.update(cx, |space, _| {
                                    if accepted {
                                        space.set_drag_target(tab_id, pane_id, direction)
                                    } else {
                                        space.clear_drag_target()
                                    }
                                })
                            });
                            if changed {
                                cx.notify();
                            }
                        });
                    })
                    .on_drag_move::<chrome::DraggedNewItem>(move |event, _, cx| {
                        let Some(direction) = pane_drop_direction_for_drag(event) else {
                            return;
                        };
                        let accepted = event.drag(cx).space == target_space;
                        let _ = drag_new.update(cx, |this, cx| {
                            if let Some(space) = this.active.clone() {
                                let changed = space.update(cx, |space, _| {
                                    if accepted {
                                        space.set_drag_target(tab_id, pane_id, direction)
                                    } else {
                                        space.clear_drag_target()
                                    }
                                });
                                if changed {
                                    cx.notify();
                                }
                            }
                        });
                    })
                    .child(content)
                    .child(
                        drop_target(
                            drop_direction,
                            drop_group,
                            drop_space,
                            target_space,
                            &self.pane_drop_preview,
                        )
                        .on_drop(move |dragged: &chrome::DraggedNewItem, window, cx| {
                            let _ = drop_new.update(cx, |this, cx| {
                                this.handle_new_item_drop(dragged, tab_id, pane_id, window, cx);
                            });
                        })
                        .on_drop(
                            move |dragged: &DraggedItem, window, cx| {
                                let dragged = dragged.clone();
                                let _ = drop_item.update(cx, |this, cx| {
                                    this.handle_item_drop(
                                        &dragged,
                                        tab_id,
                                        pane_id,
                                        pane_drop_index,
                                        true,
                                        window,
                                        cx,
                                    );
                                });
                            },
                        ),
                    ),
            )
            .into_any_element()
    }

    fn empty_pane_header(
        &self,
        tab_id: WorkspaceTabId,
        pane_id: LayoutPaneId,
        weak: &gpui::WeakEntity<Self>,
        cx: &App,
    ) -> AnyElement {
        let close = weak.clone();
        TabBar::new(format!("workspace-tab-{}-pane-{}-empty", tab_id.get(), pane_id.get()))
            .child(self.pane_new_item_cell(tab_id, pane_id, weak, cx))
            .child(div().h_full().flex_grow_1())
            .end_child(
                IconButton::new(
                    format!("close-empty-pane-{}-{}", tab_id.get(), pane_id.get()),
                    IconName::Close,
                )
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .icon_size(IconSize::XSmall)
                .aria_label("Close Empty Pane")
                .tooltip(Tooltip::text("Close Empty Pane"))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    let _ = close.update(cx, |this, cx| {
                        if let Some(space) = this.active.clone() {
                            space.update(cx, |space, _| space.remove_empty_pane(tab_id, pane_id));
                        }
                        cx.notify();
                    });
                }),
            )
            .into_any_element()
    }

    fn pane_header(
        &self,
        space: &Space,
        tab_id: WorkspaceTabId,
        layout: &Workspace,
        pane_id: LayoutPaneId,
        on: &chrome::Emit,
        weak: &gpui::WeakEntity<Self>,
        cx: &App,
    ) -> AnyElement {
        let Some(pane) = layout.pane(pane_id) else {
            return div().into_any_element();
        };

        let active_index =
            pane.active().and_then(|active| pane.items().iter().position(|item| *item == active));
        let space_key = space.key();
        let middle_click_closes_tab = self.settings.resolved().middle_click_closes_tab;
        let tabs = pane
            .items()
            .iter()
            .enumerate()
            .filter_map(|(index, id)| {
                let item = space.item(*id)?;
                let selected = pane.active() == Some(*id);
                let status = item.status();
                let process_running = item.process_running();
                let ended = item.ended();
                let bell = item.as_session().is_some_and(crate::item::SessionItem::bell);
                let position = chrome::tab_position(index, pane.items().len(), active_index);
                let select = *id;
                let close = *id;
                let select_item = on.clone();
                let close_item = on.clone();
                let middle_close_item = on.clone();
                let drop_item = weak.clone();
                let drop_space = space_key.clone();
                let dragged = DraggedItem {
                    space: space_key.clone(),
                    tab: tab_id,
                    pane: pane_id,
                    index,
                    item: *id,
                    top_level: false,
                    grouped: false,
                };
                let close_slot = IconButton::new(
                    format!("close-pane-{}-item-{}", pane_id.get(), id.get()),
                    IconName::Close,
                )
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .icon_size(IconSize::XSmall)
                .tooltip(Tooltip::text("Close"))
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    close_item(Action::Close { space: None, item: close }, window, cx)
                })
                .into_any_element();
                let tab = chrome::ItemTab::new(
                    format!("pane-{}-item-{}", pane_id.get(), id.get()),
                    item.title(),
                    selected,
                    position,
                    &space_key,
                    *id,
                )
                .activity(chrome::Activity { status, process_running, ended, bell })
                .icon_path(item.icon_path())
                .close_slot(Some(close_slot))
                .build(cx)
                .on_click(move |_, window, cx| {
                    select_item(Action::Select { space: None, item: select }, window, cx)
                })
                .when(middle_click_closes_tab, |tab| {
                    tab.on_aux_click(move |event, window, cx| {
                        if event.is_middle_click() {
                            cx.stop_propagation();
                            middle_close_item(
                                Action::Close { space: None, item: close },
                                window,
                                cx,
                            );
                        }
                    })
                })
                .can_drop(move |value, _, _| {
                    value
                        .downcast_ref::<DraggedItem>()
                        .is_some_and(|dragged| !dragged.grouped && dragged.space == drop_space)
                })
                .drag_over::<DraggedItem>(move |tab, dragged, _, cx| {
                    if !dragged.top_level && dragged.tab == tab_id && dragged.pane == pane_id {
                        return tab;
                    }
                    let mut tab = tab
                        .bg(cx.theme().colors().drop_target_background)
                        .border_color(cx.theme().colors().drop_target_border)
                        .border_0();
                    if index < dragged.index {
                        tab = tab.border_l_2();
                    } else if index > dragged.index {
                        tab = tab.border_r_2();
                    }
                    tab
                })
                .on_drop(move |dragged: &DraggedItem, window, cx| {
                    let dragged = dragged.clone();
                    let _ = drop_item.update(cx, |this, cx| {
                        this.handle_item_drop(&dragged, tab_id, pane_id, index, false, window, cx);
                    });
                });
                Some(SortableTab::new(dragged, selected, move |placement, _| {
                    tab.position(chrome::tab_position(
                        placement.index,
                        placement.count,
                        placement.active_index,
                    ))
                    .into_any_element()
                }))
            })
            .collect();
        let append_drop = weak.clone();
        let append_index = pane.items().len();
        let append_space = space_key.clone();
        let sort_drop = weak.clone();
        let sorted_tabs = SortableTabList::new(
            format!("pane-tab-sorter-{space_key}-{}-{}", tab_id.get(), pane_id.get()),
            h_flex()
                .id(format!("pane-{}-tab-list", pane_id.get()))
                .min_w_0()
                .flex_shrink_1()
                .overflow_x_scroll(),
            SortAxis::Horizontal,
            gpui::rems(0.),
            tabs,
            move |dragged, index, window, cx| {
                let _ = sort_drop.update(cx, |this, cx| {
                    this.handle_item_drop(dragged, tab_id, pane_id, index, false, window, cx);
                });
            },
        );
        let sorted_tabs = sorted_tabs.tab_min_width(chrome::ItemTab::min_width(false, cx));
        let drag_lane = h_flex()
            .id(format!("pane-{}-tab-bar-drop-target", pane_id.get()))
            .w_full()
            .min_w_0()
            .h_full()
            .can_drop(move |value, _, _| {
                value
                    .downcast_ref::<DraggedItem>()
                    .is_some_and(|dragged| !dragged.grouped && dragged.space == append_space)
            })
            .drag_over::<DraggedItem>(move |bar, dragged, _, cx| {
                if !dragged.top_level && dragged.tab == tab_id && dragged.pane == pane_id {
                    bar
                } else {
                    bar.bg(cx.theme().colors().drop_target_background)
                }
            })
            .on_drop(move |dragged: &DraggedItem, window, cx| {
                let dragged = dragged.clone();
                let _ = append_drop.update(cx, |this, cx| {
                    this.handle_item_drop(
                        &dragged,
                        tab_id,
                        pane_id,
                        append_index,
                        false,
                        window,
                        cx,
                    );
                });
            });
        TabBar::new(format!("workspace-tab-{}-pane-{}-tabs", tab_id.get(), pane_id.get()))
            .child(
                sorted_tabs
                    .drag_lane(drag_lane, self.pane_new_item_cell(tab_id, pane_id, weak, cx)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod creation_drag_tests {
    use super::*;

    struct Harness {
        preview: pane_drop_preview::PaneDropPreview,
        kind: chrome::NewItemKind,
        clicks: usize,
        drops: Vec<Option<SplitDirection>>,
        direction: Option<SplitDirection>,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let click = cx.listener(|this, _, _, cx| {
                this.clicks += 1;
                cx.notify();
            });
            let button = match self.kind {
                chrome::NewItemKind::Terminal => {
                    chrome::new_item_button("new").on_click(click).into_any_element()
                }
                chrome::NewItemKind::Plugin => {
                    chrome::new_plugin_pane_button("new", IconSize::Small)
                        .on_click(click)
                        .into_any_element()
                }
            };
            v_flex()
                .size_full()
                .child(
                    div().debug_selector(|| "NEW_ITEM_SOURCE".into()).w(px(40.)).h(px(32.)).child(
                        chrome::new_item_drag_handle(
                            "new",
                            Some(cx.entity_id()),
                            self.kind,
                            button,
                        ),
                    ),
                )
                .child(
                    div()
                        .debug_selector(|| "NEW_ITEM_BODY".into())
                        .relative()
                        .flex_1()
                        .w_full()
                        .min_h_0()
                        .group("new-item-target")
                        .on_drag_move::<chrome::DraggedNewItem>(cx.listener(
                            |this, event, _, cx| {
                                if let Some(direction) = pane_drop_direction_for_drag(event) {
                                    this.direction = direction;
                                    cx.notify();
                                }
                            },
                        ))
                        .child(
                            drop_target(
                                self.direction,
                                "new-item-target".into(),
                                "test".into(),
                                cx.entity_id(),
                                &self.preview,
                            )
                            .debug_selector(|| "NEW_ITEM_HIGHLIGHT".into())
                            .on_drop(cx.listener(
                                |this, _: &chrome::DraggedNewItem, _, cx| {
                                    this.drops.push(this.direction);
                                    cx.notify();
                                },
                            )),
                        ),
                )
                .child(self.preview.overlay())
        }
    }

    fn init(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
    }

    #[gpui::test]
    fn both_buttons_preview_edge_splits_without_creating_on_drag_start(
        cx: &mut gpui::TestAppContext,
    ) {
        init(cx);
        for kind in [chrome::NewItemKind::Terminal, chrome::NewItemKind::Plugin] {
            let (view, cx) = cx.add_window_view(|_, _| Harness {
                preview: Default::default(),
                kind,
                clicks: 0,
                drops: Vec::new(),
                direction: None,
            });
            cx.run_until_parked();
            let source = cx.debug_bounds("NEW_ITEM_SOURCE").unwrap().center();
            let body = cx.debug_bounds("NEW_ITEM_BODY").unwrap();
            cx.simulate_mouse_down(source, MouseButton::Left, gpui::Modifiers::none());
            cx.simulate_mouse_move(
                source + gpui::point(px(24.), px(0.)),
                Some(MouseButton::Left),
                gpui::Modifiers::none(),
            );
            assert!(cx.read(|cx| cx.has_active_drag()));
            assert_eq!(view.read_with(cx, |view, _| (view.clicks, view.drops.len())), (0, 0));
            let edge = gpui::point(body.right() - px(2.), body.center().y);
            cx.simulate_mouse_move(edge, Some(MouseButton::Left), gpui::Modifiers::none());
            cx.run_until_parked();
            let highlight = cx.debug_bounds("NEW_ITEM_HIGHLIGHT").unwrap();
            assert_eq!(highlight.size.width, body.size.width * 0.5);
            let preview = view.read_with(cx, |view, _| view.preview.target_bounds()).unwrap();
            assert_eq!(preview.origin, highlight.origin + gpui::point(px(6.), px(6.)));
            assert_eq!(preview.size, highlight.size - gpui::size(px(12.), px(12.)));
            cx.simulate_mouse_up(edge, MouseButton::Left, gpui::Modifiers::none());
            cx.run_until_parked();
            assert!(view.read_with(cx, |view, _| view.preview.target_bounds()).is_none());
            assert_eq!(
                view.read_with(cx, |view, _| view.drops.clone()),
                vec![Some(SplitDirection::Right)]
            );
            assert_eq!(view.read_with(cx, |view, _| view.clicks), 0);
        }
    }

    #[gpui::test]
    fn clicks_still_work_and_cancelled_drags_do_not_turn_into_clicks(
        cx: &mut gpui::TestAppContext,
    ) {
        init(cx);
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            preview: Default::default(),
            kind: chrome::NewItemKind::Plugin,
            clicks: 0,
            drops: Vec::new(),
            direction: None,
        });
        cx.run_until_parked();
        let source = cx.debug_bounds("NEW_ITEM_SOURCE").unwrap().center();
        cx.simulate_click(source, gpui::Modifiers::none());
        assert_eq!(view.read_with(cx, |view, _| view.clicks), 1);
        cx.simulate_mouse_down(source, MouseButton::Left, gpui::Modifiers::none());
        cx.simulate_mouse_move(
            source + gpui::point(px(24.), px(0.)),
            Some(MouseButton::Left),
            gpui::Modifiers::none(),
        );
        assert!(cx.read(|cx| cx.has_active_drag()));
        cx.update(|window, cx| {
            cx.stop_active_drag(window);
        });
        cx.simulate_mouse_move(source, Some(MouseButton::Left), gpui::Modifiers::none());
        cx.simulate_mouse_up(source, MouseButton::Left, gpui::Modifiers::none());
        assert_eq!(view.read_with(cx, |view, _| (view.clicks, view.drops.len())), (1, 0));
    }
}
