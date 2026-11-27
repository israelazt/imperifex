//! The rows that left the screen through its top, kept small: most of a long
//! history is never looked at again.

use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

use super::row::{Row, Wrap};
use super::{Cell, Span, Style};

/// Cells in a row that look the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Run {
    /// Cells that show something: the second column of a double one does not
    /// count.
    cells: u16,
    style: Style,
}

/// A row as text, with its styles apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Line {
    text: Box<str>,
    /// Empty when no cell has any style.
    runs: Box<[Run]>,
    pub wrap: Wrap,
}

impl Line {
    /// A line that ends here loses the empty cells at its end.
    pub fn pack(cells: &[Cell], wrap: Wrap) -> Self {
        let length = if wrap == Wrap::End {
            let empty = Cell::default();
            cells
                .iter()
                .rposition(|cell| *cell != empty)
                .map_or(0, |at| at + 1)
        } else {
            cells.len()
        };
        let mut text = String::with_capacity(length);
        let mut runs: Vec<Run> = Vec::new();
        for cell in &cells[..length] {
            if cell.span() == Span::Continuation {
                continue;
            }
            cell.push_to(&mut text);
            match runs.last_mut() {
                Some(run) if run.style == cell.style() => run.cells += 1,
                _ => runs.push(Run {
                    cells: 1,
                    style: cell.style(),
                }),
            }
        }
        if runs.iter().all(|run| run.style == Style::default()) {
            runs.clear();
        }
        Self {
            text: text.into_boxed_str(),
            runs: runs.into_boxed_slice(),
            wrap,
        }
    }

    /// The cells back, as they were packed.
    pub fn cells(&self) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::with_capacity(self.text.len());
        let mut styles = self
            .runs
            .iter()
            .flat_map(|run| std::iter::repeat_n(run.style, usize::from(run.cells)));
        for character in self.text.chars() {
            let width = character.width().unwrap_or(0);
            if width == 0 {
                if let Some(cell) = cells
                    .iter_mut()
                    .rfind(|cell| cell.span() != Span::Continuation)
                {
                    cell.add_mark(character);
                }
                continue;
            }
            let style = styles.next().unwrap_or_default();
            if width == 1 {
                cells.push(Cell::new(character, Span::Single, style));
            } else {
                cells.push(Cell::new(character, Span::Double, style));
                cells.push(Cell::continuation(style));
            }
        }
        cells
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct History {
    lines: VecDeque<Line>,
    /// How many lines are kept; 0 keeps none.
    limit: usize,
}

impl History {
    pub fn new(limit: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            limit,
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn get(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }

    /// Adds a line, and forgets the oldest ones past the limit.
    pub fn push(&mut self, line: Line) {
        if self.limit == 0 {
            return;
        }
        while self.lines.len() >= self.limit {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn push_row(&mut self, row: &Row) {
        if self.limit > 0 {
            self.push(Line::pack(&row.cells, row.wrap));
        }
    }

    /// Takes the oldest line out.
    pub fn pop_oldest(&mut self) -> Option<Line> {
        self.lines.pop_front()
    }

    pub fn clear(&mut self) {
        self.lines = VecDeque::new();
    }
}
