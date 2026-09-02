//! zeddy's only VT boundary.
//!
//! Output bytes go in and a [`Screen`] comes out; normalized [`KeyEvent`]s go
//! in and terminal input bytes come out. This is the whole reason the crate
//! exists: neither the renderer nor the keyboard adapter above it sees an
//! escape sequence or an upstream terminal type.
//!
//! # Two deliberately different cores
//!
//! Alacritty parses output because it is the parser Zed's own terminal uses and
//! its grid semantics already agree with Zeddy's renderer. libghostty encodes
//! input because it implements the legacy, xterm, fixterms, and Kitty keyboard
//! protocols as one mode-aware encoder. Both are private implementation
//! details of this boundary.
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
    grid::{Dimensions, Scroll as AlacrittyScroll},
    index::{Column, Line, Point},
    term::{Config, TermMode, cell::Flags},
    vte::ansi::{Color as AnsiColor, NamedColor, Processor},
};
use libghostty_vt::key;

/// An error produced while turning a normalized key event into terminal bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEncodingError(String);

impl std::fmt::Display for KeyEncodingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for KeyEncodingError {}

impl From<libghostty_vt::Error> for KeyEncodingError {
    fn from(error: libghostty_vt::Error) -> Self {
        Self(error.to_string())
    }
}

/// Declares the normalized keyboard and its Ghostty counterpart together, so
/// adding a key cannot leave either the physical identity or its unshifted
/// character behind.
macro_rules! key_codes {
    ($($name:ident => $upstream:ident, $unshifted:expr;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[allow(missing_docs)]
        pub enum KeyCode {
            $($name,)*
        }

        impl KeyCode {
            const ALL: &'static [Self] = &[$(Self::$name,)*];

            /// Find the physical key which types `character` without modifiers.
            pub fn typing(character: char) -> Option<Self> {
                Self::ALL.iter().copied().find(|key| key.unshifted() == Some(character))
            }

            /// The character this key types without modifiers, when it has one.
            pub fn unshifted(self) -> Option<char> {
                match self {
                    $(Self::$name => $unshifted,)*
                }
            }

            fn upstream(self) -> key::Key {
                match self {
                    $(Self::$name => key::Key::$upstream,)*
                }
            }
        }
    };
}

key_codes! {
    A => A, Some('a'); B => B, Some('b'); C => C, Some('c'); D => D, Some('d');
    E => E, Some('e'); F => F, Some('f'); G => G, Some('g'); H => H, Some('h');
    I => I, Some('i'); J => J, Some('j'); K => K, Some('k'); L => L, Some('l');
    M => M, Some('m'); N => N, Some('n'); O => O, Some('o'); P => P, Some('p');
    Q => Q, Some('q'); R => R, Some('r'); S => S, Some('s'); T => T, Some('t');
    U => U, Some('u'); V => V, Some('v'); W => W, Some('w'); X => X, Some('x');
    Y => Y, Some('y'); Z => Z, Some('z');

    Digit0 => Digit0, Some('0'); Digit1 => Digit1, Some('1');
    Digit2 => Digit2, Some('2'); Digit3 => Digit3, Some('3');
    Digit4 => Digit4, Some('4'); Digit5 => Digit5, Some('5');
    Digit6 => Digit6, Some('6'); Digit7 => Digit7, Some('7');
    Digit8 => Digit8, Some('8'); Digit9 => Digit9, Some('9');

    Backquote => Backquote, Some('`'); Backslash => Backslash, Some('\\');
    BracketLeft => BracketLeft, Some('['); BracketRight => BracketRight, Some(']');
    Comma => Comma, Some(','); Equal => Equal, Some('='); Minus => Minus, Some('-');
    Period => Period, Some('.'); Quote => Quote, Some('\''); Semicolon => Semicolon, Some(';');
    Slash => Slash, Some('/'); Space => Space, Some(' ');

    Enter => Enter, None; Tab => Tab, None; Escape => Escape, None;
    Backspace => Backspace, None; Delete => Delete, None; Insert => Insert, None;
    Home => Home, None; End => End, None; PageUp => PageUp, None; PageDown => PageDown, None;
    ArrowUp => ArrowUp, None; ArrowDown => ArrowDown, None;
    ArrowLeft => ArrowLeft, None; ArrowRight => ArrowRight, None;

    F1 => F1, None; F2 => F2, None; F3 => F3, None; F4 => F4, None;
    F5 => F5, None; F6 => F6, None; F7 => F7, None; F8 => F8, None;
    F9 => F9, None; F10 => F10, None; F11 => F11, None; F12 => F12, None;
    F13 => F13, None; F14 => F14, None; F15 => F15, None; F16 => F16, None;
    F17 => F17, None; F18 => F18, None; F19 => F19, None; F20 => F20, None;
    F21 => F21, None; F22 => F22, None; F23 => F23, None; F24 => F24, None;
    F25 => F25, None;

    BrowserBack => BrowserBack, None; BrowserForward => BrowserForward, None;
    Copy => Copy, None; Cut => Cut, None; Paste => Paste, None;
    Unidentified => Unidentified, None;
}

/// One keyboard event after the window system's spelling has been normalized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    /// Text after Shift/layout processing but before Control/Alt transformations.
    pub text: Option<String>,
    pub action: KeyAction,
    pub modifiers: Modifiers,
    pub consumed_modifiers: Modifiers,
    pub composing: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyAction {
    #[default]
    Press,
    Repeat,
    Release,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
    pub super_key: bool,
}

/// The terminal modes which affect keyboard encoding.
///
/// This copyable snapshot is the seam between Zeddy's background-owned output
/// parser and the window-thread-only Ghostty encoder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyboardModes {
    cursor_key_application: bool,
    keypad_key_application: bool,
    disambiguate_escape_codes: bool,
    report_event_types: bool,
    report_alternate_keys: bool,
    report_all_keys: bool,
    report_associated_text: bool,
}

