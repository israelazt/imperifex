//! A fake desktop. Whatever was on it before the test is the author's: asking
//! to touch any of it fails the test on the spot (safety rule S1).
//!
//! A launch opens one window of a new terminal process, which shows up after
//! some looks at the desktop, as a real one takes a while.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use imperifex_bench_domain::desktop::{Rect, WindowEntry};
use imperifex_bench_domain::ports::{Desktop, ProcessProbe};
use imperifex_bench_domain::process::{ProcessEntry, ProcessUsage};

/// The windows a launch opens get handles from here up.
pub const FIRST_OPENED: u64 = 0x100;
/// The processes a launch starts get numbers from here up.
pub const FIRST_PID: u32 = 1000;

/// How the fake behaves, as a test scripts it.
#[derive(Debug, Clone)]
pub struct Behaviour {
    /// Looks at the desktop after a launch before the window is seen; `None` if
    /// it never shows up.
    pub looks_to_show: Option<usize>,
    /// Whether a window goes away when asked to.
    pub closes: bool,
    /// Whether its process leaves with it.
    pub exits_on_close: bool,
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            looks_to_show: Some(1),
            closes: true,
            exits_on_close: true,
        }
    }
}

/// What the fake was asked and what is on it.
#[derive(Debug, Default)]
pub struct State {
    /// Every request that changes something, in order, such as `close 0x100`.
    pub asked: Vec<String>,
    /// The windows opened by launches that are still there.
    pub opened: Vec<WindowEntry>,
    waiting: Vec<(usize, WindowEntry)>,
    launches: usize,
    /// The processes the launches started that are still there.
    alive: Vec<u32>,
}

#[derive(Clone, Default)]
pub struct FakeDesktop {
    /// The author's windows.
    before: Vec<WindowEntry>,
    behaviour: Behaviour,
    pub state: Rc<RefCell<State>>,
}

pub fn window(hwnd: u64, pid: u32, title: &str) -> WindowEntry {
    WindowEntry {
        hwnd,
        pid,
        title: title.into(),
        visible: true,
        ..Default::default()
    }
}

impl FakeDesktop {
    pub fn new(before: Vec<WindowEntry>, behaviour: Behaviour) -> Self {
        Self {
            before,
            behaviour,
            state: Rc::default(),
        }
    }

    fn note(&self, what: &str, hwnd: u64) {
        assert!(
            !self.before.iter().any(|w| w.hwnd == hwnd),
            "S1 broken: asked to {what} the window {hwnd:#x}, which was already there"
        );
        self.state
            .borrow_mut()
            .asked
            .push(format!("{what} {hwnd:#x}"));
    }
}

impl Desktop for FakeDesktop {
    fn launch(&self, exe: &Path, _args: &[String]) -> Result<u32, String> {
        let mut state = self.state.borrow_mut();
        let number = state.launches;
        state.launches += 1;
        state.asked.push(format!("launch {}", exe.display()));
        let pid = FIRST_PID + number as u32;
        state.alive.push(pid);
        if let Some(looks) = self.behaviour.looks_to_show {
            let opened = window(FIRST_OPENED + number as u64, pid, "Terminal");
            state.waiting.push((looks, opened));
        }
        Ok(pid)
    }

    fn windows(&self) -> Result<Vec<WindowEntry>, String> {
        let mut state = self.state.borrow_mut();
        let waiting = std::mem::take(&mut state.waiting);
        for (looks, opened) in waiting {
            match looks.checked_sub(1) {
                Some(left) if left > 0 => state.waiting.push((left, opened)),
                _ => state.opened.push(opened),
            }
        }
        Ok(self.before.iter().chain(&state.opened).cloned().collect())
    }

    fn place(&self, hwnd: u64, _rect: Rect) -> Result<(), String> {
        self.note("place", hwnd);
        Ok(())
    }

    fn focus(&self, hwnd: u64) -> Result<(), String> {
        self.note("focus", hwnd);
        Ok(())
    }

    fn foreground(&self) -> Result<Option<u64>, String> {
        Ok(self.state.borrow().opened.last().map(|w| w.hwnd))
    }

    fn close(&self, hwnd: u64) -> Result<(), String> {
        self.note("close", hwnd);
        if self.behaviour.closes {
            let mut state = self.state.borrow_mut();
            let closed: Vec<u32> = state
                .opened
                .iter()
                .filter(|w| w.hwnd == hwnd)
                .map(|w| w.pid)
                .collect();
            state.opened.retain(|w| w.hwnd != hwnd);
            if self.behaviour.exits_on_close {
                state.alive.retain(|pid| !closed.contains(pid));
            }
        }
        Ok(())
    }
}

/// The processes that go with a fake desktop: the author's and the ones its
/// launches started. All are `warp.exe` unless renamed.
#[derive(Clone)]
pub struct DesktopProcesses {
    author: Vec<u32>,
    state: Rc<RefCell<State>>,
    images: Rc<RefCell<HashMap<u32, String>>>,
    calls: Rc<Cell<usize>>,
}

impl DesktopProcesses {
    pub fn new(desktop: &FakeDesktop, author: &[u32]) -> Self {
        Self {
            author: author.to_vec(),
            state: Rc::clone(&desktop.state),
            images: Rc::default(),
            calls: Rc::default(),
        }
    }

    pub fn rename(&self, pid: u32, image: &str) {
        self.images.borrow_mut().insert(pid, image.into());
    }

    /// How many times the list was asked for.
    pub fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl ProcessProbe for DesktopProcesses {
    fn processes(&self) -> Result<Vec<ProcessEntry>, String> {
        self.calls.set(self.calls.get() + 1);
        let state = self.state.borrow();
        let images = self.images.borrow();
        Ok(self
            .author
            .iter()
            .chain(&state.alive)
            .map(|&pid| ProcessEntry {
                pid,
                parent: 0,
                image: images.get(&pid).map_or("warp.exe", String::as_str).into(),
            })
            .collect())
    }

    fn usage(&self, _pid: u32) -> Result<ProcessUsage, String> {
        Ok(ProcessUsage::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_the_authors_window() -> FakeDesktop {
        FakeDesktop::new(vec![window(0xA, 1, "Session")], Behaviour::default())
    }

    #[test]
    #[should_panic(expected = "S1 broken")]
    fn closing_a_window_that_was_there_fails_the_test() {
        let _ = with_the_authors_window().close(0xA);
    }

    #[test]
    #[should_panic(expected = "S1 broken")]
    fn focusing_a_window_that_was_there_fails_the_test() {
        let _ = with_the_authors_window().focus(0xA);
    }

    #[test]
    #[should_panic(expected = "S1 broken")]
    fn moving_a_window_that_was_there_fails_the_test() {
        let _ = with_the_authors_window().place(0xA, Rect::default());
    }

    #[test]
    fn a_launched_window_shows_up_after_its_looks_and_closes() {
        let desktop = FakeDesktop::new(
            vec![],
            Behaviour {
                looks_to_show: Some(2),
                ..Default::default()
            },
        );
        desktop.launch(Path::new("t.exe"), &[]).unwrap();
        assert!(desktop.windows().unwrap().is_empty());
        assert_eq!(desktop.windows().unwrap().len(), 1);
        assert_eq!(desktop.foreground().unwrap(), Some(FIRST_OPENED));
        desktop.focus(FIRST_OPENED).unwrap();
        desktop.place(FIRST_OPENED, Rect::default()).unwrap();
        desktop.close(FIRST_OPENED).unwrap();
        assert!(desktop.windows().unwrap().is_empty());
        assert_eq!(desktop.state.borrow().asked.len(), 4);
    }
}
