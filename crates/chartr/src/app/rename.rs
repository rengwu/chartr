//! Space, group and terminal rename dialogs, including native modal hosting.

use super::*;
use crate::workspace::ItemId;

#[derive(Clone)]
pub(super) struct TerminalRenameTarget {
    space: EntityId,
    item: ItemId,
    terminal: chartr_herdr::TerminalId,
}

impl WorkspaceWindow {
    fn terminal_rename_info(
        &self,
        target: &TerminalRenameTarget,
        cx: &App,
    ) -> Option<chartr_herdr::control::Session> {
        let space = self.spaces.iter().find(|space| space.entity_id() == target.space)?;
        let item = space.read(cx).item(target.item)?.as_session()?;
        (item.session.info.terminal == target.terminal).then(|| item.session.info.clone())
    }

    pub(super) fn rename_active_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(space) = &self.active
            && let Some(item) = space.read(cx).active()
        {
            self.begin_terminal_rename(space.entity_id(), item, window, cx);
        }
    }

    pub(super) fn begin_terminal_rename(
        &mut self,
        space_id: EntityId,
        item: ItemId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(info) = self
            .spaces
            .iter()
            .find(|space| space.entity_id() == space_id)
            .and_then(|space| space.read(cx).item(item))
            .and_then(crate::item::Item::as_session)
            .map(|item| item.session.info.clone())
        else {
            return;
        };
        self.rename_space = None;
        self.rename_group = None;
        self.rename_terminal =
            Some(TerminalRenameTarget { space: space_id, item, terminal: info.terminal.clone() });
        self.rename_query = self.terminal_names.title(&info);
        self.rename_input
            .update(cx, |input, cx| input.set_text(self.rename_query.clone(), true, cx));
        self.open_rename_window(RenameKind::Terminal, window, cx);
        cx.notify();
    }

    pub(super) fn automatic_terminal_name(
        &mut self,
        space_id: EntityId,
        item: ItemId,
        cx: &mut Context<Self>,
    ) {
        if let Some(info) = self
            .spaces
            .iter()
            .find(|space| space.entity_id() == space_id)
            .and_then(|space| space.read(cx).item(item))
            .and_then(crate::item::Item::as_session)
            .map(|item| item.session.info.clone())
        {
            let _ = self.terminal_names.rename(&info, "");
            cx.notify();
        }
    }

    pub(super) fn commit_terminal_rename(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(target) = &self.rename_terminal else {
            return true;
        };
        if let Some(info) = self.terminal_rename_info(target, cx)
            && self.terminal_names.rename(&info, self.rename_input.read(cx).text()).is_err()
        {
            return false;
        }
        self.cancel_rename(window, cx);
        true
    }

    pub(super) fn open_rename_window(
        &mut self,
        kind: RenameKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let owner = cx.weak_entity();
        let parent = window.window_handle();
        let input = self.rename_input.clone();
        let popup_input = input.clone();
        match crate::components::open_native_modal(window, cx, move |window, cx| {
            let modal_window = window.window_handle();
            let view =
                cx.new(|cx| RenameWindow::new(kind, owner, parent, modal_window, popup_input, cx));
            window.focus(&input.focus_handle(cx), cx);
            view
        }) {
            Ok(popup) => self.rename_window = Some(popup.into()),
            Err(error) => {
                eprintln!("chartr could not open the rename dialog above native content: {error}");
                self.rename_window = None;
                window.focus(&self.rename_input.focus_handle(cx), cx);
            }
        }
    }

    fn cancel_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rename_space = None;
        self.rename_group = None;
        self.rename_terminal = None;
        self.rename_query.clear();
        self.rename_input.update(cx, |input, cx| input.clear(cx));
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn rename_window_closed(
        &mut self,
        kind: RenameKind,
        modal_window: AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.rename_window != Some(modal_window) {
            return;
        }
        self.rename_window = None;
        let rename_still_open = match kind {
            RenameKind::Terminal => self.rename_terminal.is_some(),
            RenameKind::Space => self.rename_space.is_some(),
            RenameKind::Group => self.rename_group.is_some(),
        };
        if rename_still_open {
            self.cancel_rename(window, cx);
        } else {
            window.focus(&self.focus, cx);
            cx.notify();
        }
    }

    pub(super) fn commit_space_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.rename_space.take() else {
            return;
        };
        // Read from the input directly so Enter always commits the latest IME
        // transaction, even before the subscription's mirrored value flushes.
        let name = self.rename_input.read(cx).text().trim().to_owned();
        self.rename_query.clear();
        self.rename_input.update(cx, |input, cx| input.clear(cx));
        window.focus(&self.focus, cx);
        let Some(space) = self.spaces.iter().find(|space| space.entity_id() == id).cloned() else {
            return;
        };
        let path = space.read(cx).path().clone();
        let result = self
            .registry
            .as_mut()
            .map(|registry| registry.rename(&path, name.clone()))
            .unwrap_or(Ok(()));
        match result {
            Ok(()) => {
                space.update(cx, |space, _| space.set_name(name));
                self.problem = None;
            }
            Err(error) => self.problem = Some(error.to_string()),
        }
        cx.notify();
    }

    pub(super) fn commit_group_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((space_id, tab)) = self.rename_group.take() else {
            return;
        };
        // Read from the input directly so Enter always commits the latest IME
        // transaction, even before the subscription's mirrored value flushes.
        let name = self.rename_input.read(cx).text().trim().to_owned();
        let name = (!name.is_empty()).then_some(name);
        self.rename_query.clear();
        self.rename_input.update(cx, |input, cx| input.clear(cx));
        window.focus(&self.focus, cx);
        if let Some(space) = self.spaces.iter().find(|space| space.entity_id() == space_id).cloned()
        {
            space.update(cx, |space, _| space.rename_group(tab, name));
        }
        cx.notify();
    }

    pub(super) fn rename_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.rename_window.is_some() {
            return None;
        }
        let kind = if self.rename_space.is_some() {
            RenameKind::Space
        } else if self.rename_group.is_some() {
            RenameKind::Group
        } else if self.rename_terminal.is_some() {
            RenameKind::Terminal
        } else {
            return None;
        };
        let cancel_scrim = cx.listener(|this, _, window, cx| this.cancel_rename(window, cx));
        let cancel_button = cx.listener(|this, _, window, cx| this.cancel_rename(window, cx));
        let save = cx.listener(move |this, _, window, cx| match kind {
            RenameKind::Terminal => {
                this.commit_terminal_rename(window, cx);
            }
            RenameKind::Space => this.commit_space_rename(window, cx),
            RenameKind::Group => this.commit_group_rename(window, cx),
        });
        Some(
            chartr_plugin::ui::ModalOverlay::new(
                match kind {
                    RenameKind::Terminal => "rename-terminal-scrim",
                    RenameKind::Space => "rename-space-scrim",
                    RenameKind::Group => "rename-group-scrim",
                },
                cancel_scrim,
            )
            .child(rename_dialog(kind, self.rename_input.clone(), cancel_button, save, cx))
            .into_any_element(),
        )
    }
}

