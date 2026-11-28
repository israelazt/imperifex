//! A terminal as a program sees it: the screen it writes to, the other screen
//! kept aside, and the modes it asked for.

use crate::grid::Grid;
use crate::shell_state::ShellState;

/// How many rows of history are kept when nobody says another number.
pub const DEFAULT_HISTORY: usize = 10_000;

/// What a program asked for that the window, not the screen, has to follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modes {
    pub cursor_visible: bool,
    /// The cursor keys send the application sequences.
    pub application_cursor_keys: bool,
    /// Pasted text goes between marks.
    pub bracketed_paste: bool,
    /// The program is to be told what the mouse does. So far only its wheel is.
    pub mouse_reports: bool,
    /// It is told in digits (SGR) and not in the old way.
    pub mouse_digits: bool,
}

impl Default for Modes {
    fn default() -> Self {
        Self {
            cursor_visible: true,
            application_cursor_keys: false,
            bracketed_paste: false,
            mouse_reports: false,
            mouse_digits: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Terminal {
    screen: Grid,
    /// The main screen, kept while the alternate one is in use.
    main: Option<Grid>,
    pub modes: Modes,
    /// What the shell says it is doing, if it has the integration.
    pub shell: ShellState,
    /// The folder it says it is in (OSC 9;9).
    pub folder: Option<String>,
    history_limit: usize,
}

impl Terminal {
    pub fn new(columns: u16, rows: u16) -> Self {
        Self::with_history(columns, rows, DEFAULT_HISTORY)
    }

    /// Keeps up to `lines` rows of history.
    pub fn with_history(columns: u16, rows: u16, lines: usize) -> Self {
        Self {
            screen: Grid::with_history(columns, rows, lines),
            main: None,
            modes: Modes::default(),
            shell: ShellState::default(),
            folder: None,
            history_limit: lines,
        }
    }

    /// The main screen, also while the alternate one is in use. It is the one
    /// with the history.
    pub fn main_screen(&self) -> &Grid {
        self.main.as_ref().unwrap_or(&self.screen)
    }

    /// The main screen joins and cuts its long lines again; the alternate one
    /// is redrawn by its program.
    pub fn resize(&mut self, columns: u16, rows: u16) {
        match &mut self.main {
            Some(main) => {
                main.resize(columns, rows);
                self.screen.resize_cutting(columns, rows);
            }
            None => self.screen.resize(columns, rows),
        }
    }

    /// The screen in use.
    pub fn screen(&self) -> &Grid {
        &self.screen
    }

    pub fn screen_mut(&mut self) -> &mut Grid {
        &mut self.screen
    }

    pub fn on_alternate_screen(&self) -> bool {
        self.main.is_some()
    }

    /// Goes to an empty alternate screen. On it already, nothing happens.
    pub fn enter_alternate_screen(&mut self) {
        if self.main.is_none() {
            let blank = self.screen.blank_copy();
            self.main = Some(std::mem::replace(&mut self.screen, blank));
        }
    }

    /// Goes back to the main screen as it was left.
    pub fn leave_alternate_screen(&mut self) {
        if let Some(mut main) = self.main.take() {
            // How to write is the program's choice, not the screen's.
            main.pen = self.screen.pen;
            main.autowrap = self.screen.autowrap;
            main.insert = self.screen.insert;
            main.damage_all();
            self.screen = main;
        }
    }

    /// As new, with the same size.
    pub fn reset(&mut self) {
        let columns = u16::try_from(self.screen.columns()).unwrap_or(u16::MAX);
        let rows = u16::try_from(self.screen.rows()).unwrap_or(u16::MAX);
        *self = Self::with_history(columns, rows, self.history_limit);
    }
}
