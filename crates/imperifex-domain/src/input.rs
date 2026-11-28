//! What is typed, as the bytes a program in a terminal reads: the text as the
//! system gives it, and the keys that are not text, with their modifiers as
//! xterm writes them. The part of T3.1 that programs need; IME comes later.

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
    Insert,
    Delete,
    PageUp,
    PageDown,
    /// `F1` to `F12`.
    Function(u8),
}

/// One key going down.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyInput {
    /// What the system says it writes, with `Ctrl` taken into account:
    /// `Ctrl`+`C` writes the control character 3.
    pub text: Option<String>,
    /// Which one it is, if it is one of those above.
    pub key: Option<NamedKey>,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyInput {
    /// `AltGr` is `Ctrl` and `Alt` to Windows. When with them a key writes a
    /// character that is seen, it was `AltGr` writing it: neither of the two is
    /// held for what follows.
    pub fn without_alt_graph(mut self) -> Self {
        let writes = |text: &String| !text.is_empty() && !text.chars().any(char::is_control);
        if self.ctrl && self.alt && self.key.is_none() && self.text.as_ref().is_some_and(writes) {
            (self.ctrl, self.alt) = (false, false);
        }
        self
    }

    /// The number xterm gives the modifiers held: 1 with none.
    fn modifiers(&self) -> u8 {
        1 + u8::from(self.shift) + 2 * u8::from(self.alt) + 4 * u8::from(self.ctrl)
    }
}

/// What to send to the program, with the `modes` it asked for. Empty for a key
/// that sends nothing.
pub fn bytes(input: &KeyInput, modes: Modes) -> Vec<u8> {
    match input.key {
        Some(key) => of_key(key, input, modes),
        None => of_text(input),
    }
}

/// The text as it is; with `Alt`, after an escape.
fn of_text(input: &KeyInput) -> Vec<u8> {
    let text = input.text.as_deref().unwrap_or_default().as_bytes();
    if input.alt && !input.ctrl && !text.is_empty() {
        return [b"\x1b", text].concat();
    }
    text.to_vec()
}

fn of_key(key: NamedKey, input: &KeyInput, modes: Modes) -> Vec<u8> {
    // The arrows, Home and End say ESC [ x, or ESC O x if the program asked.
    let cursor = |letter: char| letter_key(letter, input, modes.application_cursor_keys);
    match key {
        NamedKey::Enter => enter(input, modes),
        NamedKey::Backspace if input.ctrl => vec![0x08],
        NamedKey::Backspace if input.alt => vec![0x1B, 0x7F],
        NamedKey::Backspace => vec![0x7F],
        NamedKey::Tab if input.shift => b"\x1b[Z".to_vec(),
        NamedKey::Tab => vec![b'\t'],
        NamedKey::Escape => vec![0x1B],
        NamedKey::Up => cursor('A'),
        NamedKey::Down => cursor('B'),
        NamedKey::Right => cursor('C'),
        NamedKey::Left => cursor('D'),
        NamedKey::Home => cursor('H'),
        NamedKey::End => cursor('F'),
        NamedKey::Insert => numbered_key(2, input),
        NamedKey::Delete => numbered_key(3, input),
        NamedKey::PageUp => numbered_key(5, input),
        NamedKey::PageDown => numbered_key(6, input),
        NamedKey::Function(number) => function(number, input),
    }
}

/// `Shift` or `Alt` with `Enter` asks for a new line of a program that takes
/// pastes, such as Claude Code, which reads an escape and then `Enter`. Others
/// would take the escape as their own key.
fn enter(input: &KeyInput, modes: Modes) -> Vec<u8> {
    if input.alt || (input.shift && modes.bracketed_paste) {
        b"\x1b\r".to_vec()
    } else if input.ctrl {
        vec![b'\n']
    } else {
        vec![b'\r']
    }
}

/// A key named by a letter; with modifiers their number goes before it.
fn letter_key(letter: char, input: &KeyInput, application: bool) -> Vec<u8> {
    let sequence = match input.modifiers() {
        1 if application => format!("\x1bO{letter}"),
        1 => format!("\x1b[{letter}"),
        held => format!("\x1b[1;{held}{letter}"),
    };
    sequence.into_bytes()
}

/// A key named by a number, ending in `~`.
fn numbered_key(number: u8, input: &KeyInput) -> Vec<u8> {
    let sequence = match input.modifiers() {
        1 => format!("\x1b[{number}~"),
        held => format!("\x1b[{number};{held}~"),
    };
    sequence.into_bytes()
}

fn function(number: u8, input: &KeyInput) -> Vec<u8> {
    match number {
        // The first four are named by letters, always after `ESC O` when alone.
        1..=4 => {
            let letter = ['P', 'Q', 'R', 'S'][usize::from(number - 1)];
            letter_key(letter, input, true)
        }
        5 => numbered_key(15, input),
        // 17 to 21 and then 23 and 24: xterm skips some numbers.
        6..=10 => numbered_key(number + 11, input),
        11 | 12 => numbered_key(number + 12, input),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
