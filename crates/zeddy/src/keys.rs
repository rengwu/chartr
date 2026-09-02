//! The platform keyboard normalized for the VT boundary.
//!
//! GPUI describes a keystroke with its platform key name, produced text and
//! modifiers. This module preserves those facts in Zeddy's normalized event;
//! `zeddy-vt` and libghostty decide which terminal bytes they mean.

use gpui::{Keystroke, Modifiers as WindowModifiers};
use zeddy_vt::{KeyAction, KeyCode, KeyEvent, Modifiers};

/// Normalize a key press or auto-repeat without choosing a terminal encoding.
pub fn normalize(keystroke: &Keystroke, held: bool) -> KeyEvent {
    KeyEvent {
        code: code_of(&keystroke.key),
        text: typed(keystroke),
        action: if held { KeyAction::Repeat } else { KeyAction::Press },
        modifiers: modifiers_of(keystroke.modifiers),
        // GPUI does not currently expose consumed modifiers or IME composition
        // state on a Keystroke. Keeping them explicit avoids inventing facts and
        // leaves the encoder boundary ready when the platform API grows them.
        consumed_modifiers: Modifiers::default(),
        composing: false,
    }
}

/// Turn a press previously delivered to the terminal into its matching release.
pub fn released(mut pressed: KeyEvent) -> KeyEvent {
    pressed.text = None;
    pressed.action = KeyAction::Release;
    pressed
}

/// Text after Shift/layout processing but before Control/Alt transformations.
///
/// Platforms commonly put the already-encoded C0 byte in `key_char` for
/// Control chords. Passing that through would decide the protocol before
/// Ghostty sees the event. Recover the printable physical character instead;
/// Ghostty can then choose C0, fixterms, modifyOtherKeys, or Kitty encoding.
fn typed(keystroke: &Keystroke) -> Option<String> {
    match keystroke.key_char.as_deref() {
        Some(text) if !text.is_empty() && !text.chars().any(char::is_control) => {
            Some(text.to_owned())
        }
        Some(_) | None if keystroke.modifiers.control => physical_text(keystroke),
        _ => None,
    }
}

fn physical_text(keystroke: &Keystroke) -> Option<String> {
    if keystroke.key == "space" {
        return Some(" ".to_owned());
    }
    let character = single(&keystroke.key)?;
    let character = if keystroke.modifiers.shift { shifted_ascii(character) } else { character };
    Some(character.to_string())
}

/// The conventional shifted ASCII face of a physical key. This is needed only
/// when a platform replaced a Control chord's text with its C0 byte.
fn shifted_ascii(character: char) -> char {
    match character {
        'a'..='z' => character.to_ascii_uppercase(),
        '`' => '~',
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        _ => character,
    }
}

fn code_of(key: &str) -> KeyCode {
    match key {
        "space" => KeyCode::Space,
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "escape" => KeyCode::Escape,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "up" => KeyCode::ArrowUp,
        "down" => KeyCode::ArrowDown,
        "left" => KeyCode::ArrowLeft,
        "right" => KeyCode::ArrowRight,
        "back" => KeyCode::BrowserBack,
        "forward" => KeyCode::BrowserForward,
        "copy" => KeyCode::Copy,
        "cut" => KeyCode::Cut,
        "paste" => KeyCode::Paste,
        _ => function_key(key)
            .or_else(|| single(key).and_then(KeyCode::typing))
            .unwrap_or(KeyCode::Unidentified),
    }
}

fn function_key(key: &str) -> Option<KeyCode> {
    let number: u8 = key.strip_prefix('f')?.parse().ok()?;
    Some(match number {
        1 => KeyCode::F1,
        2 => KeyCode::F2,
        3 => KeyCode::F3,
        4 => KeyCode::F4,
        5 => KeyCode::F5,
        6 => KeyCode::F6,
        7 => KeyCode::F7,
        8 => KeyCode::F8,
        9 => KeyCode::F9,
        10 => KeyCode::F10,
        11 => KeyCode::F11,
        12 => KeyCode::F12,
        13 => KeyCode::F13,
        14 => KeyCode::F14,
        15 => KeyCode::F15,
        16 => KeyCode::F16,
        17 => KeyCode::F17,
        18 => KeyCode::F18,
        19 => KeyCode::F19,
        20 => KeyCode::F20,
        21 => KeyCode::F21,
        22 => KeyCode::F22,
        23 => KeyCode::F23,
        24 => KeyCode::F24,
        25 => KeyCode::F25,
        _ => return None,
    })
}

