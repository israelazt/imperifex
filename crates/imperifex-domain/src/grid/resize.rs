//! Changing the size of a screen.

use super::reflow::Rewrap;
use super::row::{Row, Wrap};
use super::{Cell, Grid, Position, Span, MAX_SIDE};

fn side(asked: u16) -> usize {
    usize::from(asked.clamp(1, MAX_SIDE))
}

impl Grid {
    /// For the main screen: the lines that were cut because they did not fit
    /// are joined and cut again, history included. The cursor stays on its
    /// text, and the scroll region goes back to the whole screen.
    pub fn resize(&mut self, columns: u16, rows: u16) {
        let (columns, rows) = (side(columns), side(rows));
        if (columns, rows) == (self.columns, self.rows()) {
            return;
        }
        // The empty rows under the cursor are not text: they are not moved.
        let written = self
            .rows
            .iter()
            .rposition(|row| !row.is_untouched())
            .map_or(0, |at| at + 1);
        let used = written.max(self.cursor.row + 1);
        let mut rewrap = Rewrap::new(columns, rows, self.history.limit());
        while let Some(line) = self.history.pop_oldest() {
            rewrap.row(line.cells(), line.wrap, None);
        }
        let Position { row, column } = self.cursor;
        let cursor = column + usize::from(self.wrap_pending);
        for (index, old) in self.rows.drain(..used).enumerate() {
            rewrap.row(old.cells, old.wrap, (index == row).then_some(cursor));
        }
        let done = rewrap.finish();
        self.history = done.history;
        self.columns = columns;
        self.rows = done.screen;
        self.finish_resize(done.cursor, done.wrap_pending);
    }

    /// For the alternate screen: the program redraws it, so rows and columns
    /// are only cut or added.
    pub fn resize_cutting(&mut self, columns: u16, rows: u16) {
        let (columns, rows) = (side(columns), side(rows));
        let blank = Cell::default();
        for row in &mut self.rows {
            if row.cells.get(columns).map(Cell::span) == Some(Span::Continuation) {
                row.cells[columns - 1] = blank.clone();
            }
            row.cells.resize(columns, blank.clone());
            row.wrap = Wrap::End;
        }
        self.rows.resize(rows, Row::empty(columns, &blank));
        self.columns = columns;
        self.finish_resize(self.cursor, false);
    }

    fn finish_resize(&mut self, cursor: Position, wrap_pending: bool) {
        self.top = 0;
        self.bottom = self.rows() - 1;
        self.move_to(cursor.row, cursor.column);
        self.wrap_pending = wrap_pending && self.autowrap;
    }
}

#[cfg(test)]
mod tests;
