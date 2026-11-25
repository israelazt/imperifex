//! Fakes of the ports, for the tests of this crate. A fake records what it
//! was asked and answers what the test scripts.

use std::cell::Cell;
use std::time::Duration;

use imperifex_domain::ports::Clock;

/// A clock that moves `step_ms` every time it is read, and jumps when waited on.
pub struct FakeClock {
    step_ms: u64,
    now_ms: Cell<u64>,
}

impl FakeClock {
    pub fn ticking(step_ms: u64) -> Self {
        Self {
            step_ms,
            now_ms: Cell::new(0),
        }
    }
}

impl Clock for FakeClock {
    fn monotonic(&self) -> Duration {
        let now = self.now_ms.get();
        self.now_ms.set(now + self.step_ms);
        Duration::from_millis(now)
    }

    fn sleep(&self, time: Duration) {
        let waited = u64::try_from(time.as_millis()).unwrap_or(u64::MAX);
        self.now_ms.set(self.now_ms.get().saturating_add(waited));
    }
}
