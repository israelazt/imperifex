//! Throwaway probe (T2.6): what winit allows for a title bar of our own. It
//! opens one window with no system title bar and paints only plain colours: a
//! blue strip, the top 40 logical pixels, that acts as the bar: drag to move, double click to maximize or
//! restore, right click for the window's menu. The 8 logical pixels of each
//! edge resize. The orange slot, the second one of 46 pixels from the right of
//! the bar, is where the maximize button would be: a click there toggles it.
//! Esc closes.
//!
//! It prints what the window is told, with the time since it opened.
//!
//! `cargo run -p imperifex --example probe_titlebar`

#[cfg(windows)]
#[path = "probe_titlebar/window.rs"]
mod window;

#[cfg(windows)]
fn main() -> Result<(), winit::error::EventLoopError> {
    window::run()
}

#[cfg(not(windows))]
fn main() {
    println!("solo en Windows");
}
