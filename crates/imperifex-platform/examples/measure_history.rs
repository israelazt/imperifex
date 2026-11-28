//! T1.5: what a history of a million lines costs, in memory and time, going the
//! whole way: bytes, parser, interpreter and grid. It opens no window.
//!
//! `cargo run --release -p imperifex-platform --example measure_history -- <case>`
//! with `plain` or `colour`, and `-default` after it for the history a terminal
//! keeps by default (T2.9). One case per run, so that the peak is its own.

#[cfg(windows)]
#[path = "measure_history/memory.rs"]
mod memory;

#[cfg(windows)]
#[path = "measure_history/run.rs"]
mod run;

#[cfg(windows)]
fn main() -> Result<(), String> {
    run::run(&std::env::args().nth(1).unwrap_or_default())
}

#[cfg(not(windows))]
fn main() {
    println!("esta medida solo está hecha para Windows");
}
