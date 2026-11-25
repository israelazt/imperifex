//! What the bench needs from the system, said by what it needs. The clock is
//! the terminal's own port, `imperifex_domain::ports::Clock`. The real adapters
//! live in `imperifex-bench-platform`; the tests use fakes.

use std::path::{Path, PathBuf};

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
}

/// The processes of the system. Reading them changes nothing.
pub trait ProcessProbe {
    /// Every process there is right now.
    fn processes(&self) -> Result<Vec<ProcessEntry>, String>;
    /// What one of the processes of the last list is using. It is an error if it
    /// is gone, or if the system does not let it be read.
    fn usage(&self, pid: u32) -> Result<ProcessUsage, String>;
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
