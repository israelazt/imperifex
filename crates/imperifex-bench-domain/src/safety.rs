//! Safety rule S1: the bench only touches the windows it opened. The author
//! works inside the terminal being measured, and its new windows belong to the
//! very process that holds their session, so processes cannot tell them apart.
//! A window may be touched only once it has been adopted, and nothing that was
//! there before a launch can be adopted. There is nothing here about
//! processes: the bench has no operation that acts on one.

use std::collections::HashSet;

use crate::desktop::WindowEntry;

/// What existed before a launch, and what has been adopted since.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyGuard {
    before: HashSet<u64>,
    adopted: HashSet<u64>,
}

impl SafetyGuard {
    /// Takes the photo: every window there is, hidden ones too.
    pub fn new(before: &[WindowEntry]) -> Self {
        Self {
            before: before.iter().map(|w| w.hwnd).collect(),
            adopted: HashSet::new(),
        }
    }

    /// Takes a window as the bench's own. A handle of the photo is refused even
    /// if its window is gone and Windows gave the number to another.
    pub fn adopt(&mut self, hwnd: u64) -> Result<(), String> {
        if self.before.contains(&hwnd) {
            return Err(format!(
                "la ventana {hwnd:#x} ya existía: no es del banco (regla S1)"
            ));
        }
        self.adopted.insert(hwnd);
        Ok(())
    }

    /// To be asked before touching a window. The error says why not.
    pub fn check(&self, hwnd: u64) -> Result<(), String> {
        if self.adopted.contains(&hwnd) {
            return Ok(());
        }
        let why = if self.before.contains(&hwnd) {
            "ya existía antes de la medida"
        } else {
            "el banco nunca la adoptó"
        };
        Err(format!("la ventana {hwnd:#x} no se toca: {why} (regla S1)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THEIRS: u64 = 0xA;
    const MINE: u64 = 0xC;

    fn guard() -> SafetyGuard {
        SafetyGuard::new(&[WindowEntry {
            hwnd: THEIRS,
            ..Default::default()
        }])
    }

    #[test]
    fn a_window_of_the_photo_is_neither_adopted_nor_touched() {
        let mut guard = guard();
        assert!(guard.adopt(THEIRS).unwrap_err().contains("ya existía"));
        let error = guard.check(THEIRS).unwrap_err();
        assert!(error.contains("0xa") && error.contains("antes"), "{error}");
    }

    #[test]
    fn a_new_window_is_not_touched_until_it_is_adopted() {
        let error = guard().check(MINE).unwrap_err();
        assert!(error.contains("nunca la adoptó"), "{error}");
    }

    #[test]
    fn an_adopted_window_is_touched() {
        let mut guard = guard();
        guard.adopt(MINE).unwrap();
        assert_eq!(guard.check(MINE), Ok(()));
    }

    #[test]
    fn a_number_of_the_photo_that_shows_up_again_cannot_be_adopted() {
        // The window of the photo closed and Windows reused its number: the
        // guard cannot know, and stays on the safe side.
        let mut guard = guard();
        assert!(guard.adopt(THEIRS).is_err());
        assert!(guard.check(THEIRS).is_err());
    }
}
