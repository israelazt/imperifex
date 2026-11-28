//! What a screen shows to someone looking at it from another row than its
//! first one: back into its history, or with its first rows out of sight.

use std::borrow::Cow;

use super::{Cell, Grid};

impl Grid {
    /// The cells of the row `row` of a view whose top is the row `first` of the
    /// screen: negative ones are of the history. A row of history has no empty
    /// cells at its end. Above the history and past the screen there are none.
    pub fn seen_row(&self, first: isize, row: usize) -> Cow<'_, [Cell]> {
        let at = first.saturating_add_unsigned(row);
        match usize::try_from(at) {
            Ok(on_screen) => {
                let cells = self.rows.get(on_screen).map(|row| row.cells.as_slice());
                Cow::Borrowed(cells.unwrap_or_default())
            }
            Err(_) => {
                let index = self.history_len().checked_add_signed(at);
                let cells = index.and_then(|index| self.history_cells(index));
                Cow::Owned(cells.unwrap_or_default())
            }
        }
    }

    /// How many rows from the top of the screen have something, or the cursor:
    /// the empty ones under them are not shown when what is seen is anchored at
    /// the bottom.
    pub fn used_rows(&self) -> usize {
        let written = self
            .rows
            .iter()
            .rposition(|row| !row.is_untouched())
            .map_or(0, |at| at + 1);
        written.max(self.cursor.row + 1)
    }
}
