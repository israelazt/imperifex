//! The processes of Windows and what each is using. It only reads: a process is
//! opened with the right to ask about it and nothing else.

use std::cell::RefCell;
use std::collections::HashMap;

use imperifex_bench_domain::ports::ProcessProbe;
use imperifex_bench_domain::process::{ProcessEntry, ProcessUsage};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, FILETIME, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows::Win32::System::Threading::{
    GetProcessHandleCount, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::decode::utf16;

/// A handle that is closed once, however the function that owns it ends.
struct Owned(HANDLE);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: the handle was given to this guard, which closes it once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn failed(what: &str) -> impl Fn(windows::core::Error) -> String + '_ {
    move |error| format!("{what} falló: {error}")
}

/// Reads the processes with the tool help snapshot and the process API.
#[derive(Debug, Default)]
pub struct WindowsProcesses {
    /// The threads of each process in the last list, which only the snapshot says.
    threads: RefCell<HashMap<u32, u32>>,
}

impl WindowsProcesses {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Every process with its threads.
fn snapshot() -> Result<Vec<(ProcessEntry, u32)>, String> {
    // SAFETY: no pointers; the handle goes to the guard.
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(failed("CreateToolhelp32Snapshot"))?;
    let snapshot = Owned(handle);
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut found = Vec::new();
    // SAFETY: the snapshot is alive and the entry says its size.
    let mut next = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    while next.is_ok() {
        let process = ProcessEntry {
            pid: entry.th32ProcessID,
            parent: entry.th32ParentProcessID,
            image: utf16(&entry.szExeFile),
        };
        found.push((process, entry.cntThreads));
        // SAFETY: the same snapshot and entry as above.
        next = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    match next {
        Err(error) if error.code() != ERROR_NO_MORE_FILES.to_hresult() => {
            Err(failed("recorrer los procesos")(error))
        }
        _ => Ok(found),
    }
}

fn memory_of(process: &Owned) -> Result<PROCESS_MEMORY_COUNTERS_EX, String> {
    let size = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let mut counters = PROCESS_MEMORY_COUNTERS_EX {
        cb: size,
        ..Default::default()
    };
    let out = std::ptr::from_mut(&mut counters).cast::<PROCESS_MEMORY_COUNTERS>();
    // SAFETY: the extended form begins as the plain one, and `size` says that
    // the whole of it can be written.
    unsafe { GetProcessMemoryInfo(process.0, out, size) }
        .map_err(failed("GetProcessMemoryInfo"))?;
    Ok(counters)
}

fn ticks(time: FILETIME) -> u64 {
    u64::from(time.dwHighDateTime) << 32 | u64::from(time.dwLowDateTime)
}

/// Kernel and user time, in ticks of 100 ns.
fn cpu_of(process: &Owned) -> Result<u64, String> {
    let [mut created, mut exited, mut kernel, mut user] = [FILETIME::default(); 4];
    // SAFETY: four out pointers to locals that live through the call.
    unsafe { GetProcessTimes(process.0, &mut created, &mut exited, &mut kernel, &mut user) }
        .map_err(failed("GetProcessTimes"))?;
    Ok(ticks(kernel) + ticks(user))
}

fn handles_of(process: &Owned) -> Result<u32, String> {
    let mut count = 0;
    // SAFETY: the out pointer is to a local that lives through the call.
    unsafe { GetProcessHandleCount(process.0, &mut count) }
        .map_err(failed("GetProcessHandleCount"))?;
    Ok(count)
}

impl ProcessProbe for WindowsProcesses {
    fn processes(&self) -> Result<Vec<ProcessEntry>, String> {
        let found = snapshot()?;
        *self.threads.borrow_mut() = found.iter().map(|(p, t)| (p.pid, *t)).collect();
        Ok(found.into_iter().map(|(process, _)| process).collect())
    }

    fn usage(&self, pid: u32) -> Result<ProcessUsage, String> {
        // SAFETY: no pointers; the handle goes to the guard.
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
            .map_err(failed("OpenProcess"))?;
        let process = Owned(handle);
        let memory = memory_of(&process)?;
        Ok(ProcessUsage {
            private_bytes: memory.PrivateUsage as u64,
            working_set_bytes: memory.WorkingSetSize as u64,
            cpu_100ns: cpu_of(&process)?,
            threads: self.threads.borrow().get(&pid).copied().unwrap_or_default(),
            handles: handles_of(&process)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_process_is_listed_and_can_be_read() {
        let probe = WindowsProcesses::new();
        let own = std::process::id();
        assert!(probe.processes().unwrap().iter().any(|p| p.pid == own));
        let usage = probe.usage(own).unwrap();
        assert!(usage.private_bytes > 0 && usage.working_set_bytes > 0);
        assert!(usage.threads > 0 && usage.handles > 0);
    }

    #[test]
    fn a_pid_that_is_nobodys_is_an_error() {
        assert!(WindowsProcesses::new().usage(u32::MAX - 3).is_err());
    }
}
