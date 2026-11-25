//! Telling which window a launch opened.

use std::collections::HashMap;

use crate::desktop::WindowEntry;

/// A smaller window is a popup, a notice or a console's placeholder, not a
/// terminal's window.
pub const MIN_WINDOW: (i32, i32) = (400, 300);

/// What a look at the desktop says about the window a launch should open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewWindow {
    /// Nothing yet.
    None,
    /// Exactly one that was not there before.
    One(WindowEntry),
    /// More than one: another could be one the author opened meanwhile.
    Many(Vec<WindowEntry>),
    /// No new one, but one that was there hidden is now shown. It is not the
    /// bench's to adopt.
    Reshown(WindowEntry),
}

/// A window of one of `pids` that is visible, has a title and the size of a
/// terminal's.
fn could_be(window: &WindowEntry, pids: &[u32]) -> bool {
    window.visible
        && !window.title.is_empty()
        && window.rect.w >= MIN_WINDOW.0
        && window.rect.h >= MIN_WINDOW.1
        && pids.contains(&window.pid)
}

/// Compares the desktop with the photo from before the launch. `pids` are the
/// terminal's processes as of `now`.
pub fn new_window(before: &[WindowEntry], now: &[WindowEntry], pids: &[u32]) -> NewWindow {
    let was_visible: HashMap<u64, bool> = before.iter().map(|w| (w.hwnd, w.visible)).collect();
    let candidates = now.iter().filter(|w| could_be(w, pids));
    let mut new = Vec::new();
    let mut reshown = None;
    for window in candidates {
        match was_visible.get(&window.hwnd) {
            None => new.push(window.clone()),
            Some(false) => reshown = reshown.or(Some(window.clone())),
            Some(true) => {}
        }
    }
    match new.len() {
        0 => reshown.map_or(NewWindow::None, NewWindow::Reshown),
        1 => NewWindow::One(new.remove(0)),
        _ => NewWindow::Many(new),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::Rect;

    const TERMINAL: u32 = 7;

    fn window(hwnd: u64, pid: u32, title: &str, visible: bool, w: i32, h: i32) -> WindowEntry {
        WindowEntry {
            hwnd,
            pid,
            title: title.into(),
            visible,
            rect: Rect { x: 0, y: 0, w, h },
            ..Default::default()
        }
    }

    fn shown(hwnd: u64) -> WindowEntry {
        window(hwnd, TERMINAL, "Warp", true, 1280, 800)
    }

    fn hidden(hwnd: u64) -> WindowEntry {
        WindowEntry {
            visible: false,
            ..shown(hwnd)
        }
    }

    fn look(before: &[WindowEntry], now: &[WindowEntry]) -> NewWindow {
        new_window(before, now, &[TERMINAL])
    }

    #[test]
    fn the_one_new_window_of_the_terminal_is_found() {
        assert_eq!(
            look(&[shown(1)], &[shown(1), shown(2)]),
            NewWindow::One(shown(2))
        );
    }

    #[test]
    fn the_authors_window_that_was_already_there_never_comes_out() {
        assert_eq!(look(&[shown(1)], &[shown(1)]), NewWindow::None);
    }

    #[test]
    fn hidden_untitled_and_foreign_windows_do_not_count() {
        let now = [
            hidden(2),
            window(3, TERMINAL, "", true, 1280, 800),
            window(4, 99, "Notas", true, 1280, 800),
        ];
        assert_eq!(look(&[], &now), NewWindow::None);
    }

    #[test]
    fn a_window_smaller_than_a_terminals_does_not_count() {
        // The 0x0 window of the shell's console, seen in probe 1.3, and popups.
        let now = [
            window(2, TERMINAL, "PseudoConsoleWindow", true, 0, 0),
            window(3, TERMINAL, "Aviso", true, 399, 800),
            window(4, TERMINAL, "Aviso", true, 800, 299),
        ];
        assert_eq!(look(&[], &now), NewWindow::None);
        let smallest = [window(5, TERMINAL, "Warp", true, 400, 300)];
        assert_eq!(look(&[], &smallest), NewWindow::One(smallest[0].clone()));
    }

    #[test]
    fn two_new_ones_are_many() {
        assert_eq!(
            look(&[], &[shown(2), shown(3)]),
            NewWindow::Many(vec![shown(2), shown(3)])
        );
    }

    #[test]
    fn a_hidden_window_that_shows_up_is_reshown_and_not_new() {
        assert_eq!(
            look(&[hidden(1)], &[shown(1)]),
            NewWindow::Reshown(shown(1))
        );
    }

    #[test]
    fn a_new_window_wins_over_a_reshown_one() {
        assert_eq!(
            look(&[hidden(1)], &[shown(1), shown(2)]),
            NewWindow::One(shown(2))
        );
    }
}
