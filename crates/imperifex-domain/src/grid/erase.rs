//! Emptying, inserting and deleting cells of a row. Nothing here moves the
//! cursor, and half a double character is never left.

use super::{Cell, Grid, Span, Wrap};

/// What to empty, seen from the cursor. Its own cell is always included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Erase {
    ToEnd,
    ToStart,
    All,
}

impl Grid {
    fn blank(&self) -> Cell {
        Cell::blank(self.pen.for_emptied())
    }

    /// Empties the cells from `from` up to, not including, `to`. A double
    /// character cut by either edge goes whole.
    pub(super) fn clear_cells(&mut self, row: usize, mut from: usize, mut to: usize) {
        let blank = self.blank();
        let cells = &mut self.rows[row].cells;
        if cells.get(from).map(Cell::span) == Some(Span::Continuation) {
            from -= 1;
        }
        if cells.get(to).map(Cell::span) == Some(Span::Continuation) {
            to += 1;
        }
        cells[from..to].fill(blank);
        if to == self.columns {
            self.rows[row].wrap = Wrap::End;
        }
    }

    /// Empties a double character whose two halves are on both sides of the
    /// edge before `column`.
    fn break_at(&mut self, row: usize, column: usize) {
        let cell = self.rows[row].cells.get(column);
        if cell.map(Cell::span) == Some(Span::Continuation) {
            self.clear_cells(row, column, column);
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

    /// Pushes the rest of the row to the right; what passes the end is lost.
    pub fn insert_chars(&mut self, count: usize) {
        let (row, column) = (self.cursor.row, self.cursor.column);
        let count = count.min(self.columns - column);
        if count == 0 {
            return;
        }
        self.break_at(row, column);
        self.break_at(row, self.columns - count);
        let blank = self.blank();
        self.rows[row].wrap = Wrap::End;
        let rest = &mut self.rows[row].cells[column..];
        rest.rotate_right(count);
        rest[..count].fill(blank);
    }

    /// Takes cells out at the cursor; empty ones come in at the end.
    pub fn delete_chars(&mut self, count: usize) {
        let (row, column) = (self.cursor.row, self.cursor.column);
        let count = count.min(self.columns - column);
        if count == 0 {
            return;
        }
        self.break_at(row, column);
        self.break_at(row, column + count);
        let blank = self.blank();
        self.rows[row].wrap = Wrap::End;
        let rest = &mut self.rows[row].cells[column..];
        rest.rotate_left(count);
        let kept = rest.len() - count;
        rest[kept..].fill(blank);
    }

    pub fn erase_in_display(&mut self, what: Erase) {
        let row = self.cursor.row;
        match what {
            Erase::ToEnd => {
                self.erase_in_line(what);
                self.empty_rows(row + 1, self.rows());
            }
            Erase::ToStart => {
                self.erase_in_line(what);
                self.empty_rows(0, row);
            }
            Erase::All => self.empty_rows(0, self.rows()),
        }
    }
}
