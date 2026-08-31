//! zeddy's only VT boundary.
//!
//! Bytes go in, a [`Screen`] comes out. That is the whole contract, and it is
//! the whole reason this crate exists: the renderer above it never sees an
//! escape sequence, and swapping the parser underneath it is a change to one
//! file rather than to the window.
//!
//! # Why alacritty's core
//!
//! It is the parser Zed's own terminal uses, and zeddy is built on Zed's
//! frontend. Taking the same one means the grid semantics the renderer assumes
//! and the grid semantics the parser produces already agree — and, unlike
//! libghostty-vt, it needs no Zig in the build.
//!
//! # Snapshots, not references
//!
//! [`Terminal::screen`] copies. A borrowed grid would be faster and would tie
//! the render pass to the lifetime of the emulator, which is owned by a
//! different thread than the one painting. At the sizes a terminal actually
//! runs — a few thousand cells — the copy is not what makes a frame slow, and
//! the freedom is worth more than the memcpy.

#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use alacritty_terminal::{
    event::{Event, EventListener},
    grid::Dimensions,
    index::{Column, Line, Point},
    term::{Config, cell::Flags},
    vte::ansi::{Color as AnsiColor, NamedColor, Processor},
};

/// A terminal grid, in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

impl Size {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols: cols.max(1), rows: rows.max(1) }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

/// `alacritty_terminal` asks for dimensions through a trait, so [`Size`] answers.
impl Dimensions for Size {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }

    fn screen_lines(&self) -> usize {
        self.rows as usize
    }

    fn columns(&self) -> usize {
        self.cols as usize
    }
}

/// A cell's colour, in the terms the theme resolves rather than in RGB.
///
/// `Default` is deliberately not "black": which colour the default foreground
/// is belongs to the theme, and resolving it here would hard-code one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// The theme's default foreground or background for this position.
    Default,
    /// One of the sixteen ANSI colours, or the 256-colour cube.
    Indexed(u8),
    /// A true-colour value the program asked for exactly.
    Rgb(u8, u8, u8),
}

/// How a cell is drawn, beyond its colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
    /// Foreground and background swap. Resolved by the renderer, because only
    /// it knows what [`Color::Default`] actually is.
    pub inverse: bool,
}

/// One cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: Color::Default, bg: Color::Default, style: Style::default() }
    }
}

/// Where the cursor is, when it is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub col: u16,
    pub row: u16,
}

/// A whole screen, ready to paint, owing nothing to the emulator that made it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    pub size: Size,
    /// `size.rows` rows of `size.cols` cells, top row first.
    pub rows: Vec<Vec<Cell>>,
    pub cursor: Option<Cursor>,
    /// The window title the program last set, if it set one.
    pub title: Option<String>,
}

