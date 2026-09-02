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
//! writes the measured grid into a shared [`Fit`] and the view reads it. A
//! changed fit explicitly schedules that follow-up frame: pane-tree edits such
//! as splits and tab moves are one-shot events, so there may be no mouse event
//! or terminal repaint to schedule it for us.

use std::{cell::Cell as StdCell, rc::Rc};

use gpui::{
    App, Bounds, Element, ElementId, Font, FontWeight, GlobalElementId, Hsla, InspectorElementId,
    IntoElement, LayoutId, Pixels, Point, SharedString, Style, TextAlign, TextRun, UnderlineStyle,
    Window, fill, point, px, size,
};
use zeddy_vt::{CellPosition, Screen, Size};

/// The grid the last paint found room for.
///
/// Shared between the element that measures it and the view that acts on it.
/// A plain `Cell` because both ends are on the window thread.
///
/// Read rather than consumed: the view checks it on every frame and the session
/// ignores a size it is already running at, so a steady window costs one
/// comparison per frame and a dragged one costs a resize per frame.
#[derive(Debug, Clone, Default)]
pub struct Fit {
    size: Rc<StdCell<Option<Size>>>,
    line_height: Rc<StdCell<Option<Pixels>>>,
    bounds: Rc<StdCell<Option<Bounds<Pixels>>>>,
    cell_width: Rc<StdCell<Option<Pixels>>>,
    scroll_px: Rc<StdCell<f32>>,
}

impl Fit {
    pub fn get(&self) -> Option<Size> {
        self.size.get()
    }

    fn set(&self, size: Size) -> bool {
        self.size.replace(Some(size)) != Some(size)
    }

    fn measure(
        &self,
        size: Size,
        bounds: Bounds<Pixels>,
        cell_width: Pixels,
        line_height: Pixels,
    ) -> bool {
        self.bounds.set(Some(bounds));
        self.cell_width.set(Some(cell_width));
        self.line_height.set(Some(line_height));
        self.set(size)
    }

    /// Resolve a window-space pointer position to the nearest visible cell.
    pub fn cell_at(&self, position: Point<Pixels>) -> Option<CellPosition> {
        let bounds = self.bounds.get()?;
        let cell_width = self.cell_width.get()?;
        let line_height = self.line_height.get()?;
        let size = self.size.get()?;
        let local = position - bounds.origin;
        let col = (local.x / cell_width).floor().clamp(0., f32::from(size.cols - 1)) as u16;
        let row = (local.y / line_height).floor().clamp(0., f32::from(size.rows - 1)) as u16;
        Some(CellPosition::new(col, row))
    }

    /// Quantize a wheel or trackpad gesture into terminal lines.
    ///
    /// Pixel deltas accumulate until they cross a full row, while traditional
    /// mouse-wheel line deltas pass through exactly.
    pub fn wheel_lines(&self, event: &gpui::ScrollWheelEvent) -> Option<i32> {
        let line_height = self.line_height.get()?;
        match event.touch_phase {
            gpui::TouchPhase::Started => {
                self.scroll_px.set(0.);
            }
            gpui::TouchPhase::Ended | gpui::TouchPhase::Cancelled => return None,
            gpui::TouchPhase::Moved => {}
        }

        let line_height = line_height / px(1.);
        let accumulated =
            self.scroll_px.get() + event.delta.pixel_delta(px(line_height)).y / px(1.);
        let lines = (accumulated / line_height).trunc() as i32;
        self.scroll_px.set(accumulated - lines as f32 * line_height);
        (lines != 0).then_some(lines)
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
        cx: &mut App,
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

        let measured = Size::new(
            (bounds.size.width / cell.width).floor() as u16,
            (bounds.size.height / cell.height).floor() as u16,
        );
        let fit_changed = self.fit.measure(measured, bounds, cell.width, cell.height);
        if fit_changed {
            // `Window::refresh` is intentionally ignored while GPUI is in a
            // draw pass. Defer it until the pass completes so the next render
            // can apply this fit before taking the terminal screen snapshot.
            window.defer(cx, |window, _| window.refresh());
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_reports_only_real_grid_changes() {
        let fit = Fit::default();
        assert!(fit.set(Size::new(80, 24)));
        assert!(!fit.set(Size::new(80, 24)));
        assert!(fit.set(Size::new(120, 40)));
        assert_eq!(fit.get(), Some(Size::new(120, 40)));
    }

    #[test]
    fn wheel_deltas_are_measured_in_terminal_lines() {
        let fit = Fit::default();
        fit.measure(
            Size::new(80, 24),
            Bounds::new(point(px(0.), px(0.)), size(px(800.), px(480.))),
            px(10.),
            px(20.),
        );
        let event = gpui::ScrollWheelEvent {
            delta: gpui::ScrollDelta::Lines(point(0., 2.)),
            ..Default::default()
        };

        assert_eq!(fit.wheel_lines(&event), Some(2));
    }

    #[test]
    fn trackpad_pixels_accumulate_to_complete_rows() {
        let fit = Fit::default();
        fit.measure(
            Size::new(80, 24),
            Bounds::new(point(px(0.), px(0.)), size(px(800.), px(480.))),
            px(10.),
            px(20.),
        );
        let event = |pixels| gpui::ScrollWheelEvent {
            delta: gpui::ScrollDelta::Pixels(point(px(0.), px(pixels))),
            ..Default::default()
        };

        assert_eq!(fit.wheel_lines(&event(9.)), None);
        assert_eq!(fit.wheel_lines(&event(11.)), Some(1));
    }

    #[test]
    fn a_trackpad_gestures_first_delta_is_not_dropped() {
        let fit = Fit::default();
        fit.measure(
            Size::new(80, 24),
            Bounds::new(point(px(0.), px(0.)), size(px(800.), px(480.))),
            px(10.),
            px(20.),
        );
        let event = gpui::ScrollWheelEvent {
            delta: gpui::ScrollDelta::Pixels(point(px(0.), px(20.))),
            touch_phase: gpui::TouchPhase::Started,
            ..Default::default()
        };

        assert_eq!(fit.wheel_lines(&event), Some(1));
    }

    #[test]
    fn pointer_positions_resolve_to_bounded_terminal_cells() {
        let fit = Fit::default();
        fit.measure(
            Size::new(80, 24),
            Bounds::new(point(px(10.), px(20.)), size(px(800.), px(480.))),
            px(10.),
            px(20.),
        );

        assert_eq!(fit.cell_at(point(px(35.), px(65.))), Some(CellPosition::new(2, 2)));
        assert_eq!(fit.cell_at(point(px(0.), px(0.))), Some(CellPosition::new(0, 0)));
        assert_eq!(fit.cell_at(point(px(900.), px(600.))), Some(CellPosition::new(79, 23)));
    }
}