fn rename_dialog(
    kind: RenameKind,
    input: Entity<TextInput>,
    cancel: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    save: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    let (title, help, dialog_id, cancel_id, save_id) = match kind {
        RenameKind::Terminal => (
            "Rename Terminal",
            Some("Leave blank to use the automatic name."),
            "rename-terminal-dialog",
            "cancel-terminal-rename",
            "save-terminal-rename",
        ),
        RenameKind::Space => (
            "Rename Space",
            None,
            "rename-space-dialog",
            "cancel-space-rename",
            "save-space-rename",
        ),
        RenameKind::Group => (
            "Rename Group",
            Some("Leave blank to use the tab count."),
            "rename-group-dialog",
            "cancel-group-rename",
            "save-group-rename",
        ),
    };

    let error = (kind == RenameKind::Terminal)
        .then(|| crate::terminal_names::clean_name(input.read(cx).text()).err())
        .flatten();
    let invalid = error.is_some();
    chartr_plugin::ui::DialogSurface::new(dialog_id)
        .compact()
        .child(chartr_plugin::ui::dialog_header(title, div(), cx))
        .child(
            chartr_plugin::ui::dialog_body()
                .gap_1()
                .child(crate::components::input_field(format!("{dialog_id}-input"), input, cx))
                .children(help.map(|help| {
                    Label::new(help).size(UI_LABEL_SMALL).color(Color::Muted).into_any_element()
                }))
                .children(error.map(|error| {
                    Label::new(error.to_string())
                        .size(UI_LABEL_SMALL)
                        .color(Color::Error)
                        .into_any_element()
                })),
        )
        .child(
            chartr_plugin::ui::dialog_actions(cx)
                .child(chartr_plugin::ui::action(cancel_id, "Cancel").on_click(cancel))
                .child(
                    chartr_plugin::ui::action(save_id, "Rename").disabled(invalid).on_click(save),
                ),
        )
}

