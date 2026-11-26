//! The presses themselves: each one only after the screen has been calm.

use std::time::Duration;

use imperifex_bench_domain::change::{ChangeDetector, DEFAULT_MIN_PIXELS};
use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::loads::XorShift64;
use imperifex_bench_domain::ports::{Desktop, Keyboard, ProcessProbe, ScreenWatch, Watch};
use imperifex_bench_domain::screen::ScreenFrame;
use imperifex_bench_domain::stats::summarize;
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
const LEARNING: Duration = Duration::from_secs(2);
/// A sample this many times the median is told apart in the notes (D28).
const SLOW: f64 = 4.0;
/// Over this share of presses without a sample, the measure is no good.
const MOST_FAILURES_PERCENT: u32 = 5;

/// The watched screen: what is left out of it and its last picture.
pub(super) struct Screen<'a> {
    watch: &'a mut dyn Watch,
    /// The part of each picture that is looked at; all of it until it is known.
    zone: Option<Rect>,
    pub(super) detector: ChangeDetector,
    latest: ScreenFrame,
}

fn detector_for(frame: &ScreenFrame) -> Result<ChangeDetector, String> {
    ChangeDetector::learn(&[], frame.width, frame.height, DEFAULT_MIN_PIXELS)
}

fn cut(frame: &ScreenFrame, zone: Rect) -> Result<ScreenFrame, String> {
    frame
        .crop(zone)
        .ok_or_else(|| "la imagen no contiene la zona de la tecla".to_string())
}

/// What a press changed: the screen as it was, and as it showed first.
pub(super) struct Change {
    pub(super) before: Vec<u8>,
    pub(super) after: ScreenFrame,
}

impl<'a> Screen<'a> {
    /// `latest` is the picture on the screen now.
    pub(super) fn new(watch: &'a mut dyn Watch, latest: ScreenFrame) -> Result<Self, String> {
        Ok(Self {
            watch,
            zone: None,
            detector: detector_for(&latest)?,
            latest,
        })
    }

    /// The watch, to wait for something other than a key.
    pub(super) fn watch(&mut self) -> &mut dyn Watch {
        self.watch
    }

    /// The size of what is being looked at.
    pub(super) fn size(&self) -> (usize, usize) {
        (self.latest.width, self.latest.height)
    }

    /// From now on only `zone` is looked at, with nothing left out of it yet.
    pub(super) fn narrow(&mut self, zone: Rect) -> Result<(), String> {
        self.latest = cut(&self.latest, zone)?;
        self.detector = detector_for(&self.latest)?;
        self.zone = Some(zone);
        Ok(())
    }

    fn next(&mut self) -> Result<Option<ScreenFrame>, String> {
        match (self.watch.next(POLL)?, self.zone) {
            (Some(frame), Some(zone)) => cut(&frame, zone).map(Some),
            (frame, _) => Ok(frame),
        }
    }

    /// Watches for a while with nothing being typed, to leave out what changes
    /// on its own.
    pub(super) fn learn(&mut self, clock: &impl Clock) -> Result<(), String> {
        let start = clock.monotonic();
        while clock.monotonic().saturating_sub(start) < LEARNING {
            if let Some(frame) = self.next()? {
                self.detector.ignore_changes(&self.latest.bgra, &frame.bgra);
                self.latest = frame;
            }
        }
        Ok(())
    }

