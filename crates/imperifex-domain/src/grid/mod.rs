//! The visible screen of a terminal: the cells and the cursor, and what
//! writing and moving do to them.

mod cell;
mod erase;

use unicode_width::UnicodeWidthChar;

pub use cell::{Cell, Span};
pub use erase::Erase;

/// The most columns or rows a grid has: a size comes from outside.
const MAX_SIDE: u16 = 1000;
const TAB_WIDTH: usize = 8;

/// A place in the grid, counted from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Grid {
    columns: usize,
    /// Every row has exactly `columns` cells, and half a double character is
    /// never left in one.
    rows: Vec<Vec<Cell>>,
    cursor: Position,
    /// The last column was written: the next character goes to the next row.
    /// Until then the cursor stays; any movement forgets it.
    wrap_pending: bool,
}

impl Grid {
    /// An empty grid. Each side is kept between 1 and 1000.
    pub fn new(columns: u16, rows: u16) -> Self {
        let columns = usize::from(columns.clamp(1, MAX_SIDE));
        let rows = usize::from(rows.clamp(1, MAX_SIDE));
        Self {
            columns,
            rows: vec![vec![Cell::default(); columns]; rows],
            cursor: Position::default(),
            wrap_pending: false,
        }
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn rows(&self) -> usize {
        self.rows.len()
    }

    pub fn cursor(&self) -> Position {
        self.cursor
    }

    pub fn cell(&self, row: usize, column: usize) -> Option<&Cell> {
        self.rows.get(row)?.get(column)
    }

    /// What a row shows, without the spaces at its end. Empty past the last row.
    pub fn row_text(&self, row: usize) -> String {
        let mut text = String::new();
        for cell in self.rows.get(row).into_iter().flatten() {
            cell.push_to(&mut text);
        }
        text.truncate(text.trim_end_matches(' ').len());
        text
    }

    /// Every row, one per line.
    pub fn text(&self) -> String {
        (0..self.rows())
            .map(|row| self.row_text(row))
            .collect::<Vec<_>>()
            .join("\n")
    }

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
        if self.wrap_pending || self.cursor.column + width > self.columns {
            self.carriage_return();
            self.line_feed();
        }
        let Position { row, column } = self.cursor;
        self.clear_cells(row, column, column + width);
        if width == 2 {
            self.rows[row][column] = Cell::new(character, Span::Double);
            self.rows[row][column + 1] = Cell::continuation();
        } else {
            self.rows[row][column] = Cell::new(character, Span::Single);
        }
        if column + width == self.columns {
            self.cursor.column = self.columns - 1;
            self.wrap_pending = true;
        } else {
            self.cursor.column = column + width;
        }
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
        if self.rows[row][at].span() == Span::Continuation {
            at -= 1;
        }
        self.rows[row][at].add_mark(mark);
    }

    pub fn carriage_return(&mut self) {
        self.move_to(self.cursor.row, 0);
    }

    /// Goes down one row. On the last one everything moves up and the top row
    /// is lost.
    pub fn line_feed(&mut self) {
        self.wrap_pending = false;
        if self.cursor.row + 1 < self.rows() {
            self.cursor.row += 1;
        } else {
            self.rows.rotate_left(1);
            let last = self.rows() - 1;
            self.rows[last].fill(Cell::default());
        }
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
}

#[cfg(test)]
mod tests;
