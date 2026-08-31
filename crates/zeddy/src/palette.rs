//! Turning a cell's colour into a colour the window can paint.
//!
//! [`zeddy_vt::Color`] deliberately does not resolve anything: it says
//! "indexed 4" or "default", and *what those are* belongs to the theme. This is
//! where that is decided, and it is the only place — so switching themes is a
//! re-render rather than a re-parse.

use gpui::{Hsla, Rgba};
use theme::Theme;
use zeddy_vt::{Cell, Color, Style};

/// Whether a colour is standing in for the foreground or the background.
///
/// [`Color::Default`] means different things in the two positions, and this is
/// how the caller says which one it is asking about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Foreground,
    Background,
}

/// Resolve one colour against the active theme.
pub fn resolve(color: Color, slot: Slot, theme: &Theme) -> Hsla {
    match color {
        Color::Default => match slot {
            Slot::Foreground => theme.colors().terminal_foreground,
            Slot::Background => theme.colors().terminal_background,
        },
        Color::Rgb(r, g, b) => {
            Rgba { r: f32::from(r) / 255., g: f32::from(g) / 255., b: f32::from(b) / 255., a: 1. }
                .into()
        }
        Color::Indexed(index) => indexed(index, slot, theme),
    }
}

/// The foreground and background a cell is actually painted with, after
/// `inverse` and `dim` have been applied.
///
/// Applied here rather than in the VT crate because both depend on what the
/// theme's defaults are, and the VT crate does not have a theme.
pub fn cell_colors(cell: &Cell, theme: &Theme) -> (Hsla, Hsla) {
    let Style { inverse, dim, .. } = cell.style;
    let (fg_color, bg_color) = if inverse { (cell.bg, cell.fg) } else { (cell.fg, cell.bg) };
    let (fg_slot, bg_slot) = if inverse {
        (Slot::Background, Slot::Foreground)
    } else {
        (Slot::Foreground, Slot::Background)
    };

    let mut fg = resolve(fg_color, fg_slot, theme);
    if dim {
        fg.a *= 0.7;
    }
    (fg, resolve(bg_color, bg_slot, theme))
}

/// The sixteen ANSI slots, plus the 256-colour cube and greyscale ramp.
///
/// Zed's theme names the sixteen; 16..=255 are the xterm cube, which is defined
/// arithmetically and is not a theme's to override.
fn indexed(index: u8, slot: Slot, theme: &Theme) -> Hsla {
    let colors = theme.colors();
    match index {
        0 => colors.terminal_ansi_black,
        1 => colors.terminal_ansi_red,
        2 => colors.terminal_ansi_green,
        3 => colors.terminal_ansi_yellow,
        4 => colors.terminal_ansi_blue,
        5 => colors.terminal_ansi_magenta,
        6 => colors.terminal_ansi_cyan,
        7 => colors.terminal_ansi_white,
        8 => colors.terminal_ansi_bright_black,
        9 => colors.terminal_ansi_bright_red,
        10 => colors.terminal_ansi_bright_green,
        11 => colors.terminal_ansi_bright_yellow,
        12 => colors.terminal_ansi_bright_blue,
        13 => colors.terminal_ansi_bright_magenta,
        14 => colors.terminal_ansi_bright_cyan,
        15 => colors.terminal_ansi_bright_white,
        16..=231 => {
            // The 6×6×6 cube. The steps are xterm's, not evenly spaced: the
            // first is 0 and the rest are 95 + 40n.
            let value = index - 16;
            let step = |n: u8| match n {
                0 => 0u8,
                n => 95 + 40 * (n - 1),
            };
            let (r, g, b) = (step(value / 36), step((value % 36) / 6), step(value % 6));
            resolve(Color::Rgb(r, g, b), slot, theme)
        }
        232..=255 => {
            let level = 8 + 10 * (index - 232);
            resolve(Color::Rgb(level, level, level), slot, theme)
        }
    }
}

#[cfg(test)]
mod tests {
    //! Run against the theme the app actually boots with, rather than a
    //! hand-built one: what these assert is that the mapping agrees with Zed's
    //! palette, and a fixture theme could not tell us that.

    use super::*;
    use gpui::TestAppContext;
    use theme::ActiveTheme as _;
    use zeddy_vt::Style;

    fn cell(fg: Color, bg: Color, style: Style) -> Cell {
        Cell { ch: 'x', fg, bg, style }
    }

    fn with_theme<R>(cx: &mut TestAppContext, f: impl FnOnce(&Theme) -> R) -> R {
        cx.update(|cx| {
            theme::init(theme::LoadThemes::JustBase, cx);
            f(cx.theme())
        })
    }

    #[gpui::test]
    fn default_means_something_different_in_each_slot(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            assert_ne!(
                resolve(Color::Default, Slot::Foreground, theme),
                resolve(Color::Default, Slot::Background, theme)
            );
        });
    }

    #[gpui::test]
    fn true_colour_is_passed_through_untouched(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            let painted = resolve(Color::Rgb(255, 0, 0), Slot::Foreground, theme);
            assert_eq!(painted, Hsla::from(Rgba { r: 1., g: 0., b: 0., a: 1. }));
        });
    }

    #[gpui::test]
    fn the_sixteen_ansi_slots_come_from_the_theme(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            assert_eq!(
                resolve(Color::Indexed(1), Slot::Foreground, theme),
                theme.colors().terminal_ansi_red
            );
            assert_eq!(
                resolve(Color::Indexed(9), Slot::Foreground, theme),
                theme.colors().terminal_ansi_bright_red
            );
        });
    }

    #[gpui::test]
    fn the_cube_follows_xterms_uneven_steps(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            // 16 is the cube's black corner, 231 its white one.
            assert_eq!(
                resolve(Color::Indexed(16), Slot::Foreground, theme),
                resolve(Color::Rgb(0, 0, 0), Slot::Foreground, theme)
            );
            assert_eq!(
                resolve(Color::Indexed(231), Slot::Foreground, theme),
                resolve(Color::Rgb(255, 255, 255), Slot::Foreground, theme)
            );
        });
    }

    #[gpui::test]
    fn the_greyscale_ramp_is_grey(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            let grey = resolve(Color::Indexed(240), Slot::Foreground, theme);
            assert_eq!(grey.s, 0., "a ramp entry with saturation is not grey");
        });
    }

    #[gpui::test]
    fn inverse_swaps_the_two_slots_and_not_merely_the_two_colours(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            let plain = cell(Color::Default, Color::Default, Style::default());
            let inverted =
                cell(Color::Default, Color::Default, Style { inverse: true, ..Default::default() });
            assert_eq!(cell_colors(&plain, theme), {
                let (fg, bg) = cell_colors(&inverted, theme);
                (bg, fg)
            });
        });
    }

    #[gpui::test]
    fn dim_fades_the_foreground_and_leaves_the_background_alone(cx: &mut TestAppContext) {
        with_theme(cx, |theme| {
            let dimmed =
                cell(Color::Indexed(2), Color::Default, Style { dim: true, ..Default::default() });
            let (fg, bg) = cell_colors(&dimmed, theme);
            assert!(fg.a < 1.0);
            assert_eq!(bg, theme.colors().terminal_background);
        });
    }
}
