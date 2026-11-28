//! Writing characters at the cursor.

use unicode_width::UnicodeWidthChar;

use super::{Cell, Grid, Position, Span, Wrap};

impl Grid {
    /// Writes one character at the cursor. Control characters do nothing here.
    pub fn print(&mut self, character: char) {
        match character.width() {
            None => {}
            Some(0) => self.add_mark(character),
            Some(1) => self.put(character, 1),
            Some(_) => self.put(character, 2),
        }
    }

    /// Writes a character that takes `width` columns, 1 or 2.
    fn put(&mut self, character: char, width: usize) {
        if width > self.columns {
            return;
        }
        if self.autowrap && (self.wrap_pending || self.cursor.column + width > self.columns) {
            self.wrap_line();
        }
        // Without autowrap, what does not fit stays on the last columns.
        let column = self.cursor.column.min(self.columns - width);
        let row = self.cursor.row;
        self.cursor.column = column;
        if self.insert {
            self.insert_chars(width);
        }
        self.clear_cells(row, column, column + width);
        let span = if width == 2 {
            Span::Double
        } else {
            Span::Single
        };
        self.rows[row].cells[column] = Cell::new(character, span, self.pen);
        self.damage(row, row + 1);
        if width == 2 {
            self.rows[row].cells[column + 1] = Cell::continuation(self.pen);
        }
        if column + width == self.columns {
            self.cursor.column = self.columns - 1;
            self.wrap_pending = self.autowrap;
        } else {
            self.cursor.column = column + width;
            self.wrap_pending = false;
        }
    }

    /// Goes on in the next row, and remembers that the line does.
    fn wrap_line(&mut self) {
        let Position { row, column } = self.cursor;
        // Below the scroll region, the last row has no next one.
        if row == self.bottom || row + 1 < self.rows() {
            let left_empty = !self.wrap_pending
                && column + 1 == self.columns
                && self.rows[row].cells[column] == Cell::default();
            self.rows[row].wrap = if left_empty {
                Wrap::BeforeWide
            } else {
                Wrap::GoesOn
            };
        }
        self.carriage_return();
        self.line_feed();
    }

    /// A character without a column joins the one just written. With none to
    /// its left in the row, it is dropped.
    fn add_mark(&mut self, mark: char) {
        let Position { row, column } = self.cursor;
        let written = if self.wrap_pending {
            Some(column)
        } else {
            column.checked_sub(1)
        };
        let Some(mut at) = written else {
            return;
        };
        if self.rows[row].cells[at].span() == Span::Continuation {
            at -= 1;
        }
        self.rows[row].cells[at].add_mark(mark);
        self.damage(row, row + 1);
    }
}
