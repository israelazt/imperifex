//! The log of the program, to the standard error stream (a file on disk,
//! `%APPDATA%\Imperifex\imperifex.log`, comes when the app needs it).

use std::io::Write;

use imperifex_domain::log::{set_sink, Level, LogSink};

struct Stderr;

impl LogSink for Stderr {
    fn write(&self, level: Level, message: &str) {
        let _ = writeln!(std::io::stderr(), "{:<5} {message}", level.label());
    }
}

/// Sends the log to the standard error stream.
pub fn install_stderr_log() {
    set_sink(Box::new(Stderr));
}
