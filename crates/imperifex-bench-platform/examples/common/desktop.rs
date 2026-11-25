//! The top level windows of the desktop, and asking one to close.

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone)]
pub struct Window {
    pub hwnd: isize,
    pub pid: u32,
    pub visible: bool,
    pub title: String,
    pub class: String,
    pub width: i32,
    pub height: i32,
}

pub fn handle(hwnd: isize) -> HWND {
    HWND(hwnd as *mut core::ffi::c_void)
}

/// # Safety
/// `list` must be the `Vec<isize>` that [`windows`] passed, alive and unshared.
unsafe extern "system" fn collect(hwnd: HWND, list: LPARAM) -> BOOL {
    // SAFETY: the contract of this function.
    let found = unsafe { &mut *(list.0 as *mut Vec<isize>) };
    found.push(hwnd.0 as isize);
    true.into()
}

/// Every top level window, hidden ones too. Reading them changes nothing.
pub fn windows() -> Result<Vec<Window>, String> {
    let mut found: Vec<isize> = Vec::new();
    let list = LPARAM(std::ptr::from_mut(&mut found) as isize);
    // SAFETY: the callback only runs during this call, while `found` is alive.
    unsafe { EnumWindows(Some(collect), list) }
        .map_err(|error| format!("EnumWindows falló: {error}"))?;
    Ok(found.into_iter().map(describe).collect())
}

fn describe(hwnd: isize) -> Window {
    let window = handle(hwnd);
    let mut pid = 0;
    // SAFETY: a window that is gone is answered with 0.
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    // SAFETY: a window that is gone is answered with false.
    let visible = unsafe { IsWindowVisible(window) }.as_bool();
    let mut title = [0u16; 256];
    // SAFETY: the buffer's length goes with it.
    let title_len = unsafe { GetWindowTextW(window, &mut title) };
    let mut class = [0u16; 256];
    // SAFETY: the buffer's length goes with it.
    let class_len = unsafe { GetClassNameW(window, &mut class) };
    let (width, height) = size(window);
    Window {
        hwnd,
        pid,
        visible,
        title: String::from_utf16_lossy(&title[..usize::try_from(title_len).unwrap_or(0)]),
        class: String::from_utf16_lossy(&class[..usize::try_from(class_len).unwrap_or(0)]),
        width,
        height,
    }
}

/// The size of the frame the user sees; zero if the window will not say.
fn size(window: HWND) -> (i32, i32) {
    let mut area = RECT::default();
    // SAFETY: the pointer and the size describe the same local rectangle.
    let read = unsafe {
        DwmGetWindowAttribute(
            window,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            std::ptr::from_mut(&mut area).cast(),
            size_of::<RECT>() as u32,
        )
    };
    match read {
        Ok(()) => (area.right - area.left, area.bottom - area.top),
        Err(_) => (0, 0),
    }
}

/// Asks a window to close, as its close button does. The caller must have
/// checked that the window is one it opened (S1).
pub fn ask_to_close(hwnd: isize) -> Result<(), String> {
    // SAFETY: a posted message with no pointers.
    unsafe { PostMessageW(Some(handle(hwnd)), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        .map_err(|error| format!("PostMessageW falló: {error}"))
}
