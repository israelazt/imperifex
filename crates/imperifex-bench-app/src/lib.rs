//! The services of the bench. They work only against ports, so they are tested
//! with fakes and without touching the system.

#![forbid(unsafe_code)]

pub mod command;
pub mod echo;
pub mod emit;
pub mod footprint;
pub mod latency;
pub mod measure_window;
pub mod memory_load;
pub mod output_seen;
pub mod reports;
pub mod screen_wait;
pub mod startup;
#[cfg(test)]
mod testing;
mod usage_reading;
pub mod window_session;
pub mod workdir;

pub use command::{Adapters, Command, Output};
pub use emit::EmitService;
pub use footprint::FootprintService;
pub use startup::StartupService;
