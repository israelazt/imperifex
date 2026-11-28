//! What the rest of the program needs from the system, said by what it needs
//! and not by how Windows (or another system) gives it. The real adapters live
//! in `imperifex-platform`; the tests use fakes.

use std::sync::Arc;
use std::time::Duration;

use crate::draw::{DrawList, UiFont};
use crate::picture::TerminalPicture;
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

/// How much room the texts of the interface take, in physical pixels, with
/// letters of `size` of them.
pub trait TextMeasure {
    fn width(&self, text: &str, font: UiFont, size: u32) -> u32;
    fn line_height(&self, font: UiFont, size: u32) -> u32;
}

/// Putting on the window what the interface asks to have drawn.
pub trait Renderer {
    /// The window now measures this, in physical pixels. A size with nothing in
    /// it, as of a minimized window, is ignored.
    fn resize(&mut self, width: u32, height: u32) -> Result<(), String>;
    /// Draws a whole frame and shows it. After an error the renderer may be of
    /// no more use: a new one is made.
    fn draw(
        &mut self,
        list: &DrawList,
        terminal: Option<&TerminalPicture<'_>>,
    ) -> Result<(), String>;
}

/// Telling whoever waits that there is something new. It may be called from any
/// thread, so it only passes the word on.
pub type Wake = Arc<dyn Fn() + Send + Sync>;

/// Whether the system is set to light or dark.
pub trait SystemAppearance {
    fn dark(&self) -> Result<bool, String>;
    /// From now until the program ends, calls `changed` when the setting may
    /// have changed: `dark` says what it is now.
    fn watch(&self, changed: Wake) -> Result<(), String>;
}

/// The language the system speaks to the user.
pub trait SystemLanguage {
    /// A tag such as `es-MX`; `None` when it cannot be known.
    fn locale(&self) -> Option<String>;
}

/// Where the settings file is kept.
pub trait SettingsStore {
    /// The text of the file; `None` if there is none yet. Bytes that are not
    /// text give the replacement character.
    fn load(&self) -> Result<Option<String>, String>;
    /// Replaces the file, whole or not at all.
    fn save(&self, text: &str) -> Result<(), String>;
    /// Keeps a copy of a file that was damaged, without writing over any other
    /// copy, and says where it is.
    fn keep_damaged(&self, text: &str) -> Result<String, String>;
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
    /// `wake` is called, from another thread, when there is output to read,
    /// when it has ended and when the program leaves. For output, after a call
    /// there is no other until [`Pty::read`] was called, so one call may stand
    /// for much of it.
    fn start(
        &self,
        command_line: &str,
        columns: u16,
        rows: u16,
        wake: Wake,
    ) -> Result<Box<dyn Pty>, String>;
}
