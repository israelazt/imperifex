//! Safety rule S1: the bench only touches what it opened. The author works
//! inside the terminal being measured, so closing, moving, focusing or typing
//! into something that was already there could hurt their session.

use std::collections::HashSet;

/// What existed before a measure. None of it may ever be touched.
///
/// Processes are known by their number. A number Windows hands out again for a
/// new process stays out of reach, which errs on the safe side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyGuard {
    pids: HashSet<u32>,
    windows: HashSet<u64>,
}

impl SafetyGuard {
    /// Takes the photo: every process and every window there is, hidden ones
    /// too, as a hidden window that shows up later is still not the bench's.
    pub fn new(pids: impl IntoIterator<Item = u32>, hwnds: impl IntoIterator<Item = u64>) -> Self {
        Self {
            pids: pids.into_iter().collect(),
            windows: hwnds.into_iter().collect(),
        }
    }

    pub fn may_touch_window(&self, hwnd: u64) -> bool {
        !self.windows.contains(&hwnd)
    }

    pub fn may_touch_process(&self, pid: u32) -> bool {
        !self.pids.contains(&pid)
    }

    /// To be asked before touching a window. The error says why not.
    pub fn check(&self, hwnd: u64) -> Result<(), String> {
        if self.may_touch_window(hwnd) {
            Ok(())
        } else {
            Err(format!(
                "la ventana {hwnd:#x} ya existía antes de la medida: no se toca (regla S1)"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> SafetyGuard {
        SafetyGuard::new([10, 11], [0xA, 0xB])
    }

    #[test]
    fn a_window_that_was_there_cannot_be_touched_and_the_error_says_why() {
        assert!(!guard().may_touch_window(0xA));
        let error = guard().check(0xA).unwrap_err();
        assert!(error.contains("0xa") && error.contains("S1"), "{error}");
    }

    #[test]
    fn a_new_window_can_be_touched() {
        assert!(guard().may_touch_window(0xC));
        assert_eq!(guard().check(0xC), Ok(()));
    }

    #[test]
    fn processes_that_were_there_cannot_be_touched_and_new_ones_can() {
        assert!(!guard().may_touch_process(10));
        assert!(guard().may_touch_process(12));
    }
}
