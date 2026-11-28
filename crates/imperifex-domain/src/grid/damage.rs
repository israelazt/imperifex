//! Which rows of a screen changed since the last time they were drawn: only
//! those need drawing again.

use super::Grid;

/// One flag per row of the screen, by its place on it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Damage {
    pub rows: Vec<bool>,
}

impl Damage {
    pub fn any(&self) -> bool {
        self.rows.contains(&true)
    }
}

impl Grid {
    /// What changed since the last call, which it forgets. A new screen, one
    /// of another size or one brought back has all its rows changed. Moving
    /// the cursor changes no row.
    pub fn take_damage(&mut self) -> Damage {
        let rows = self.damaged.clone();
        self.damaged.fill(false);
        Damage { rows }
    }

    /// Whether anything changed since [`Grid::take_damage`], without
    /// forgetting it.
    pub fn is_damaged(&self) -> bool {
        self.damaged.contains(&true)
    }

    pub(crate) fn damage_all(&mut self) {
        self.damaged = vec![true; self.rows()];
    }

    /// The rows from `from` up to, not including, `to`.
    pub(super) fn damage(&mut self, from: usize, to: usize) {
        if let Some(rows) = self.damaged.get_mut(from..to) {
            rows.fill(true);
        }
    }
}

#[cfg(test)]
mod tests;
