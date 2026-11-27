//! One screen of a terminal: its cells and cursor, and what writing does to
//! them.

mod cell;
mod cursor;
mod erase;
mod history;
mod reflow;
mod resize;
mod row;
mod scroll;
mod style;
mod write;

pub use cell::{Cell, Span};
pub use erase::Erase;
pub use style::{Attributes, Color, Style};

use history::History;
use row::{Row, Wrap};

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
    rows: Vec<Row>,
    history: History,
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
    /// It keeps no history.
    pub fn new(columns: u16, rows: u16) -> Self {
        Self::with_history(columns, rows, 0)
    }

    /// Keeps up to `lines` rows of those that leave through the top.
    pub fn with_history(columns: u16, rows: u16, lines: usize) -> Self {
        let columns = usize::from(columns.clamp(1, MAX_SIDE));
        let rows = usize::from(rows.clamp(1, MAX_SIDE));
        Self {
            columns,
            rows: vec![Row::empty(columns, &Cell::default()); rows],
            history: History::new(lines),
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

    /// An empty screen of the same size, without history, that goes on with
    /// this one's cursor, pen and writing modes.
    pub fn blank_copy(&self) -> Self {
        let mut copy = Self {
            rows: vec![Row::empty(self.columns, &Cell::default()); self.rows()],
            history: History::default(),
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
        self.rows.get(row)?.cells.get(column)
    }

    /// What a row shows, without the spaces at its end. Empty past the last row.
    pub fn row_text(&self, row: usize) -> String {
        let mut text = String::new();
        for cell in self.rows.get(row).into_iter().flat_map(|row| &row.cells) {
            cell.push_to(&mut text);
        }
        text.truncate(text.trim_end_matches(' ').len());
        text
    }

    /// How many rows of history there are.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// A row of history, 0 being the oldest, without the spaces at its end.
    pub fn history_text(&self, index: usize) -> Option<&str> {
        let line = self.history.get(index)?;
        Some(line.text().trim_end_matches(' '))
    }

    /// The cells of a row of history; the empty ones at its end are not kept.
    pub fn history_cells(&self, index: usize) -> Option<Vec<Cell>> {
        self.history.get(index).map(history::Line::cells)
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// The history and the screen, one row per line.
    pub fn text_with_history(&self) -> String {
        (0..self.history_len())
            .filter_map(|index| self.history_text(index))
            .map(str::to_string)
            .chain((0..self.rows()).map(|row| self.row_text(row)))
            .collect::<Vec<_>>()
            .join("\n")
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
