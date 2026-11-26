//! A fake desktop. Whatever was on it before the test is the author's: asking
//! to touch any of it fails the test on the spot (safety rule S1).
//!
//! A launch opens a window of a new terminal process, which shows up after some
//! looks at the desktop, as a real one takes a while.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use imperifex_bench_domain::desktop::{Rect, WindowEntry};
use imperifex_bench_domain::ports::Desktop;

use super::processes_of::DesktopProcesses;
use super::{Events, FakeClock};
use crate::window_session::SessionPorts;

/// The windows a launch opens get handles from here up.
pub const FIRST_OPENED: u64 = 0x100;
/// The processes a launch starts get numbers from here up.
pub const FIRST_PID: u32 = 1000;
/// The author's window, its process, and a window of it that is hidden.
pub const AUTHORS_WINDOW: u64 = 0xA;
pub const AUTHORS_PID: u32 = 1;
pub const AUTHORS_HIDDEN: u64 = 0xB;
/// The handle of the question a window asks before closing.
const QUESTION: u64 = 0xD1A;

/// How the fake behaves, as a test scripts it.
#[derive(Debug, Clone)]
pub struct Behaviour {
    /// Looks at the desktop after a launch before the window is seen; `None` if
    /// it never shows up.
    pub looks_to_show: Option<usize>,
    pub windows_per_launch: usize,
    /// A launch shows the author's hidden window.
    pub reshows_hidden: bool,
    /// Whether a window goes away when asked to.
    pub closes: bool,
    /// Whether asking to close it brings up a question instead.
    pub asks_to_confirm: bool,
    /// Whether its process leaves once it is closed.
    pub exits_on_close: bool,
    /// How many looks at the processes it stays for after the close.
    pub lingers_for: usize,
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            looks_to_show: Some(1),
            windows_per_launch: 1,
            reshows_hidden: false,
            closes: true,
            asks_to_confirm: false,
            exits_on_close: true,
            lingers_for: 0,
        }
    }
}

/// What the fake was asked and what is on it.
#[derive(Debug, Default)]
pub struct State {
    /// Every request that changes something, in order, such as `close 0x100`.
    pub asked: Vec<String>,
    /// The folder of each launch.
    pub dirs: Vec<PathBuf>,
    /// The time of each launch, on the rig's clock.
    pub launched_at: Vec<Duration>,
    clock: Option<FakeClock>,
    /// The windows opened by launches that are still there.
    pub opened: Vec<WindowEntry>,
    waiting: Vec<(usize, WindowEntry)>,
    windows_made: u64,
    launches: usize,
    reshown: bool,
    /// The processes the launches started that are still there.
    pub alive: Vec<u32>,
    /// Processes on their way out, with the looks they have left.
    pub leaving: Vec<(usize, u32)>,
    /// A window to answer as the foreground instead of the real one.
    pub foreground_taken_by: Option<u64>,
}

#[derive(Debug, Clone, Default)]
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
        rect: Rect {
            x: 0,
            y: 0,
            w: 1280,
            h: 800,
        },
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

    fn drop_window(&self, hwnd: u64) {
        let mut state = self.state.borrow_mut();
        let Some(at) = state.opened.iter().position(|w| w.hwnd == hwnd) else {
            return;
        };
        let pid = state.opened.remove(at).pid;
        if !self.behaviour.exits_on_close {
            return;
        }
        match self.behaviour.lingers_for {
            0 => state.alive.retain(|&p| p != pid),
            looks => state.leaving.push((looks, pid)),
        }
    }
}

