//! Taking the foreground and typing, only on windows the probe opened.

use std::collections::HashSet;
use std::time::Duration;

use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::ports::Keyboard;
use imperifex_bench_platform::WindowsKeyboard;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
};

/// How long the system is given to move the foreground.
const SETTLE: Duration = Duration::from_millis(300);

fn handle(hwnd: u64) -> HWND {
    HWND(hwnd as usize as *mut core::ffi::c_void)
}

pub fn foreground() -> u64 {
    // SAFETY: no arguments; it may answer with no window.
    unsafe { GetForegroundWindow() }.0 as usize as u64
}

fn thread_of(hwnd: u64) -> u32 {
    // SAFETY: a window that is gone is answered with 0.
    unsafe { GetWindowThreadProcessId(handle(hwnd), None) }
}

fn set_foreground(hwnd: u64) {
    // SAFETY: the system refuses what it does not allow; the result is read back.
    let accepted = unsafe { SetForegroundWindow(handle(hwnd)) }.as_bool();
    println!("  SetForegroundWindow respondió {accepted}");
}

fn arrived(hwnd: u64) -> bool {
    std::thread::sleep(SETTLE);
    foreground() == hwnd
}

fn attach(from: u32, to: u32, on: bool) -> bool {
    // SAFETY: two thread ids; a failure is answered with false.
    unsafe { AttachThreadInput(from, to, on) }.as_bool()
}

fn with_attached_threads(hwnd: u64) {
    let current = foreground();
    let (holder, target) = (thread_of(current), thread_of(hwnd));
    println!("  hilo en primer plano {holder}, hilo de la ventana pedida {target}");
    // Two windows of one thread cannot be attached to each other: then it is
    // this probe's thread that joins the one in the foreground.
    let from = if holder == target {
        println!("  es el mismo hilo: se une el hilo de la sonda al de primer plano");
        // SAFETY: no arguments.
        unsafe { GetCurrentThreadId() }
    } else {
        target
    };
    println!(
        "  AttachThreadInput respondió {}",
        attach(from, holder, true)
    );
    set_foreground(hwnd);
    attach(from, holder, false);
}

fn key(code: VIRTUAL_KEY, unicode: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: code,
                wScan: unicode,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(events: &[INPUT]) -> Result<(), String> {
    // SAFETY: the slice and the size of one element describe the events.
    let sent = unsafe { SendInput(events, size_of::<INPUT>() as i32) };
    if sent as usize == events.len() {
        Ok(())
    } else {
        Err(format!(
            "SendInput mandó {sent} de {} eventos",
            events.len()
        ))
    }
}

/// The adapter itself refuses unless `target` is in the foreground (S3).
pub fn type_char(target: u64, c: char) -> Result<(), String> {
    WindowsKeyboard.press(target, Key::Char(c))
}

pub fn press_backspace(target: u64) -> Result<(), String> {
    WindowsKeyboard.press(target, Key::Backspace)
}

/// `Alt` goes to whatever is in the foreground, so it is pressed only when that
/// is one of the probe's own windows.
fn with_alt(hwnd: u64, own: &HashSet<u64>) -> Result<(), String> {
    let current = foreground();
    if !own.contains(&current) {
        return Err("el primer plano no es una ventana propia: no se pulsa Alt".into());
    }
    send(&[
        key(VK_MENU, 0, KEYBD_EVENT_FLAGS(0)),
        key(VK_MENU, 0, KEYEVENTF_KEYUP),
    ])?;
    set_foreground(hwnd);
    Ok(())
}

/// Tries the three ways in order and names the first that worked.
pub fn take_foreground(hwnd: u64, own: &HashSet<u64>) -> Result<&'static str, String> {
    println!("método 1: SetForegroundWindow");
    set_foreground(hwnd);
    if arrived(hwnd) {
        return Ok("SetForegroundWindow solo");
    }
    println!("método 2: AttachThreadInput");
    with_attached_threads(hwnd);
    if arrived(hwnd) {
        return Ok("AttachThreadInput y SetForegroundWindow");
    }
    println!("método 3: Alt y SetForegroundWindow");
    with_alt(hwnd, own)?;
    if arrived(hwnd) {
        return Ok("Alt y SetForegroundWindow");
    }
    Err("ningún método consiguió el primer plano".into())
}
