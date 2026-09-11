//! Terminal selection, focus, and search.

use super::*;

impl WorkspaceWindow {
    fn active_terminal(&self, cx: &App) -> Option<Entity<terminal::Terminal>> {
        self.active
            .as_ref()
            .and_then(|space| {
                let space = space.read(cx);
                space.active().and_then(|id| space.item(id))
            })
            .and_then(crate::item::Item::as_session)
            .map(|item| item.session.terminal())
    }

    pub(super) fn toggle_terminal_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Mode::Conversations {
            return;
        }
        if self.terminal_search_open {
            self.close_terminal_search(window, cx);
            return;
        }
        let Some(terminal) = self.active_terminal(cx) else {
            return;
        };
        let suggestion =
            terminal.read(cx).last_content().selection_text.clone().unwrap_or_default();
        self.terminal_search_open = true;
        self.terminal_search_target = Some(terminal);
        self.terminal_search_query = suggestion.clone();
        self.terminal_search_input.update(cx, |input, cx| {
            input.set_text(suggestion, true, cx);
        });
        self.start_terminal_search(cx);
        window.focus(&self.terminal_search_input.focus_handle(cx), cx);
        cx.notify();
    }

    pub(super) fn start_terminal_search(&mut self, cx: &mut Context<Self>) {
        if !self.terminal_search_open {
            return;
        }
        self.terminal_search_generation = self.terminal_search_generation.wrapping_add(1);
        let generation = self.terminal_search_generation;
        let Some(terminal) = self.terminal_search_target.clone() else {
            return;
        };
        let query = self.terminal_search_query.clone();
        if query.is_empty() {
            terminal.update(cx, |terminal, _| terminal.matches.clear());
            self.terminal_search_matches.clear();
            self.terminal_search_active = None;
            cx.notify();
            return;
        }
        let Some(search) = terminal::Search::new(&regex_escape_literal(&query)) else {
            return;
        };
        let debounce = cx.background_executor().timer(Duration::from_millis(60));
        cx.spawn(async move |this, cx| {
            debounce.await;
            let Ok(Some(find)) = this.update(cx, |this, cx| {
                if !this.terminal_search_open
                    || this.terminal_search_generation != generation
                    || this.terminal_search_target.as_ref() != Some(&terminal)
                {
                    return None;
                }
                Some(terminal.update(cx, |terminal, cx| terminal.find_matches(search, cx)))
            }) else {
                return;
            };
            let matches = find.await;
            let _ = this.update(cx, |this, cx| {
                if !this.terminal_search_open
                    || this.terminal_search_generation != generation
                    || this.terminal_search_target.as_ref() != Some(&terminal)
                {
                    return;
                }
                let active = matches.len().checked_sub(1);
                terminal.update(cx, |terminal, _| {
                    terminal.matches = matches.clone();
                    if let Some(active) = active {
                        terminal.activate_match(active);
                    }
                });
                this.terminal_search_matches = matches;
                this.terminal_search_active = active;
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn navigate_terminal_search(&mut self, forward: bool, cx: &mut Context<Self>) {
        let count = self.terminal_search_matches.len();
        if count == 0 {
            return;
        }
        let active = match (self.terminal_search_active, forward) {
            (Some(active), true) => (active + 1) % count,
            (Some(0), false) | (None, false) => count - 1,
            (Some(active), false) => active - 1,
            (None, true) => 0,
        };
        self.terminal_search_active = Some(active);
        if let Some(terminal) = self.terminal_search_target.as_ref() {
            terminal.update(cx, |terminal, _| terminal.activate_match(active));
        }
        cx.notify();
    }

    pub(super) fn close_terminal_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal_search_open = false;
        self.terminal_search_generation = self.terminal_search_generation.wrapping_add(1);
        self.terminal_search_query.clear();
        self.terminal_search_matches.clear();
        self.terminal_search_active = None;
        if let Some(terminal) = self.terminal_search_target.take() {
            terminal.update(cx, |terminal, _| terminal.matches.clear());
        }
        self.terminal_search_input.update(cx, |input, cx| input.clear(cx));
        self.focus_active_terminal(window, cx);
        cx.notify();
    }

    pub(super) fn terminal_search_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.terminal_search_open {
            return None;
        }
        let count = self.terminal_search_matches.len();
        let current = self.terminal_search_active.map_or(0, |active| active + 1);
        let previous = cx.listener(|this, _, _, cx| this.navigate_terminal_search(false, cx));
        let next = cx.listener(|this, _, _, cx| this.navigate_terminal_search(true, cx));
        let close = cx.listener(|this, _, window, cx| this.close_terminal_search(window, cx));
        Some(
            h_flex()
                .id("terminal-search")
                .key_context("chartrTerminalSearch")
                .absolute()
                .top_2()
                .right_2()
                .gap_1()
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border)
                .bg(cx.theme().colors().elevated_surface_background)
                .child(div().w(px(220.)).child(self.terminal_search_input.clone()))
                .child(
                    Label::new(format!("{current}/{count}"))
                        .size(UI_LABEL_SMALL)
                        .color(Color::Muted),
                )
                .child(Button::new("terminal-search-previous", "Prev").on_click(previous))
                .child(Button::new("terminal-search-next", "Next").on_click(next))
                .child(Button::new("terminal-search-close", "Close").on_click(close))
                .into_any_element(),
        )
    }
}
