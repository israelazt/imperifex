//! The services of Imperifex. They work only against the ports of
//! `imperifex-domain`, so they are tested with fakes and without touching the
//! system. See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

pub mod command;
pub mod status;
#[cfg(test)]
mod testing;

pub use command::{Command, Output};
pub use status::StatusService;
