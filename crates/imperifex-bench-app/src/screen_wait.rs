//! Waiting for the screen to settle: for nothing that counts as a change for a
//! while. A blinking cursor does not count (D17).

use std::time::Duration;

use imperifex_bench_domain::change::ChangeDetector;
use imperifex_bench_domain::ports::Watch;
use imperifex_bench_domain::screen::ScreenFrame;
use imperifex_bench_domain::settle::{settled_at, SETTLE_SHARE};
use imperifex_domain::ports::Clock;

/// How long each wait for a picture lasts before looking at the time.
const POLL: Duration = Duration::from_millis(10);

/// What was seen until the screen settled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settled {
    /// The last change, or the start if there was none.
    pub at: Duration,
    /// The last picture, if any was presented.
    pub frame: Option<ScreenFrame>,
    pub changes: usize,
    /// Changes whose picture added up several presentations.
    pub joined: usize,
}

/// The pictures and changes so far.
#[derive(Default)]
struct Seen {
    detector: Option<ChangeDetector>,
    last: Option<ScreenFrame>,
    changes: Vec<Duration>,
    joined: usize,
}

impl Seen {
    fn add(&mut self, frame: ScreenFrame) -> Result<(), String> {
        let Some(last) = &self.last else {
            let pixels = frame.width * frame.height;
            let min = ((pixels as f64 * SETTLE_SHARE).ceil() as usize).max(1);
            self.detector = Some(ChangeDetector::learn(&[], frame.width, frame.height, min)?);
            self.last = Some(frame);
            return Ok(());
        };
        let changed = self
            .detector
            .as_ref()
            .is_some_and(|d| d.changed(&last.bgra, &frame.bgra));
        if changed {
            self.changes.push(frame.at);
            self.joined += usize::from(frame.accumulated > 1);
        }
        self.last = Some(frame);
        Ok(())
    }
}

/// Reads pictures until there have been `quiet` without a change that counts,
/// measuring the first silence from `start`. It gives up `max` after `start`.
pub fn settle(
    watch: &mut dyn Watch,
    clock: &impl Clock,
    start: Duration,
    quiet: Duration,
    max: Duration,
) -> Result<Settled, String> {
    settle_when(watch, clock, start, quiet, max, || true)
}

/// Like [`settle`], but the screen is only taken as settled once `ready` says
/// that what was being waited for has happened. Pictures are read all along, so
/// no change goes unseen meanwhile.
pub fn settle_when(
    watch: &mut dyn Watch,
    clock: &impl Clock,
    start: Duration,
    quiet: Duration,
    max: Duration,
    mut ready: impl FnMut() -> bool,
) -> Result<Settled, String> {
    let mut seen = Seen::default();
    loop {
        if let Some(frame) = watch.next(POLL)? {
            seen.add(frame)?;
        }
        let now = clock.monotonic();
        // The silence that counts is the one going on now, not an earlier one.
        let since = seen.changes.last().copied().unwrap_or(start);
        if now.saturating_sub(since) >= quiet && ready() {
            let at = settled_at(&seen.changes, start, quiet, now.max(since)).unwrap_or(since);
            return Ok(Settled {
                at,
                frame: seen.last,
                changes: seen.changes.len(),
                joined: seen.joined,
            });
        }
        if now.saturating_sub(start) >= max {
            return Err(format!(
                "la pantalla no se asentó en {} s ({} cambios)",
                max.as_secs(),
                seen.changes.len()
            ));
        }
    }
}

#[cfg(test)]
mod tests;
