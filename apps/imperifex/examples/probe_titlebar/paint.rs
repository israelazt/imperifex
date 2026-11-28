//! Plain colours with GDI, only so that the probe's window can be seen: the
//! background, the strip that acts as the title bar and the slot where the
//! maximize button would be, with a square in it.

use windows::Win32::Foundation::{COLORREF, HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, DeleteObject, FillRect, FrameRect, GetDC, ReleaseDC, HDC,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use super::zones::{maximize_slot, BAR};

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

pub fn paint(window: &Window) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut _);
    let size = window.inner_size();
    let (width, height) = (size.width as i32, size.height as i32);
    let scale = window.scale_factor();
    let bar = (BAR * scale).round() as i32;
    let (left, right) = maximize_slot(f64::from(size.width), scale);
    let (left, right) = (left as i32, right as i32);
    let icon = (5.0 * scale).round() as i32;
    let (middle_x, middle_y) = ((left + right) / 2, bar / 2);

    // SAFETY: the handle is this window's, alive while `window` is.
    let dc = unsafe { GetDC(Some(hwnd)) };
    fill(dc, rect(0, 0, width, height), rgb(32, 32, 36));
    fill(dc, rect(0, 0, width, bar), rgb(40, 90, 140));
    fill(dc, rect(left, 0, right, bar), rgb(220, 140, 30));
    let square = rect(
        middle_x - icon,
        middle_y - icon,
        middle_x + icon,
        middle_y + icon,
    );
    outline(dc, square, rgb(0, 0, 0));
    // SAFETY: the context came from `GetDC` on this window.
    unsafe { ReleaseDC(Some(hwnd), dc) };
}

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

fn fill(dc: HDC, area: RECT, colour: COLORREF) {
    // SAFETY: nothing is read from memory.
    let brush = unsafe { CreateSolidBrush(colour) };
    // SAFETY: a live context and brush, and `area` lives through the call.
    unsafe { FillRect(dc, &area, brush) };
    // SAFETY: the brush made above, no longer used.
    let _ = unsafe { DeleteObject(brush.into()) };
}

fn outline(dc: HDC, area: RECT, colour: COLORREF) {
    // SAFETY: nothing is read from memory.
    let brush = unsafe { CreateSolidBrush(colour) };
    // SAFETY: a live context and brush, and `area` lives through the call.
    unsafe { FrameRect(dc, &area, brush) };
    // SAFETY: the brush made above, no longer used.
    let _ = unsafe { DeleteObject(brush.into()) };
}