/// Rename dialogs are hosted in their own parent-anchored window so their scrim and content are
/// composited above native plugin views. `WorkspaceWindow` remains the state owner; this view only
/// routes popup input back to the originating workspace window.
struct RenameWindow {
    kind: RenameKind,
    owner: WeakEntity<WorkspaceWindow>,
    parent: AnyWindowHandle,
    input: Entity<TextInput>,
}

impl RenameWindow {
    fn new(
        kind: RenameKind,
        owner: WeakEntity<WorkspaceWindow>,
        parent: AnyWindowHandle,
        modal_window: AnyWindowHandle,
        input: Entity<TextInput>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        let release_owner = owner.clone();
        cx.on_release(move |_, cx| {
            let _ = parent.update(cx, |_, window, cx| {
                let _ = release_owner.update(cx, |owner, cx| {
                    owner.rename_window_closed(kind, modal_window, window, cx)
                });
            });
        })
        .detach();
        Self { kind, owner, parent, input }
    }

    fn finish(&self, save: bool, window: &mut Window, cx: &mut Context<Self>) {
        let owner = self.owner.clone();
        let kind = self.kind;
        let finished = self.parent.update(cx, |_, parent_window, cx| {
            owner
                .update(cx, |owner, cx| {
                    if save {
                        match kind {
                            RenameKind::Terminal => {
                                return owner.commit_terminal_rename(parent_window, cx);
                            }
                            RenameKind::Space => owner.commit_space_rename(parent_window, cx),
                            RenameKind::Group => owner.commit_group_rename(parent_window, cx),
                        }
                    } else {
                        owner.cancel_rename(parent_window, cx);
                    }
                    true
                })
                .unwrap_or(true)
        });
        if finished.unwrap_or(true) {
            window.remove_window();
        }
    }
}

impl Render for RenameWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ui_font = Fonts::setup_ui(window, cx);
        let cancel_scrim = cx.listener(|this, _, window, cx| this.finish(false, window, cx));
        let cancel_button = cx.listener(|this, _, window, cx| this.finish(false, window, cx));
        let save = cx.listener(|this, _, window, cx| this.finish(true, window, cx));
        let on_key = cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
            match event.keystroke.key.as_str() {
                "escape" => {
                    cx.stop_propagation();
                    this.finish(false, window, cx);
                }
                "enter" => {
                    cx.stop_propagation();
                    this.finish(true, window, cx);
                }
                _ => {}
            }
        });
        let key_context = match self.kind {
            RenameKind::Terminal => "RenameTerminal",
            RenameKind::Space => "RenameSpace",
            RenameKind::Group => "RenameGroup",
        };

        div()
            .id("native-rename-scrim")
            .relative()
            .key_context(key_context)
            .size_full()
            .font(ui_font)
            .text_size(UI_TEXT_DEFAULT)
            .text_color(cx.theme().colors().text)
            .on_key_down(on_key)
            .child(
                chartr_plugin::ui::ModalOverlay::new("rename-modal-overlay", cancel_scrim)
                    .child(rename_dialog(self.kind, self.input.clone(), cancel_button, save, cx)),
            )
    }
}
