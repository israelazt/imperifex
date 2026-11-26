//! When a new window showed its first content and when it was ready (D19),
//! told from the screen: against a picture of the primary monitor taken before
//! the launch. `startup` times it; what types into a window waits for it (D25).

use std::cell::Cell;
use std::time::Duration;

use imperifex_bench_domain::change::ChangeDetector;
use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::ports::{ScreenWatch, Watch};
use imperifex_bench_domain::screen::ScreenFrame;
use imperifex_bench_domain::settle::SETTLE_SHARE;
use imperifex_domain::ports::Clock;

use crate::screen_wait::{settle, settle_when};

/// How long the monitor has to be still before the launch.
const CALM_BEFORE: Duration = Duration::from_secs(1);
const TO_BE_CALM: Duration = Duration::from_secs(10);
/// A ready window is one that has been still this long. Warp stays still for
/// about 1 s while its shell starts, before it is ready.
const READY_QUIET: Duration = Duration::from_secs(3);
/// Counted from when the window is visible.
const TO_BE_READY: Duration = Duration::from_secs(15);

/// A picture that marks one of the times of a start.
#[derive(Debug, Clone, Copy)]
pub struct Moment {
    /// From the launch until it was presented.
    pub time: Duration,
    /// It added up several presentations, so it may say too late a time.
    pub joined: bool,
}

/// What the screen told of one start; each time, or why there is none.
pub struct Drawn {
    pub content: Result<Moment, String>,
    pub ready: Result<Moment, String>,
}

impl Drawn {
    pub fn unseen(why: String) -> Self {
        Self {
            content: Err(why.clone()),
            ready: Err(why),
        }
    }
}

/// The primary monitor as it was before the launch, still being watched.
pub struct Before {
    watch: Box<dyn Watch>,
    picture: ScreenFrame,
}

/// Watches the whole primary monitor and waits for it to be still. A new
/// duplication gives its picture at once, even of a still screen.
pub fn look_before(
    screen: &impl ScreenWatch,
    clock: &impl Clock,
    primary: Rect,
) -> Result<Before, String> {
    let mut watch = screen.watch(primary)?;
    let calm = settle(
        watch.as_mut(),
        clock,
        clock.monotonic(),
        CALM_BEFORE,
        TO_BE_CALM,
    )
    .map_err(|error| format!("antes de lanzar, {error}"))?;
    let picture = calm
        .frame
        .ok_or("no llegó ninguna imagen del monitor principal antes de lanzar")?;
    Ok(Before { watch, picture })
}

/// The pictures of the monitor cut to the window, the first of them the one of
/// before the launch. It notes the first that differs from that one.
struct WindowWatch<'a> {
    monitor: &'a mut dyn Watch,
    area: Rect,
    detector: ChangeDetector,
    /// The window's place as it was before the launch.
    base: ScreenFrame,
    base_handed: bool,
    launched: Duration,
    /// When it was presented, and whether it added up several.
    content: &'a Cell<Option<(Duration, bool)>>,
    /// The moments of the pictures that added up several presentations.
    joined: Vec<Duration>,
}

impl Watch for WindowWatch<'_> {
    fn next(&mut self, timeout: Duration) -> Result<Option<ScreenFrame>, String> {
        if !self.base_handed {
            self.base_handed = true;
            return Ok(Some(self.base.clone()));
        }
        let Some(frame) = self.monitor.next(timeout)? else {
            return Ok(None);
        };
        let cut = frame
            .crop(self.area)
            .ok_or("la imagen del monitor no contiene la ventana")?;
        if cut.accumulated > 1 {
            self.joined.push(cut.at);
        }
        if self.content.get().is_none()
            && cut.at >= self.launched
            && self.detector.changed(&self.base.bgra, &cut.bgra)
        {
            self.content.set(Some((cut.at, cut.accumulated > 1)));
        }
        Ok(Some(cut))
    }
}

fn not_on_it(window: Rect) -> String {
    format!(
        "la ventana no cabe entera en el monitor principal: abrió en ({}, {}), de {}x{}",
        window.x, window.y, window.w, window.h
    )
}

/// Reads the screen until the window that was found is ready, or until it has
/// had long enough. The window is left where it opened.
pub fn watch_window(
    mut before: Before,
    clock: &impl Clock,
    window: Rect,
    launched: Duration,
    visible: Duration,
) -> Drawn {
    let Some(base) = before.picture.crop(window) else {
        return Drawn::unseen(not_on_it(window));
    };
    let pixels = base.width * base.height;
    let min = ((pixels as f64 * SETTLE_SHARE).ceil() as usize).max(1);
    let detector = match ChangeDetector::learn(&[], base.width, base.height, min) {
        Ok(detector) => detector,
        Err(error) => return Drawn::unseen(error),
    };
    let content = Cell::new(None);
    let mut watch = WindowWatch {
        monitor: before.watch.as_mut(),
        area: window,
        detector,
        base,
        base_handed: false,
        launched,
        content: &content,
        joined: Vec::new(),
    };
    let longest = visible.saturating_sub(launched) + TO_BE_READY;
    let settled = settle_when(&mut watch, clock, launched, READY_QUIET, longest, || {
        content.get().is_some()
    });
    let Some((first, joined)) = content.get() else {
        let why = settled.err().unwrap_or_default();
        return Drawn::unseen(format!(
            "la ventana no cambió respecto a lo que había antes de lanzar. {why}"
        ));
    };
    let since_launch = |at: Duration, joined| Moment {
        time: at.saturating_sub(launched),
        joined,
    };
    Drawn {
        content: Ok(since_launch(first, joined)),
        ready: settled.map(|settled| {
            // Content that came in steps too small to count one by one leaves
            // no later change: it was ready no sooner than it showed.
            let at = settled.at.max(first);
            since_launch(at, watch.joined.contains(&at))
        }),
    }
}
