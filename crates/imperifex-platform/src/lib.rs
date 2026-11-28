//! The adapters of Imperifex: they implement the ports of `imperifex-domain`.
//! Nothing here knows the application. Calls to the system live in
//! `#[cfg(windows)]` modules; decoding what the system returns is plain code,
//! tested everywhere.

#[cfg(windows)]
mod appearance;
#[cfg(windows)]
mod clipboard;
mod clock;
mod folders;
mod history_file;
#[cfg(any(windows, test))]
mod instances;
mod language;
mod log_console;
mod parser;
#[cfg(windows)]
mod pty;
#[cfg(windows)]
mod render;
mod settings_file;
#[cfg(windows)]
mod text;

#[cfg(windows)]
pub use appearance::WindowsAppearance;
#[cfg(windows)]
pub use clipboard::WindowsClipboard;
pub use clock::SystemClock;
pub use folders::DiskFolders;
pub use history_file::FileCommandHistory;
#[cfg(windows)]
pub use language::WindowsLanguage;
pub use log_console::install_stderr_log;
pub use parser::VteParser;
#[cfg(windows)]
pub use pty::ConPtyLauncher;
#[cfg(windows)]
pub use render::D3dRenderer;
pub use settings_file::FileSettingsStore;
#[cfg(windows)]
pub use text::DirectWriteText;
