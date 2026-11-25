//! The clock of the system: a monotonic time and waiting.

use std::time::{Duration, Instant};

use imperifex_domain::ports::Clock;

pub struct SystemClock {
    start: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn monotonic(&self) -> Duration {
        self.start.elapsed()
    }

    fn sleep(&self, time: Duration) {
        std::thread::sleep(time);
    }
}
