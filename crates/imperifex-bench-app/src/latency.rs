//! How long a key takes to show on screen, at the prompt and inside a running
//! program (D14).

use std::path::Path;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::key_zone::key_zone;
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
/// How many presses that do not count have to change the same place for it to
/// be where the key shows (D27), and in how many at the most.
const TO_AGREE: usize = 5;
const MOST_TO_AGREE: usize = 10;

/// Where the key is typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyMode {
    Prompt,
    /// Into the bench's echo program, running in the window.
    Program,
}

/// Where the key shows, and the presses it took to learn it.
struct Zone {
    area: Rect,
    pressed: usize,
}

impl Zone {
    fn note(&self) -> String {
        let Rect { x, y, w, h } = self.area;
        format!(
            "Zona de la tecla: {w}x{h} en ({x}, {y}) de la región vigilada. Lo de fuera no cuenta. Pulsaciones previas, que no cuentan: {}; de ellas, descartadas por cambiar otro sitio: {}.",
            self.pressed,
            self.pressed - TO_AGREE
        )
    }
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
        let before = self.ports.look()?;
        let window = WindowSession::open(&self.ports.session, profile, dir)?;
        let (((timed, zone), ready_note), _) = then_close(window, |window| {
            let mut prepared = self.ports.prepare(window, before)?;
            let timed = self.timed(window, prepared.watch.as_mut(), presses, mode)?;
            Ok((timed, prepared.ready_note()))
        })?;
        let metric = match mode {
            LatencyMode::Prompt => Metric::KeyLatencyPrompt,
            LatencyMode::Program => Metric::KeyLatencyProgram,
        };
        let mut notes = timed.notes();
        notes.push(zone.note());
        notes.push(ready_note);
        Measurement::new(metric, timed.samples_ms, notes)
            .ok_or_else(|| "ninguna pulsación dio una muestra".to_string())
    }

    fn timed(
        &self,
        window: &WindowSession<'_, C, D, P>,
        watch: &mut dyn Watch,
        presses: u32,
        mode: LatencyMode,
    ) -> Result<(presses::Timed, Zone), String> {
        let latest = self.get_ready(window, watch, mode)?;
        let mut screen = presses::Screen::new(watch, latest)?;
        let zone = self.find_zone(window, &mut screen)?;
        let timed = presses::time(&self.ports, window, &mut screen, presses)?;
        if mode == LatencyMode::Program {
            self.ports.press(window, Key::Char('q'))?;
            settle_now(screen.watch(), &self.ports.session.clock)?;
        }
        Ok((timed, zone))
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

    /// Learns where the key shows and leaves the screen narrowed to it, with
    /// what changes there on its own left out.
    fn find_zone(
        &self,
        window: &WindowSession<'_, C, D, P>,
        screen: &mut presses::Screen<'_>,
    ) -> Result<Zone, String> {
        let clock = &self.ports.session.clock;
        screen.learn(clock)?;
        let found = presses::areas(&self.ports, window, screen, TO_AGREE, MOST_TO_AGREE)?;
        let (width, height) = screen.size();
        let side = |length: usize| i32::try_from(length).map_err(|e| e.to_string());
        let zone = key_zone(&found.areas, side(width)?, side(height)?)?;
        screen.narrow(zone)?;
        screen.learn(clock)?;
        Ok(Zone {
            area: zone,
            pressed: found.pressed,
        })
    }
}

#[cfg(test)]
mod tests;
