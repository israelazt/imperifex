//! The standard input of the program, key by key: no line editing, no echo of
//! its own, and `Ctrl+C` arriving as one more byte.

use std::io::Read;

use imperifex_bench_domain::ports::KeyInput;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_ECHO_INPUT,
    ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT, STD_INPUT_HANDLE,
};

/// Puts the console's input mode back as it was when dropped.
pub struct ConsoleIn {
    console: HANDLE,
    previous: CONSOLE_MODE,
}

impl ConsoleIn {
    pub fn new() -> Result<Self, String> {
        // SAFETY: no pointers; the handle belongs to the process and is not closed.
        let console = unsafe { GetStdHandle(STD_INPUT_HANDLE) }
            .map_err(|error| format!("GetStdHandle falló: {error}"))?;
        let mut previous = CONSOLE_MODE::default();
        // SAFETY: the out pointer is to a local that lives through the call.
        unsafe { GetConsoleMode(console, &mut previous) }.map_err(|error| {
            format!("la entrada no es una consola; corre la orden en una terminal ({error})")
        })?;
        let raw = (previous & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT))
            | ENABLE_VIRTUAL_TERMINAL_INPUT;
        // SAFETY: a console handle, checked above, and a combination of its flags.
        unsafe { SetConsoleMode(console, raw) }
            .map_err(|error| format!("SetConsoleMode falló: {error}"))?;
        Ok(Self { console, previous })
    }
}

impl Drop for ConsoleIn {
    fn drop(&mut self) {
        // SAFETY: the same handle, and the mode it had.
        let _ = unsafe { SetConsoleMode(self.console, self.previous) };
    }
}

impl KeyInput for ConsoleIn {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        std::io::stdin()
            .lock()
            .read(buf)
            .map_err(|error| format!("no se pudo leer la entrada: {error}"))
    }
}
