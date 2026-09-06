//! Command palette hosted above native child surfaces, with window-owned input and selection.

use super::*;

type FinishPalette = Box<dyn FnOnce(Option<PaletteCommand>, &mut App)>;

impl Zeddy {
    pub(super) fn toggle_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(palette) = self.command_palette_window.take() {
            let _ = palette.update(cx, |_, window, _| window.remove_window());
            window.focus(&self.focus, cx);
            cx.notify();
            return;
        }
        let owner = cx.weak_entity();
        let parent = window.window_handle();
        match crate::components::open_native_modal(window, cx, move |window, cx| {
            let modal = window.window_handle();
            let palette = cx.new(|cx| {
                CommandPaletteWindow::new(
                    Box::new(move |command, cx| {
                        let _ = parent.update(cx, |_, window, cx| {
                            let _ = owner.update(cx, |owner, cx| {
                                if owner.command_palette_window != Some(modal) {
                                    return;
                                }
                                owner.command_palette_window = None;
                                window.focus(&owner.focus, cx);
                                if let Some(command) = command {
                                    owner.invoke_palette_command(command, window, cx);
                                }
                                cx.notify();
                            });
                        });
                    }),
                    cx,
                )
            });
            window.focus(&palette.read(cx).input.focus_handle(cx), cx);
            palette
        }) {
            Ok(palette) => self.command_palette_window = Some(palette.into()),
            Err(error) => {
                self.problem = Some(format!("Could not open the command palette: {error}"))
            }
        }
        cx.notify();
    }

    fn invoke_palette_command(
        &mut self,
        command: PaletteCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let action: Box<dyn gpui::Action> = match command {
            PaletteCommand::NewTerminal => Box::new(actions::workspace::NewTerminal),
            PaletteCommand::CloseItem => Box::new(actions::pane::CloseActiveItem),
            PaletteCommand::CloseAllItems => Box::new(actions::pane::CloseAllItems),
            PaletteCommand::MoveLeft => Box::new(actions::pane::MoveLeft),
            PaletteCommand::MoveRight => Box::new(actions::pane::MoveRight),
            PaletteCommand::MoveUp => Box::new(actions::pane::MoveUp),
            PaletteCommand::MoveDown => Box::new(actions::pane::MoveDown),
            PaletteCommand::JoinPane => Box::new(actions::pane::JoinIntoNext),
            PaletteCommand::FocusLeft => Box::new(actions::workspace::ActivatePaneLeft),
            PaletteCommand::FocusRight => Box::new(actions::workspace::ActivatePaneRight),
            PaletteCommand::FocusUp => Box::new(actions::workspace::ActivatePaneUp),
            PaletteCommand::FocusDown => Box::new(actions::workspace::ActivatePaneDown),
            PaletteCommand::OpenSettings => Box::new(actions::settings::Open),
        };
        window.dispatch_action(action, cx);
        cx.notify();
    }
}

fn filtered_commands(query: &str) -> Vec<(PaletteCommand, &'static str, &'static str)> {
    let query = query.trim().to_lowercase();
    PaletteCommand::ALL
        .into_iter()
        .filter(|(_, label, _)| query.is_empty() || label.to_lowercase().contains(&query))
        .collect()
}

struct CommandPaletteWindow {
    input: Entity<TextInput>,
    selected: usize,
    finish: Option<FinishPalette>,
}

