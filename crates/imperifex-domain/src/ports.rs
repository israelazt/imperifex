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

/// What reading from a pseudoconsole gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyRead {
    Data(Vec<u8>),
    /// Nothing arrived in the time waited.
    Nothing,
    /// The console is closed and all its output has been read.
    End,
}

/// A program running in a pseudoconsole.
///
/// It is never terminated: closing the console is what asks it to leave, and
/// dropping this closes the console.
pub trait Pty {
    /// Waits up to `wait` for output.
    fn read(&mut self, wait: Duration) -> PtyRead;
    /// Sends what the user types.
    fn write(&mut self, bytes: &[u8]) -> Result<(), String>;
    fn resize(&mut self, columns: u16, rows: u16) -> Result<(), String>;
    /// The exit code, once the program has left.
    fn exit_code(&mut self) -> Result<Option<u32>, String>;
    /// Starts closing the console and returns at once. What is left of the
    /// output still arrives through `read`, until [`PtyRead::End`]. Calling it
    /// again does nothing.
    fn close(&mut self);
    /// The number of its process, to name it to the user.
    fn process_id(&self) -> u32;
}

/// Starting programs in pseudoconsoles.
pub trait PtyLauncher {
    /// `command_line` is the whole line, program included, already quoted.
    fn start(&self, command_line: &str, columns: u16, rows: u16) -> Result<Box<dyn Pty>, String>;
}
