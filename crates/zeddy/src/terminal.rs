//! Painting a [`Screen`].
//!
//! This is a custom [`Element`] rather than a tree of styled `div`s. A terminal
//! is a grid of thousands of cells that changes many times a second, and a div
//! per cell would put a Taffy layout node per cell on the frame path. Here the
//! whole screen is one element: one shaped line per row, and the runs inside it
//! carry the colours.
//!
//! # The grid is measured here and used elsewhere
//!
//! How many cells fit is a question only the paint pass can answer — it depends
//! on the font metrics and on the bounds the layout gave us. But the *answer*
//! belongs to the session, which has to tell herdr about it. So the element
//! writes the measured grid into a shared [`Fit`] and the view reads it, which
//! is why a resize takes effect on the frame after the one that noticed it.

use std::{cell::Cell as StdCell, rc::Rc};

use gpui::{
    App, Bounds, Element, ElementId, Font, FontWeight, GlobalElementId, Hsla, InspectorElementId,
    IntoElement, LayoutId, Pixels, SharedString, Style, TextAlign, TextRun, UnderlineStyle, Window,
    fill, point, px, size,
};
use zeddy_vt::{Screen, Size};

/// The grid the last paint found room for.
///
/// Shared between the element that measures it and the view that acts on it.
/// A plain `Cell` because both ends are on the window thread.
///
/// Read rather than consumed: the view checks it on every frame and the session
/// ignores a size it is already running at, so a steady window costs one
/// comparison per frame and a dragged one costs a resize per frame.
#[derive(Debug, Clone, Default)]
pub struct Fit(Rc<StdCell<Option<Size>>>);

impl Fit {
    pub fn get(&self) -> Option<Size> {
        self.0.get()
    }

    fn set(&self, size: Size) {
        self.0.set(Some(size));
    }
}

/// How a terminal is drawn: the font, and the colours a cell's `Default` means.
#[derive(Debug, Clone)]
pub struct Appearance {
    pub font: Font,
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub background: Hsla,
    pub cursor: Hsla,
}

/// One screen, painted.
pub struct TerminalElement {
    screen: Screen,
    appearance: Appearance,
    /// A blurred terminal draws a hollow cursor, the way every native terminal
    /// does — it is how you tell at a glance which pane has the keyboard.
    focused: bool,
    fit: Fit,
    /// Resolved by the caller, because only it has the theme.
    colors: Vec<Vec<(Hsla, Hsla)>>,
}

impl TerminalElement {
    pub fn new(
        screen: Screen,
        colors: Vec<Vec<(Hsla, Hsla)>>,
        appearance: Appearance,
        focused: bool,
        fit: Fit,
    ) -> Self {
        Self { screen, appearance, focused, fit, colors }
    }
}

/// What [`Element::prepaint`] worked out and [`Element::paint`] needs.
pub struct Metrics {
    cell: gpui::Size<Pixels>,
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = Metrics;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        // Full width, and *grown* into the remaining height rather than sized
        // at 100% of it: a percentage height against a parent whose own height
        // comes from a flex line resolves to zero, and a terminal one cell tall
        // is not an obvious-looking bug — it looks like a terminal that will not
        // scroll. The parent is a column, so growing is what fills it.
        let style = Style {
            flex_grow: 1.,
            size: size(gpui::relative(1.).into(), gpui::Length::Auto),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        _: &mut App,
    ) -> Metrics {
        // The font is monospace, so one glyph's advance is every glyph's.
        let em = window
            .text_system()
            .shape_line(
                SharedString::from("M"),
                self.appearance.font_size,
                &[Look {
                    fg: gpui::black(),
                    bg: None,
                    bold: false,
                    italic: false,
                    underline: false,
                }
                .run(1, &self.appearance)],
                None,
            )
            .width
            .max(px(1.));
        let cell = size(em, self.appearance.line_height);

        self.fit.set(Size::new(
            (bounds.size.width / cell.width).floor() as u16,
            (bounds.size.height / cell.height).floor() as u16,
        ));

        Metrics { cell }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        metrics: &mut Metrics,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_quad(fill(bounds, self.appearance.background));

        for (index, row) in self.screen.rows.iter().enumerate() {
            let origin = bounds.origin + point(px(0.), metrics.cell.height * index as f32);
            if origin.y > bounds.bottom() {
                break;
            }

            let colors = &self.colors[index];
            let text: String = row.iter().map(|cell| cell.ch).collect();

            // One run per cell would shape every glyph separately; merging
            // neighbours that look alike is what makes a line of plain text one
            // run instead of eighty.
            let mut runs: Vec<TextRun> = Vec::new();
            let mut last: Option<Look> = None;
            for (cell, &(fg, bg)) in row.iter().zip(colors) {
                let look = Look {
                    fg,
                    bg: (bg != self.appearance.background).then_some(bg),
                    bold: cell.style.bold,
                    italic: cell.style.italic,
                    underline: cell.style.underline,
                };
                match (&last, runs.last_mut()) {
                    (Some(previous), Some(run)) if *previous == look => {
                        run.len += cell.ch.len_utf8()
                    }
                    _ => {
                        runs.push(look.run(cell.ch.len_utf8(), &self.appearance));
                        last = Some(look);
                    }
                }
            }

            let line = window.text_system().shape_line(
                SharedString::from(text),
                self.appearance.font_size,
                &runs,
                None,
            );
            let _ = line.paint_background(
                origin,
                metrics.cell.height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
            let _ = line.paint(origin, metrics.cell.height, TextAlign::Left, None, window, cx);
        }

        if let Some(cursor) = self.screen.cursor {
            let origin = bounds.origin
                + point(
                    metrics.cell.width * cursor.col as f32,
                    metrics.cell.height * cursor.row as f32,
                );
            let cell = Bounds { origin, size: metrics.cell };
            if self.focused {
                window.paint_quad(fill(cell, self.appearance.cursor));
            } else {
                let mut hollow =
                    gpui::outline(cell, self.appearance.cursor, gpui::BorderStyle::Solid);
                hollow.border_widths = px(1.).into();
                window.paint_quad(hollow);
            }
        }
    }
}

/// Everything about a cell that decides which run it belongs to.
///
/// Two adjacent cells share a run exactly when their `Look`s are equal, which
/// is a single comparison rather than a rule spread across five fields.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Look {
    fg: Hsla,
    bg: Option<Hsla>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Look {
    fn run(&self, len: usize, appearance: &Appearance) -> TextRun {
        TextRun {
            len,
            font: Font {
                weight: if self.bold { FontWeight::BOLD } else { appearance.font.weight },
                style: if self.italic { gpui::FontStyle::Italic } else { appearance.font.style },
                ..appearance.font.clone()
            },
            color: self.fg,
            background_color: self.bg,
            underline: self.underline.then(|| UnderlineStyle {
                color: Some(self.fg),
                thickness: px(1.),
                wavy: false,
            }),
            strikethrough: None,
        }
    }
}
