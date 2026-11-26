//! The window of a measure that types into it: where it goes, what is watched
//! of it, and closing it whatever happens.

use std::time::Duration;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::ports::{Desktop, Keyboard, ProcessProbe, ScreenWatch, Watch};
use imperifex_domain::ports::Clock;

use crate::screen_wait::{settle, Settled};
use crate::window_session::{Closed, SessionPorts, WindowSession};

/// Where the window is put, on the primary monitor.
pub const PLACE: Rect = Rect {
    x: 100,
    y: 100,
    w: 1280,
    h: 720,
};
/// The top of the window, with its title and tabs, is not watched.
const TITLE_BAR: i32 = 40;
const QUIET: Duration = Duration::from_millis(500);
const TO_SETTLE: Duration = Duration::from_secs(10);

/// What a measure that types needs.
#[derive(Debug, Clone)]
pub struct TypingPorts<C, D, P, S, K> {
    pub session: SessionPorts<C, D, P>,
    pub screen: S,
    pub keyboard: K,
}

/// Waits for the screen to be still, counting from now.
pub fn settle_now(watch: &mut dyn Watch, clock: &impl Clock) -> Result<Settled, String> {
    settle(watch, clock, clock.monotonic(), QUIET, TO_SETTLE)
}

/// Does `work` on the window and closes it afterwards, however the work went.
pub fn then_close<C: Clock, D: Desktop, P: ProcessProbe, T>(
    session: WindowSession<'_, C, D, P>,
    work: impl FnOnce(&WindowSession<'_, C, D, P>) -> Result<T, String>,
) -> Result<(T, Closed), String> {
    let done = work(&session);
    match (done, session.close()) {
        (Ok(done), Ok(closed)) => Ok((done, closed)),
        (Err(error), Ok(_)) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(closing)) => Err(format!("{error}; además, al cerrar: {closing}")),
    }
}

impl<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>
    TypingPorts<C, D, P, S, K>
{
    /// Places the window, starts watching it, waits for it to be still and
    /// brings it to the front.
    pub fn prepare(&self, window: &WindowSession<'_, C, D, P>) -> Result<Box<dyn Watch>, String> {
        window.place(PLACE)?;
        let rect = window.rect_now()?;
        let mut watch = self.screen.watch(Rect {
            y: rect.y + TITLE_BAR,
            h: rect.h - TITLE_BAR,
            ..rect
        })?;
        settle_now(watch.as_mut(), &self.session.clock)?;
        window.focus()?;
        Ok(watch)
    }

    pub fn type_text(&self, window: &WindowSession<'_, C, D, P>, text: &str) -> Result<(), String> {
        self.keyboard.type_text(window.ready_to_type()?, text)
    }

    pub fn press(&self, window: &WindowSession<'_, C, D, P>, key: Key) -> Result<(), String> {
        self.keyboard.press(window.ready_to_type()?, key)
    }

    /// Types a line and runs it.
    pub fn enter_line(
        &self,
        window: &WindowSession<'_, C, D, P>,
        line: &str,
    ) -> Result<(), String> {
        self.type_text(window, line)?;
        self.press(window, Key::Enter)
    }
}
