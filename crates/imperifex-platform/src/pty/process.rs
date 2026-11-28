//! Starting a program attached to a pseudoconsole, and knowing how it ended.

use std::ffi::c_void;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};

use imperifex_domain::ports::Wake;
use windows::core::PWSTR;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Console::HPCON;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
    InitializeProcThreadAttributeList, UpdateProcThreadAttribute, WaitForSingleObject,
    EXTENDED_STARTUPINFO_PRESENT, INFINITE, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTF_USESTDHANDLES, STARTUPINFOEXW,
};

pub(super) struct Process {
    handle: OwnedHandle,
    id: u32,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Calls `left` from another thread once the program has left. The
    /// console stays open after it, and then its output says nothing.
    pub fn tell_when_it_leaves(&self, left: Wake) -> Result<(), String> {
        let handle = self
            .handle
            .try_clone()
            .map_err(|e| format!("no se pudo duplicar el proceso: {e}"))?;
        std::thread::Builder::new()
            .name("conpty-exit".into())
            .spawn(move || {
                let raw = HANDLE(handle.as_raw_handle());
                // SAFETY: this thread owns the handle, open until it ends.
                if unsafe { WaitForSingleObject(raw, INFINITE) } == WAIT_OBJECT_0 {
                    left();
                }
            })
            .map(drop)
            .map_err(|e| format!("no se pudo vigilar el proceso: {e}"))
    }

    pub fn exit_code(&self) -> Result<Option<u32>, String> {
        let handle = HANDLE(self.handle.as_raw_handle());
        // SAFETY: the handle is the process we created, still open.
        if unsafe { WaitForSingleObject(handle, 0) } != WAIT_OBJECT_0 {
            return Ok(None);
        }
        let mut code = 0;
        // SAFETY: the same handle, and `code` lives through the call.
        unsafe { GetExitCodeProcess(handle, &mut code) }
            .map_err(|e| format!("GetExitCodeProcess: {e}"))?;
        Ok(Some(code))
    }
}

/// A list of attributes for a new process, deleted when dropped.
struct Attributes {
    /// Holds the list; `usize` keeps it aligned.
    buffer: Vec<usize>,
}

impl Attributes {
    /// A list with room for one attribute.
    fn new() -> Result<Self, String> {
        let mut bytes = 0usize;
        // SAFETY: with no list, the call only writes the size it needs.
        let _ = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut bytes) };
        let mut attributes = Self {
            buffer: vec![0usize; bytes.div_ceil(size_of::<usize>())],
        };
        let list = attributes.list();
        // SAFETY: the buffer is at least as long as the size asked.
        unsafe { InitializeProcThreadAttributeList(Some(list), 1, None, &mut bytes) }
            .map_err(|e| format!("InitializeProcThreadAttributeList: {e}"))?;
        Ok(attributes)
    }

    fn list(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        LPPROC_THREAD_ATTRIBUTE_LIST(self.buffer.as_mut_ptr().cast())
    }

    fn attach(&mut self, console: *const c_void) -> Result<(), String> {
        let list = self.list();
        let attribute = PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize;
        // SAFETY: for this attribute the value is the console handle itself,
        // and the console outlives the process creation.
        unsafe {
            UpdateProcThreadAttribute(
                list,
                0,
                attribute,
                Some(console),
                size_of::<HPCON>(),
                None,
                None,
            )
        }
        .map_err(|e| format!("UpdateProcThreadAttribute: {e}"))
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        let list = self.list();
        // SAFETY: only a list that was initialized is ever dropped.
        unsafe { DeleteProcThreadAttributeList(list) };
    }
}

/// Starts `command_line` in the console whose handle is `console`.
pub(super) fn launch(console: *const c_void, command_line: &str) -> Result<Process, String> {
    let mut attributes = Attributes::new()?;
    attributes.attach(console)?;
    let mut startup = STARTUPINFOEXW {
        lpAttributeList: attributes.list(),
        ..Default::default()
    };
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    // Empty standard handles: the program takes those of its console, not ours.
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    create(&startup, command_line)
}

fn create(startup: &STARTUPINFOEXW, command_line: &str) -> Result<Process, String> {
    let mut line: Vec<u16> = command_line.encode_utf16().chain([0]).collect();
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: the command line is writable and ends in zero; the startup data
    // is the extended one, as the flag and its `cb` say.
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
    .map_err(|e| format!("no se pudo lanzar «{command_line}»: {e}"))?;
    // SAFETY: the call gave us the thread handle; dropping it closes it once.
    drop(unsafe { OwnedHandle::from_raw_handle(info.hThread.0) });
    // SAFETY: the same, for the process handle.
    let handle = unsafe { OwnedHandle::from_raw_handle(info.hProcess.0) };
    Ok(Process {
        handle,
        id: info.dwProcessId,
    })
}
