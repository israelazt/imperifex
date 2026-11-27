//! Probe T1.1: what ConPTY gives back, and whether the OSC sequences the
//! blocks and the notices need arrive whole and in order. It opens no window.
//!
//! `cargo run --release -p imperifex-platform --example probe_conpty`

#[cfg(windows)]
#[path = "probe_conpty/pty.rs"]
mod pty;

#[cfg(windows)]
#[path = "probe_conpty/cases.rs"]
mod cases;

#[cfg(windows)]
fn main() -> Result<(), String> {
    cases::run()
}

#[cfg(not(windows))]
fn main() {
    println!("ConPTY solo existe en Windows");
}
