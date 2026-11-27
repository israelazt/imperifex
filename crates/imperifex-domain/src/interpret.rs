//! What each sequence does to the grid. What is not known here is ignored: a
//! program may write anything.

use crate::grid::{Erase, Grid};
use crate::sequence::Sequence;

pub fn apply(grid: &mut Grid, sequence: Sequence<'_>) {
    match sequence {
        Sequence::Print(character) => grid.print(character),
        Sequence::Control(byte) => control(grid, byte),
        // With a marker the same letter means something else.
        Sequence::Csi {
            params,
            intermediates: [],
            action,
        } => csi(grid, params, action),
        Sequence::Esc {
            intermediates: [],
            action,
        } => esc(grid, action),
        Sequence::Csi { .. } | Sequence::Esc { .. } | Sequence::Osc(_) => {}
    }
}

fn control(grid: &mut Grid, byte: u8) {
    match byte {
        0x08 => grid.backspace(),
        0x09 => grid.tab(),
        // Line feed, vertical tab and form feed.
        0x0A..=0x0C => grid.line_feed(),
        0x0D => grid.carriage_return(),
        _ => {}
    }
}

fn esc(grid: &mut Grid, action: u8) {
    match action {
        b'D' => grid.line_feed(),
        b'E' => {
            grid.carriage_return();
            grid.line_feed();
        }
        _ => {}
    }
}

fn param(params: &[u16], index: usize) -> usize {
    usize::from(params.get(index).copied().unwrap_or(0))
}

/// A number of steps or a position counted from 1: nothing and 0 mean 1.
fn count(params: &[u16], index: usize) -> usize {
    param(params, index).max(1)
}

fn csi(grid: &mut Grid, params: &[u16], action: char) {
    match action {
        'A'..='H' | 'd' | 'f' => move_cursor(grid, params, action),
        'J' => {
            if let Some(what) = erase(params) {
                grid.erase_in_display(what);
            }
        }
        'K' => {
            if let Some(what) = erase(params) {
                grid.erase_in_line(what);
            }
        }
        'X' => grid.erase_chars(count(params, 0)),
        _ => {}
    }
}

fn move_cursor(grid: &mut Grid, params: &[u16], action: char) {
    let here = grid.cursor();
    let n = count(params, 0);
    let (row, column) = match action {
        'A' => (here.row.saturating_sub(n), here.column),
        'B' => (here.row + n, here.column),
        'C' => (here.row, here.column + n),
        'D' => (here.row, here.column.saturating_sub(n)),
        'E' => (here.row + n, 0),
        'F' => (here.row.saturating_sub(n), 0),
        'G' => (here.row, n - 1),
        'd' => (n - 1, here.column),
        'H' | 'f' => (n - 1, count(params, 1) - 1),
        _ => return,
    };
    grid.move_to(row, column);
}

/// `None` for what is not about the visible screen (3 is the history).
fn erase(params: &[u16]) -> Option<Erase> {
    match param(params, 0) {
        0 => Some(Erase::ToEnd),
        1 => Some(Erase::ToStart),
        2 => Some(Erase::All),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
