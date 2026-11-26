//! What the bench needs from the system, said by what it needs. The clock is
//! the terminal's own port, `imperifex_domain::ports::Clock`. The real adapters
//! live in `imperifex-bench-platform`; the tests use fakes.

use std::path::{Path, PathBuf};

use crate::desktop::{Rect, WindowEntry};
use crate::process::{ProcessEntry, ProcessUsage};
use crate::report::Machine;

/// The output of the program, which the terminal being measured takes in.
pub trait Output {
    /// Hands over all the bytes; it returns when they have been accepted, which
    /// may be before they are shown.
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), String>;
    /// Hands over whatever was still held back.
    fn flush(&mut self) -> Result<(), String>;
}

/// The files the bench reads and writes.
pub trait FileStore {
    /// Writes the whole file, replacing it, and creating its folder if needed.
    /// A reader never sees it half written.
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String>;
    /// `Ok(None)` if there is no such file.
    fn read(&self, path: &Path) -> Result<Option<Vec<u8>>, String>;
    /// A folder for the bench's passing files. It may not exist yet.
    fn temp_dir(&self) -> Result<PathBuf, String>;
    /// Whether there is a file or a folder there; one that cannot be looked at
    /// counts as not there.
    fn exists(&self, path: &Path) -> bool;
    /// Makes the folder and those above it. One that is already there is fine.
    fn make_dir(&self, path: &Path) -> Result<(), String>;
}

/// The processes of the system. Reading them changes nothing.
pub trait ProcessProbe {
    /// Every process there is right now.
    fn processes(&self) -> Result<Vec<ProcessEntry>, String>;
    /// What one of the processes of the last list is using. It is an error if it
    /// is gone, or if the system does not let it be read.
    fn usage(&self, pid: u32) -> Result<ProcessUsage, String>;
}

/// The desktop: opening a terminal and handling the windows it shows. What may
/// be touched is not decided here: every service asks the `SafetyGuard` first.
pub trait Desktop {
    /// Starts a program in `dir` without waiting for it, and gives its process.
    fn launch(&self, exe: &Path, args: &[String], dir: &Path) -> Result<u32, String>;
    /// Every top level window, hidden ones too.
    fn windows(&self) -> Result<Vec<WindowEntry>, String>;
    /// Moves and sizes a window without bringing it forward.
    fn place(&self, hwnd: u64, rect: Rect) -> Result<(), String>;
    /// Brings a window to the foreground; it is an error if it does not get there.
    fn focus(&self, hwnd: u64) -> Result<(), String>;
    /// The window in the foreground, if there is one.
    fn foreground(&self) -> Result<Option<u64>, String>;
    /// Asks a window to close, as its close button does. It returns at once:
    /// the window may take a while, or stay. Nothing is ever forced (S2).
    fn close(&self, hwnd: u64) -> Result<(), String>;
}

/// What a report says about where and when it was taken.
pub trait MachineProbe {
    /// The processor, the graphics card of the primary monitor, the system and
    /// the monitors.
    fn machine(&self) -> Result<Machine, String>;
    /// The version a program's file declares as its product's.
    fn product_version(&self, exe: &Path) -> Result<String, String>;
    /// Today, local time, as `YYYY-MM-DD`.
    fn today(&self) -> String;
    /// An environment variable, to expand the paths of the profiles.
    fn env_var(&self, name: &str) -> Option<String>;
}

/// How much the graphics card is working for some processes.
pub trait GpuProbe {
    /// The percent of the 3D engines those processes used since the previous
    /// call, or since the probe was made.
    fn utilization(&mut self, pids: &[u32]) -> Result<f64, String>;
}
