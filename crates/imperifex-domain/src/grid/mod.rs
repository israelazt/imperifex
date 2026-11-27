//! One screen of a terminal: its cells and cursor, and what writing does to
//! them.

mod cell;
mod cursor;
mod erase;
mod scroll;
mod style;
mod write;

pub use cell::{Cell, Span};
pub use erase::Erase;
pub use style::{Attributes, Color, Style};

/// The most columns or rows a grid has: a size comes from outside.
const MAX_SIDE: u16 = 1000;

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
    /// What is written from now on looks like this.
    pub pen: Style,
    /// Writing past the last column goes on in the next row.
    pub autowrap: bool,
    /// Writing pushes the rest of the row instead of covering it.
    pub insert: bool,
    /// Row positions count from the top of the scroll region and stay in it.
    origin: bool,
    /// First and last row that scroll.
    top: usize,
    bottom: usize,
    saved: cursor::Saved,
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
            pen: Style::default(),
            autowrap: true,
            insert: false,
            origin: false,
            top: 0,
            bottom: rows - 1,
            saved: cursor::Saved::default(),
        }
    }

    /// An empty screen of the same size that goes on with this one's cursor,
    /// pen and writing modes.
    pub fn blank_copy(&self) -> Self {
        let mut copy = Self {
            rows: vec![vec![Cell::default(); self.columns]; self.rows()],
            origin: false,
            top: 0,
            bottom: self.rows() - 1,
            saved: cursor::Saved::default(),
            ..*self
        };
        copy.move_to(self.cursor.row, self.cursor.column);
        copy
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
}

#[cfg(test)]
mod tests;
