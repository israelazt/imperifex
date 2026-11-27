//! The `ESC [` sequences without a private marker.

use crate::grid::{Erase, Grid};
use crate::sequence::Params;
use crate::terminal::Terminal;

use super::{modes, sgr};

pub(super) fn apply(terminal: &mut Terminal, params: Params<'_>, action: char) {
    let screen = terminal.screen_mut();
    match action {
        'm' => sgr::apply(&mut screen.pen, params),
        'h' | 'l' => modes::ansi(screen, params, action == 'h'),
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | '`' | 'a' | 'd' | 'e' | 'f' => {
            move_cursor(screen, params, action);
        }
        _ => edit(screen, params, action),
    }
}

/// A number of times or a position counted from 1: nothing and 0 mean 1.
fn count(params: Params<'_>, index: usize) -> usize {
    usize::from(params.get(index).max(1))
}

fn move_cursor(screen: &mut Grid, params: Params<'_>, action: char) {
    let here = screen.cursor();
    let n = count(params, 0);
    match action {
        'A' => screen.move_up(n),
        'B' | 'e' => screen.move_down(n),
        'C' | 'a' => screen.move_to(here.row, here.column + n),
        'D' => screen.move_to(here.row, here.column.saturating_sub(n)),
        'E' => {
            screen.move_down(n);
            screen.carriage_return();
        }
        'F' => {
            screen.move_up(n);
            screen.carriage_return();
        }
        'G' | '`' => screen.move_to(here.row, n - 1),
        'd' => screen.move_to_asked(n - 1, here.column),
        _ => screen.move_to_asked(n - 1, count(params, 1) - 1),
    }
}

/// Erasing, inserting, deleting and scrolling.
fn edit(screen: &mut Grid, params: Params<'_>, action: char) {
    let n = count(params, 0);
    match action {
        'J' if params.get(0) == 3 => screen.clear_history(),
        'J' => {
            if let Some(what) = erase(params) {
                screen.erase_in_display(what);
            }
        }
        'K' => {
            if let Some(what) = erase(params) {
                screen.erase_in_line(what);
            }
        }
        'X' => screen.erase_chars(n),
        '@' => screen.insert_chars(n),
        'P' => screen.delete_chars(n),
        'L' => screen.insert_lines(n),
        'M' => screen.delete_lines(n),
        'S' => screen.scroll_up(n),
        // With more parameters, `T` is about the mouse.
        'T' if params.groups().count() <= 1 => screen.scroll_down(n),
        'r' => {
            let bottom = match params.get(1) {
                0 => screen.rows(),
                asked => usize::from(asked),
            };
            screen.set_region(n - 1, bottom - 1);
        }
        's' => screen.save_cursor(),
        'u' => screen.restore_cursor(),
        _ => {}
    }
}

/// `None` for what means nothing here.
fn erase(params: Params<'_>) -> Option<Erase> {
    match params.get(0) {
        0 => Some(Erase::ToEnd),
        1 => Some(Erase::ToStart),
        2 => Some(Erase::All),
        _ => None,
    }
}
