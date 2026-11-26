//! Typing with `SendInput`. Keys go to whatever is in the foreground, so each
//! batch is sent only while that is the window it is meant for (rule S3).

use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::ports::Keyboard;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use crate::key_events::{of_key, of_text, Code, KeyEvent};

fn input(event: KeyEvent) -> INPUT {
    let (code, scan, kind) = match event.code {
        Code::Unicode(unit) => (0, unit, KEYEVENTF_UNICODE),
        Code::Virtual(code) => (code, 0, KEYBD_EVENT_FLAGS(0)),
    };
    let flags = if event.up {
        kind | KEYEVENTF_KEYUP
    } else {
        kind
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(code),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn foreground() -> u64 {
    // SAFETY: no arguments; it may answer with no window.
    unsafe { GetForegroundWindow() }.0 as usize as u64
}

fn send(hwnd: u64, events: &[KeyEvent]) -> Result<(), String> {
    let inputs: Vec<INPUT> = events.iter().copied().map(input).collect();
    if foreground() != hwnd {
        return Err(format!(
            "la ventana {hwnd:#x} no está en primer plano: no se pulsa nada (regla S3)"
        ));
    }
    // SAFETY: the slice and the size of one element describe the events.
    let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(format!(
            "SendInput mandó {sent} de {} eventos",
            inputs.len()
        ))
    }
}

/// The real keyboard.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsKeyboard;

impl Keyboard for WindowsKeyboard {
    fn type_text(&self, hwnd: u64, text: &str) -> Result<(), String> {
        send(hwnd, &of_text(text))
    }

    fn press(&self, hwnd: u64, key: Key) -> Result<(), String> {
        send(hwnd, &of_key(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Safe to run anywhere: no window has this number, so nothing is sent.
    #[test]
    fn nothing_is_typed_into_a_window_that_is_not_in_the_foreground() {
        let nobody = u64::from(u32::MAX - 7);
        for result in [
            WindowsKeyboard.type_text(nobody, "x"),
            WindowsKeyboard.press(nobody, Key::Enter),
        ] {
            assert!(result.unwrap_err().contains("S3"));
        }
    }
}
