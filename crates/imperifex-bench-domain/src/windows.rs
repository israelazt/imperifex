//! Telling which window a launch opened.

use std::collections::HashSet;

use crate::desktop::WindowEntry;

/// The window that is in `now` and was not in `before`, among those that are
/// visible, have a title and belong to one of the terminal's processes.
///
/// `None` if there is none, and also if there is more than one: another could be
/// one the author opened meanwhile, and the bench must not close that.
pub fn new_window(
    before: &[WindowEntry],
    now: &[WindowEntry],
    pids_of_profile: &[u32],
) -> Option<WindowEntry> {
    let mut found = candidates(before, now).filter(|w| pids_of_profile.contains(&w.pid));
    let first = found.next()?;
    found.next().is_none().then(|| first.clone())
}

/// The new windows that could be a terminal's, whoever they belong to. Asking
/// this first spares listing the processes while nothing has shown up.
pub fn candidates<'a>(
    before: &[WindowEntry],
    now: &'a [WindowEntry],
) -> impl Iterator<Item = &'a WindowEntry> {
    let known: HashSet<u64> = before.iter().map(|w| w.hwnd).collect();
    now.iter()
        .filter(move |w| w.visible && !w.title.is_empty() && !known.contains(&w.hwnd))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TERMINAL: u32 = 7;

    fn window(hwnd: u64, pid: u32, title: &str, visible: bool) -> WindowEntry {
        WindowEntry {
            hwnd,
            pid,
            title: title.into(),
            visible,
            ..Default::default()
        }
    }

    fn shown(hwnd: u64) -> WindowEntry {
        window(hwnd, TERMINAL, "Warp", true)
    }

    fn found(before: &[WindowEntry], now: &[WindowEntry]) -> Option<u64> {
        new_window(before, now, &[TERMINAL]).map(|w| w.hwnd)
    }

    #[test]
    fn the_new_window_of_the_terminal_is_found() {
        assert_eq!(found(&[shown(1)], &[shown(1), shown(2)]), Some(2));
    }

    #[test]
    fn a_window_that_was_already_there_is_never_it() {
        assert_eq!(found(&[shown(1)], &[shown(1)]), None);
        let hidden = window(1, TERMINAL, "Warp", false);
        assert_eq!(found(&[hidden], &[shown(1)]), None);
    }

    #[test]
    fn hidden_untitled_and_foreign_windows_are_not_it() {
        let now = [
            window(2, TERMINAL, "Warp", false),
            // The 0x0 console window the shell showed on the reference machine.
            window(3, TERMINAL, "", true),
            window(4, 99, "Notas", true),
        ];
        assert_eq!(found(&[], &now), None);
    }

    #[test]
    fn with_two_new_ones_there_is_no_telling_which_to_touch() {
        assert_eq!(found(&[], &[shown(2), shown(3)]), None);
    }

    #[test]
    fn the_candidates_are_the_new_visible_titled_windows_of_anyone() {
        let now = [shown(1), window(4, 99, "Notas", true)];
        let handles: Vec<u64> = candidates(&[shown(1)], &now).map(|w| w.hwnd).collect();
        assert_eq!(handles, [4]);
    }
}
