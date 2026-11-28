//! The adapters of Imperifex: they implement the ports of `imperifex-domain`.
//! Nothing here knows the application. Calls to the system live in
//! `#[cfg(windows)]` modules; decoding what the system returns is plain code,
//! tested everywhere.

#[cfg(windows)]
mod appearance;
mod clock;
mod language;
mod log_console;
mod parser;
#[cfg(windows)]
mod pty;
mod settings_file;

#[cfg(windows)]
pub use appearance::WindowsAppearance;
pub use clock::SystemClock;
#[cfg(windows)]
pub use language::WindowsLanguage;
pub use log_console::install_stderr_log;
pub use parser::VteParser;
#[cfg(windows)]
pub use pty::ConPtyLauncher;
pub use settings_file::FileSettingsStore;
