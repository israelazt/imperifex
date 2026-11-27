//! Moving the cursor, and remembering where it was.

use super::{Grid, Position, Style};

const TAB_WIDTH: usize = 8;

/// What saving the cursor keeps. With nothing saved, restoring goes to the
/// start with no style.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Saved {
    position: Position,
    pen: Style,
    wrap_pending: bool,
    origin: bool,
}

impl Grid {
    pub fn carriage_return(&mut self) {
        self.move_to(self.cursor.row, 0);
    }

    pub fn backspace(&mut self) {
        self.move_to(self.cursor.row, self.cursor.column.saturating_sub(1));
    }

    /// Goes to the next column that is a multiple of 8, or to the last one.
    pub fn tab(&mut self) {
        let next = (self.cursor.column / TAB_WIDTH + 1) * TAB_WIDTH;
        self.move_to(self.cursor.row, next);
    }

    /// Moves the cursor, kept inside the grid.
    pub fn move_to(&mut self, row: usize, column: usize) {
        self.cursor = Position {
            row: row.min(self.rows() - 1),
            column: column.min(self.columns - 1),
        };
        self.wrap_pending = false;
    }

    /// Moves to a position a program names: with origin mode on, its rows count
    /// from the top of the scroll region and do not leave it.
    pub fn move_to_asked(&mut self, row: usize, column: usize) {
        let row = if self.origin {
            self.top.saturating_add(row).min(self.bottom)
        } else {
            row
        };
        self.move_to(row, column);
    }

    /// The row as [`Self::move_to_asked`] counts it.
    pub fn asked_row(&self) -> usize {
        if self.origin {
            self.cursor.row.saturating_sub(self.top)
        } else {
            self.cursor.row
        }
    }

    /// Goes up `count` rows. From inside the scroll region it does not pass its
    /// top.
    pub fn move_up(&mut self, count: usize) {
        let limit = if self.cursor.row >= self.top {
            self.top
        } else {
            0
        };
        let row = self.cursor.row.saturating_sub(count).max(limit);
        self.move_to(row, self.cursor.column);
    }

    /// Goes down `count` rows. From inside the scroll region it does not pass
    /// its bottom.
    pub fn move_down(&mut self, count: usize) {
        let limit = if self.cursor.row <= self.bottom {
            self.bottom
        } else {
            self.rows() - 1
        };
        let row = self.cursor.row.saturating_add(count).min(limit);
        self.move_to(row, self.cursor.column);
    }

    /// Turns origin mode on or off, and goes to the start it names.
    pub fn set_origin(&mut self, on: bool) {
        self.origin = on;
        self.move_to_asked(0, 0);
    }

    pub fn save_cursor(&mut self) {
        self.saved = Saved {
            position: self.cursor,
            pen: self.pen,
            wrap_pending: self.wrap_pending,
            origin: self.origin,
        };
    }

    pub fn restore_cursor(&mut self) {
        let saved = self.saved;
        self.move_to(saved.position.row, saved.position.column);
        self.pen = saved.pen;
        self.origin = saved.origin;
        self.wrap_pending = saved.wrap_pending && self.cursor == saved.position;
    }
}
