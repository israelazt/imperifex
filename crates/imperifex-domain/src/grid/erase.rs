//! Emptying cells. Nothing here moves the cursor.

use super::{Cell, Grid, Span};

/// What to empty, seen from the cursor. Its own cell is always included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Erase {
    ToEnd,
    ToStart,
    All,
}

impl Grid {
    /// Empties the cells from `from` up to, not including, `to`. A double
    /// character cut by either edge goes whole.
    pub(super) fn clear_cells(&mut self, row: usize, mut from: usize, mut to: usize) {
        let cells = &mut self.rows[row];
        if cells.get(from).map(Cell::span) == Some(Span::Continuation) {
            from -= 1;
        }
        if cells.get(to).map(Cell::span) == Some(Span::Continuation) {
            to += 1;
        }
        cells[from..to].fill(Cell::default());
    }

    fn clear_rows(&mut self, from: usize, to: usize) {
        for row in &mut self.rows[from..to] {
            row.fill(Cell::default());
        }
    }

    pub fn erase_in_line(&mut self, what: Erase) {
        let (row, column) = (self.cursor.row, self.cursor.column);
        let (from, to) = match what {
            Erase::ToEnd => (column, self.columns),
            Erase::ToStart => (0, column + 1),
            Erase::All => (0, self.columns),
        };
        self.clear_cells(row, from, to);
    }

    /// Empties `count` cells from the cursor on, without leaving its row.
    pub fn erase_chars(&mut self, count: usize) {
        let (row, column) = (self.cursor.row, self.cursor.column);
        let to = self.columns.min(column.saturating_add(count));
        if to > column {
            self.clear_cells(row, column, to);
        }
    }

    pub fn erase_in_display(&mut self, what: Erase) {
        let row = self.cursor.row;
        match what {
            Erase::ToEnd => {
                self.erase_in_line(what);
                self.clear_rows(row + 1, self.rows());
            }
            Erase::ToStart => {
                self.erase_in_line(what);
                self.clear_rows(0, row);
            }
            Erase::All => self.clear_rows(0, self.rows()),
        }
    }
}
