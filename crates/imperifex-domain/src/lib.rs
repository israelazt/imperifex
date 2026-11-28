//! The rules and the data of Imperifex. No Windows, no threads, no disk, no
//! network: the system is reached only through the traits in [`ports`].
//! See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

pub mod command_line;
pub mod example;
pub mod grid;
pub mod i18n;
pub mod interpret;
pub mod log;
pub mod ports;
pub mod sequence;
pub mod shell;
pub mod terminal;
