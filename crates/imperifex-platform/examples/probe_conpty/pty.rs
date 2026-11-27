//! A program running inside a pseudoconsole, for the probe only.

use std::ffi::c_void;
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use windows::core::PWSTR;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, WaitForSingleObject, EXTENDED_STARTUPINFO_PRESENT,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
    STARTF_USESTDHANDLES, STARTUPINFOEXW,
};

pub struct Pty {
    console: Option<HPCON>,
    input: File,
    seen: Arc<Mutex<Vec<u8>>>,
    reader: Option<JoinHandle<()>>,
    process: OwnedHandle,
    pub pid: u32,
}

/// What closing the console did.
pub struct Closed {
    /// How long the program took to leave; `None` if it stayed 5 s.
    pub left_in: Option<Duration>,
    pub reading_ended: bool,
}

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

fn size(columns: i16, rows: i16) -> COORD {
    COORD {
        X: columns,
        Y: rows,
    }
}

/// Starts `command` attached to `console`.
fn launch(console: HPCON, command: &str) -> Result<(OwnedHandle, u32), String> {
    let mut bytes = 0usize;
    // SAFETY: with no list, the call only writes the size it needs.
    let _ = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut bytes) };
    let mut buffer = vec![0usize; bytes.div_ceil(size_of::<usize>())];
    let list = LPPROC_THREAD_ATTRIBUTE_LIST(buffer.as_mut_ptr().cast());
    // SAFETY: the buffer is at least as long as the size asked, and aligned.
    unsafe { InitializeProcThreadAttributeList(Some(list), 1, None, &mut bytes) }
        .map_err(|e| format!("InitializeProcThreadAttributeList: {e}"))?;
    let started = attach_and_create(list, console, command);
    // SAFETY: the list was initialized above and is not used after this.
    unsafe { DeleteProcThreadAttributeList(list) };
    started
}

fn attach_and_create(
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
    console: HPCON,
    command: &str,
) -> Result<(OwnedHandle, u32), String> {
    let value = console.0 as *const c_void;
    let attribute = PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize;
    // SAFETY: for this attribute the value is the console handle itself, and
    // the console outlives the process creation.
    unsafe {
        UpdateProcThreadAttribute(
            list,
            0,
            attribute,
            Some(value),
            size_of::<HPCON>(),
            None,
            None,
        )
    }
    .map_err(|e| format!("UpdateProcThreadAttribute: {e}"))?;

    let mut startup = STARTUPINFOEXW {
        lpAttributeList: list,
        ..Default::default()
    };
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    // Empty standard handles: the program takes those of its console, not ours.
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    let mut line: Vec<u16> = command.encode_utf16().chain([0]).collect();
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: the command line is writable and ends in zero; the startup data
    // is the extended one, as the flag and `cb` say.
    unsafe {
        CreateProcessW(
            None,
            Some(PWSTR(line.as_mut_ptr())),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            None,
            &startup.StartupInfo,
            &mut info,
        )
    }
    .map_err(|e| format!("CreateProcessW({command}): {e}"))?;
    // SAFETY: the call gave us the thread handle; dropping it closes it once.
    drop(unsafe { OwnedHandle::from_raw_handle(info.hThread.0) });
    // SAFETY: the same, for the process handle.
    let process = unsafe { OwnedHandle::from_raw_handle(info.hProcess.0) };
    Ok((process, info.dwProcessId))
}

impl Pty {
    pub fn start(command: &str, columns: i16, rows: i16) -> Result<Self, String> {
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
        let seen = Arc::new(Mutex::new(Vec::new()));
        // Reading starts before the program: a full pipe would stop the console.
        let reader = read_into(File::from(our_output), Arc::clone(&seen));
        let (process, pid) = launch(console, command)?;
        Ok(Self {
            console: Some(console),
            input: File::from(our_input),
            seen,
            reader: Some(reader),
            process,
            pid,
        })
    }

    pub fn output(&self) -> Vec<u8> {
        self.seen.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn send(&mut self, text: &str) -> Result<(), String> {
        self.input
            .write_all(text.as_bytes())
            .map_err(|e| format!("escribir en la consola: {e}"))
    }

    /// Waits until `text` is in the output; `false` after `limit`.
    pub fn wait_for(&self, text: &str, limit: Duration) -> bool {
        let start = Instant::now();
        loop {
            if String::from_utf8_lossy(&self.output()).contains(text) {
                return true;
            }
            if start.elapsed() > limit {
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Waits until nothing new arrives for `quiet`, or `limit` passes.
    pub fn wait_quiet(&self, quiet: Duration, limit: Duration) {
        let start = Instant::now();
        let mut last = (self.output().len(), Instant::now());
        while start.elapsed() < limit && (last.0 == 0 || last.1.elapsed() < quiet) {
            std::thread::sleep(Duration::from_millis(20));
            let length = self.output().len();
            if length != last.0 {
                last = (length, Instant::now());
            }
        }
    }

    pub fn has_left_within(&self, limit: Duration) -> bool {
        let millis = u32::try_from(limit.as_millis()).unwrap_or(u32::MAX - 1);
        // SAFETY: the handle is the process we created, still open.
        unsafe { WaitForSingleObject(raw(&self.process), millis) == WAIT_OBJECT_0 }
    }

    pub fn resize(&self, columns: i16, rows: i16) -> Result<(), String> {
        let console = self.console.ok_or("la consola ya está cerrada")?;
        // SAFETY: the console is open: it is only closed by taking it.
        unsafe { ResizePseudoConsole(console, size(columns, rows)) }
            .map_err(|e| format!("ResizePseudoConsole: {e}"))
    }

    fn close_console(&mut self) {
        if let Some(console) = self.console.take() {
            // SAFETY: taken out of the option, so it is closed only once.
            unsafe { ClosePseudoConsole(console) };
        }
    }

    /// Closes the console and says whether the program left by itself. It is
    /// never terminated.
    pub fn close(&mut self) -> Closed {
        let start = Instant::now();
        self.close_console();
        let left_in = self
            .has_left_within(Duration::from_secs(5))
            .then(|| start.elapsed());
        let limit = Instant::now() + Duration::from_secs(2);
        while self.reader.as_ref().is_some_and(|r| !r.is_finished()) && Instant::now() < limit {
            std::thread::sleep(Duration::from_millis(20));
        }
        Closed {
            left_in,
            reading_ended: self.reader.as_ref().is_some_and(JoinHandle::is_finished),
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        self.close_console();
    }
}

fn read_into(mut from: File, seen: Arc<Mutex<Vec<u8>>>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut chunk = [0u8; 4096];
        // An error here is the pipe closing: the end of the output.
        while let Ok(read @ 1..) = from.read(&mut chunk) {
            if let Ok(mut seen) = seen.lock() {
                seen.extend_from_slice(&chunk[..read]);
            }
        }
    })
}
