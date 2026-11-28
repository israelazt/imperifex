//! Marks on rows, which travel with them into the history and through a change
//! of width: where the commands of a shell start.
//!
//! Here a *line* is a row of the history or of the screen, counted from the
//! oldest one kept.

use super::{Cell, Grid, Wrap};

/// What starts on a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// The prompt of the block with this number. `silent` is that of the
    /// block before it when its command wrote nothing: its output would have
    /// started on this very row.
    Prompt { id: u32, silent: Option<u32> },
    /// What the command of the block with this number wrote.
    Output(u32),
}

impl Grid {
    /// Marks the row of the cursor. A prompt does not forget that the output of
    /// the block before started there.
    pub fn set_mark(&mut self, mark: Mark) {
        let row = self.cursor.row;
        let mark = match (self.rows[row].mark, mark) {
            (Some(Mark::Output(before)), Mark::Prompt { id, .. }) => Mark::Prompt {
                id,
                silent: Some(before),
            },
            _ => mark,
        };
        self.rows[row].mark = Some(mark);
        self.damage(row, row + 1);
    }

    /// How many lines there are.
    pub fn lines(&self) -> usize {
        self.history_len() + self.rows()
    }

    pub fn mark_of(&self, line: usize) -> Option<Mark> {
        match line.checked_sub(self.history_len()) {
            Some(row) => self.rows.get(row)?.mark,
            None => self.history.get(line)?.mark,
        }
    }

    /// The cells of a line; those of the history have no empty ones at the end.
    pub fn line_cells(&self, line: usize) -> Vec<Cell> {
        match line.checked_sub(self.history_len()) {
            Some(row) => self
                .rows
                .get(row)
                .map(|row| row.cells.clone())
                .unwrap_or_default(),
            None => self.history_cells(line).unwrap_or_default(),
        }
    }

    fn goes_on(&self, line: usize) -> bool {
        let wrap = match line.checked_sub(self.history_len()) {
            Some(row) => self.rows.get(row).map(|row| row.wrap),
            None => self.history.get(line).map(|line| line.wrap),
        };
        wrap.is_some_and(|wrap| wrap != Wrap::End)
    }

    /// The text of the lines from `from` up to, not including, `to`, as it was
    /// written: what did not fit in a row is joined again. With no empty lines
    /// at the end.
    pub fn text_of_lines(&self, from: usize, to: usize) -> String {
        let mut text = String::new();
        for line in from..to.min(self.lines()) {
            let mut row = String::new();
            for cell in self.line_cells(line) {
                cell.push_to(&mut row);
            }
            if self.goes_on(line) {
                text.push_str(&row);
            } else {
                text.push_str(row.trim_end_matches(' '));
                text.push('\n');
            }
        }
        text.trim_end_matches('\n').to_string()
    }
}