fn single(key: &str) -> Option<char> {
    let mut characters = key.chars();
    let first = characters.next()?;
    characters.next().is_none().then_some(first)
}

fn modifiers_of(modifiers: WindowModifiers) -> Modifiers {
    Modifiers {
        shift: modifiers.shift,
        alt: modifiers.alt,
        control: modifiers.control,
        super_key: modifiers.platform,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keystroke(key: &str, text: Option<&str>) -> Keystroke {
        Keystroke {
            modifiers: WindowModifiers::default(),
            key: key.to_owned(),
            key_char: text.map(str::to_owned),
        }
    }

    #[test]
    fn names_printable_navigation_and_function_keys() {
        assert_eq!(code_of("a"), KeyCode::A);
        assert_eq!(code_of("/"), KeyCode::Slash);
        assert_eq!(code_of("left"), KeyCode::ArrowLeft);
        assert_eq!(code_of("f20"), KeyCode::F20);
        assert_eq!(code_of("f25"), KeyCode::F25);
        assert_eq!(code_of("f26"), KeyCode::Unidentified);
    }

    #[test]
    fn preserves_platform_text_instead_of_rederiving_it() {
        let event = normalize(&keystroke("e", Some("é")), false);
        assert_eq!(event.code, KeyCode::E);
        assert_eq!(event.text.as_deref(), Some("é"));
    }

    #[test]
    fn recovers_printable_text_from_platform_control_bytes() {
        let mut control_i = keystroke("i", Some("\t"));
        control_i.modifiers.control = true;
        assert_eq!(normalize(&control_i, false).text.as_deref(), Some("i"));

        let mut control_question = keystroke("/", Some("\u{7f}"));
        control_question.modifiers.control = true;
        control_question.modifiers.shift = true;
        assert_eq!(normalize(&control_question, false).text.as_deref(), Some("?"));
    }

    #[test]
    fn held_and_released_keys_keep_their_identity() {
        let mut input = keystroke("left", None);
        input.modifiers.alt = true;
        let repeated = normalize(&input, true);
        assert_eq!(repeated.action, KeyAction::Repeat);
        assert!(repeated.modifiers.alt);

        let release = released(repeated);
        assert_eq!(release.action, KeyAction::Release);
        assert_eq!(release.code, KeyCode::ArrowLeft);
        assert_eq!(release.text, None);
        assert!(release.modifiers.alt);
    }

    #[test]
    fn the_original_missing_chords_reach_ghostty_intact() {
        let mut encoder = zeddy_vt::KeyEncoder::new().unwrap();

        let mut shifted_enter = keystroke("enter", Some("\n"));
        shifted_enter.modifiers.shift = true;
        let event = normalize(&shifted_enter, false);
        assert_eq!(
            encoder.encode(&event, zeddy_vt::KeyboardModes::default()).unwrap(),
            b"\x1b[27;2;13~",
        );

        let mut option_left = keystroke("left", None);
        option_left.modifiers.alt = true;
        let event = normalize(&option_left, false);
        assert_eq!(
            encoder.encode(&event, zeddy_vt::KeyboardModes::default()).unwrap(),
            b"\x1b[1;3D",
        );
    }

    #[test]
    fn control_i_remains_distinct_from_tab() {
        let mut encoder = zeddy_vt::KeyEncoder::new().unwrap();
        let mut control_i = keystroke("i", Some("\t"));
        control_i.modifiers.control = true;

        assert_eq!(
            encoder
                .encode(&normalize(&control_i, false), zeddy_vt::KeyboardModes::default())
                .unwrap(),
            b"\x1b[105;5u",
        );
    }
}