impl CommandPaletteWindow {
    fn new(finish: FinishPalette, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new("Type a command…", cx));
        cx.subscribe(&input, |this, _, _: &InputEvent, cx| {
            this.selected = 0;
            cx.notify();
        })
        .detach();
        cx.on_release(|this, cx| {
            if let Some(finish) = this.finish.take() {
                finish(None, cx);
            }
        })
        .detach();
        Self { input, selected: 0, finish: Some(finish) }
    }

    fn finish(
        &mut self,
        command: Option<PaletteCommand>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.remove_window();
        if let Some(finish) = self.finish.take() {
            finish(command, cx);
        }
    }

    fn on_key(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let commands = filtered_commands(self.input.read(cx).text());
        match event.keystroke.key.as_str() {
            "escape" => self.finish(None, window, cx),
            "up" if !commands.is_empty() => {
                self.selected = self.selected.checked_sub(1).unwrap_or(commands.len() - 1);
            }
            "down" if !commands.is_empty() => self.selected = (self.selected + 1) % commands.len(),
            "enter" => {
                if let Some((command, _, _)) = commands.get(self.selected) {
                    self.finish(Some(*command), window, cx);
                }
            }
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}

impl Render for CommandPaletteWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let font = Fonts::setup_ui(window, cx);
        let commands = filtered_commands(self.input.read(cx).text());
        if self.selected >= commands.len() {
            self.selected = 0;
        }
        let rows = commands
            .into_iter()
            .enumerate()
            .map(|(index, (command, label, shortcut))| {
                ListItem::new(("command-palette-item", index))
                    .spacing(ListItemSpacing::Dense)
                    .toggle_state(index == self.selected)
                    .aria_role(gpui::Role::ListBoxOption)
                    .aria_label(label)
                    .when(!shortcut.is_empty(), |item| item.aria_keyshortcuts(shortcut))
                    .when(index == self.selected, ListItem::aria_active_descendant)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.finish(Some(command), window, cx)
                    }))
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .child(Label::new(label).size(UI_LABEL_DEFAULT))
                            .when(!shortcut.is_empty(), |row| {
                                row.child(
                                    Label::new(shortcut).size(UI_LABEL_SMALL).color(Color::Muted),
                                )
                            }),
                    )
            })
            .collect();
        let dismiss = cx.listener(|this, _, window, cx| this.finish(None, window, cx));
        div()
            .relative()
            .size_full()
            .font(font)
            .text_size(UI_TEXT_DEFAULT)
            .text_color(cx.theme().colors().text)
            .key_context("Chartr CommandPalette")
            .track_focus(&self.input.focus_handle(cx))
            .on_key_down(cx.listener(Self::on_key))
            .on_action(cx.listener(|this, _: &actions::command_palette::Toggle, window, cx| {
                this.finish(None, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::pane::CloseActiveItem, window, cx| {
                this.finish(None, window, cx)
            }))
            .on_action(cx.listener(|this, _: &actions::settings::Open, window, cx| {
                this.finish(Some(PaletteCommand::OpenSettings), window, cx)
            }))
            .child(palette_surface(
                self.input.clone(),
                rows,
                dismiss,
                window.viewport_size().height,
                cx,
            ))
    }
}

