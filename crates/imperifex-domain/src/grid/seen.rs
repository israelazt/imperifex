//! What a screen shows to someone looking back at its history.

use std::borrow::Cow;

use super::{Cell, Grid};

impl Grid {
    /// The cells of the row `row` of a view `back` rows above the screen: from
    /// the history at the top, and then from the screen. A row of history has
    /// no empty cells at its end. Past the last row there are none.
    pub fn seen_row(&self, back: usize, row: usize) -> Cow<'_, [Cell]> {
        let back = back.min(self.history_len());
        match row.checked_sub(back) {
            Some(on_screen) => {
                let cells = self.rows.get(on_screen).map(|row| row.cells.as_slice());
                Cow::Borrowed(cells.unwrap_or_default())
            }
            None => {
                let at = self.history_len() - back + row;
                Cow::Owned(self.history_cells(at).unwrap_or_default())
            }
        }
    }
}
