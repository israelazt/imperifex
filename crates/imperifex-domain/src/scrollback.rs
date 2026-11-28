//! Looking back at what left the screen: how far back the view is, and what a
//! turn of the mouse wheel does.

use crate::grid::Grid;
use crate::input::{bytes, KeyInput, NamedKey};
use crate::terminal::Modes;

/// How many rows one notch of the wheel moves, as Windows has it unless told
/// otherwise.
const ROWS_PER_NOTCH: f32 = 3.0;
/// The buttons a mouse report names the wheel with.
const WHEEL_UP: u8 = 64;
const WHEEL_DOWN: u8 = 65;
/// The furthest place the old kind of mouse report can name, counted from 0.
const FURTHEST_OLD: u16 = 222;

/// How many rows above the screen the view is: 0 shows the screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct View {
    back: usize,
    /// Rows at the top of the screen that are not seen either, when what is
    /// seen of it is anchored at the bottom of less room: one can go back that
    /// much further than the history.
    hidden: usize,
    /// How many rows the history had been given when last looked at.
    seen: u64,
}

impl View {
    pub fn back(self) -> usize {
        self.back
    }

    /// Moves `rows` back in time; forward when negative. It stays between the
    /// oldest row of the history and the screen.
    pub fn scroll(&mut self, rows: i32, screen: &Grid) {
        self.follow(screen);
        let rows = isize::try_from(rows).unwrap_or(0);
        self.back = self
            .back
            .saturating_add_signed(rows)
            .min(screen.history_len() + self.hidden);
    }

    /// After new output: a view that was back stays on the same text while the
    /// history still has it.
    pub fn follow(&mut self, screen: &Grid) {
        let added = screen.history_added();
        if self.back > 0 {
            let new = added.saturating_sub(self.seen);
            let new = usize::try_from(new).unwrap_or(usize::MAX);
            self.back = self.back.saturating_add(new);
        }
        self.back = self.back.min(screen.history_len() + self.hidden);
        self.seen = added;
    }

    pub fn hide(&mut self, rows: usize) {
        self.hidden = rows;
    }

    pub fn to_screen(&mut self) {
        self.back = 0;
    }
}

/// What a turn of the wheel is to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Turned {
    /// Not enough yet for one step.
    Nothing,
    /// Move the view this many rows back, or forward when negative.
    Scroll(i32),
    /// Tell the program.
    Send(Vec<u8>),
}

/// The wheel of the mouse. A touchpad turns it a little at a time: what does
/// not make a whole step is kept for the next turn.
#[derive(Debug, Clone, Copy, Default)]
pub struct Wheel {
    rest: f32,
}

impl Wheel {
    /// `notches` of turning, up when positive, with the pointer on the cell
    /// `at` (column and row, from 0).
    ///
    /// A program that asked for the mouse gets one report per notch. One on the
    /// alternate screen that did not, which has no history to look at, gets the
    /// arrow keys. Otherwise the view moves.
    pub fn turn(&mut self, notches: f32, modes: Modes, alternate: bool, at: (u16, u16)) -> Turned {
        let per_notch = if modes.mouse_reports {
            1.0
        } else {
            ROWS_PER_NOTCH
        };
        // Turning the other way drops what was left over.
        if self.rest * notches < 0.0 {
            self.rest = 0.0;
        }
        let total = self.rest + notches * per_notch;
        // A turn of the wheel is far inside what these hold.
        let steps = total.trunc() as i32;
        self.rest = total.fract();
        if steps == 0 {
            return Turned::Nothing;
        }
        let times = usize::try_from(steps.unsigned_abs()).unwrap_or(0);
        if modes.mouse_reports {
            let button = if steps > 0 { WHEEL_UP } else { WHEEL_DOWN };
            Turned::Send(report(button, at, modes).repeat(times))
        } else if alternate {
            Turned::Send(arrow(steps > 0, modes).repeat(times))
        } else {
            Turned::Scroll(steps)
        }
    }
}

/// A button on a cell, as the program asked to be told: in digits (SGR), or in
/// the old way, one byte each.
fn report(button: u8, (column, row): (u16, u16), modes: Modes) -> Vec<u8> {
    if modes.mouse_digits {
        let (column, row) = (u32::from(column) + 1, u32::from(row) + 1);
        return format!("\x1b[<{button};{column};{row}M").into_bytes();
    }
    // 32 is added to each, and places count from 1.
    let place = |at: u16| u8::try_from(at.min(FURTHEST_OLD) + 33).unwrap_or(u8::MAX);
    vec![0x1B, b'[', b'M', button + 32, place(column), place(row)]
}

fn arrow(up: bool, modes: Modes) -> Vec<u8> {
    let key = if up { NamedKey::Up } else { NamedKey::Down };
    let input = KeyInput {
        key: Some(key),
        ..KeyInput::default()
    };
    bytes(&input, modes)
}

#[cfg(test)]
mod tests;
