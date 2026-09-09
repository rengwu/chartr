//! A single native buffer with atomic inline template objects. Its private,
//! self-contained token representation survives the editor's normal clipboard
//! and undo stack; persisted documents still use structured `Part` values.
use super::{TemplateChip, document::Part};
use editor::{Editor, FoldPlaceholder, MultiBufferOffset, SelectionEffects, display_map::Crease};
use gpui::{
    App, Context, Entity, Focusable, InteractiveElement, IntoElement, Pixels, Point, Styled,
    Window, div,
};
use std::{ops::Range, sync::Arc};
use ui::{ActiveTheme, Label, Tooltip, prelude::*};

const START: char = '\u{e000}';
const END: char = '\u{e001}';

pub fn token(part: &Part) -> String {
    let bytes = serde_json::to_vec(part).expect("parts serialize");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("{START}{hex}{END}")
}
pub fn encode(parts: &[Part]) -> String {
    parts
        .iter()
        .map(|part| match part {
            Part::Text { text } => text.clone(),
            _ => token(part),
        })
        .collect()
}
pub fn objects(text: &str) -> Vec<(Range<usize>, Part)> {
    let mut items = Vec::new();
    let mut cursor = 0;
    while let Some(start) = text[cursor..].find(START).map(|i| i + cursor) {
        let body = start + START.len_utf8();
        let Some(end) = text[body..].find(END).map(|i| i + body) else { break };
        let hex = &text[body..end];
        let decoded = if hex.is_ascii() && hex.len().is_multiple_of(2) {
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                .collect::<Result<Vec<_>, _>>()
                .ok()
        } else {
            None
        };
        if let Some(Part::Template { provider, id, title }) =
            decoded.and_then(|bytes| serde_json::from_slice::<Part>(&bytes).ok())
        {
            items.push((start..end + END.len_utf8(), Part::Template { provider, id, title }));
            cursor = end + END.len_utf8();
        } else {
            cursor = body;
        }
    }
    items
}
pub fn decode(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    for (range, part) in objects(text) {
        if cursor < range.start {
            parts.push(Part::Text { text: text[cursor..range.start].into() });
        }
        parts.push(part);
        cursor = range.end;
    }
    if cursor < text.len() || parts.is_empty() {
        parts.push(Part::Text { text: text[cursor..].into() });
    }
    parts
}

pub fn decorate(
    editor: &Entity<Editor>,
    templates: &[TemplateChip],
    window: &mut Window,
    cx: &mut App,
) {
    let text = editor.read(cx).text(cx);
    let weak = editor.downgrade();
    let entity_id = editor.entity_id();
    let creases = objects(&text)
        .into_iter()
        .map(|(range, part)| {
            let Part::Template { provider, id, title } = part else { unreachable!() };
            let live = templates.iter().find(|t| t.provider == provider && t.id == id);
            let title = live.map(|t| t.title.clone()).unwrap_or(title);
            let unavailable = live.is_none();
            let drag = TemplateChip {
                provider: provider.clone(),
                id,
                title: title.clone(),
                origin: Some((entity_id, range.clone())),
            };
            let weak = weak.clone();
            let placeholder = FoldPlaceholder {
                constrain_width: false,
                merge_adjacent: false,
                render: Arc::new(move |fold_id, anchors, cx| {
                    let weak = weak.clone();
                    let label = title.clone();
                    let tooltip = format!(
                        "{} · {}",
                        provider,
                        if unavailable {
                            "Unavailable — enable its provider or remove this item"
                        } else {
                            "Template · select and delete like text"
                        }
                    );
                    div()
                        .id(fold_id)
                        .px_1()
                        .rounded_sm()
                        .border_1()
                        .border_color(if unavailable {
                            cx.theme().status().error
                        } else {
                            cx.theme().colors().border
                        })
                        .bg(cx.theme().colors().element_background)
                        .text_color(cx.theme().colors().text)
                        .cursor_default()
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            if let Some(editor) = weak.upgrade() {
                                window.focus(&editor.focus_handle(cx), cx);
                                editor.update(cx, |editor, cx| {
                                    editor.change_selections(
                                        SelectionEffects::no_scroll(),
                                        window,
                                        cx,
                                        |s| s.select_anchor_ranges([anchors.clone()]),
                                    )
                                });
                            }
                        })
                        .on_drag(drag.clone(), |drag, _, _, cx| cx.new(|_| drag.clone()))
                        .tooltip(Tooltip::text(tooltip))
                        .child(Label::new(label))
                        .into_any_element()
                }),
                ..FoldPlaceholder::default()
            };
            Crease::simple(
                MultiBufferOffset(range.start)..MultiBufferOffset(range.end),
                placeholder,
            )
        })
        .collect();
    editor.update(cx, |editor, cx| {
        editor.unfold_ranges(
            &[MultiBufferOffset(0)..MultiBufferOffset(text.len())],
            true,
            false,
            cx,
        );
        editor.fold_creases(creases, false, window, cx);
    });
}

