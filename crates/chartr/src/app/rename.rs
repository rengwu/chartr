//! Space and group rename dialogs, including native modal hosting.

use super::*;

impl WorkspaceWindow {
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
        } else {
            return None;
        };
        let cancel_scrim = cx.listener(|this, _, window, cx| this.cancel_rename(window, cx));
        let cancel_button = cx.listener(|this, _, window, cx| this.cancel_rename(window, cx));
        let save = cx.listener(move |this, _, window, cx| match kind {
            RenameKind::Space => this.commit_space_rename(window, cx),
            RenameKind::Group => this.commit_group_rename(window, cx),
        });
        Some(
            chartr_plugin::ui::ModalOverlay::new(
                match kind {
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

    chartr_plugin::ui::DialogSurface::new(dialog_id)
        .compact()
        .child(Label::new(title).size(UI_LABEL_LARGE))
        .child(
            v_flex()
                .gap_1()
                .child(crate::components::input_field(format!("{dialog_id}-input"), input, cx))
                .children(help.map(|help| {
                    Label::new(help).size(UI_LABEL_SMALL).color(Color::Muted).into_any_element()
                })),
        )
        .child(
            h_flex()
                .justify_end()
                .gap_1()
                .child(Button::new(cancel_id, "Cancel").on_click(cancel))
                .child(Button::new(save_id, "Rename").on_click(save)),
        )
}

/// Rename dialogs are hosted in their own parent-anchored window so their scrim and content are
/// composited above native browser/plugin views. `WorkspaceWindow` remains the state owner; this view only
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
        let _ = self.parent.update(cx, |_, parent_window, cx| {
            let _ = owner.update(cx, |owner, cx| {
                if save {
                    match kind {
                        RenameKind::Space => owner.commit_space_rename(parent_window, cx),
                        RenameKind::Group => owner.commit_group_rename(parent_window, cx),
                    }
                } else {
                    owner.cancel_rename(parent_window, cx);
                }
            });
        });
        window.remove_window();
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
