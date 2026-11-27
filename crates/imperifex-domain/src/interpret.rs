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
        Sequence::Esc { .. } | Sequence::Osc(_) => {}
    }
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
