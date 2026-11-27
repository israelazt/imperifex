//! Modes a program turns on (`h`) and off (`l`). The mouse and focus modes are
//! not here yet: nothing reads them.

use crate::grid::Grid;
use crate::sequence::Params;
use crate::terminal::Terminal;

/// `ESC [ n h`, without a marker.
pub(super) fn ansi(screen: &mut Grid, params: Params<'_>, on: bool) {
    for group in params.groups() {
        if group[0] == 4 {
            screen.insert = on;
        }
    }
}

/// `ESC [ ? n h`. Several modes may come in one sequence.
pub(super) fn private(terminal: &mut Terminal, params: Params<'_>, action: char) {
    let on = match action {
        'h' => true,
        'l' => false,
        _ => return,
    };
    for group in params.groups() {
        set(terminal, group[0], on);
    }
}

fn set(terminal: &mut Terminal, mode: u16, on: bool) {
    match mode {
        1 => terminal.modes.application_cursor_keys = on,
        6 => terminal.screen_mut().set_origin(on),
        7 => terminal.screen_mut().autowrap = on,
        25 => terminal.modes.cursor_visible = on,
        2004 => terminal.modes.bracketed_paste = on,
        47 | 1047 => alternate(terminal, on),
        1048 => save_or_restore(terminal.screen_mut(), on),
        // The cursor is saved on the main screen, before leaving it, and
        // restored there, after coming back.
        1049 if on => {
            terminal.screen_mut().save_cursor();
            alternate(terminal, true);
        }
        1049 => {
            alternate(terminal, false);
            terminal.screen_mut().restore_cursor();
        }
        _ => {}
    }
}

fn alternate(terminal: &mut Terminal, on: bool) {
    if on {
        terminal.enter_alternate_screen();
    } else {
        terminal.leave_alternate_screen();
    }
}

fn save_or_restore(screen: &mut Grid, save: bool) {
    if save {
        screen.save_cursor();
    } else {
        screen.restore_cursor();
    }
}
