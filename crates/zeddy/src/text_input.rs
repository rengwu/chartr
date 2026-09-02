//! A native-behaving, single-line GPUI text input.
//!
//! This follows GPUI's canonical `examples/input.rs` architecture: the model
//! implements [`EntityInputHandler`], so text composition, dead keys, input
//! methods, and accessibility cross the platform text-input bridge instead of
//! being reconstructed from key-down events. The bindings below mirror the
//! single-line subset of Zed's platform editor keymaps.

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable, GlobalElementId, KeyBinding,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, Style, TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window,
    actions, fill, point, prelude::*, px, relative, size,
};
use ui::prelude::*;
use unicode_segmentation::UnicodeSegmentation as _;

actions!(
    chartr_text_input,
    [
        Backspace,
        Delete,
        DeleteWordBackward,
        DeleteWordForward,
        DeleteToBeginning,
        DeleteToEnd,
        Left,
        Right,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        Home,
        End,
        SelectHome,
        SelectEnd,
        SelectAll,
        Paste,
        Cut,
        Copy,
        Undo,
        Redo,
        ShowCharacterPalette,
    ]
);

/// Register the native platform bindings in this input's narrow key context.
pub fn init(cx: &mut App) {
    let context = Some("NativeTextInput");
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("shift-backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-home", SelectHome, context),
        KeyBinding::new("shift-end", SelectEnd, context),
    ]);

    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-a", SelectAll, context),
        KeyBinding::new("cmd-c", Copy, context),
        KeyBinding::new("cmd-x", Cut, context),
        KeyBinding::new("cmd-v", Paste, context),
        KeyBinding::new("cmd-z", Undo, context),
        KeyBinding::new("cmd-shift-z", Redo, context),
        KeyBinding::new("alt-left", WordLeft, context),
        KeyBinding::new("alt-right", WordRight, context),
        KeyBinding::new("alt-shift-left", SelectWordLeft, context),
        KeyBinding::new("alt-shift-right", SelectWordRight, context),
        KeyBinding::new("alt-backspace", DeleteWordBackward, context),
        KeyBinding::new("alt-delete", DeleteWordForward, context),
        KeyBinding::new("cmd-left", Home, context),
        KeyBinding::new("cmd-right", End, context),
        KeyBinding::new("cmd-up", Home, context),
        KeyBinding::new("cmd-down", End, context),
        KeyBinding::new("cmd-shift-left", SelectHome, context),
        KeyBinding::new("cmd-shift-right", SelectEnd, context),
        KeyBinding::new("cmd-shift-up", SelectHome, context),
        KeyBinding::new("cmd-shift-down", SelectEnd, context),
        KeyBinding::new("cmd-backspace", DeleteToBeginning, context),
        KeyBinding::new("cmd-delete", DeleteToEnd, context),
        KeyBinding::new("ctrl-a", Home, context),
        KeyBinding::new("ctrl-e", End, context),
        KeyBinding::new("ctrl-b", Left, context),
        KeyBinding::new("ctrl-f", Right, context),
        KeyBinding::new("ctrl-h", Backspace, context),
        KeyBinding::new("ctrl-d", Delete, context),
        KeyBinding::new("ctrl-w", DeleteWordBackward, context),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, context),
    ]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-a", SelectAll, context),
        KeyBinding::new("ctrl-c", Copy, context),
        KeyBinding::new("ctrl-x", Cut, context),
        KeyBinding::new("ctrl-v", Paste, context),
        KeyBinding::new("cut", Cut, context),
        KeyBinding::new("copy", Copy, context),
        KeyBinding::new("paste", Paste, context),
        KeyBinding::new("ctrl-insert", Copy, context),
        KeyBinding::new("shift-delete", Cut, context),
        KeyBinding::new("shift-insert", Paste, context),
        KeyBinding::new("ctrl-z", Undo, context),
        KeyBinding::new("ctrl-y", Redo, context),
        KeyBinding::new("ctrl-shift-z", Redo, context),
        KeyBinding::new("undo", Undo, context),
        KeyBinding::new("redo", Redo, context),
        KeyBinding::new("ctrl-left", WordLeft, context),
        KeyBinding::new("ctrl-right", WordRight, context),
        KeyBinding::new("ctrl-shift-left", SelectWordLeft, context),
        KeyBinding::new("ctrl-shift-right", SelectWordRight, context),
        KeyBinding::new("ctrl-backspace", DeleteWordBackward, context),
        KeyBinding::new("ctrl-delete", DeleteWordForward, context),
        KeyBinding::new("ctrl-home", Home, context),
        KeyBinding::new("ctrl-end", End, context),
        KeyBinding::new("ctrl-shift-home", SelectHome, context),
        KeyBinding::new("ctrl-shift-end", SelectEnd, context),
        KeyBinding::new("ctrl-alt-space", ShowCharacterPalette, context),
    ]);
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Edited,
}