/// Ghostty's keyboard encoder, kept separate because the safe binding is not
/// `Send` and must remain on the window thread which created it.
#[derive(Debug)]
pub struct KeyEncoder(key::Encoder<'static>);

impl KeyEncoder {
    pub fn new() -> Result<Self, KeyEncodingError> {
        Ok(Self(key::Encoder::new()?))
    }

    /// Encode one normalized event under an active terminal's mode snapshot.
    pub fn encode(
        &mut self,
        input: &KeyEvent,
        modes: KeyboardModes,
    ) -> Result<Vec<u8>, KeyEncodingError> {
        self.0
            .set_cursor_key_application(modes.cursor_key_application)
            .set_keypad_key_application(modes.keypad_key_application)
            .set_alt_esc_prefix(true)
            .set_modify_other_keys_state_2(false)
            .set_kitty_flags(kitty_flags(modes))
            .set_macos_option_as_alt(key::OptionAsAlt::True)
            .set_backarrow_key_mode(false);

        let mut event = key::Event::new()?;
        event
            .set_action(match input.action {
                KeyAction::Press => key::Action::Press,
                KeyAction::Repeat => key::Action::Repeat,
                KeyAction::Release => key::Action::Release,
            })
            .set_key(input.code.upstream())
            .set_mods(key_modifiers(input.modifiers))
            .set_consumed_mods(key_modifiers(input.consumed_modifiers))
            .set_composing(input.composing);
        if let Some(text) = &input.text {
            event.set_utf8(Some(text.clone()));
        }
        if let Some(codepoint) = input
            .code
            .unshifted()
            .or_else(|| input.text.as_ref().and_then(|text| text.chars().next()))
        {
            event.set_unshifted_codepoint(codepoint);
        }

        let mut encoded = Vec::new();
        self.0.encode_to_vec(&event, &mut encoded)?;
        Ok(encoded)
    }
}

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

struct Emulation {
    term: alacritty_terminal::Term<TitleSink>,
    parser: Processor,
}

impl Emulation {
    fn new(size: Size, scrolling_history: usize, title: TitleSink) -> Self {
        // This permits applications to negotiate Kitty keyboard modes. It does
        // not enable any flag by itself; legacy encoding remains the default.
        let config = Config { scrolling_history, kitty_keyboard: true, ..Config::default() };
        Self { term: alacritty_terminal::Term::new(config, &size, title), parser: Processor::new() }
    }
}

/// The outcome of trying to move the visible viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollResult {
    Changed,
    NeedsHistory,
    Unchanged,
}