impl Screen {
    /// The screen as plain text, one line per row, trailing blanks trimmed.
    ///
    /// Not a rendering path — this is what tests assert against and what a
    /// plugin reading a session gets.
    pub fn to_text(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.iter().map(|cell| cell.ch).collect::<String>().trim_end().to_owned())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The emulator reports the window title as an *event*, not as grid state, so
/// something has to be listening for one to be readable at all.
///
/// Everything else the emulator emits — clipboard requests, colour queries,
/// PTY writebacks — is a reply zeddy does not owe: herdr owns the PTY, and a
/// reply written here would never reach it. They are dropped deliberately.
#[derive(Debug, Clone, Default)]
struct TitleSink(Arc<Mutex<Option<String>>>);

impl EventListener for TitleSink {
    fn send_event(&self, event: Event) {
        match event {
            Event::Title(title) => *self.0.lock().expect("title mutex") = Some(title),
            Event::ResetTitle => *self.0.lock().expect("title mutex") = None,
            _ => {}
        }
    }
}

/// A terminal emulator fed by [`Terminal::feed`].
pub struct Terminal {
    term: alacritty_terminal::Term<TitleSink>,
    parser: Processor,
    size: Size,
    title: TitleSink,
}

impl Terminal {
    pub fn new(size: Size) -> Self {
        // No scrollback. herdr's frame stream sends the viewport and has no way
        // to move it back through history, so a scrollback buffer here would be
        // a buffer nothing can ever scroll to. History comes from the control
        // plane instead, and is a different rendering.
        let config = Config { scrolling_history: 0, ..Config::default() };
        let title = TitleSink::default();
        Self {
            term: alacritty_terminal::Term::new(config, &size, title.clone()),
            parser: Processor::new(),
            size,
            title,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    /// Apply a repaint. Bytes must arrive in the order they were produced.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    /// Re-run the grid at a new size.
    ///
    /// Whatever is driving this should expect a full repaint next: a resize
    /// invalidates the diffs the previous frames were measured against.
    pub fn resize(&mut self, size: Size) {
        if size == self.size {
            return;
        }
        self.size = size;
        self.term.resize(size);
    }

    /// Copy the current screen out.
    pub fn screen(&self) -> Screen {
        let grid = self.term.grid();
        let mut rows = Vec::with_capacity(self.size.rows as usize);
        for line in 0..self.size.rows as i32 {
            let mut cells = Vec::with_capacity(self.size.cols as usize);
            for column in 0..self.size.cols as usize {
                cells.push(convert(&grid[Point::new(Line(line), Column(column))]));
            }
            rows.push(cells);
        }

        let cursor = {
            let point = grid.cursor.point;
            let visible =
                self.term.mode().contains(alacritty_terminal::term::TermMode::SHOW_CURSOR);
            visible.then(|| Cursor { col: point.column.0 as u16, row: point.line.0.max(0) as u16 })
        };

        Screen {
            size: self.size,
            rows,
            cursor,
            title: self.title.0.lock().expect("title mutex").clone(),
        }
    }
}

impl std::fmt::Debug for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terminal").field("size", &self.size).finish_non_exhaustive()
    }
}

fn convert(cell: &alacritty_terminal::term::cell::Cell) -> Cell {
    let flags = cell.flags;
    Cell {
        ch: cell.c,
        fg: color(cell.fg),
        bg: color(cell.bg),
        style: Style {
            bold: flags.contains(Flags::BOLD),
            italic: flags.contains(Flags::ITALIC),
            underline: flags.intersects(Flags::ALL_UNDERLINES),
            dim: flags.contains(Flags::DIM),
            inverse: flags.contains(Flags::INVERSE),
        },
    }
}

fn color(color: AnsiColor) -> Color {
    match color {
        AnsiColor::Spec(rgb) => Color::Rgb(rgb.r, rgb.g, rgb.b),
        AnsiColor::Indexed(index) => Color::Indexed(index),
        // The named slots that mean "whatever the theme says" stay `Default`;
        // the sixteen real ANSI names become their indices, which is what a
        // palette is indexed by anyway.
        AnsiColor::Named(
            NamedColor::Foreground
            | NamedColor::Background
            | NamedColor::Cursor
            | NamedColor::DimForeground
            | NamedColor::BrightForeground,
        ) => Color::Default,
        AnsiColor::Named(named) => Color::Indexed(named as u8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen_of(bytes: &[u8]) -> Screen {
        let mut term = Terminal::new(Size::new(20, 3));
        term.feed(bytes);
        term.screen()
    }

    #[test]
    fn plain_text_lands_on_the_grid() {
        assert_eq!(screen_of(b"hello").to_text().lines().next(), Some("hello"));
    }

    #[test]
    fn a_screen_is_always_exactly_its_size() {
        let screen = screen_of(b"hi");
        assert_eq!(screen.rows.len(), 3);
        assert!(screen.rows.iter().all(|row| row.len() == 20));
    }

    #[test]
    fn sgr_colours_reach_the_cells() {
        let screen = screen_of(b"\x1b[31mred");
        assert_eq!(screen.rows[0][0].fg, Color::Indexed(NamedColor::Red as u8));
        assert_eq!(screen.rows[0][0].bg, Color::Default, "background was never set");
    }

    #[test]
    fn true_colour_survives_as_true_colour() {
        let screen = screen_of(b"\x1b[38;2;10;20;30mx");
        assert_eq!(screen.rows[0][0].fg, Color::Rgb(10, 20, 30));
    }

    #[test]
    fn attributes_are_carried_not_flattened_into_colour() {
        let screen = screen_of(b"\x1b[1;3;4mstyled");
        let style = screen.rows[0][0].style;
        assert!(style.bold && style.italic && style.underline);
    }

    #[test]
    fn cursor_addressing_moves_the_cursor() {
        let screen = screen_of(b"\x1b[2;5H");
        assert_eq!(screen.cursor, Some(Cursor { col: 4, row: 1 }));
    }

    #[test]
    fn a_hidden_cursor_is_absent_rather_than_placed_somewhere() {
        assert_eq!(screen_of(b"\x1b[?25l").cursor, None);
    }

    #[test]
    fn an_osc_title_is_picked_up() {
        assert_eq!(screen_of(b"\x1b]0;a session\x07").title.as_deref(), Some("a session"));
    }

    #[test]
    fn feeding_a_repaint_in_two_writes_is_the_same_as_one() {
        let mut split = Terminal::new(Size::new(20, 3));
        split.feed(b"\x1b[3");
        split.feed(b"1mred");
        assert_eq!(split.screen(), screen_of(b"\x1b[31mred"));
    }

    #[test]
    fn resizing_changes_the_shape_of_the_next_snapshot() {
        let mut term = Terminal::new(Size::new(20, 3));
        term.resize(Size::new(40, 10));
        let screen = term.screen();
        assert_eq!(screen.size, Size::new(40, 10));
        assert_eq!(screen.rows.len(), 10);
        assert_eq!(screen.rows[0].len(), 40);
    }

    #[test]
    fn a_zero_sized_grid_is_never_handed_to_the_emulator() {
        assert_eq!(Size::new(0, 0), Size::new(1, 1));
    }
}
