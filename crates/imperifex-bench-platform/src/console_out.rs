//! The standard output as the terminal being measured gets it, with the
//! console reading escape sequences while it is held.

use std::io::{StdoutLock, Write};

use imperifex_bench_domain::ports::Output;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    STD_OUTPUT_HANDLE,
};

/// Puts the console's mode back as it was when dropped.
struct Mode {
    console: HANDLE,
    previous: CONSOLE_MODE,
}

impl Mode {
    fn with_escape_sequences() -> Result<Self, String> {
        // SAFETY: no pointers; the handle belongs to the process and is not closed.
        let console = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }
            .map_err(|error| format!("GetStdHandle falló: {error}"))?;
        let mut previous = CONSOLE_MODE::default();
        // SAFETY: the out pointer is to a local that lives through the call.
        unsafe { GetConsoleMode(console, &mut previous) }.map_err(|error| {
            format!("la salida no es una consola; corre la orden dentro de la terminal que mides, sin redirigirla ({error})")
        })?;
        // SAFETY: a console handle, checked above, and a combination of its flags.
        unsafe { SetConsoleMode(console, previous | ENABLE_VIRTUAL_TERMINAL_PROCESSING) }
            .map_err(|error| format!("SetConsoleMode falló: {error}"))?;
        Ok(Self { console, previous })
    }
}

impl Drop for Mode {
    fn drop(&mut self) {
        // SAFETY: the same handle, and the mode it had.
        let _ = unsafe { SetConsoleMode(self.console, self.previous) };
    }
}

/// The program's standard output, when it is a console.
pub struct ConsoleOut {
    out: StdoutLock<'static>,
    _mode: Mode,
}

impl ConsoleOut {
    /// It fails if the output is not a console: a speed measured against a file
    /// or a pipe would say nothing about a terminal.
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            _mode: Mode::with_escape_sequences()?,
            out: std::io::stdout().lock(),
        })
    }
}

impl Output for ConsoleOut {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.out
            .write_all(bytes)
            .map_err(|error| format!("no se pudo escribir en la consola: {error}"))
    }

    fn flush(&mut self) -> Result<(), String> {
        self.out
            .flush()
            .map_err(|error| format!("no se pudo vaciar la salida: {error}"))
    }
}
