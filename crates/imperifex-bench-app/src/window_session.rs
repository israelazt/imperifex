//! One window of the bench's own, from its launch to what it leaves behind.
//! Every measure that opens a window does it here: only an adopted window is
//! ever touched (S1), nothing is forced (S2), and the processes a closed window
//! leaves are waited for (D16).

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::desktop::{Rect, WindowEntry};
use imperifex_bench_domain::ports::{Desktop, ProcessProbe};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::safety::SafetyGuard;
use imperifex_bench_domain::tree::{new_since, terminal_processes};
use imperifex_bench_domain::windows::{new_window, NewWindow};
use imperifex_domain::ports::Clock;

const EVERY_LOOK: Duration = Duration::from_millis(5);
const TO_SHOW: Duration = Duration::from_secs(20);
const EVERY_CLOSE_LOOK: Duration = Duration::from_millis(50);
const TO_CLOSE: Duration = Duration::from_secs(5);
const EVERY_LEFTOVER_LOOK: Duration = Duration::from_secs(1);
/// A closed window of Warp leaves its shell for 40 to 62 s (probe 1.3).
const TO_LEAVE: Duration = Duration::from_secs(90);

/// Looks every `every` until `found` answers, or gives up after `limit`.
pub fn wait_for<T>(
    clock: &impl Clock,
    every: Duration,
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
        clock.sleep(every);
    }
}

/// The terminal's processes right now.
pub fn running(
    processes: &impl ProcessProbe,
    profile: &TerminalProfile,
) -> Result<Vec<u32>, String> {
    Ok(terminal_processes(&processes.processes()?, profile))
}

/// What opening and closing a window needs.
#[derive(Debug, Clone)]
pub struct SessionPorts<C, D, P> {
    pub clock: C,
    pub desktop: D,
    pub processes: P,
}

/// What was left once a window was closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closed {
    /// Processes of the terminal that were not there before the launch and
    /// are still there.
    pub leftovers: usize,
    /// How long they were waited for.
    pub waited: Duration,
}

/// A window the bench opened and adopted.
#[derive(Debug)]
pub struct WindowSession<'a, C, D, P> {
    ports: &'a SessionPorts<C, D, P>,
    profile: &'a TerminalProfile,
    guard: SafetyGuard,
    window: WindowEntry,
    /// Just before the launch.
    pub started_at: Duration,
    /// The look at the desktop that found the window.
    pub visible_at: Duration,
    /// The terminal's processes before the launch.
    processes_before: Vec<u32>,
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

impl<'a, C: Clock, D: Desktop, P: ProcessProbe> WindowSession<'a, C, D, P> {
    /// Takes the photo, launches the terminal in `dir` and adopts its one new
    /// window. The profile's `exe` must already have its variables expanded.
    pub fn open(
        ports: &'a SessionPorts<C, D, P>,
        profile: &'a TerminalProfile,
        dir: &Path,
    ) -> Result<Self, String> {
        let before = ports.desktop.windows()?;
        let processes_before = running(&ports.processes, profile)?;
        let mut guard = SafetyGuard::new(&before);
        let started_at = ports.clock.monotonic();
        ports
            .desktop
            .launch(Path::new(&profile.exe), &profile.new_window_args, dir)?;
        let (window, visible_at) = Self::find(ports, profile, &before)?;
        guard.adopt(window.hwnd)?;
        Ok(Self {
            ports,
            profile,
            guard,
            window,
            started_at,
            visible_at,
            processes_before,
        })
    }

    fn find(
        ports: &SessionPorts<C, D, P>,
        profile: &TerminalProfile,
        before: &[WindowEntry],
    ) -> Result<(WindowEntry, Duration), String> {
        let found = wait_for(&ports.clock, EVERY_LOOK, TO_SHOW, || {
            let now = ports.desktop.windows()?;
            let seen = ports.clock.monotonic();
            // Listing the processes is slow: only once some window, of
            // anyone's, could be the one.
            let anyones: Vec<u32> = now.iter().map(|w| w.pid).collect();
            if new_window(before, &now, &anyones) == NewWindow::None {
                return Ok(None);
            }
            let pids = running(&ports.processes, profile)?;
            Ok(only_one(new_window(before, &now, &pids))?.map(|w| (w, seen)))
        })?;
        found.ok_or_else(|| {
            format!(
                "no apareció ninguna ventana nueva de {} en {} s; si quedó abierta alguna que abrió el banco, ciérrala a mano",
                profile.name,
                TO_SHOW.as_secs()
            )
        })
    }

