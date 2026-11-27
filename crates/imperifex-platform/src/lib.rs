//! The adapters of Imperifex: they implement the ports of `imperifex-domain`.
//! Nothing here knows the application. Calls to the system live in
//! `#[cfg(windows)]` modules; decoding what the system returns is plain code,
//! tested everywhere.

mod clock;
mod log_console;
mod parser;
#[cfg(windows)]
mod pty;

pub use clock::SystemClock;
pub use log_console::install_stderr_log;
pub use parser::VteParser;
#[cfg(windows)]
pub use pty::ConPtyLauncher;