impl EventEmitter<InputEvent> for TextInput {}

#[derive(Clone)]
struct Snapshot {
    content: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

/// A reusable single-line input model and view.
pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll_x: Pixels,
    alignment_offset: Pixels,
    text_align: TextAlign,
    is_selecting: bool,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
}

impl TextInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: placeholder.into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            scroll_x: px(0.),
            alignment_offset: px(0.),
            text_align: TextAlign::Left,
            is_selecting: false,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text_align(&mut self, text_align: TextAlign, cx: &mut Context<Self>) {
        self.text_align = text_align;
        cx.notify();
    }

    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        select_all: bool,
        cx: &mut Context<Self>,
    ) {
        self.content = text.into();
        self.marked_range = None;
        self.undo.clear();
        self.redo.clear();
        self.scroll_x = px(0.);
        if select_all {
            self.selected_range = 0..self.content.len();
        } else {
            self.selected_range = self.content.len()..self.content.len();
        }
        self.selection_reversed = false;
        cx.emit(InputEvent::Edited);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_text("", false, cx);
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            content: self.content.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        }
    }

    fn restore(&mut self, snapshot: Snapshot, cx: &mut Context<Self>) {
        self.content = snapshot.content;
        self.selected_range = snapshot.selected_range;
        self.selection_reversed = snapshot.selection_reversed;
        self.marked_range = None;
        cx.emit(InputEvent::Edited);
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed { self.selected_range.start } else { self.selected_range.end }
    }

    fn anchor_offset(&self) -> usize {
        if self.selection_reversed { self.selected_range.end } else { self.selected_range.start }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let anchor = self.anchor_offset();
        let head = offset.min(self.content.len());
        self.selected_range = anchor.min(head)..anchor.max(head);
        self.selection_reversed = head < anchor;
        cx.notify();
    }

    fn previous_grapheme(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(index, _)| (index < offset).then_some(index))
            .unwrap_or(0)
    }

    fn next_grapheme(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(index, _)| (index > offset).then_some(index))
            .unwrap_or(self.content.len())
    }

    fn previous_word_start(&self, offset: usize) -> usize {
        self.content[..offset]
            .unicode_word_indices()
            .map(|(index, _)| index)
            .next_back()
            .unwrap_or(0)
    }

    fn next_word_end(&self, offset: usize) -> usize {
        self.content[offset..]
            .unicode_word_indices()
            .next()
            .map(|(index, word)| offset + index + word.len())
            .unwrap_or(self.content.len())
    }

    fn word_range_at(&self, offset: usize) -> Range<usize> {
        if self.content.is_empty() {
            return 0..0;
        }
        let offset = offset.min(self.content.len().saturating_sub(1));
        self.content
            .split_word_bound_indices()
            .find_map(|(start, segment)| {
                let end = start + segment.len();
                (start <= offset && offset < end).then_some(start..end)
            })
            .unwrap_or(offset..self.next_grapheme(offset))
    }

    fn replace_range(&mut self, range: Range<usize>, new_text: &str, cx: &mut Context<Self>) {
        let new_text = new_text.replace(['\r', '\n'], " ");
        if range.is_empty() && new_text.is_empty() {
            return;
        }
        self.undo.push(self.snapshot());
        self.redo.clear();
        self.content =
            format!("{}{}{}", &self.content[..range.start], new_text, &self.content[range.end..])
                .into();
        let cursor = range.start + new_text.len();
        self.selected_range = cursor..cursor;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.emit(InputEvent::Edited);
        cx.notify();
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let target = if self.selected_range.is_empty() {
            self.previous_grapheme(self.cursor_offset())
        } else {
            self.selected_range.start
        };
        self.move_to(target, cx);
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let target = if self.selected_range.is_empty() {
            self.next_grapheme(self.cursor_offset())
        } else {
            self.selected_range.end
        };
        self.move_to(target, cx);
    }

    fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        let target = if self.selected_range.is_empty() {
            self.previous_word_start(self.cursor_offset())
        } else {
            self.selected_range.start
        };
        self.move_to(target, cx);
    }

    fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        let target = if self.selected_range.is_empty() {
            self.next_word_end(self.cursor_offset())
        } else {
            self.selected_range.end
        };
        self.move_to(target, cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_grapheme(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_grapheme(self.cursor_offset()), cx);
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_word_start(self.cursor_offset()), cx);
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_word_end(self.cursor_offset()), cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(0, cx);
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.content.len(), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        let range = if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            self.previous_grapheme(cursor)..cursor
        } else {
            self.selected_range.clone()
        };
        if range.is_empty() {
            window.play_system_bell();
        } else {
            self.replace_range(range, "", cx);
        }
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        let range = if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            cursor..self.next_grapheme(cursor)
        } else {
            self.selected_range.clone()
        };
        if range.is_empty() {
            window.play_system_bell();
        } else {
            self.replace_range(range, "", cx);
        }
    }

    fn delete_word_backward(
        &mut self,
        _: &DeleteWordBackward,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            self.previous_word_start(cursor)..cursor
        } else {
            self.selected_range.clone()
        };
        if range.is_empty() {
            window.play_system_bell();
        } else {
            self.replace_range(range, "", cx);
        }
    }

    fn delete_word_forward(
        &mut self,
        _: &DeleteWordForward,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            cursor..self.next_word_end(cursor)
        } else {
            self.selected_range.clone()
        };
        if range.is_empty() {
            window.play_system_bell();
        } else {
            self.replace_range(range, "", cx);
        }
    }

    fn delete_to_beginning(
        &mut self,
        _: &DeleteToBeginning,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = if self.selected_range.is_empty() {
            0..self.cursor_offset()
        } else {
            self.selected_range.clone()
        };
        if range.is_empty() {
            window.play_system_bell();
        } else {
            self.replace_range(range, "", cx);
        }
    }

    fn delete_to_end(&mut self, _: &DeleteToEnd, window: &mut Window, cx: &mut Context<Self>) {
        let range = if self.selected_range.is_empty() {
            self.cursor_offset()..self.content.len()
        } else {
            self.selected_range.clone()
        };
        if range.is_empty() {
            window.play_system_bell();
        } else {
            self.replace_range(range, "", cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_owned(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_owned(),
            ));
            self.replace_range(self.selected_range.clone(), "", cx);
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_range(self.selected_range.clone(), &text, cx);
        }
    }

    fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(self.snapshot());
            self.restore(previous, cx);
        }
    }

    fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.snapshot());
            self.restore(next, cx);
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        self.is_selecting = true;
        let index = self.index_for_mouse_position(event.position);
        match event.click_count {
            1 if event.modifiers.shift => self.select_to(index, cx),
            1 => self.move_to(index, cx),
            2 => {
                self.selected_range = self.word_range_at(index);
                self.selection_reversed = false;
                cx.notify();
            }
            _ => {
                self.selected_range = 0..self.content.len();
                self.selection_reversed = false;
                cx.notify();
            }
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds, self.last_layout.as_ref()) else {
            return 0;
        };
        if position.x <= bounds.left() {
            return 0;
        }
        if position.x >= bounds.right() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left() - self.alignment_offset + self.scroll_x)
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn offset_from_utf16_in(text: &str, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        self.replace_range(range, text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        let snapshot = self.snapshot();
        self.undo.push(snapshot);
        self.redo.clear();
        let text = text.replace(['\r', '\n'], " ");
        self.content =
            format!("{}{}{}", &self.content[..range.start], text, &self.content[range.end..])
                .into();
        self.marked_range = (!text.is_empty()).then_some(range.start..range.start + text.len());
        self.selected_range = selected
            .as_ref()
            .map(|selection| {
                Self::offset_from_utf16_in(&text, selection.start)
                    ..Self::offset_from_utf16_in(&text, selection.end)
            })
            .map(|selection| range.start + selection.start..range.start + selection.end)
            .unwrap_or_else(|| range.start + text.len()..range.start + text.len());
        self.selection_reversed = false;
        cx.emit(InputEvent::Edited);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range);
        Some(Bounds::from_corners(
            point(
                bounds.left() + self.alignment_offset + line.x_for_index(range.start)
                    - self.scroll_x,
                bounds.top(),
            ),
            point(
                bounds.left() + self.alignment_offset + line.x_for_index(range.end) - self.scroll_x,
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        let index =
            line.index_for_x(point.x - bounds.left() - self.alignment_offset + self.scroll_x)?;
        Some(self.offset_to_utf16(index))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_x: Pixels,
    alignment_offset: Pixels,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let style = Style {
            size: size(relative(1.).into(), window.line_height().into()),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PrepaintState {
        let style = window.text_style();
        let colors = cx.theme().colors();
        let (
            display_text,
            text_color,
            selected_range,
            cursor,
            marked_range,
            previous_scroll,
            text_align,
        ) = {
            let input = self.input.read(cx);
            let display = if input.content.is_empty() {
                (input.placeholder.clone(), colors.text_muted)
            } else {
                (input.content.clone(), style.color)
            };
            (
                display.0,
                display.1,
                input.selected_range.clone(),
                input.cursor_offset(),
                input.marked_range.clone(),
                input.scroll_x,
                input.text_align,
            )
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked) = marked_range {
            vec![
                TextRun { len: marked.start, ..run.clone() },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun { len: display_text.len() - marked.end, ..run },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window.text_system().shape_line(display_text, font_size, &runs, None);
        let cursor_x = line.x_for_index(cursor);
        let viewport = bounds.size.width.max(px(1.));
        let max_scroll = (line.width - viewport).max(px(0.));
        let mut scroll_x = previous_scroll.min(max_scroll);
        if cursor_x < scroll_x {
            scroll_x = cursor_x;
        } else if cursor_x > scroll_x + viewport - px(2.) {
            scroll_x = (cursor_x - viewport + px(2.)).min(max_scroll);
        }
        let remaining = (viewport - line.width).max(px(0.));
        let alignment_offset = match text_align {
            TextAlign::Left => px(0.),
            TextAlign::Center => remaining / 2.,
            TextAlign::Right => remaining,
        };

        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + alignment_offset + cursor_x - scroll_x, bounds.top()),
                        size(px(1.), bounds.size.height),
                    ),
                    cx.theme().players().local().cursor,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left()
                                + alignment_offset
                                + line.x_for_index(selected_range.start)
                                - scroll_x,
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + alignment_offset + line.x_for_index(selected_range.end)
                                - scroll_x,
                            bounds.bottom(),
                        ),
                    ),
                    colors.element_selection_background,
                )),
                None,
            )
        };
        PrepaintState { line: Some(line), cursor, selection, scroll_x, alignment_offset }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        state: &mut PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.input.read(cx).focus_handle.clone();
        window.handle_input(&focus, ElementInputHandler::new(bounds, self.input.clone()), cx);
        if let Some(selection) = state.selection.take() {
            window.paint_quad(selection);
        }
        let line = state.line.take().expect("prepaint shaped the input line");
        let _ = line.paint(
            point(bounds.left() + state.alignment_offset - state.scroll_x, bounds.top()),
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        );
        if focus.is_focused(window)
            && let Some(cursor) = state.cursor.take()
        {
            window.paint_quad(cursor);
        }
        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.scroll_x = state.scroll_x;
            input.alignment_offset = state.alignment_offset;
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("native-text-input")
            .key_context("NativeTextInput")
            .role(gpui::Role::TextInput)
            .aria_label(self.placeholder.clone())
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .w_full()
            .min_w_0()
            .overflow_hidden()
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word_backward))
            .on_action(cx.listener(Self::delete_word_forward))
            .on_action(cx.listener(Self::delete_to_beginning))
            .on_action(cx.listener(Self::delete_to_end))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_word_left))
            .on_action(cx.listener(Self::select_word_right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::show_character_palette))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(TextElement { input: cx.entity() })
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    struct Harness {
        input: Entity<TextInput>,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(self.input.clone())
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<TextInput>, &mut gpui::VisualTestContext) {
        cx.update(|cx| {
            theme::init(theme::LoadThemes::JustBase, cx);
            init(cx);
        });
        let (harness, cx) = cx.add_window_view(|_, cx| Harness {
            input: cx.new(|cx| TextInput::new("Type here…", cx)),
        });
        let input = cx.read_entity(&harness, |harness, _| harness.input.clone());
        cx.update(|window, cx| window.focus(&input.focus_handle(cx), cx));
        (input, cx)
    }

    fn platform(shortcut: &'static str) -> &'static str {
        #[cfg(target_os = "macos")]
        return match shortcut {
            "select_all" => "cmd-a",
            "copy" => "cmd-c",
            "paste" => "cmd-v",
            "undo" => "cmd-z",
            "word_left" => "alt-left",
            _ => unreachable!(),
        };

        #[cfg(not(target_os = "macos"))]
        return match shortcut {
            "select_all" => "ctrl-a",
            "copy" => "ctrl-c",
            "paste" => "ctrl-v",
            "undo" => "ctrl-z",
            "word_left" => "ctrl-left",
            _ => unreachable!(),
        };
    }

    #[gpui::test]
    fn select_all_and_clipboard_use_platform_shortcuts(cx: &mut TestAppContext) {
        let (input, cx) = setup(cx);
        cx.simulate_input("copy me");
        cx.simulate_keystrokes(platform("select_all"));
        cx.read_entity(&input, |input, _| assert_eq!(input.selected_range, 0..7));
        cx.simulate_keystrokes(platform("copy"));

        input.update(cx, |input, cx| input.clear(cx));
        cx.simulate_keystrokes(platform("paste"));
        cx.read_entity(&input, |input, _| assert_eq!(input.text(), "copy me"));

        cx.simulate_keystrokes(platform("select_all"));
        cx.simulate_input("replaced");
        cx.read_entity(&input, |input, _| assert_eq!(input.text(), "replaced"));
    }

    #[gpui::test]
    fn word_motion_grapheme_deletion_and_undo_match_native_edits(cx: &mut TestAppContext) {
        let (input, cx) = setup(cx);
        cx.simulate_input("alpha beta");
        cx.simulate_keystrokes(platform("word_left"));
        cx.read_entity(&input, |input, _| assert_eq!(input.cursor_offset(), 6));

        input.update(cx, |input, cx| input.set_text("a👨‍👩‍👧‍👦", false, cx));
        cx.simulate_keystrokes("backspace");
        cx.read_entity(&input, |input, _| assert_eq!(input.text(), "a"));
        cx.simulate_keystrokes(platform("undo"));
        cx.read_entity(&input, |input, _| assert_eq!(input.text(), "a👨‍👩‍👧‍👦"));
    }
}