    /// The window as it was when found.
    pub fn window(&self) -> &WindowEntry {
        &self.window
    }

    /// The handle, only through the guard: whatever touches the window asks
    /// for it here.
    pub fn touch(&self) -> Result<u64, String> {
        self.guard.check(self.window.hwnd)?;
        Ok(self.window.hwnd)
    }

    /// The handle for typing: adopted, and in the foreground right now (S3).
    pub fn ready_to_type(&self) -> Result<u64, String> {
        let hwnd = self.touch()?;
        if self.ports.desktop.foreground()? == Some(hwnd) {
            Ok(hwnd)
        } else {
            Err(format!(
                "la ventana {hwnd:#x} ya no está en primer plano: no se pulsa nada (regla S3)"
            ))
        }
    }

    /// Where the window is now.
    pub fn rect_now(&self) -> Result<Rect, String> {
        let hwnd = self.window.hwnd;
        self.ports
            .desktop
            .windows()?
            .iter()
            .find(|w| w.hwnd == hwnd)
            .map(|w| w.rect)
            .ok_or_else(|| format!("la ventana {hwnd:#x} ya no está"))
    }

    pub fn place(&self, rect: Rect) -> Result<(), String> {
        self.ports.desktop.place(self.touch()?, rect)
    }

    pub fn focus(&self) -> Result<(), String> {
        self.ports.desktop.focus(self.touch()?)
    }

    /// Asks the window to close, waits until it is gone and then for the
    /// processes it leaves. Leftovers are not an error: they are told.
    pub fn close(self) -> Result<Closed, String> {
        let hwnd = self.touch()?;
        let shown: Vec<u64> = self.visible_now()?.iter().map(|w| w.hwnd).collect();
        self.ports.desktop.close(hwnd)?;
        self.wait_until_gone(hwnd, &shown)?;
        self.wait_for_leftovers()
    }

    fn visible_now(&self) -> Result<Vec<WindowEntry>, String> {
        let mut windows = self.ports.desktop.windows()?;
        windows.retain(|w| w.visible);
        Ok(windows)
    }

    fn wait_until_gone(&self, hwnd: u64, shown: &[u64]) -> Result<(), String> {
        let gone = wait_for(&self.ports.clock, EVERY_CLOSE_LOOK, TO_CLOSE, || {
            if !self.ports.desktop.windows()?.iter().any(|w| w.hwnd == hwnd) {
                return Ok(Some(()));
            }
            let asked = self
                .visible_now()?
                .iter()
                .any(|w| w.pid == self.window.pid && !shown.contains(&w.hwnd));
            if asked {
                return Err(format!(
                    "{} pidió confirmar el cierre; el banco no pulsa nada: ciérrala a mano",
                    self.profile.name
                ));
            }
            Ok(None)
        })?;
        gone.ok_or_else(|| {
            format!(
                "la ventana {hwnd:#x} no se cerró en {} s: no se fuerza (regla S2); ciérrala a mano",
                TO_CLOSE.as_secs()
            )
        })
    }

    fn wait_for_leftovers(&self) -> Result<Closed, String> {
        let start = self.ports.clock.monotonic();
        loop {
            let now = running(&self.ports.processes, self.profile)?;
            let leftovers = new_since(&self.processes_before, &now).len();
            let waited = self.ports.clock.monotonic().saturating_sub(start);
            if leftovers == 0 || waited >= TO_LEAVE {
                return Ok(Closed { leftovers, waited });
            }
            self.ports.clock.sleep(EVERY_LEFTOVER_LOOK);
        }
    }
}

#[cfg(test)]
mod tests;
