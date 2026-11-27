//! One row of the screen: its cells, and whether its text goes on below.

use super::Cell;

/// Whether a row's line goes on in the next row because it did not fit. A
/// change of width joins and cuts those lines again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Wrap {
    /// The line ends here.
    #[default]
    End,
    /// It goes on in the next row.
    GoesOn,
    /// It goes on, and the last cell was left empty because a character two
    /// columns wide did not fit: that cell is not part of the line.
    BeforeWide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Row {
    pub cells: Vec<Cell>,
    pub wrap: Wrap,
}

impl Row {
    pub fn empty(columns: usize, blank: &Cell) -> Self {
        Self {
            cells: vec![blank.clone(); columns],
            wrap: Wrap::End,
        }
    }

    /// Nothing written and no style: as a new row.
    pub fn is_untouched(&self) -> bool {
        self.wrap == Wrap::End && self.cells.iter().all(|cell| *cell == Cell::default())
    }
}
