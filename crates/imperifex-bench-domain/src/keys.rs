//! The keys the bench presses.

/// A key held with others, such as `Ctrl+Shift+T`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// An ASCII letter, in upper case, or a digit.
    pub key: char,
}

/// One thing to press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Enter,
    Backspace,
    /// A character, typed whatever the keyboard's layout.
    Char(char),
    Chord(Chord),
}

/// Reads keys written as `Ctrl+Shift+T`: modifiers, then one letter or digit.
/// Case does not matter. The error shows what was not understood.
pub fn parse_chord(text: &str) -> Result<Chord, String> {
    let bad = |what: &str| format!("no se entiende «{what}» en las teclas «{text}»");
    let mut parts: Vec<&str> = text.split('+').map(str::trim).collect();
    let last = parts.pop().unwrap_or_default();
    let mut chars = last.chars();
    let key = match (chars.next(), chars.next()) {
        (Some(key), None) if key.is_ascii_alphanumeric() => key.to_ascii_uppercase(),
        _ => return Err(bad(last)),
    };
    let mut chord = Chord {
        ctrl: false,
        shift: false,
        alt: false,
        key,
    };
    for part in parts {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" => chord.ctrl = true,
            "shift" => chord.shift = true,
            "alt" => chord.alt = true,
            _ => return Err(bad(part)),
        }
    }
    Ok(chord)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(ctrl: bool, shift: bool, alt: bool, key: char) -> Chord {
        Chord {
            ctrl,
            shift,
            alt,
            key,
        }
    }

    #[test]
    fn modifiers_and_a_letter_are_read_whatever_their_case() {
        assert_eq!(
            parse_chord("Ctrl+Shift+T"),
            Ok(chord(true, true, false, 'T'))
        );
        assert_eq!(parse_chord("alt + 1"), Ok(chord(false, false, true, '1')));
        assert_eq!(parse_chord("x"), Ok(chord(false, false, false, 'X')));
    }

    #[test]
    fn what_is_not_understood_is_an_error_that_shows_it() {
        for (text, shown) in [
            ("Win+T", "Win"),
            ("Ctrl+Tab", "Tab"),
            ("Ctrl+", "«»"),
            ("", "«»"),
            ("Ctrl+ñ", "ñ"),
        ] {
            let error = parse_chord(text).unwrap_err();
            assert!(error.contains(shown), "{text}: {error}");
        }
    }
}
