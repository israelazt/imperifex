//! What each sequence does to the terminal. What is not known here is ignored:
//! a program may write anything.

mod csi;
mod modes;
mod sgr;

use crate::grid::Grid;
use crate::sequence::Sequence;
use crate::terminal::Terminal;

pub fn apply(terminal: &mut Terminal, sequence: Sequence<'_>) {
    match sequence {
        Sequence::Print(character) => terminal.screen_mut().print(character),
        Sequence::Control(byte) => control(terminal.screen_mut(), byte),
        Sequence::Csi {
            params,
            intermediates,
            action,
        } => match intermediates {
            [] => csi::apply(terminal, params, action),
            [b'?'] => modes::private(terminal, params, action),
            // With any other marker the letter means something else.
            _ => {}
        },
        Sequence::Esc {
            intermediates: [],
            action,
        } => esc(terminal, action),
        Sequence::Osc([code, letter, more @ ..]) if *code == b"133" => {
            terminal.mark(letter, more);
        }
        Sequence::Osc([code, kind, path @ ..]) if *code == b"9" && *kind == b"9" => {
            terminal.folder = folder_named(path);
        }
        Sequence::Esc { .. } | Sequence::Osc(_) => {}
    }
}

/// The path of an OSC 9;9, which was cut at each `;` it had. Some programs
/// quote it.
fn folder_named(parts: &[&[u8]]) -> Option<String> {
    let path = String::from_utf8_lossy(&parts.join(&b';')).into_owned();
    let path = path.trim_matches('"');
    (!path.is_empty()).then(|| path.to_string())
}

fn control(screen: &mut Grid, byte: u8) {
    match byte {
        0x08 => screen.backspace(),
        0x09 => screen.tab(),
        // Line feed, vertical tab and form feed.
        0x0A..=0x0C => screen.line_feed(),
        0x0D => screen.carriage_return(),
        _ => {}
    }
}

fn esc(terminal: &mut Terminal, action: u8) {
    let screen = terminal.screen_mut();
    match action {
        b'D' => screen.line_feed(),
        b'E' => {
            screen.carriage_return();
            screen.line_feed();
        }
        b'M' => screen.reverse_line_feed(),
        b'7' => screen.save_cursor(),
        b'8' => screen.restore_cursor(),
        b'c' => terminal.reset(),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