/// A terminal emulator fed by [`Terminal::feed`].
pub struct Terminal {
    live: Emulation,
    history: Option<Emulation>,
    history_stale: bool,
    generation: u64,
    size: Size,
    title: TitleSink,
}

impl Terminal {
    pub fn new(size: Size) -> Self {
        let title = TitleSink::default();
        Self {
            // Repaint frames describe only the live viewport. Letting them
            // manufacture local history retains arbitrary repaint artifacts,
            // so real history is loaded separately from Herdr's control plane.
            live: Emulation::new(size, 0, title.clone()),
            history: None,
            history_stale: false,
            generation: 0,
            size,
            title,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    /// Apply a repaint. Bytes must arrive in the order they were produced.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.live.parser.advance(&mut self.live.term, bytes);
        self.generation = self.generation.wrapping_add(1);
        self.history_stale = self.history.is_some();
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
        self.live.term.resize(size);
        self.history = None;
        self.history_stale = false;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Move through the most recently loaded host scrollback.
    pub fn scroll(&mut self, lines: i32) -> ScrollResult {
        if lines == 0 {
            return ScrollResult::Unchanged;
        }
        let Some(history) = self.history.as_mut() else {
            return if lines > 0 { ScrollResult::NeedsHistory } else { ScrollResult::Unchanged };
        };
        if lines > 0 && self.history_stale && history.term.grid().display_offset() == 0 {
            return ScrollResult::NeedsHistory;
        }

        let before = history.term.grid().display_offset();
        history.term.scroll_display(AlacrittyScroll::Delta(lines));
        if before != history.term.grid().display_offset() {
            ScrollResult::Changed
        } else {
            ScrollResult::Unchanged
        }
    }

    /// A token for deciding whether live output arrived during an asynchronous
    /// history request.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Take a copyable snapshot of the modes which affect keyboard encoding.
    pub fn keyboard_modes(&self) -> KeyboardModes {
        let mode = self.live.term.mode();
        KeyboardModes {
            cursor_key_application: mode.contains(TermMode::APP_CURSOR),
            keypad_key_application: mode.contains(TermMode::APP_KEYPAD),
            disambiguate_escape_codes: mode.contains(TermMode::DISAMBIGUATE_ESC_CODES),
            report_event_types: mode.contains(TermMode::REPORT_EVENT_TYPES),
            report_alternate_keys: mode.contains(TermMode::REPORT_ALTERNATE_KEYS),
            report_all_keys: mode.contains(TermMode::REPORT_ALL_KEYS_AS_ESC),
            report_associated_text: mode.contains(TermMode::REPORT_ASSOCIATED_TEXT),
        }
    }

    /// Replace the historical snapshot with ANSI-styled rows from Herdr, then
    /// apply the wheel movement that requested them.
    pub fn load_history(&mut self, ansi: &str, lines: i32, requested_at: u64) -> bool {
        let mut history =
            Emulation::new(self.size, Config::default().scrolling_history, TitleSink::default());
        let ansi = crlf(ansi);
        history.parser.advance(&mut history.term, &ansi);

        // This snapshot may have become stale while it was in flight, but it
        // is still the answer to the gesture that requested it. Apply that
        // gesture once; `scroll` will require a refresh after returning to the
        // live viewport.
        let before = history.term.grid().display_offset();
        history.term.scroll_display(AlacrittyScroll::Delta(lines));
        let changed = before != history.term.grid().display_offset();
        self.history = Some(history);
        self.history_stale = self.generation != requested_at;
        changed
    }

    /// Copy the current screen out.
    pub fn screen(&self) -> Screen {
        let term = self
            .history
            .as_ref()
            .filter(|history| history.term.grid().display_offset() > 0)
            .map(|history| &history.term)
            .unwrap_or(&self.live.term);
        screen(term, self.size, &self.title)
    }
}

fn key_modifiers(modifiers: Modifiers) -> key::Mods {
    let mut result = key::Mods::empty();
    result.set(key::Mods::SHIFT, modifiers.shift);
    result.set(key::Mods::ALT, modifiers.alt);
    result.set(key::Mods::CTRL, modifiers.control);
    result.set(key::Mods::SUPER, modifiers.super_key);
    result
}

fn kitty_flags(modes: KeyboardModes) -> key::KittyKeyFlags {
    let mut flags = key::KittyKeyFlags::DISABLED;
    flags.set(key::KittyKeyFlags::DISAMBIGUATE, modes.disambiguate_escape_codes);
    flags.set(key::KittyKeyFlags::REPORT_EVENTS, modes.report_event_types);
    flags.set(key::KittyKeyFlags::REPORT_ALTERNATES, modes.report_alternate_keys);
    flags.set(key::KittyKeyFlags::REPORT_ALL, modes.report_all_keys);
    flags.set(key::KittyKeyFlags::REPORT_ASSOCIATED, modes.report_associated_text);
    flags
}

fn screen(term: &alacritty_terminal::Term<TitleSink>, size: Size, title: &TitleSink) -> Screen {
    let grid = term.grid();
    let mode = term.mode();
    let display_offset = i32::try_from(grid.display_offset()).unwrap_or(i32::MAX);
    let mut rows = Vec::with_capacity(size.rows as usize);
    for line in 0..size.rows as i32 {
        let mut cells = Vec::with_capacity(size.cols as usize);
        for column in 0..size.cols as usize {
            let line = Line(line.saturating_sub(display_offset));
            cells.push(convert(&grid[Point::new(line, Column(column))]));
        }
        rows.push(cells);
    }

    let cursor = {
        let point = grid.cursor.point;
        let visible = mode.contains(alacritty_terminal::term::TermMode::SHOW_CURSOR);
        let viewport_row = point.line.0.saturating_add(display_offset);
        (visible && viewport_row >= 0 && viewport_row < i32::from(size.rows))
            .then_some(Cursor { col: point.column.0 as u16, row: viewport_row as u16 })
    };

    Screen { size, rows, cursor, title: title.0.lock().expect("title mutex").clone() }
}

fn crlf(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut previous = None;
    for byte in text.bytes() {
        if byte == b'\n' && previous != Some(b'\r') {
            bytes.push(b'\r');
        }
        bytes.push(byte);
        previous = Some(byte);
    }
    bytes
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

    fn press(code: KeyCode, text: Option<&str>, modifiers: Modifiers) -> KeyEvent {
        KeyEvent {
            code,
            text: text.map(str::to_owned),
            action: KeyAction::Press,
            modifiers,
            consumed_modifiers: Modifiers::default(),
            composing: false,
        }
    }

    fn encoded(term: &Terminal, event: &KeyEvent) -> Vec<u8> {
        KeyEncoder::new().unwrap().encode(event, term.keyboard_modes()).unwrap()
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
    fn host_history_can_move_the_visible_viewport() {
        let mut term = Terminal::new(Size::new(20, 3));
        term.feed(b"one\r\ntwo\r\nthree\r\nfour");
        assert_eq!(term.screen().to_text(), "two\nthree\nfour");
        assert_eq!(term.scroll(1), ScrollResult::NeedsHistory);

        let requested_at = term.generation();
        assert!(term.load_history("\x1b[31mone\x1b[0m\ntwo\nthree\nfour", 1, requested_at,));
        let history = term.screen();
        assert_eq!(history.to_text(), "one\ntwo\nthree");
        assert_eq!(history.rows[0][0].fg, Color::Indexed(NamedColor::Red as u8));
        assert_eq!(history.cursor, None, "the live cursor is outside the historical viewport");

        assert_eq!(term.scroll(-1), ScrollResult::Changed);
        assert_eq!(term.screen().to_text(), "two\nthree\nfour");
        assert_eq!(term.scroll(-1), ScrollResult::Unchanged);
    }

    #[test]
    fn live_output_marks_a_bottomed_history_snapshot_for_refresh() {
        let mut term = Terminal::new(Size::new(20, 3));
        let requested_at = term.generation();
        assert!(term.load_history("one\ntwo\nthree\nfour", 1, requested_at));
        assert_eq!(term.scroll(-1), ScrollResult::Changed);

        term.feed(b"new output");
        assert_eq!(term.scroll(1), ScrollResult::NeedsHistory);
    }

    #[test]
    fn history_loaded_after_live_output_is_already_stale() {
        let mut term = Terminal::new(Size::new(20, 3));
        let requested_at = term.generation();

        term.feed(b"latest\r\n");
        assert!(term.load_history("one\ntwo\nthree\nfour", 1, requested_at));
        assert_eq!(term.scroll(-1), ScrollResult::Changed);
        assert_eq!(term.scroll(1), ScrollResult::NeedsHistory);
    }

    #[test]
    fn a_zero_sized_grid_is_never_handed_to_the_emulator() {
        assert_eq!(Size::new(0, 0), Size::new(1, 1));
    }

    #[test]
    fn ghostty_encodes_the_legacy_terminal_key_matrix() {
        let term = Terminal::new(Size::default());
        let alt = Modifiers { alt: true, ..Modifiers::default() };
        let shift = Modifiers { shift: true, ..Modifiers::default() };
        let control = Modifiers { control: true, ..Modifiers::default() };

        for (event, expected) in [
            (press(KeyCode::Enter, None, Modifiers::default()), b"\r".to_vec()),
            (press(KeyCode::Enter, None, shift), b"\x1b[27;2;13~".to_vec()),
            (press(KeyCode::ArrowLeft, None, alt), b"\x1b[1;3D".to_vec()),
            (press(KeyCode::ArrowRight, None, control), b"\x1b[1;5C".to_vec()),
            (press(KeyCode::Tab, None, shift), b"\x1b[Z".to_vec()),
            (press(KeyCode::F5, None, Modifiers::default()), b"\x1b[15~".to_vec()),
            (press(KeyCode::B, Some("b"), alt), b"\x1bb".to_vec()),
            (
                press(
                    KeyCode::Slash,
                    Some("?"),
                    Modifiers { control: true, shift: true, ..Modifiers::default() },
                ),
                vec![0x7f],
            ),
        ] {
            assert_eq!(
                encoded(&term, &event),
                expected,
                "unexpected encoding for {:?} with {:?}",
                event.code,
                event.modifiers,
            );
        }
    }

    #[test]
    fn application_cursor_mode_changes_unmodified_arrows() {
        let mut term = Terminal::new(Size::default());
        let left = press(KeyCode::ArrowLeft, None, Modifiers::default());
        assert_eq!(encoded(&term, &left), b"\x1b[D");

        term.feed(b"\x1b[?1h");
        assert_eq!(encoded(&term, &left), b"\x1bOD");
    }

    #[test]
    fn kitty_mode_reports_modified_enter_and_key_releases() {
        let mut term = Terminal::new(Size::default());
        // Disambiguate, report event types, and report every key. The latter is
        // required by the Kitty protocol before Enter releases are reported.
        term.feed(b"\x1b[>11u");
        let shifted_enter =
            press(KeyCode::Enter, None, Modifiers { shift: true, ..Modifiers::default() });
        assert_eq!(encoded(&term, &shifted_enter), b"\x1b[13;2u");

        let release = KeyEvent { action: KeyAction::Release, ..shifted_enter };
        assert_eq!(encoded(&term, &release), b"\x1b[13;2:3u");
    }

    #[test]
    fn a_release_is_silent_until_an_application_requests_it() {
        let term = Terminal::new(Size::default());
        let release = KeyEvent {
            action: KeyAction::Release,
            ..press(KeyCode::A, None, Modifiers::default())
        };
        assert!(encoded(&term, &release).is_empty());
    }
}
