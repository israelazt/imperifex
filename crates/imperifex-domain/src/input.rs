//! What is typed, as the bytes a shell reads. The least to be able to use one
//! (T-D18): the text as the system gives it, and a few keys. The rest of the
//! keyboard (the arrows with modifiers, function keys, shortcuts, IME) comes
//! with T3.1, which replaces this.

use crate::terminal::Modes;

/// The keys that are not the text they give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedKey {
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Right,
    Left,
    Home,
    End,
    Delete,
}

/// One key going down.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyInput {
    /// What the system says it writes.
    pub text: Option<String>,
    /// Which one it is, if it is one of those above.
    pub key: Option<NamedKey>,
    /// A Shift key is down. The shell is not told; the editor is.
    pub shift: bool,
    /// A Ctrl key is down; the same.
    pub ctrl: bool,
}

/// What to send to the shell, with the `modes` its program asked for. Empty
/// for a key that sends nothing.
pub fn bytes(input: &KeyInput, modes: Modes) -> Vec<u8> {
    match input.key {
        Some(key) => of_key(key, modes),
        None => input
            .text
            .as_deref()
            .unwrap_or_default()
            .as_bytes()
            .to_vec(),
    }
}

fn of_key(key: NamedKey, modes: Modes) -> Vec<u8> {
    // The arrows say ESC [ x, or ESC O x if the program asked for it.
    let arrow = |letter: u8| {
        let second = if modes.application_cursor_keys {
            b'O'
        } else {
            b'['
        };
        vec![0x1B, second, letter]
    };
    match key {
        NamedKey::Enter => vec![b'\r'],
        NamedKey::Backspace => vec![0x7F],
        NamedKey::Tab => vec![b'\t'],
        NamedKey::Escape => vec![0x1B],
        NamedKey::Up => arrow(b'A'),
        NamedKey::Down => arrow(b'B'),
        NamedKey::Right => arrow(b'C'),
        NamedKey::Left => arrow(b'D'),
        NamedKey::Home => arrow(b'H'),
        NamedKey::End => arrow(b'F'),
        NamedKey::Delete => b"\x1b[3~".to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(text: &str) -> KeyInput {
        KeyInput {
            text: Some(text.into()),
            ..KeyInput::default()
        }
    }

    fn key(key: NamedKey) -> KeyInput {
        KeyInput {
            key: Some(key),
            ..KeyInput::default()
        }
    }

    #[test]
    fn text_goes_as_it_is_in_utf_8() {
        let sent = |typed| bytes(&text(typed), Modes::default());
        assert_eq!(sent("a"), b"a");
        assert_eq!(sent("ñ€"), "ñ€".as_bytes());
        assert_eq!(sent(" "), b" ");
        assert_eq!(bytes(&KeyInput::default(), Modes::default()), b"");
    }

    #[test]
    fn each_named_key_sends_its_bytes_whatever_text_comes_with_it() {
        for (named, expected) in [
            (NamedKey::Enter, &b"\r"[..]),
            (NamedKey::Backspace, b"\x7f"),
            (NamedKey::Tab, b"\t"),
            (NamedKey::Escape, b"\x1b"),
            (NamedKey::Up, b"\x1b[A"),
            (NamedKey::Down, b"\x1b[B"),
            (NamedKey::Right, b"\x1b[C"),
            (NamedKey::Left, b"\x1b[D"),
            (NamedKey::Home, b"\x1b[H"),
            (NamedKey::End, b"\x1b[F"),
            (NamedKey::Delete, b"\x1b[3~"),
        ] {
            let with_text = KeyInput {
                text: Some("\u{8}".into()),
                ..key(named)
            };
            for input in [key(named), with_text] {
                assert_eq!(bytes(&input, Modes::default()), expected, "{named:?}");
            }
        }
    }

    #[test]
    fn the_arrows_change_when_the_program_asks_for_application_keys() {
        let modes = Modes {
            application_cursor_keys: true,
            ..Modes::default()
        };
        assert_eq!(bytes(&key(NamedKey::Up), modes), b"\x1bOA");
        assert_eq!(bytes(&key(NamedKey::Left), modes), b"\x1bOD");
        assert_eq!(bytes(&key(NamedKey::Enter), modes), b"\r");
    }
}
