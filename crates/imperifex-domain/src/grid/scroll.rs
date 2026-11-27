//! The scroll region, and everything that moves whole rows.

use super::{Cell, Grid, Wrap};

impl Grid {
    /// Sets the first and last row that scroll, counted from 0, and goes to the
    /// start. A region of less than two rows is not one: nothing happens.
    pub fn set_region(&mut self, top: usize, bottom: usize) {
        let bottom = bottom.min(self.rows() - 1);
        if top < bottom {
            self.top = top;
            self.bottom = bottom;
            self.move_to_asked(0, 0);
        }
    }

    /// Goes down one row. On the last one of the region, the region moves up.
    pub fn line_feed(&mut self) {
        self.wrap_pending = false;
        if self.cursor.row == self.bottom {
            self.scroll_up(1);
        } else if self.cursor.row + 1 < self.rows() {
            self.cursor.row += 1;
        }
    }

    /// Goes up one row. On the first one of the region, the region moves down.
    pub fn reverse_line_feed(&mut self) {
        self.wrap_pending = false;
        if self.cursor.row == self.top {
            self.scroll_down(1);
        } else {
            self.cursor.row = self.cursor.row.saturating_sub(1);
        }
    }

    /// The rows of the region move up and empty ones come in at the bottom.
    /// Those that leave through the top of the screen go to the history; from
    /// a region that starts lower they are lost.
    pub fn scroll_up(&mut self, count: usize) {
        if self.top == 0 {
            let leaving = count.min(self.bottom + 1);
            for row in &self.rows[..leaving] {
                self.history.push_row(row);
            }
        }
        self.shift_up(self.top, self.bottom, count);
    }

    pub fn scroll_down(&mut self, count: usize) {
        self.shift_down(self.top, self.bottom, count);
    }

    /// Empty rows at the cursor push the rest of the region down. Outside the
    /// region it does nothing.
    pub fn insert_lines(&mut self, count: usize) {
        let row = self.cursor.row;
        if (self.top..=self.bottom).contains(&row) {
            self.shift_down(row, self.bottom, count);
            self.carriage_return();
        }
    }

    /// The rows at the cursor go away and the rest of the region comes up.
    pub fn delete_lines(&mut self, count: usize) {
        let row = self.cursor.row;
        if (self.top..=self.bottom).contains(&row) {
            self.shift_up(row, self.bottom, count);
            self.carriage_return();
        }
    }

    /// The row above a block that moves no longer goes on in the same row.
    fn cut_above(&mut self, from: usize) {
        if let Some(above) = from.checked_sub(1) {
            self.rows[above].wrap = Wrap::End;
        }
    }

    fn shift_up(&mut self, from: usize, to: usize, count: usize) {
        self.cut_above(from);
        let rows = &mut self.rows[from..=to];
        let count = count.min(rows.len());
        rows.rotate_left(count);
        let kept = rows.len() - count;
        self.empty_rows(from + kept, from + kept + count);
    }

    fn shift_down(&mut self, from: usize, to: usize, count: usize) {
        self.cut_above(from);
        let rows = &mut self.rows[from..=to];
        let count = count.min(rows.len());
        rows.rotate_right(count);
        self.empty_rows(from, from + count);
        // What ends the block now has another row under it.
        self.rows[to].wrap = Wrap::End;
    }

    pub(super) fn empty_rows(&mut self, from: usize, to: usize) {
        let blank = Cell::blank(self.pen.for_emptied());
        for row in &mut self.rows[from..to] {
            row.cells.fill(blank.clone());
            row.wrap = Wrap::End;
        }
    }
}
