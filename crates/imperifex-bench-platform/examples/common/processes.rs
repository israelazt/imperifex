//! The processes of the system, read once. Reading them changes nothing.

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub parent: u32,
    pub image: String,
}

impl Process {
    pub fn is(&self, image: &str) -> bool {
        self.image.eq_ignore_ascii_case(image)
    }
}

struct Snapshot(HANDLE);

impl Drop for Snapshot {
    fn drop(&mut self) {
        // SAFETY: the handle came from CreateToolhelp32Snapshot and is closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

pub fn processes() -> Result<Vec<Process>, String> {
    // SAFETY: no pointers; the handle is owned by the guard.
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|error| format!("CreateToolhelp32Snapshot falló: {error}"))?;
    let snapshot = Snapshot(handle);
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut found = Vec::new();
    // SAFETY: the snapshot is alive and the entry says its size.
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) }.is_ok();
    while more {
        let name = &entry.szExeFile;
        let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        found.push(Process {
            pid: entry.th32ProcessID,
            parent: entry.th32ParentProcessID,
            image: String::from_utf16_lossy(&name[..end]),
        });
        // SAFETY: the same snapshot and entry as above.
        more = unsafe { Process32NextW(snapshot.0, &mut entry) }.is_ok();
    }
    Ok(found)
}
