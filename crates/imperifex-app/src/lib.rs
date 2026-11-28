//! The services of Imperifex. They work only against the ports of
//! `imperifex-domain`, so they are tested with fakes and without touching the
//! system. See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

pub mod command;
pub mod headless;
pub mod session;
pub mod settings;
pub mod shell_tab;
pub mod status;
#[cfg(test)]
mod testing;
pub mod window;

pub use command::{Command, Output, Shells};
pub use status::StatusService;
