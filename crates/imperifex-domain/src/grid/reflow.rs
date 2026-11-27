//! Joining the rows of each line and cutting it again at a new width. Rows go
//! in one at a time and come out packed, so a long history is never held as
//! cells all at once.

use super::history::{History, Line};
use super::row::{Row, Wrap};
use super::{Cell, Position, Span};

pub(super) struct Rewrap {
    columns: usize,
    rows: usize,
    /// The cut rows, the oldest first: the history and then the screen.
    out: History,
    /// How many rows were cut, the forgotten ones included.
    made: usize,
    /// The cells of the line being joined.
    line: Vec<Cell>,
    /// Where the cursor is in `line`, if it is on this line.
    pending: Option<usize>,
    /// The cursor's row, counted as `made`, and its column.
    cursor: Option<(usize, usize)>,
}

/// A row cut before a wide character keeps its empty last cell, as one that
/// was written does: joining takes exactly that cell out.
fn pack_cut(cells: &[Cell], wrap: Wrap) -> Line {
    if wrap == Wrap::BeforeWide {
        let mut row = cells.to_vec();
        row.push(Cell::default());
        Line::pack(&row, wrap)
    } else {
        Line::pack(cells, wrap)
    }
}

/// The screen and history after cutting again.
pub(super) struct Done {
    pub history: History,
    pub screen: Vec<Row>,
    pub cursor: Position,
    pub wrap_pending: bool,
}

impl Rewrap {
    pub fn new(columns: usize, rows: usize, history_limit: usize) -> Self {
        Self {
            columns,
            rows,
            out: History::new(history_limit + rows),
            made: 0,
            line: Vec::new(),
            pending: None,
            cursor: None,
        }
    }

    /// Takes the next old row; `cursor` is its column if the cursor is there.
    pub fn row(&mut self, cells: Vec<Cell>, wrap: Wrap, cursor: Option<usize>) {
        if let Some(column) = cursor {
            self.pending = Some(self.line.len() + column);
        }
        self.line.extend(cells);
        match wrap {
            Wrap::End => self.cut_line(),
            Wrap::GoesOn => {}
            Wrap::BeforeWide => {
                self.line.pop();
            }
        }
    }

    fn cut_line(&mut self) {
        let mut line = std::mem::take(&mut self.line);
        let cursor = self.pending.take();
        let empty = Cell::default();
        let written = line
            .iter()
            .rposition(|cell| *cell != empty)
            .map_or(0, |at| at + 1);
        // The line keeps reaching the cursor, though nothing is written there.
        line.resize(written.max(cursor.unwrap_or(0)), empty);
        let mut from = 0;
        loop {
            let (to, wrap) = self.end_of_row(&line, from);
            if to == from && wrap != Wrap::End {
                // Two columns do not fit in a screen of one: it is dropped.
                from += 2;
                continue;
            }
            self.note_cursor(cursor, from, to, wrap == Wrap::End);
            self.out.push(pack_cut(&line[from..to], wrap));
            self.made += 1;
            if wrap == Wrap::End {
                return;
            }
            from = to;
        }
    }

    /// Where the row that starts at `from` ends, and whether the line goes on.
    fn end_of_row(&self, line: &[Cell], from: usize) -> (usize, Wrap) {
        let to = from + self.columns;
        if to >= line.len() {
            (line.len().max(from), Wrap::End)
        } else if line[to].span() == Span::Continuation {
            (to - 1, Wrap::BeforeWide)
        } else {
            (to, Wrap::GoesOn)
        }
    }

    fn note_cursor(&mut self, cursor: Option<usize>, from: usize, to: usize, last: bool) {
        if let Some(at) = cursor {
            if at >= from && (at < to || last) {
                self.cursor = Some((self.made, at - from));
            }
        }
    }

    /// The last rows are the screen, and the cursor's row is one of them.
    pub fn finish(mut self) -> Done {
        if !self.line.is_empty() || self.pending.is_some() {
            self.cut_line();
        }
        let held = self.out.len();
        let forgotten = self.made - held;
        let (cursor_row, column) = self.cursor.unwrap_or((forgotten, 0));
        let cursor_row = cursor_row.saturating_sub(forgotten);
        let start = held.saturating_sub(self.rows).min(cursor_row);
        let limit = self.out.limit() - self.rows;
        let mut history = History::new(limit);
        for _ in 0..start {
            if let Some(line) = self.out.pop_oldest() {
                history.push(line);
            }
        }
        let mut screen = Vec::with_capacity(self.rows);
        while screen.len() < self.rows {
            let line = self.out.pop_oldest();
            screen.push(self.unpack(line));
        }
        Done {
            history,
            screen,
            cursor: Position {
                row: cursor_row - start,
                column: column.min(self.columns - 1),
            },
            // The cursor just past a full row waits there for the next one.
            wrap_pending: column == self.columns,
        }
    }

    fn unpack(&self, line: Option<Line>) -> Row {
        let blank = Cell::default();
        let Some(line) = line else {
            return Row::empty(self.columns, &blank);
        };
        let mut cells = line.cells();
        cells.resize(self.columns, blank);
        Row {
            cells,
            wrap: line.wrap,
        }
    }
}