/// Find the closest legal caret boundary using the editor's shaped layout.
/// Binary search keeps hit testing cheap even for long, wrapped compositions.
pub fn place_caret(
    editor: &mut Editor,
    position: Point<Pixels>,
    window: &mut Window,
    cx: &mut Context<Editor>,
) {
    let Some(bounds) = editor.last_bounds().copied() else { return };
    let snapshot = editor.snapshot(window, cx);
    let text = editor.text(cx);
    let objects = objects(&text);
    let mut item = 0;
    let mut offsets = Vec::new();
    for offset in text.char_indices().map(|(i, _)| i).chain(std::iter::once(text.len())) {
        while item < objects.len() && objects[item].0.end <= offset {
            item += 1;
        }
        if item == objects.len() || offset <= objects[item].0.start {
            offsets.push(offset);
        }
    }
    let point_at = |editor: &mut Editor, index: usize, window: &mut Window, cx: &mut App| {
        let anchor = snapshot.buffer_snapshot().anchor_before(MultiBufferOffset(offsets[index]));
        editor
            .to_pixel_point(anchor, &snapshot, window, cx)
            .unwrap_or(Point::new(gpui::px(-100000.), gpui::px(-100000.)))
    };
    let position = position - bounds.origin;
    // The y coordinate of a caret is the line's top, not its baseline.
    let mut low = 0;
    let mut high = offsets.len();
    while low < high {
        let mid = (low + high) / 2;
        if point_at(editor, mid, window, cx).y <= position.y {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    let row_end = low.saturating_sub(1);
    let row_y = point_at(editor, row_end, window, cx).y;
    low = 0;
    high = row_end;
    while low < high {
        let mid = (low + high) / 2;
        if point_at(editor, mid, window, cx).y < row_y {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    let row_start = low;
    low = row_start;
    high = row_end;
    while low < high {
        let mid = (low + high) / 2;
        if point_at(editor, mid, window, cx).x < position.x {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    let mut nearest = low;
    if low > row_start
        && (point_at(editor, low - 1, window, cx).x - position.x).abs()
            < (point_at(editor, low, window, cx).x - position.x).abs()
    {
        nearest -= 1;
    }
    let offset = MultiBufferOffset(offsets[nearest]);
    editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
        s.select_ranges([offset..offset])
    });
}

pub fn insert(
    editor: &mut Editor,
    chip: &TemplateChip,
    window: &mut Window,
    cx: &mut Context<Editor>,
) {
    let part = Part::Template {
        provider: chip.provider.clone(),
        id: chip.id.clone(),
        title: chip.title.clone(),
    };
    let encoded = token(&part);
    editor.transact(window, cx, |editor, window, cx| {
        if let Some((source, range)) = &chip.origin {
            if *source == cx.entity_id() {
                let text = editor.text(cx);
                if text.get(range.clone()).is_some_and(|value| !objects(value).is_empty()) {
                    let snapshot = editor.snapshot(window, cx);
                    let target = editor.selections.newest::<MultiBufferOffset>(&snapshot).head().0;
                    if range.contains(&target) || target == range.end {
                        return;
                    }
                    editor.edit(
                        [(MultiBufferOffset(range.start)..MultiBufferOffset(range.end), "")],
                        cx,
                    );
                    let target = MultiBufferOffset(if target > range.end {
                        target - range.len()
                    } else {
                        target
                    });
                    editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                        s.select_ranges([target..target])
                    });
                }
            }
        }
        editor.insert(&encoded, window, cx);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structured_compositions_round_trip_and_invalid_tokens_stay_text() {
        let parts = vec![
            Part::Text { text: "Before 🦀 ".into() },
            Part::Template {
                provider: "skills".into(),
                id: "sources".into(),
                title: "资料".into(),
            },
            Part::Text { text: " after\nnext line".into() },
        ];
        assert_eq!(decode(&encode(&parts)), parts);
        let invalid = format!("hello {START}not-a-template{END}");
        assert_eq!(decode(&invalid), vec![Part::Text { text: invalid }]);
    }
}

#[cfg(test)]
mod editing_tests {
    use super::*;
    #[gpui::test]
    fn inline_objects_delete_atomically_and_undo_restores_identity(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
            crate::text_input::init(cx);
            crate::prompts_plugin::init(cx);
        });
        let (editor, cx) = cx.add_window_view(|window, cx| Editor::auto_height(4, 20, window, cx));
        let item = Part::Template {
            provider: "example".into(),
            id: "stable".into(),
            title: "Inline item".into(),
        };
        let initial = vec![
            Part::Text { text: "Before 🦀 ".into() },
            item.clone(),
            Part::Text { text: " after\nnext line".into() },
        ];
        let text = encode(&initial);
        let range = objects(&text)[0].0.clone();
        editor.update_in(cx, |editor, window, cx| {
            editor.set_text(text.clone(), window, cx);
            editor.finalize_last_transaction(cx);
            let offset = MultiBufferOffset(range.end);
            editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                s.select_ranges([offset..offset])
            });
        });
        cx.update(|window, cx| {
            decorate(&editor, &[], window, cx);
            window.focus(&editor.focus_handle(cx), cx);
        });
        cx.simulate_keystrokes("backspace");
        assert_eq!(
            cx.read_entity(&editor, |editor, cx| editor.text(cx)),
            "Before 🦀  after\nnext line"
        );
        cx.simulate_keystrokes("cmd-z");
        assert_eq!(cx.read_entity(&editor, |editor, cx| decode(&editor.text(cx))), initial);
        cx.update(|window, cx| decorate(&editor, &[], window, cx));
        editor.update_in(cx, |editor, window, cx| {
            let start = MultiBufferOffset(range.start);
            let end = MultiBufferOffset(range.end);
            editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                s.select_ranges([start..end])
            });
        });
        cx.simulate_keystrokes("cmd-c");
        editor.update_in(cx, |editor, window, cx| {
            let end = MultiBufferOffset(editor.text(cx).len());
            editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                s.select_ranges([end..end])
            });
        });
        cx.simulate_keystrokes("cmd-v");
        let pasted = cx.read_entity(&editor, |editor, cx| decode(&editor.text(cx)));
        assert_eq!(pasted.last(), Some(&item));
        assert_eq!(pasted.iter().filter(|part| matches!(part, Part::Template { .. })).count(), 2);
        editor.update_in(cx, |editor, window, cx| {
            let text = editor.text(cx);
            let range = objects(&text).last().unwrap().0.clone();
            editor.finalize_last_transaction(cx);
            let offset = MultiBufferOffset(0);
            editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                s.select_ranges([offset..offset])
            });
            let chip = TemplateChip {
                provider: "example".into(),
                id: "stable".into(),
                title: "Inline item".into(),
                origin: Some((cx.entity_id(), range)),
            };
            insert(editor, &chip, window, cx);
        });
        let moved = cx.read_entity(&editor, |editor, cx| decode(&editor.text(cx)));
        assert_eq!(moved.first(), Some(&item));
        assert_eq!(moved.iter().filter(|part| matches!(part, Part::Template { .. })).count(), 2);
        cx.simulate_keystrokes("cmd-z");
        assert_eq!(cx.read_entity(&editor, |editor, cx| decode(&editor.text(cx))), pasted);
    }
}
