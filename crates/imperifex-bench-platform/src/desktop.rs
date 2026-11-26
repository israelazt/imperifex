//! The windows of Windows. It does as it is told: what may be touched is
//! decided by the services, with the `SafetyGuard`.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use imperifex_bench_domain::desktop::{Rect, WindowEntry};
use imperifex_bench_domain::ports::Desktop;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, PostMessageW, SetForegroundWindow, SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
    WM_CLOSE,
};

use crate::decode::utf16;

/// How long the foreground has to move to a window.
const TO_FOCUS: Duration = Duration::from_millis(500);

fn handle(hwnd: u64) -> HWND {
    HWND(hwnd as usize as *mut core::ffi::c_void)
}

fn number(hwnd: HWND) -> u64 {
    hwnd.0 as usize as u64
}

/// # Safety
/// `list` must be the `Vec<HWND>` that [`top_level`] passed, alive and not
/// shared during the call.
unsafe extern "system" fn collect(hwnd: HWND, list: LPARAM) -> BOOL {
    // SAFETY: the contract of this function.
    let found = unsafe { &mut *(list.0 as *mut Vec<HWND>) };
    found.push(hwnd);
    true.into()
}

fn top_level() -> Result<Vec<HWND>, String> {
    let mut found: Vec<HWND> = Vec::new();
    let list = LPARAM(std::ptr::from_mut(&mut found) as isize);
    // SAFETY: the callback only runs during this call, while `found` is alive.
    unsafe { EnumWindows(Some(collect), list) }
        .map_err(|error| format!("EnumWindows falló: {error}"))?;
    Ok(found)
}

/// The frame the user sees; empty if the system will not say.
fn frame(window: HWND) -> Rect {
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
        Ok(()) => Rect {
            x: area.left,
            y: area.top,
            w: area.right - area.left,
            h: area.bottom - area.top,
        },
        Err(_) => Rect::default(),
    }
}

/// A window that goes away while it is read comes out with empty values.
fn describe(window: HWND) -> WindowEntry {
    let mut pid = 0;
    // SAFETY: the out pointer is to a local; a window that is gone leaves it 0.
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    // SAFETY: a window that is gone is answered with false.
    let visible = unsafe { IsWindowVisible(window) }.as_bool();
    let mut title = [0u16; 256];
    // SAFETY: the buffer's length goes with it. For a window of another process
    // this reads the stored caption, without asking the window.
    unsafe { GetWindowTextW(window, &mut title) };
    let mut class = [0u16; 256];
    // SAFETY: the buffer's length goes with it.
    unsafe { GetClassNameW(window, &mut class) };
    WindowEntry {
        hwnd: number(window),
        pid,
        title: utf16(&title),
        class: utf16(&class),
        visible,
        rect: frame(window),
    }
}

fn foreground_window() -> Option<u64> {
    // SAFETY: no arguments; it may answer with no window.
    let window = unsafe { GetForegroundWindow() };
    (!window.is_invalid()).then(|| number(window))
}

/// The real desktop.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsDesktop;

impl Desktop for WindowsDesktop {
    fn launch(&self, exe: &Path, args: &[String], dir: &Path) -> Result<u32, String> {
        // With no handles in common, what is launched cannot hold the bench's
        // output open after it ends.
        Command::new(exe)
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|child| child.id())
            .map_err(|error| format!("no se pudo abrir {}: {error}", exe.display()))
    }

    fn windows(&self) -> Result<Vec<WindowEntry>, String> {
        Ok(top_level()?.into_iter().map(describe).collect())
    }

    fn place(&self, hwnd: u64, rect: Rect) -> Result<(), String> {
        // SAFETY: no pointers; a window that is gone is answered with an error.
        unsafe {
            SetWindowPos(
                handle(hwnd),
                None,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        }
        .map_err(|error| format!("no se pudo colocar la ventana {hwnd:#x}: {error}"))
    }

    /// `SetForegroundWindow` alone, which was enough in probe 1.4; the result is
    /// read back, as the system may refuse without saying why.
    fn focus(&self, hwnd: u64) -> Result<(), String> {
        // SAFETY: no pointers; the system refuses what it does not allow.
        let _ = unsafe { SetForegroundWindow(handle(hwnd)) };
        let start = Instant::now();
        while foreground_window() != Some(hwnd) {
            if start.elapsed() >= TO_FOCUS {
                return Err(format!(
                    "la ventana {hwnd:#x} no llegó al primer plano: no se sigue"
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    fn foreground(&self) -> Result<Option<u64>, String> {
        Ok(foreground_window())
    }

    fn close(&self, hwnd: u64) -> Result<(), String> {
        // SAFETY: a posted message with no pointers.
        unsafe { PostMessageW(Some(handle(hwnd)), WM_CLOSE, WPARAM(0), LPARAM(0)) }
            .map_err(|error| format!("no se pudo pedir el cierre de {hwnd:#x}: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only reading is tested here: the rest touches windows.
    #[test]
    fn a_window_that_is_not_there_cannot_be_placed_or_closed() {
        let nobody = u64::from(u32::MAX - 7);
        assert!(WindowsDesktop.place(nobody, Rect::default()).is_err());
        assert!(WindowsDesktop.close(nobody).is_err());
    }

    /// Reading changes nothing. A hosted runner may have no windows to read.
    #[test]
    fn the_windows_of_the_desktop_are_read_with_their_class() {
        if std::env::var_os("IMPERIFEX_NO_HARDWARE").is_some() {
            return;
        }
        let all = WindowsDesktop.windows().unwrap();
        assert!(
            all.iter().any(|w| !w.class.is_empty()),
            "{} read",
            all.len()
        );
        assert!(all.iter().any(|w| !w.visible), "hidden ones too");
    }

    /// `cargo test --release -p imperifex-bench-platform measure_reading_the_windows -- --ignored --nocapture`
    #[test]
    #[ignore = "a measurement, run by hand on a real desktop"]
    fn measure_reading_the_windows() {
        for round in 0..10 {
            let start = Instant::now();
            let all = WindowsDesktop.windows().unwrap();
            let took = start.elapsed();
            let shown = all.iter().filter(|w| w.visible).count();
            println!(
                "{round}: {} windows ({shown} visible) in {took:?}",
                all.len()
            );
        }
    }
}
