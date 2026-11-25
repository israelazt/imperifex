//! The adapters of the bench: they implement its ports. Calls to the system
//! live in `#[cfg(windows)]` modules; decoding what it returns is plain code.

#[cfg(windows)]
mod console_out;
mod files;

#[cfg(windows)]
pub use console_out::ConsoleOut;
pub use files::DiskFiles;
