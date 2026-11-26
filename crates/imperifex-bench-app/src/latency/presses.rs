//! The presses themselves: each one only after the screen has been calm.

use std::time::Duration;

use imperifex_bench_domain::change::ChangeDetector;
use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::loads::XorShift64;
use imperifex_bench_domain::ports::{Desktop, Keyboard, ProcessProbe, ScreenWatch, Watch};
use imperifex_bench_domain::screen::ScreenFrame;
use imperifex_domain::ports::Clock;

use crate::measure_window::TypingPorts;
use crate::window_session::WindowSession;

pub(super) const POLL: Duration = Duration::from_millis(5);
/// The pause before a press is this plus up to as much again as the spread.
const LEAST_PAUSE_MS: u64 = 150;
const PAUSE_SPREAD_MS: u64 = 200;
/// The order of the pauses is the same in every run.
const PAUSE_SEED: u64 = 1;
/// A key is pressed only after this long without a change.
const CALM: Duration = Duration::from_millis(300);
/// How long that calm is waited for beyond the pause.
const TO_CALM: Duration = Duration::from_secs(3);
const TO_SHOW: Duration = Duration::from_secs(1);
/// Over this share of presses without a sample, the measure is no good.
const MOST_FAILURES_PERCENT: u32 = 5;

/// The watched screen: what is left out of it and its last picture.
pub(super) struct Screen<'a> {
    pub(super) watch: &'a mut dyn Watch,
    pub(super) detector: ChangeDetector,
    pub(super) latest: ScreenFrame,
}

impl Screen<'_> {
    /// Reads pictures for `pause` and until nothing has changed for [`CALM`].
    /// `false` if that calm never came.
    fn calm(&mut self, clock: &impl Clock, pause: Duration) -> Result<bool, String> {
        let start = clock.monotonic();
        let mut changed_at = start;
        loop {
            if let Some(frame) = self.watch.next(POLL)? {
                if self.detector.changed(&self.latest.bgra, &frame.bgra) {
                    changed_at = frame.at.max(changed_at);
                }
                self.latest = frame;
            }
            let now = clock.monotonic();
            if now.saturating_sub(start) >= pause && now.saturating_sub(changed_at) >= CALM {
                return Ok(true);
            }
            if now.saturating_sub(start) >= pause + TO_CALM {
                return Ok(false);
            }
        }
    }

    /// The first picture presented from `since` on that differs from the screen
    /// as it was then. One presented before belongs to what it was.
    fn change_from(
        &mut self,
        clock: &impl Clock,
        since: Duration,
    ) -> Result<Option<ScreenFrame>, String> {
        let mut base = self.latest.bgra.clone();
        while clock.monotonic().saturating_sub(since) < TO_SHOW {
            let Some(frame) = self.watch.next(POLL)? else {
                continue;
            };
            let changed = self.detector.changed(&base, &frame.bgra);
            self.latest = frame;
            if self.latest.at < since {
                base.clone_from(&self.latest.bgra);
            } else if changed {
                return Ok(Some(self.latest.clone()));
            }
        }
        Ok(None)
    }
}

/// What the presses gave.
pub(super) struct Timed {
    pub(super) samples_ms: Vec<f64>,
    failures: u32,
    /// Samples whose picture added up several presentations.
    joined: u32,
    masked: usize,
}

impl Timed {
    pub(super) fn notes(&self) -> Vec<String> {
        vec![
            format!(
                "Pulsaciones sin muestra (sin calma antes, o sin cambio en 1 s): {}.",
                self.failures
            ),
            format!(
                "Muestras que juntaban varias presentaciones, que pueden llegar hasta un frame tarde (6,9 ms a 144 Hz): {}.",
                self.joined
            ),
            format!(
                "Píxeles ignorados porque cambiaban solos antes de medir: {}.",
                self.masked
            ),
        ]
    }
}

pub(super) fn time<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>(
    ports: &TypingPorts<C, D, P, S, K>,
    window: &WindowSession<'_, C, D, P>,
    screen: &mut Screen<'_>,
    presses: u32,
) -> Result<Timed, String> {
    let clock = &ports.session.clock;
    let mut pauses = XorShift64::new(PAUSE_SEED);
    let mut timed = Timed {
        samples_ms: Vec::new(),
        failures: 0,
        joined: 0,
        masked: screen.detector.masked(),
    };
    for _ in 0..presses {
        let pause = LEAST_PAUSE_MS + pauses.next_u64() % (PAUSE_SPREAD_MS + 1);
        if screen.calm(clock, Duration::from_millis(pause))? {
            one(ports, window, screen, &mut timed)?;
        } else {
            timed.failures += 1;
        }
        if timed.failures * 100 > presses * MOST_FAILURES_PERCENT {
            return Err(format!(
                "fallaron {} de {presses} pulsaciones, más del {MOST_FAILURES_PERCENT} %: la pantalla no se calma o la tecla no se ve",
                timed.failures
            ));
        }
    }
    Ok(timed)
}

/// Presses the key, times it and takes it back.
fn one<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>(
    ports: &TypingPorts<C, D, P, S, K>,
    window: &WindowSession<'_, C, D, P>,
    screen: &mut Screen<'_>,
    timed: &mut Timed,
) -> Result<(), String> {
    let clock = &ports.session.clock;
    // Checked before the clock is read, so that checking is not timed (S3).
    let hwnd = window.ready_to_type()?;
    let pressed = clock.monotonic();
    ports.keyboard.press(hwnd, Key::Char('a'))?;
    match screen.change_from(clock, pressed)? {
        Some(frame) => {
            let took = frame.at.saturating_sub(pressed);
            timed.samples_ms.push(took.as_secs_f64() * 1000.0);
            timed.joined += u32::from(frame.accumulated > 1);
        }
        None => timed.failures += 1,
    }
    let erased = clock.monotonic();
    ports.press(window, Key::Backspace)?;
    screen.change_from(clock, erased)?;
    Ok(())
}
