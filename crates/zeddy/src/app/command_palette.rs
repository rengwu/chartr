//! Command palette filtering, invocation, and rendering.

use super::*;

impl Zeddy {
    pub(super) fn toggle_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = !self.command_palette_open;
        self.command_palette_query.clear();
        self.command_palette_input.update(cx, |input, cx| input.clear(cx));
        self.command_palette_selected = 0;
        if self.command_palette_open {
            window.focus(&self.command_palette_input.focus_handle(cx), cx);
        } else {
            window.focus(&self.focus, cx);
        }
        cx.notify();
    }

    pub(super) fn filtered_palette_commands(
        &self,
    ) -> Vec<(PaletteCommand, &'static str, &'static str)> {
        let query = self.command_palette_query.trim().to_lowercase();
        PaletteCommand::ALL
            .into_iter()
            .filter(|(_, label, _)| query.is_empty() || label.to_lowercase().contains(&query))
            .collect()
    }

    pub(super) fn invoke_palette_command(
        &mut self,
        command: PaletteCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_input.update(cx, |input, cx| input.clear(cx));
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
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(super) fn command_palette(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.command_palette_open {
            return None;
        }
        let commands = self.filtered_palette_commands();
        if self.command_palette_selected >= commands.len() {
            self.command_palette_selected = 0;
        }
        let selected = self.command_palette_selected;
        let weak = cx.weak_entity();
        let rows: Vec<_> = commands
            .into_iter()
            .enumerate()
            .map(|(index, (command, label, shortcut))| {
                let choose = weak.clone();
                ListItem::new(("command-palette-item", index))
                    .spacing(ListItemSpacing::Dense)
                    .toggle_state(index == selected)
                    .aria_role(gpui::Role::ListBoxOption)
                    .aria_label(label)
                    .when(!shortcut.is_empty(), |item| item.aria_keyshortcuts(shortcut))
                    .when(index == selected, ListItem::aria_active_descendant)
                    .on_click(move |_, window, cx| {
                        let _ = choose.update(cx, |this, cx| {
                            this.invoke_palette_command(command, window, cx)
                        });
                    })
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
        let dismiss = cx.listener(|this, _, window, cx| {
            this.command_palette_open = false;
            this.command_palette_query.clear();
            this.command_palette_input.update(cx, |input, cx| input.clear(cx));
            window.focus(&this.focus, cx);
            cx.notify();
        });

        Some(
            div()
                .id("command-palette-scrim")
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .bg(gpui::black().opacity(0.35))
                .on_mouse_down(gpui::MouseButton::Left, dismiss)
                .child(
                    v_flex()
                        .id("command-palette")
                        .absolute()
                        .top(px(48.))
                        .left(relative(0.5))
                        .ml(px(-320.))
                        .w(px(640.))
                        .max_h(px(480.))
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
                                .px_3()
                                .gap_2()
                                .border_b_1()
                                .border_color(cx.theme().colors().border)
                                .child(
                                    Icon::new(IconName::MagnifyingGlass)
                                        .size(IconSize::Small)
                                        .color(Color::Muted),
                                )
                                .child(self.command_palette_input.clone()),
                        )
                        .child(
                            v_flex()
                                .id("command-palette-results")
                                .role(gpui::Role::ListBox)
                                .aria_label("Commands")
                                .p_1()
                                .overflow_y_scroll()
                                .when(rows.is_empty(), |list| {
                                    list.child(div().p_3().child(
                                        Label::new("No matching commands").color(Color::Muted),
                                    ))
                                })
                                .children(rows),
                        ),
                )
                .into_any_element(),
        )
    }
}
