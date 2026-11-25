//! The rules and the data of the bench that measures terminals. No system, no
//! disk, no clock: everything outside is reached through ports.

#![forbid(unsafe_code)]

pub mod change;
pub mod desktop;
pub mod emit;
pub mod loads;
pub mod markdown;
pub mod metric;
pub mod ports;
pub mod process;
pub mod profile;
pub mod report;
pub mod safety;
pub mod settle;
pub mod stats;
pub mod tree;
pub mod usage;
pub mod windows;
