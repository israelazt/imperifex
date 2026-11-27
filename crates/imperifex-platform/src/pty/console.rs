//! The pseudoconsole itself: its pipes, its size and starting a program in it.

use std::ffi::c_void;
use std::fs::File;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows::Win32::System::Pipes::CreatePipe;

use super::process::{self, Process};

/// The console, closed once: by `close` or when dropped.
pub(super) struct Console(Option<HPCON>);

/// A pipe as (read end, write end).
fn pipe() -> Result<(OwnedHandle, OwnedHandle), String> {
    let (mut read, mut write) = (HANDLE::default(), HANDLE::default());
    // SAFETY: both pointers are to handles that live through the call.
    unsafe { CreatePipe(&mut read, &mut write, None, 0) }
        .map_err(|e| format!("CreatePipe: {e}"))?;
    // SAFETY: the call gave this handle to us, and nobody else closes it.
    let read = unsafe { OwnedHandle::from_raw_handle(read.0) };
    // SAFETY: the same, for the other end.
    let write = unsafe { OwnedHandle::from_raw_handle(write.0) };
    Ok((read, write))
}

fn raw(handle: &OwnedHandle) -> HANDLE {
    HANDLE(handle.as_raw_handle())
}

/// A size as the console takes it, with each side at least 1.
fn size(columns: u16, rows: u16) -> COORD {
    let side = |value: u16| i16::try_from(value.max(1)).unwrap_or(i16::MAX);
    COORD {
        X: side(columns),
        Y: side(rows),
    }
}

fn close_handle(console: isize) {
    // SAFETY: the number is a console taken out of its owner, so it is closed
    // only once.
    unsafe { ClosePseudoConsole(HPCON(console)) };
}

impl Console {
    /// The console, the pipe to type into and the pipe its output comes from.
    pub fn open(columns: u16, rows: u16) -> Result<(Self, File, File), String> {
        let (console_input, our_input) = pipe()?;
        let (our_output, console_output) = pipe()?;
        // SAFETY: both handles are open pipe ends of the right direction.
        let console = unsafe {
            CreatePseudoConsole(
                size(columns, rows),
                raw(&console_input),
                raw(&console_output),
                0,
            )
        }
        .map_err(|e| format!("CreatePseudoConsole: {e}"))?;
        // The console keeps its own copies of its ends.
        drop((console_input, console_output));
        Ok((
            Self(Some(console)),
            File::from(our_input),
            File::from(our_output),
        ))
    }

    pub fn launch(&self, command_line: &str) -> Result<Process, String> {
        let console = self.0.ok_or("la consola ya está cerrada")?;
        process::launch(console.0 as *const c_void, command_line)
    }

    pub fn resize(&self, columns: u16, rows: u16) -> Result<(), String> {
        let console = self.0.ok_or("la consola ya está cerrada")?;
        // SAFETY: the console is open: it is only closed by taking it.
        unsafe { ResizePseudoConsole(console, size(columns, rows)) }
            .map_err(|e| format!("ResizePseudoConsole: {e}"))
    }

    /// Closes it on another thread. Some versions of Windows do not return from
    /// closing until the output has been read, and the caller is who reads it.
    pub fn close(&mut self) {
        if let Some(console) = self.0.take() {
            let number = console.0;
            let spawned = std::thread::Builder::new()
                .name("conpty-close".into())
                .spawn(move || close_handle(number));
            if let Err(error) = spawned {
                imperifex_domain::log_warn!("la consola se cierra en el mismo hilo: {error}");
                close_handle(number);
            }
        }
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        self.close();
    }
}
