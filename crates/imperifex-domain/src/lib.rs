//! The rules and the data of Imperifex. No Windows, no threads, no disk, no
//! network: the system is reached only through the traits in [`ports`].
//! See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

pub mod example;
pub mod grid;
pub mod interpret;
pub mod log;
pub mod ports;
pub mod sequence;
