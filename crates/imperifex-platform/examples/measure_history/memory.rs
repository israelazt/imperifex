//! The memory of this process, as Windows counts it.

use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::GetCurrentProcess;

fn counters() -> Result<PROCESS_MEMORY_COUNTERS, String> {
    let mut counters = PROCESS_MEMORY_COUNTERS::default();
    let size = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    // SAFETY: the pseudo handle of this process needs no closing.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: the structure is as long as the size passed.
    let done = unsafe { K32GetProcessMemoryInfo(process, &mut counters, size) };
    done.ok()
        .map_err(|e| format!("GetProcessMemoryInfo: {e}"))?;
    Ok(counters)
}

/// What only this process uses.
pub fn private_bytes() -> Result<usize, String> {
    Ok(counters()?.PagefileUsage)
}

pub fn peak_private_bytes() -> Result<usize, String> {
    Ok(counters()?.PeakPagefileUsage)
}
