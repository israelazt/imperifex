//! What the rest of the program needs from the system, said by what it needs
//! and not by how Windows (or another system) gives it. The real adapters live
//! in `imperifex-platform`; the tests use fakes.

use std::time::Duration;

/// The clock: a monotonic time to measure how long things take, and waiting.
pub trait Clock {
    /// Time since an arbitrary start; only differences mean something.
    fn monotonic(&self) -> Duration;
    /// Waits.
    fn sleep(&self, time: Duration);
}
