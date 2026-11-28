//! What a shell with the integration says it is doing, with the marks of
//! OSC 133: `A` starts its prompt, `B` ends it and waits for a command, and
//! `C` starts running one.

use crate::grid::Grid;
use crate::terminal::Terminal;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ShellState {
    /// It has said nothing: it may have no integration.
    #[default]
    Unknown,
    /// Writing its prompt, which started on this row, counted from the first
    /// one the history was ever given, and at this column.
    Prompting(u64, usize),
    /// Waiting for a command after this prompt, one line per row.
    Waiting(Vec<String>),
    /// Running a command: what is typed goes to it.
    Running,
}

/// The row of the cursor, counted as [`ShellState::Prompting`] does.
fn cursor_row(screen: &Grid) -> u64 {
    screen.history_added() + screen.cursor().row as u64
}

/// What is from the row `start`, at `column`, to the cursor, a line per row.
fn written_since(screen: &Grid, start: u64, column: usize) -> Vec<String> {
    let cursor = screen.cursor();
    // How many rows above the cursor the first one is; a prompt is short.
    let above = cursor_row(screen).saturating_sub(start);
    let above = usize::try_from(above).unwrap_or(0).min(screen.rows());
    let top = 0isize
        .saturating_add_unsigned(cursor.row)
        .saturating_sub_unsigned(above);
    (0..=above)
        .map(|at| {
            let cells = screen.seen_row(top, at);
            let end = if at == above {
                cursor.column.min(cells.len())
            } else {
                cells.len()
            };
            // The first row may have output that ended without a line break.
            let from = if at == 0 { column.min(end) } else { 0 };
            let mut text = String::new();
            for cell in &cells[from..end] {
                cell.push_to(&mut text);
            }
            text.trim_end().to_string()
        })
        .collect()
}

impl Terminal {
    /// A mark of OSC 133, by its letter. Others, such as the `D` with how the
    /// command ended, are not used yet.
    pub(crate) fn mark(&mut self, letter: &[u8]) {
        // A program on the whole screen is not the shell's prompt.
        if self.on_alternate_screen() {
            return;
        }
        match letter {
            b"A" => {
                let screen = self.screen();
                let column = screen.cursor().column;
                self.shell = ShellState::Prompting(cursor_row(screen), column);
            }
            b"B" => {
                if let ShellState::Prompting(row, column) = self.shell {
                    let lines = written_since(self.screen(), row, column);
                    self.shell = ShellState::Waiting(lines);
                }
            }
            b"C" => self.shell = ShellState::Running,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
