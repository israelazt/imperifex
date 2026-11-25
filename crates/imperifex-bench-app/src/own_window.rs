//! A window of the bench's own: opening it and closing it without touching
//! anything that was there before (S1) and without forcing anything (S2).
//! Every measure that needs a window gets it here.

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::desktop::WindowEntry;
use imperifex_bench_domain::ports::{Desktop, ProcessProbe};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::safety::SafetyGuard;
use imperifex_bench_domain::tree::terminal_processes;
use imperifex_bench_domain::windows::{new_window, NewWindow};
use imperifex_domain::ports::Clock;

/// How often the desktop is looked at.
const EVERY: Duration = Duration::from_millis(5);
const TO_SHOW: Duration = Duration::from_secs(20);
const TO_CLOSE: Duration = Duration::from_secs(5);

/// A window the bench opened.
#[derive(Debug)]
pub struct OwnWindow {
    pub window: WindowEntry,
    /// From launching the program until the window was seen.
    pub took: Duration,
    /// The photo from before the launch.
    guard: SafetyGuard,
}

/// Looks every few milliseconds until `found` answers, or gives up after
/// `limit`.
pub fn wait_for<T>(
    clock: &impl Clock,
    limit: Duration,
    mut found: impl FnMut() -> Result<Option<T>, String>,
) -> Result<Option<T>, String> {
    let start = clock.monotonic();
    loop {
        if let Some(value) = found()? {
            return Ok(Some(value));
        }
        if clock.monotonic().saturating_sub(start) >= limit {
            return Ok(None);
        }
        clock.sleep(EVERY);
    }
}

/// The window to adopt, if there is exactly one; an error if the bench cannot
/// tell which one is its own.
fn only_one(found: NewWindow) -> Result<Option<WindowEntry>, String> {
    match found {
        NewWindow::None => Ok(None),
        NewWindow::One(window) => Ok(Some(window)),
        NewWindow::Many(windows) => Err(format!(
            "aparecieron {} ventanas a la vez; no sé cuál es la mía: ciérralas a mano",
            windows.len()
        )),
        NewWindow::Reshown(window) => Err(format!(
            "la terminal mostró una ventana que ya tenía oculta («{}»); no es del banco y no se toca",
            window.title
        )),
    }
}

/// The terminal's processes right now.
pub fn running(
    processes: &impl ProcessProbe,
    profile: &TerminalProfile,
) -> Result<Vec<u32>, String> {
    Ok(terminal_processes(&processes.processes()?, profile))
}

/// The tools to open and close windows.
pub struct Opener<'a, C, D, P> {
    pub clock: &'a C,
    pub desktop: &'a D,
    pub processes: &'a P,
}

impl<C: Clock, D: Desktop, P: ProcessProbe> Opener<'_, C, D, P> {
    /// Takes the photo, launches the terminal and waits for its one new window.
    /// The profile's `exe` must already have its variables expanded.
    pub fn open(&self, profile: &TerminalProfile, dir: &Path) -> Result<OwnWindow, String> {
        let before = self.desktop.windows()?;
        let mut guard = SafetyGuard::new(&before);
        let start = self.clock.monotonic();
        self.desktop
            .launch(Path::new(&profile.exe), &profile.new_window_args, dir)?;
        let found = wait_for(self.clock, TO_SHOW, || {
            let now = self.desktop.windows()?;
            let seen = self.clock.monotonic();
            // Listing the processes is slow: only once some window, of
            // anyone's, could be the one.
            let anyones: Vec<u32> = now.iter().map(|w| w.pid).collect();
            if new_window(&before, &now, &anyones) == NewWindow::None {
                return Ok(None);
            }
            let pids = running(self.processes, profile)?;
            only_one(new_window(&before, &now, &pids)).map(|w| w.map(|w| (w, seen)))
        })?;
        let (window, seen) = found.ok_or_else(|| {
            format!(
                "no apareció una sola ventana nueva de {} en {} s; si quedó abierta alguna que abrió el banco, ciérrala a mano",
                profile.name,
                TO_SHOW.as_secs()
            )
        })?;
        guard.adopt(window.hwnd)?;
        Ok(OwnWindow {
            window,
            took: seen.saturating_sub(start),
            guard,
        })
    }

    /// Asks the window to close and waits until it is gone.
    pub fn close(&self, own: &OwnWindow) -> Result<(), String> {
        let hwnd = own.window.hwnd;
        own.guard.check(hwnd)?;
        self.desktop.close(hwnd)?;
        let gone = wait_for(self.clock, TO_CLOSE, || {
            let there = self.desktop.windows()?;
            let shown = there.iter().any(|w| w.hwnd == hwnd && w.visible);
            Ok((!shown).then_some(()))
        })?;
        gone.ok_or_else(|| {
            format!(
                "la ventana {hwnd:#x} no se cerró en {} s: no se fuerza (regla S2); ciérrala a mano",
                TO_CLOSE.as_secs()
            )
        })
    }
}
