//! The blocks of a terminal (T5.2): each command of a shell with the
//! integration, from its prompt to the next one.

use std::collections::BTreeMap;

use crate::grid::{Cell, Grid, Mark};
use crate::i18n::{decimal, Language};
use crate::terminal::Terminal;

/// How many results are kept: as many as rows of history.
const KEPT: usize = 10_000;
/// How far above the view the start of the block at its top is looked for.
const FAR: usize = 5_000;

/// How a command ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ran {
    pub code: i32,
    /// How long it ran, if the shell said.
    pub millis: Option<u64>,
}

/// The blocks a shell has made.
#[derive(Debug, Clone, Default)]
pub struct Blocks {
    /// The number of the last one that started.
    last: u32,
    /// That of the command running.
    running: Option<u32>,
    ran: BTreeMap<u32, Ran>,
}

impl Blocks {
    /// A prompt starts: the mark for its row.
    pub fn prompt(&mut self) -> Mark {
        self.last = self.last.wrapping_add(1);
        Mark::Prompt(self.last)
    }

    /// Its command starts to write: the mark for that row.
    pub fn output(&mut self) -> Mark {
        self.running = Some(self.last);
        Mark::Output(self.last)
    }

    /// The command running ended.
    pub fn ended(&mut self, ran: Ran) {
        let Some(id) = self.running.take() else {
            return;
        };
        self.ran.insert(id, ran);
        while self.ran.len() > KEPT {
            self.ran.pop_first();
        }
    }

    pub fn ran(&self, id: u32) -> Option<Ran> {
        self.ran.get(&id).copied()
    }
}

/// A block with some row in view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeenBlock {
    pub id: u32,
    /// Its rows of the view, from the first up to, not including, the last.
    pub from: usize,
    pub to: usize,
    /// Its first row is one of the view's.
    pub starts_here: bool,
    /// The row of the view with its command, if it ran one and the row is
    /// seen, and how many of its columns are written.
    pub command: Option<(usize, usize)>,
    pub ran: Option<Ran>,
}

/// The line where the prompt of the block over `line` starts, and its number.
fn prompt_above(screen: &Grid, line: usize) -> Option<(usize, u32)> {
    (line.saturating_sub(FAR)..line)
        .rev()
        .find_map(|at| match screen.mark_of(at) {
            Some(Mark::Prompt(id)) => Some((at, id)),
            _ => None,
        })
}

/// The line of the command of the block `id`, which starts at `from`: the one
/// over where its output starts.
fn command_line(screen: &Grid, id: u32, from: usize) -> Option<usize> {
    for line in from..screen.lines().min(from + FAR) {
        match screen.mark_of(line) {
            Some(Mark::Output(found)) if found == id => {
                return Some(line.saturating_sub(1).max(from));
            }
            Some(Mark::Prompt(found)) if found != id => return None,
            _ => {}
        }
    }
    None
}

fn written_columns(cells: &[Cell]) -> usize {
    let empty = Cell::default();
    let last = cells.iter().rposition(|cell| *cell != empty);
    last.map_or(0, |at| at + 1)
}

/// The blocks with rows in a view of `rows` rows whose top is the row `first`
/// of the screen, from the top down.
pub fn seen(terminal: &Terminal, first: isize, rows: usize) -> Vec<SeenBlock> {
    let screen = terminal.screen();
    let Some(top) = screen.history_len().checked_add_signed(first) else {
        return Vec::new();
    };
    let end = screen.lines().min(top + rows);
    // Each start in view, and before them the block that reaches into it.
    let mut starts: Vec<(usize, u32)> = (top..end)
        .filter_map(|line| match screen.mark_of(line) {
            Some(Mark::Prompt(id)) => Some((line, id)),
            _ => None,
        })
        .collect();
    if starts.first().map_or(true, |&(line, _)| line > top) {
        if let Some(above) = prompt_above(screen, top) {
            starts.insert(0, above);
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(at, &(line, id))| {
            let next = starts.get(at + 1).map_or(end, |&(next, _)| next);
            let command = command_line(screen, id, line)
                .filter(|&command| (top..end).contains(&command))
                .map(|command| {
                    let cells = screen.line_cells(command);
                    (command - top, written_columns(&cells))
                });
            SeenBlock {
                id,
                from: line.saturating_sub(top),
                to: next - top,
                starts_here: line >= top,
                command,
                ran: terminal.blocks.ran(id),
            }
        })
        .collect()
}

/// What the command of the block `id` wrote, if it is still kept.
pub fn output(terminal: &Terminal, id: u32) -> Option<String> {
    let screen = terminal.screen();
    let from = (0..screen.lines())
        .rev()
        .find(|&line| screen.mark_of(line) == Some(Mark::Output(id)))?;
    let to = (from..screen.lines())
        .find(|&line| matches!(screen.mark_of(line), Some(Mark::Prompt(next)) if next != id))
        .unwrap_or_else(|| screen.history_len() + screen.used_rows());
    Some(screen.text_of_lines(from, to))
}

/// How long something took, as it is shown: `0,1 s`, or `2 min 5 s`.
pub fn duration(language: Language, millis: u64) -> String {
    if millis < 60_000 {
        // Under a minute, which an `f64` holds exactly.
        let seconds = millis as f64 / 1000.0;
        return format!("{} s", decimal(language, seconds, 1));
    }
    let seconds = millis / 1000;
    format!("{} min {} s", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests;