    /// Reads pictures for `pause` and until nothing has changed for [`CALM`].
    /// `false` if that calm never came.
    fn calm(&mut self, clock: &impl Clock, pause: Duration) -> Result<bool, String> {
        let start = clock.monotonic();
        let mut changed_at = start;
        loop {
            if let Some(frame) = self.next()? {
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
    ) -> Result<Option<Change>, String> {
        let mut base = self.latest.bgra.clone();
        while clock.monotonic().saturating_sub(since) < TO_SHOW {
            let Some(frame) = self.next()? else {
                continue;
            };
            let changed = self.detector.changed(&base, &frame.bgra);
            self.latest = frame;
            if self.latest.at < since {
                base.clone_from(&self.latest.bgra);
            } else if changed {
                return Ok(Some(Change {
                    before: base,
                    after: self.latest.clone(),
                }));
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
    /// How long the measure had been going at each sample's press.
    at: Vec<Duration>,
}

impl Timed {
    /// The samples far over the median, which stay in (D28).
    fn slow(&self) -> String {
        let median = summarize(&self.samples_ms).map_or(0.0, |s| s.p50);
        let slow: Vec<String> = (1..)
            .zip(self.samples_ms.iter().zip(&self.at))
            .filter(|(_, (ms, _))| **ms > SLOW * median)
            .map(|(number, (ms, at))| format!("la {number} ({ms:.1} ms, a los {} s)", at.as_secs()))
            .collect();
        let slow = if slow.is_empty() {
            "ninguna".to_string()
        } else {
            slow.join("; ")
        };
        format!("Muestras de más de {SLOW} veces el p50, que se quedan: {slow}.")
    }

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
            self.slow(),
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
        at: Vec::new(),
    };
    let started = clock.monotonic();
    for _ in 0..presses {
        if screen.calm(clock, pause(&mut pauses))? {
            one(ports, window, screen, started, &mut timed)?;
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

/// The next pause before a press.
fn pause(pauses: &mut XorShift64) -> Duration {
    Duration::from_millis(LEAST_PAUSE_MS + pauses.next_u64() % (PAUSE_SPREAD_MS + 1))
}

/// Presses the key, waits for it to show and takes it back. Gives when it was
/// pressed and what it changed, if it showed.
fn press_and_take_back<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>(
    ports: &TypingPorts<C, D, P, S, K>,
    window: &WindowSession<'_, C, D, P>,
    screen: &mut Screen<'_>,
) -> Result<(Duration, Option<Change>), String> {
    let clock = &ports.session.clock;
    // Checked before the clock is read, so that checking is not timed (S3).
    let hwnd = window.ready_to_type()?;
    let pressed = clock.monotonic();
    ports.keyboard.press(hwnd, Key::Char('a'))?;
    let change = screen.change_from(clock, pressed)?;
    let erased = clock.monotonic();
    ports.press(window, Key::Backspace)?;
    screen.change_from(clock, erased)?;
    Ok((pressed, change))
}

/// One press that counts.
fn one<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>(
    ports: &TypingPorts<C, D, P, S, K>,
    window: &WindowSession<'_, C, D, P>,
    screen: &mut Screen<'_>,
    started: Duration,
    timed: &mut Timed,
) -> Result<(), String> {
    let (pressed, change) = press_and_take_back(ports, window, screen)?;
    match change {
        Some(change) => {
            let took = change.after.at.saturating_sub(pressed);
            timed.samples_ms.push(took.as_secs_f64() * 1000.0);
            timed.at.push(pressed.saturating_sub(started));
            timed.joined += u32::from(change.after.accumulated > 1);
        }
        None => timed.failures += 1,
    }
    Ok(())
}

/// The areas that `count` presses changed, none of them timed.
pub(super) fn areas<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>(
    ports: &TypingPorts<C, D, P, S, K>,
    window: &WindowSession<'_, C, D, P>,
    screen: &mut Screen<'_>,
    count: u32,
) -> Result<Vec<Rect>, String> {
    let mut pauses = XorShift64::new(PAUSE_SEED);
    (1..=count)
        .map(|number| {
            if !screen.calm(&ports.session.clock, pause(&mut pauses))? {
                return Err(format!(
                    "antes de la pulsación previa {number} la pantalla no se calmó"
                ));
            }
            let (_, change) = press_and_take_back(ports, window, screen)?;
            change
                .and_then(|c| screen.detector.changed_area(&c.before, &c.after.bgra))
                .ok_or_else(|| {
                    format!("la tecla no cambia siempre el mismo sitio: la pulsación previa {number} no cambió nada en 1 s")
                })
        })
        .collect()
}
