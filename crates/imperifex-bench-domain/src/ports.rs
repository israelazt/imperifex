//! What the bench needs from the system, said by what it needs. The clock is
//! the terminal's own port, `imperifex_domain::ports::Clock`. The real adapters
//! live in `imperifex-bench-platform`; the tests use fakes.

use std::path::{Path, PathBuf};

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
