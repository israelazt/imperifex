//! The presses and releases that make up what is to be typed. Plain code,
//! tested everywhere.

use imperifex_bench_domain::keys::{Chord, Key};

/// The virtual key codes of Windows that are used. Letters and digits have the
/// code of their ASCII character, in upper case.
const VK_BACKSPACE: u16 = 0x08;
const VK_ENTER: u16 = 0x0D;
const VK_SHIFT: u16 = 0x10;
const VK_CTRL: u16 = 0x11;
const VK_ALT: u16 = 0x12;

/// What one event presses or releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    /// A UTF-16 unit, typed whatever the keyboard's layout.
    Unicode(u16),
    Virtual(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeyEvent {
    pub(crate) code: Code,
    pub(crate) up: bool,
}

fn down(code: Code) -> KeyEvent {
    KeyEvent { code, up: false }
}

fn tap(code: Code) -> [KeyEvent; 2] {
    [down(code), KeyEvent { code, up: true }]
}

/// Each UTF-16 unit pressed and released.
pub(crate) fn of_text(text: &str) -> Vec<KeyEvent> {
    text.encode_utf16()
        .flat_map(|unit| tap(Code::Unicode(unit)))
        .collect()
}

/// The modifiers go down first and come up last, in reverse order.
fn of_chord(chord: Chord) -> Vec<KeyEvent> {
    let held: Vec<Code> = [
        (chord.ctrl, VK_CTRL),
        (chord.shift, VK_SHIFT),
        (chord.alt, VK_ALT),
    ]
    .iter()
    .filter(|(wanted, _)| *wanted)
    .map(|&(_, code)| Code::Virtual(code))
    .collect();
    let mut events: Vec<KeyEvent> = held.iter().map(|&code| down(code)).collect();
    events.extend(tap(Code::Virtual(u16::from(chord.key as u8))));
    events.extend(held.iter().rev().map(|&code| KeyEvent { code, up: true }));
    events
}

pub(crate) fn of_key(key: Key) -> Vec<KeyEvent> {
    match key {
        Key::Enter => tap(Code::Virtual(VK_ENTER)).to_vec(),
        Key::Backspace => tap(Code::Virtual(VK_BACKSPACE)).to_vec(),
        Key::Char(c) => of_text(c.encode_utf8(&mut [0; 4])),
        Key::Chord(chord) => of_chord(chord),
    }
}

#[cfg(test)]
mod tests {
    use imperifex_bench_domain::keys::parse_chord;

    use super::*;

    fn shown(events: &[KeyEvent]) -> Vec<String> {
        events
            .iter()
            .map(|e| {
                let arrow = if e.up { "up" } else { "down" };
                match e.code {
                    Code::Unicode(unit) => format!("u{unit:04x} {arrow}"),
                    Code::Virtual(code) => format!("v{code:02x} {arrow}"),
                }
            })
            .collect()
    }

    #[test]
    fn text_goes_unit_by_unit_pressed_and_released() {
        // An emoji takes two UTF-16 units.
        assert_eq!(
            shown(&of_text("a🚀")),
            [
                "u0061 down",
                "u0061 up",
                "ud83d down",
                "ud83d up",
                "ude80 down",
                "ude80 up"
            ]
        );
    }

    #[test]
    fn enter_and_backspace_are_keys_and_a_character_is_text() {
        assert_eq!(shown(&of_key(Key::Enter)), ["v0d down", "v0d up"]);
        assert_eq!(shown(&of_key(Key::Backspace)), ["v08 down", "v08 up"]);
        assert_eq!(shown(&of_key(Key::Char('a'))), ["u0061 down", "u0061 up"]);
    }

    #[test]
    fn a_chord_holds_its_modifiers_around_the_key_and_lets_go_in_reverse() {
        let chord = Key::Chord(parse_chord("Ctrl+Shift+T").unwrap());
        assert_eq!(
            shown(&of_key(chord)),
            ["v11 down", "v10 down", "v54 down", "v54 up", "v10 up", "v11 up"]
        );
    }
}
