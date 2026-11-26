//! The services of the bench. They work only against ports, so they are tested
//! with fakes and without touching the system.

#![forbid(unsafe_code)]

pub mod command;
pub mod emit;
pub mod footprint;
pub mod footprint_report;
pub mod startup;
#[cfg(test)]
mod testing;
pub mod window_session;

pub use command::{Adapters, Command, Output};
pub use emit::EmitService;
pub use footprint::FootprintService;
pub use startup::StartupService;
