//! T1.6: whether the console and our grid give a text the same columns, and
//! whether the line drawings of `ESC ( 0` reach us drawn. It opens no window.
//!
//! `cargo run --release -p imperifex-platform --example probe_widths`

#[cfg(windows)]
#[path = "probe_widths/run.rs"]
mod run;

#[cfg(windows)]
fn main() -> Result<(), String> {
    run::run()
}

#[cfg(not(windows))]
fn main() {
    println!("ConPTY solo existe en Windows");
}