impl Desktop for FakeDesktop {
    fn launch(&self, exe: &Path, _args: &[String], dir: &Path) -> Result<u32, String> {
        let mut state = self.state.borrow_mut();
        let pid = FIRST_PID + state.launches as u32;
        state.launches += 1;
        state.asked.push(format!("launch {}", exe.display()));
        state.dirs.push(dir.to_path_buf());
        let now = state.clock.as_ref().map(FakeClock::now);
        state.launched_at.extend(now);
        state.alive.push(pid);
        state.reshown |= self.behaviour.reshows_hidden;
        if let Some(looks) = self.behaviour.looks_to_show {
            for _ in 0..self.behaviour.windows_per_launch {
                let hwnd = FIRST_OPENED + state.windows_made;
                state.windows_made += 1;
                state.waiting.push((looks, window(hwnd, pid, "Terminal")));
            }
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
        let mut all: Vec<WindowEntry> = self.before.clone();
        for w in &mut all {
            w.visible |= state.reshown && w.hwnd == AUTHORS_HIDDEN;
        }
        all.extend(state.opened.iter().cloned());
        Ok(all)
    }

    fn place(&self, hwnd: u64, rect: Rect) -> Result<(), String> {
        self.note("place", hwnd);
        let mut state = self.state.borrow_mut();
        for w in state.opened.iter_mut().filter(|w| w.hwnd == hwnd) {
            w.rect = rect;
        }
        Ok(())
    }

    fn focus(&self, hwnd: u64) -> Result<(), String> {
        self.note("focus", hwnd);
        Ok(())
    }

    fn foreground(&self) -> Result<Option<u64>, String> {
        let state = self.state.borrow();
        Ok(state
            .foreground_taken_by
            .or_else(|| state.opened.last().map(|w| w.hwnd)))
    }

    fn close(&self, hwnd: u64) -> Result<(), String> {
        self.note("close", hwnd);
        if self.behaviour.asks_to_confirm {
            let mut state = self.state.borrow_mut();
            let pid = state.opened.iter().find(|w| w.hwnd == hwnd).map(|w| w.pid);
            state
                .opened
                .push(window(QUESTION, pid.unwrap_or_default(), "¿Cerrar?"));
        } else if self.behaviour.closes {
            self.drop_window(hwnd);
        }
        Ok(())
    }
}

pub type FakePorts = SessionPorts<FakeClock, FakeDesktop, DesktopProcesses>;

/// A desktop with its processes and a clock that only moves when waited on.
pub struct Rig {
    pub ports: FakePorts,
}

impl Rig {
    pub fn asked(&self) -> Vec<String> {
        self.ports.desktop.state.borrow().asked.clone()
    }

    pub fn dirs(&self) -> Vec<PathBuf> {
        self.ports.desktop.state.borrow().dirs.clone()
    }
}

pub fn home() -> &'static Path {
    Path::new("home")
}

/// With `authors_terminal`, the author has the terminal open: a window, a
/// hidden one and their process, none of which may be touched.
pub fn rig(behaviour: Behaviour, authors_terminal: bool) -> Rig {
    let (before, author) = if authors_terminal {
        let hidden = WindowEntry {
            visible: false,
            ..window(AUTHORS_HIDDEN, AUTHORS_PID, "Hidden")
        };
        let session = window(AUTHORS_WINDOW, AUTHORS_PID, "Session");
        (vec![session, hidden], vec![AUTHORS_PID])
    } else {
        (vec![], vec![])
    };
    let desktop = FakeDesktop::new(before, behaviour);
    let processes = DesktopProcesses::new(&desktop, &author);
    let clock = FakeClock::ticking(0, &Events::default());
    desktop.state.borrow_mut().clock = Some(clock.clone());
    Rig {
        ports: SessionPorts {
            clock,
            desktop,
            processes,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desktop() -> FakeDesktop {
        rig(Behaviour::default(), true).ports.desktop
    }

    #[test]
    #[should_panic(expected = "S1 broken")]
    fn closing_a_window_that_was_there_fails_the_test() {
        let _ = desktop().close(AUTHORS_WINDOW);
    }

    #[test]
    #[should_panic(expected = "S1 broken")]
    fn focusing_a_window_that_was_there_fails_the_test() {
        let _ = desktop().focus(AUTHORS_HIDDEN);
    }

    #[test]
    #[should_panic(expected = "S1 broken")]
    fn moving_a_window_that_was_there_fails_the_test() {
        let _ = desktop().place(AUTHORS_WINDOW, Rect::default());
    }
}
