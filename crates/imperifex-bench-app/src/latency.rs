//! How long a key takes to show on screen, at the prompt and inside a running
//! program (D14).

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::change::{ChangeDetector, DEFAULT_MIN_PIXELS};
use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{Desktop, Keyboard, ProcessProbe, ScreenWatch, Watch};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_bench_domain::screen::ScreenFrame;
use imperifex_bench_domain::shell::powershell_command;
use imperifex_domain::ports::Clock;

use crate::measure_window::{settle_now, then_close, TypingPorts};
use crate::window_session::WindowSession;

mod presses;

/// Typed before measuring at the prompt, and not measured: it makes no word, so
/// the history has nothing to suggest after it (probe 1.4 saw a suggestion).
const NONSENSE: &str = "zqxj";
const LEARNING: Duration = Duration::from_secs(2);

/// Where the key is typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyMode {
    Prompt,
    /// Into the bench's echo program, running in the window.
    Program,
}

pub struct LatencyService<C, D, P, S, K> {
    ports: TypingPorts<C, D, P, S, K>,
    /// The bench's own program, for the window to run its echo.
    bench_exe: String,
}

impl<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>
    LatencyService<C, D, P, S, K>
{
    pub fn new(ports: TypingPorts<C, D, P, S, K>, bench_exe: String) -> Self {
        Self { ports, bench_exe }
    }

    /// Presses `a` `presses` times in a window of its own and times each until
    /// the screen changes.
    pub fn run(
        &self,
        profile: &TerminalProfile,
        presses: u32,
        mode: LatencyMode,
        dir: &Path,
    ) -> Result<Measurement, String> {
        let window = WindowSession::open(&self.ports.session, profile, dir)?;
        let (timed, _) = then_close(window, |window| self.timed(window, presses, mode))?;
        let metric = match mode {
            LatencyMode::Prompt => Metric::KeyLatencyPrompt,
            LatencyMode::Program => Metric::KeyLatencyProgram,
        };
        let notes = timed.notes();
        Measurement::new(metric, timed.samples_ms, notes)
            .ok_or_else(|| "ninguna pulsación dio una muestra".to_string())
    }

    fn timed(
        &self,
        window: &WindowSession<'_, C, D, P>,
        presses: u32,
        mode: LatencyMode,
    ) -> Result<presses::Timed, String> {
        let mut watch = self.ports.prepare(window)?;
        let latest = self.get_ready(window, watch.as_mut(), mode)?;
        let mut screen = self.learn(watch.as_mut(), latest)?;
        let timed = presses::time(&self.ports, window, &mut screen, presses)?;
        if mode == LatencyMode::Program {
            self.ports.press(window, Key::Char('q'))?;
            settle_now(screen.watch, &self.ports.session.clock)?;
        }
        Ok(timed)
    }

    /// Leaves the cursor where the measured key will go, and gives the picture
    /// of it.
    fn get_ready(
        &self,
        window: &WindowSession<'_, C, D, P>,
        watch: &mut dyn Watch,
        mode: LatencyMode,
    ) -> Result<ScreenFrame, String> {
        match mode {
            LatencyMode::Prompt => self.ports.type_text(window, NONSENSE)?,
            LatencyMode::Program => {
                let echo = powershell_command(&self.bench_exe, &["echo"]);
                self.ports.enter_line(window, &echo)?;
            }
        }
        settle_now(watch, &self.ports.session.clock)?
            .frame
            .ok_or_else(|| {
                "la pantalla no entregó ninguna imagen (¿bloqueada o apagada?)".to_string()
            })
    }

    /// Watches for a while with nothing being typed, to leave out what changes
    /// on its own.
    fn learn<'a>(
        &self,
        watch: &'a mut dyn Watch,
        latest: ScreenFrame,
    ) -> Result<presses::Screen<'a>, String> {
        let detector = ChangeDetector::learn(&[], latest.width, latest.height, DEFAULT_MIN_PIXELS)?;
        let mut screen = presses::Screen {
            watch,
            detector,
            latest,
        };
        let clock = &self.ports.session.clock;
        let start = clock.monotonic();
        while clock.monotonic().saturating_sub(start) < LEARNING {
            if let Some(frame) = screen.watch.next(presses::POLL)? {
                screen
                    .detector
                    .ignore_changes(&screen.latest.bgra, &frame.bgra);
                screen.latest = frame;
            }
        }
        Ok(screen)
    }
}

#[cfg(test)]
mod tests;
