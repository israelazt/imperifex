//! A fake screen on the fake clock: the pictures a test schedules come out
//! when their time comes, and waiting for one moves the clock.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::ports::{ScreenWatch, Watch};
use imperifex_bench_domain::screen::ScreenFrame;

use super::FakeClock;

/// The size of every fake picture.
pub const SIDE: usize = 100;

/// The size of the pictures of a window being typed into: big enough for the
/// zone of one letter to be a small part of it.
pub const KEY_SIDE: usize = 400;

/// A picture with its first `lit` pixels white.
pub fn picture(at: Duration, lit: usize) -> ScreenFrame {
    picture_of(SIDE, at, lit)
}

pub fn picture_of(side: usize, at: Duration, lit: usize) -> ScreenFrame {
    let mut bgra = vec![0; side * side * 4];
    bgra[..lit * 4].fill(255);
    ScreenFrame {
        at,
        accumulated: 1,
        width: side,
        height: side,
        bgra,
    }
}

/// A picture of a whole monitor, black but for `pixels` white ones from the
/// start of row `row`.
pub fn monitor(
    at: Duration,
    (width, height): (usize, usize),
    row: usize,
    pixels: usize,
) -> ScreenFrame {
    let mut bgra = vec![0; width * height * 4];
    bgra[row * width * 4..(row * width + pixels) * 4].fill(255);
    ScreenFrame {
        at,
        accumulated: 1,
        width,
        height,
        bgra,
    }
}

#[derive(Debug, Default)]
pub struct Shown {
    /// Pictures yet to come, in order of time.
    pub pending: VecDeque<ScreenFrame>,
    /// The regions asked to be watched.
    pub watched: Vec<Rect>,
    /// Why watching fails, if it does.
    pub refusal: Option<String>,
    /// What watching a whole monitor shows: pictures with so many pixels lit,
    /// each so long after the watch began. The same for every such watch.
    pub opening: Vec<(u64, usize)>,
}

#[derive(Debug, Clone)]
pub struct FakeScreen {
    clock: FakeClock,
    pub shown: Rc<RefCell<Shown>>,
}

impl FakeScreen {
    pub fn new(clock: &FakeClock) -> Self {
        Self {
            clock: clock.clone(),
            shown: Rc::default(),
        }
    }

    /// Schedules a picture, keeping them in order of time.
    pub fn show(&self, frame: ScreenFrame) {
        let mut shown = self.shown.borrow_mut();
        let at = shown.pending.partition_point(|f| f.at <= frame.at);
        shown.pending.insert(at, frame);
    }

    pub fn watching(&self) -> FakeWatch {
        FakeWatch(self.clone())
    }
}

pub struct FakeWatch(FakeScreen);

impl Watch for FakeWatch {
    fn next(&mut self, timeout: Duration) -> Result<Option<ScreenFrame>, String> {
        let deadline = self.0.clock.now() + timeout;
        let mut shown = self.0.shown.borrow_mut();
        match shown.pending.front() {
            Some(frame) if frame.at <= deadline => {
                self.0.clock.advance_to(frame.at);
                Ok(shown.pending.pop_front())
            }
            _ => {
                self.0.clock.advance_to(deadline);
                Ok(None)
            }
        }
    }
}

impl ScreenWatch for FakeScreen {
    fn watch(&self, region: Rect) -> Result<Box<dyn Watch>, String> {
        let mut shown = self.shown.borrow_mut();
        shown.watched.push(region);
        if let Some(why) = &shown.refusal {
            return Err(why.clone());
        }
        let opening = shown.opening.clone();
        drop(shown);
        // Only a whole monitor starts where the desktop counts from.
        if (region.x, region.y) == (0, 0) {
            let size = (region.w as usize, region.h as usize);
            let began = self.clock.now();
            for (after, lit) in opening {
                let at = began + Duration::from_millis(after);
                self.show(monitor(at, size, 0, lit));
            }
        }
        Ok(Box::new(self.watching()))
    }
}