fn palette_surface(
    input: Entity<TextInput>,
    rows: Vec<ui::ListItem>,
    dismiss: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static,
    height: gpui::Pixels,
    cx: &App,
) -> AnyElement {
    div()
        .id("command-palette-scrim")
        .absolute()
        .top_0()
        .right_0()
        .bottom_0()
        .left_0()
        .flex()
        .items_start()
        .justify_center()
        .pt(px(48.))
        .bg(gpui::black().opacity(0.35))
        .on_mouse_down(gpui::MouseButton::Left, dismiss)
        .child(
            v_flex()
                .id("command-palette")
                .debug_selector(|| "COMMAND_PALETTE".into())
                .w(px(640.))
                .max_w(relative(0.92))
                .max_h((height - px(72.)).max(px(42.)).min(px(480.)))
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().colors().border)
                .bg(cx.theme().colors().elevated_surface_background)
                .shadow_lg()
                .overflow_hidden()
                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    h_flex()
                        .h(px(42.))
                        .flex_none()
                        .px_3()
                        .gap_2()
                        .border_b_1()
                        .border_color(cx.theme().colors().border)
                        .child(
                            Icon::new(IconName::MagnifyingGlass)
                                .size(IconSize::Small)
                                .color(Color::Muted),
                        )
                        .child(input),
                )
                .child(
                    v_flex()
                        .id("command-palette-results")
                        .role(gpui::Role::ListBox)
                        .aria_label("Commands")
                        .p_1()
                        .overflow_y_scroll()
                        .when(rows.is_empty(), |list| {
                            list.child(
                                div()
                                    .p_3()
                                    .child(Label::new("No matching commands").color(Color::Muted)),
                            )
                        })
                        .children(rows),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Parent;
    impl Render for Parent {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full()
        }
    }

    fn open(
        cx: &mut gpui::TestAppContext,
    ) -> (gpui::VisualTestContext, Rc<RefCell<Vec<Option<PaletteCommand>>>>) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            crate::actions::init(&crate::keymap::KeymapStore::bare(), cx);
        });
        let (_, parent) = cx.add_window_view(|_, _| Parent);
        let events = Rc::new(RefCell::new(Vec::new()));
        let recorded = events.clone();
        let popup = parent.update(|window, cx| {
            crate::components::open_native_modal(window, cx, move |window, cx| {
                let view = cx.new(|cx| {
                    CommandPaletteWindow::new(
                        Box::new(move |command, _| recorded.borrow_mut().push(command)),
                        cx,
                    )
                });
                window.focus(&view.read(cx).input.focus_handle(cx), cx);
                view
            })
            .unwrap()
        });
        (gpui::VisualTestContext::from_window(popup.into(), parent), events)
    }

    #[gpui::test]
    fn palette_is_a_separate_modal_with_visible_bounds_at_small_sizes(
        cx: &mut gpui::TestAppContext,
    ) {
        let (mut popup, _) = open(cx);
        assert_eq!(popup.windows().len(), 2);
        for size in [gpui::size(px(1100.), px(720.)), gpui::size(px(420.), px(280.))] {
            popup.simulate_resize(size);
            popup.run_until_parked();
            let bounds = popup
                .debug_bounds("COMMAND_PALETTE")
                .expect("palette must be painted in the native modal");
            assert!(
                bounds.size.height > px(42.),
                "input and results must remain visible: {bounds:?}"
            );
            assert!(bounds.left() >= px(0.) && bounds.right() <= size.width);
            assert!(bounds.top() >= px(0.) && bounds.bottom() <= size.height);
        }
    }

    #[gpui::test]
    fn typing_and_enter_invoke_the_filtered_command_once(cx: &mut gpui::TestAppContext) {
        let (mut popup, events) = open(cx);
        popup.simulate_input("settings");
        popup.simulate_keystrokes("enter");
        popup.run_until_parked();
        assert_eq!(*events.borrow(), vec![Some(PaletteCommand::OpenSettings)]);
        assert_eq!(popup.windows().len(), 1);
    }

    #[gpui::test]
    fn escape_and_toggle_dismiss_without_invoking_a_command(cx: &mut gpui::TestAppContext) {
        let (mut popup, events) = open(cx);
        popup.simulate_input("no matching command");
        popup.simulate_keystrokes("enter");
        assert!(events.borrow().is_empty());
        popup.simulate_keystrokes("escape");
        popup.run_until_parked();
        assert_eq!(*events.borrow(), vec![None]);
        assert_eq!(popup.windows().len(), 1);

        let (mut popup, events) = open(cx);
        popup.simulate_keystrokes(if cfg!(target_os = "macos") {
            "cmd-shift-p"
        } else {
            "ctrl-shift-p"
        });
        popup.run_until_parked();
        assert_eq!(*events.borrow(), vec![None]);
    }

    #[gpui::test]
    fn arrow_keys_select_commands_and_outside_click_dismisses(cx: &mut gpui::TestAppContext) {
        let (mut popup, events) = open(cx);
        popup.simulate_keystrokes("down enter");
        popup.run_until_parked();
        assert_eq!(*events.borrow(), vec![Some(PaletteCommand::CloseItem)]);

        let (mut popup, events) = open(cx);
        popup.simulate_mouse_down(
            gpui::point(px(4.), px(4.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        popup.run_until_parked();
        assert_eq!(*events.borrow(), vec![None]);
    }
}
