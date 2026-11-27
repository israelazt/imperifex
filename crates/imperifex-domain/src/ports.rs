//! What the rest of the program needs from the system, said by what it needs
//! and not by how Windows (or another system) gives it. The real adapters live
//! in `imperifex-platform`; the tests use fakes.

use std::time::Duration;

use crate::sequence::Sequence;

/// The clock: a monotonic time to measure how long things take, and waiting.
pub trait Clock {
    /// Time since an arbitrary start; only differences mean something.
    fn monotonic(&self) -> Duration;
    /// Waits.
    fn sleep(&self, time: Duration);
}

/// Cutting what a shell writes into sequences.
///
/// The bytes may be cut anywhere, even inside a character: what is left over
/// waits for the next call. Nothing is an error: bytes that are not text give
/// the replacement character, and a sequence too long to be read whole is not
/// passed on, except an OSC, which arrives cut short.
pub trait SequenceParser {
    fn parse(&mut self, bytes: &[u8], receive: &mut dyn FnMut(Sequence<'_>));
}
