//! Telling Windows that the program works in physical pixels.

use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

/// Makes every coordinate a physical one, on every monitor. It has to be the
/// first thing the program does.
pub fn use_physical_pixels() -> Result<(), String> {
    // SAFETY: one of the system's own constants.
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
        .map_err(|error| format!("SetProcessDpiAwarenessContext falló: {error}"))
}
