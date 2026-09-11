//! Native prose rendering. Markdown is text; no HTML or remote images execute.
use gpui::{AnyElement, FontStyle, FontWeight, HighlightStyle, StyledText, Window, div};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ui::prelude::*;

#[derive(Default)]
struct Block {
    text: String,
    spans: Vec<(std::ops::Range<usize>, HighlightStyle)>,
    code: bool,
    heading: bool,
}

pub fn render(text: &str, window: &Window, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let mut blocks = Vec::new();
    let mut block = Block::default();
    let (mut bold, mut italic, mut list) = (0usize, 0usize, 0usize);
    let flush = |block: &mut Block, blocks: &mut Vec<Block>| {
        if !block.text.is_empty() {
            blocks.push(std::mem::take(block));
        }
    };
    for event in Parser::new(text) {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                flush(&mut block, &mut blocks);
                block.code = true;
            }
            Event::Start(Tag::Heading { .. }) => {
                flush(&mut block, &mut blocks);
                block.heading = true;
            }
            Event::Start(Tag::Strong) => bold += 1,
            Event::End(TagEnd::Strong) => bold -= 1,
            Event::Start(Tag::Emphasis) => italic += 1,
            Event::End(TagEnd::Emphasis) => italic -= 1,
            Event::Start(Tag::List(_)) => list += 1,
            Event::End(TagEnd::List(_)) => list -= 1,
            Event::Start(Tag::Item) => {
                flush(&mut block, &mut blocks);
                block.text.push_str(&format!("{}• ", "  ".repeat(list.saturating_sub(1))));
            }
            Event::End(
                TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::CodeBlock | TagEnd::Item,
            ) => flush(&mut block, &mut blocks),
            Event::Text(value) | Event::Html(value) | Event::InlineHtml(value) => {
                let start = block.text.len();
                block.text.push_str(&value);
                if bold > 0 || italic > 0 {
                    block.spans.push((
                        start..block.text.len(),
                        HighlightStyle {
                            font_weight: (bold > 0).then_some(FontWeight::SEMIBOLD),
                            font_style: (italic > 0).then_some(FontStyle::Italic),
                            ..Default::default()
                        },
                    ));
                }
            }
            Event::Code(value) => {
                let start = block.text.len();
                block.text.push_str(&value);
                block.spans.push((
                    start..block.text.len(),
                    HighlightStyle {
                        background_color: Some(colors.element_background),
                        ..Default::default()
                    },
                ));
            }
            Event::SoftBreak | Event::HardBreak => block.text.push('\n'),
            Event::Rule => {
                flush(&mut block, &mut blocks);
                block.text.push_str("────");
                flush(&mut block, &mut blocks);
            }
            _ => {}
        }
    }
    flush(&mut block, &mut blocks);
    v_flex()
        .w_full()
        .gap_3()
        .children(blocks.into_iter().map(|block| {
            let mut style = window.text_style();
            if block.heading {
                style.font_weight = FontWeight::SEMIBOLD;
            }
            if block.code {
                style.font_family = theme::theme_settings(cx).buffer_font(cx).family.clone();
            }
            div()
                .w_full()
                .line_height(relative(1.6))
                .when(block.code, |view| {
                    view.p_3()
                        .rounded_md()
                        .bg(colors.element_background)
                        .font_family(theme::theme_settings(cx).buffer_font(cx).family.clone())
                        .text_size(rems(0.9))
                })
                .child(StyledText::new(block.text).with_default_highlights(&style, block.spans))
        }))
        .into_any_element()
}
