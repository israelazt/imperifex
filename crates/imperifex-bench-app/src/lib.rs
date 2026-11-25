//! The services of the bench. They work only against ports, so they are tested
//! with fakes and without touching the system.

#![forbid(unsafe_code)]

pub mod command;
pub mod emit;
pub mod footprint;
#[cfg(test)]
mod testing;

pub use command::{Command, Output};
pub use emit::EmitService;
pub use footprint::FootprintService;
