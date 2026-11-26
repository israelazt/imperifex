//! The performance counter as the bench's clock: the one desktop duplication
//! stamps its frames with.

use std::time::Duration;

use imperifex_bench_domain::qpc::qpc_to_duration;
use imperifex_domain::ports::Clock;
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// How many ticks the counter makes a second; 0 if the system will not say.
pub(crate) fn frequency() -> i64 {
    let mut frequency = 0;
    // SAFETY: the out pointer is to a local that lives through the call.
    let read = unsafe { QueryPerformanceFrequency(&mut frequency) };
    read.map_or(0, |()| frequency)
}

/// Counts from the counter's own start, not from the program's, so that its
/// times and a frame's are on the same scale.
#[derive(Debug, Clone, Copy)]
pub struct QpcClock {
    frequency: i64,
}

impl QpcClock {
    pub fn new() -> Self {
        Self {
            frequency: frequency(),
        }
    }
}

impl Default for QpcClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for QpcClock {
    fn monotonic(&self) -> Duration {
        let mut ticks = 0;
        // SAFETY: the out pointer is to a local. It cannot fail on any Windows
        // the bench runs on; if it did, the time would read as zero.
        let _ = unsafe { QueryPerformanceCounter(&mut ticks) };
        qpc_to_duration(ticks, self.frequency)
    }

    fn sleep(&self, time: Duration) {
        std::thread::sleep(time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_moves_forward_by_about_what_is_slept() {
        let clock = QpcClock::new();
        let before = clock.monotonic();
        clock.sleep(Duration::from_millis(20));
        let passed = clock.monotonic() - before;
        assert!(passed >= Duration::from_millis(19), "{passed:?}");
    }
}
