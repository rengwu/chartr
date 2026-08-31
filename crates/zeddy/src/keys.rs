//! Turning a keystroke into the bytes a terminal expects.
//!
//! GPUI hands us a [`Keystroke`] — a key name and a set of modifiers. A PTY
//! wants bytes. This is the whole translation, kept in one file with tests
//! because it is the part of a terminal that is quietly wrong for years if
//! nobody checks it.
//!
//! Only the sequences an agent session actually needs are here: text, the
//! control range, the arrows and their bracketed forms, and the editing keys.
//! Mouse reporting, the kitty keyboard protocol, and application-cursor mode
//! are deliberately absent — none of them is reachable through herdr's frame
//! stream, which sends a re-render rather than the program's own output.

use gpui::Keystroke;

/// The bytes to send for a keystroke, or `None` for one that means nothing to
/// a terminal (a bare modifier, an unhandled function key).
pub fn bytes_for(keystroke: &Keystroke) -> Option<Vec<u8>> {
    let modifiers = &keystroke.modifiers;

    let named = match keystroke.key.as_str() {
        "enter" => Some("\r"),
        "tab" if modifiers.shift => Some("\x1b[Z"),
        "tab" => Some("\t"),
        "backspace" => Some("\x7f"),
        "escape" => Some("\x1b"),
        "space" => Some(" "),
        "up" => Some("\x1b[A"),
        "down" => Some("\x1b[B"),
        "right" => Some("\x1b[C"),
        "left" => Some("\x1b[D"),
        "home" => Some("\x1b[H"),
        "end" => Some("\x1b[F"),
        "pageup" => Some("\x1b[5~"),
        "pagedown" => Some("\x1b[6~"),
        "delete" => Some("\x1b[3~"),
        "insert" => Some("\x1b[2~"),
        _ => None,
    };

    if let Some(named) = named {
        return Some(with_alt(named.as_bytes(), modifiers.alt));
    }

    // Control folds a letter into the C0 range: ^A is 1, ^Z is 26. The handful
    // of punctuation controls follow the same table.
    if modifiers.control {
        let byte = match keystroke.key.as_str() {
            key if key.len() == 1 => {
                let c = key.chars().next().expect("one char");
                match c {
                    'a'..='z' => Some(c as u8 - b'a' + 1),
                    '@' | ' ' => Some(0),
                    '[' => Some(27),
                    '\\' => Some(28),
                    ']' => Some(29),
                    '^' => Some(30),
                    '_' | '?' => Some(31),
                    _ => None,
                }
            }
            _ => None,
        };
        return byte.map(|byte| with_alt(&[byte], modifiers.alt));
    }

    // Anything else is text, and GPUI already worked out what text it is —
    // including the shifted and dead-key forms this code should not re-derive.
    let text = keystroke.key_char.as_deref().filter(|text| !text.is_empty())?;
    Some(with_alt(text.as_bytes(), modifiers.alt))
}

/// Alt is a leading escape. That is what a terminal means by "meta".
fn with_alt(bytes: &[u8], alt: bool) -> Vec<u8> {
    if alt {
        let mut out = Vec::with_capacity(bytes.len() + 1);
        out.push(0x1b);
        out.extend_from_slice(bytes);
        out
    } else {
        bytes.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(spec: &str) -> Keystroke {
        Keystroke::parse(spec).expect("a parseable keystroke")
    }

    fn typed(spec: &str, text: &str) -> Keystroke {
        let mut keystroke = key(spec);
        keystroke.key_char = Some(text.to_owned());
        keystroke
    }

    #[test]
    fn plain_text_goes_through_as_itself() {
        assert_eq!(bytes_for(&typed("a", "a")), Some(b"a".to_vec()));
        assert_eq!(bytes_for(&typed("shift-a", "A")), Some(b"A".to_vec()));
    }

    #[test]
    fn enter_is_a_carriage_return_and_not_a_newline() {
        // A PTY in canonical mode reads CR as "submit"; LF would insert a line.
        assert_eq!(bytes_for(&key("enter")), Some(b"\r".to_vec()));
    }

    #[test]
    fn backspace_is_del_which_is_what_readline_expects() {
        assert_eq!(bytes_for(&key("backspace")), Some(vec![0x7f]));
    }

    #[test]
    fn control_letters_fold_into_the_c0_range() {
        assert_eq!(bytes_for(&key("ctrl-a")), Some(vec![1]));
        assert_eq!(bytes_for(&key("ctrl-c")), Some(vec![3]));
        assert_eq!(bytes_for(&key("ctrl-z")), Some(vec![26]));
    }

    #[test]
    fn the_arrows_are_csi_sequences() {
        assert_eq!(bytes_for(&key("up")), Some(b"\x1b[A".to_vec()));
        assert_eq!(bytes_for(&key("left")), Some(b"\x1b[D".to_vec()));
    }

    #[test]
    fn shift_tab_is_a_back_tab_and_not_a_tab() {
        assert_eq!(bytes_for(&key("tab")), Some(b"\t".to_vec()));
        assert_eq!(bytes_for(&key("shift-tab")), Some(b"\x1b[Z".to_vec()));
    }

    #[test]
    fn alt_prefixes_an_escape_whatever_the_key_was() {
        assert_eq!(bytes_for(&typed("alt-b", "b")), Some(b"\x1bb".to_vec()));
        assert_eq!(bytes_for(&key("alt-up")), Some(b"\x1b\x1b[A".to_vec()));
        assert_eq!(bytes_for(&key("ctrl-alt-a")), Some(vec![0x1b, 1]));
    }

    #[test]
    fn a_keystroke_with_no_text_and_no_name_sends_nothing() {
        // An unhandled function key must send nothing rather than send garbage.
        assert_eq!(bytes_for(&key("f13")), None);
    }
}
