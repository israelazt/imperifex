//! The 3D work of the graphics card, by process, from the performance counters.

use imperifex_bench_domain::ports::GpuProbe;
use windows::core::{w, PCWSTR};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W,
    PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
};

use crate::gpu_instance::pid_of_3d_engine;

const GPU_ENGINES: PCWSTR = w!(r"\GPU Engine(*)\Utilization Percentage");
/// The largest answer accepted, so that a misbehaving call cannot allocate
/// without end.
const MAX_BYTES: u32 = 16 * 1024 * 1024;
/// Instances come and go; if the list grows between asking its size and reading
/// it, it is asked again this many times.
const ATTEMPTS: usize = 3;

fn check(what: &str, status: u32) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("{what} falló: 0x{status:08x}"))
    }
}

/// A query kept open: each reading is of the time since the one before.
pub struct GpuCounters {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
}

impl Drop for GpuCounters {
    fn drop(&mut self) {
        // SAFETY: the query was opened by `new` and is closed once.
        let _ = unsafe { PdhCloseQuery(self.query) };
    }
}

impl GpuCounters {
    pub fn new() -> Result<Self, String> {
        let mut query = PDH_HQUERY::default();
        // SAFETY: the out pointer is to a local; no data source means live data.
        let status = unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) };
        check("PdhOpenQueryW", status)?;
        let mut counters = Self {
            query,
            counter: PDH_HCOUNTER::default(),
        };
        // SAFETY: the query is open and the path is a static wide string.
        let status =
            unsafe { PdhAddEnglishCounterW(counters.query, GPU_ENGINES, 0, &mut counters.counter) };
        check("PdhAddEnglishCounterW", status)?;
        counters.collect()?;
        Ok(counters)
    }

    fn collect(&self) -> Result<(), String> {
        // SAFETY: the query is open.
        let status = unsafe { PdhCollectQueryData(self.query) };
        check("PdhCollectQueryData", status)
    }

    fn array(&self, bytes: &mut u32, count: &mut u32, buffer: Option<&mut [u64]>) -> u32 {
        let buffer = buffer.map(|b| b.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>());
        // SAFETY: the counter is alive; the buffer, if any, is aligned and holds
        // at least `bytes` bytes.
        unsafe { PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, bytes, count, buffer) }
    }

    /// Every instance with a value, from the last two collections.
    fn values(&self) -> Result<Vec<(String, f64)>, String> {
        let mut status = PDH_MORE_DATA;
        for _ in 0..ATTEMPTS {
            let (mut bytes, mut count) = (0, 0);
            status = self.array(&mut bytes, &mut count, None);
            if status != PDH_MORE_DATA {
                check("PdhGetFormattedCounterArrayW (tamaño)", status)?;
                return Ok(Vec::new());
            }
            if bytes > MAX_BYTES {
                return Err(format!("PDH pide demasiados bytes: {bytes}"));
            }
            let mut buffer = vec![0u64; (bytes as usize).div_ceil(8)];
            status = self.array(&mut bytes, &mut count, Some(&mut buffer));
            if status != PDH_MORE_DATA {
                check("PdhGetFormattedCounterArrayW", status)?;
                return Ok(read_items(&buffer, count as usize));
            }
        }
        check("PdhGetFormattedCounterArrayW", status).map(|()| Vec::new())
    }
}

/// `buffer` must be what PDH filled with `count` items.
fn read_items(buffer: &[u64], count: usize) -> Vec<(String, f64)> {
    let first = buffer.as_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
    // SAFETY: PDH wrote `count` items at the start of this aligned buffer.
    let items = unsafe { std::slice::from_raw_parts(first, count) };
    items.iter().filter_map(read_item).collect()
}

/// `None` for an instance without a value yet.
fn read_item(item: &PDH_FMT_COUNTERVALUE_ITEM_W) -> Option<(String, f64)> {
    let status = item.FmtValue.CStatus;
    if status != PDH_CSTATUS_VALID_DATA && status != PDH_CSTATUS_NEW_DATA {
        return None;
    }
    // SAFETY: the name points into the buffer the item came in, ends in a zero
    // and is read while that buffer is alive.
    let name = unsafe { item.szName.to_string() }.ok()?;
    // SAFETY: the double format was asked for, so this is the member written.
    let value = unsafe { item.FmtValue.Anonymous.doubleValue };
    Some((name, value))
}

impl GpuProbe for GpuCounters {
    fn utilization(&mut self, pids: &[u32]) -> Result<f64, String> {
        self.collect()?;
        Ok(self
            .values()?
            .iter()
            .filter(|(name, _)| pid_of_3d_engine(name).is_some_and(|pid| pids.contains(&pid)))
            .map(|(_, value)| value)
            .sum())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use imperifex_bench_domain::ports::ProcessProbe;

    use super::*;
    use crate::WindowsProcesses;

    /// A hosted runner has no graphics card to count.
    fn hardware_or_skip() -> bool {
        std::env::var_os("IMPERIFEX_NO_HARDWARE").is_none()
    }

    #[test]
    fn asking_about_no_process_is_no_work_at_all() {
        if !hardware_or_skip() {
            return;
        }
        let mut counters = GpuCounters::new().unwrap();
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(counters.utilization(&[]), Ok(0.0));
    }

    /// `cargo test --release -p imperifex-bench-platform measure_ -- --ignored
    /// --nocapture`: what one whole sample of the footprint of Warp costs.
    #[test]
    #[ignore = "a measurement, run by hand with Warp open"]
    fn measure_a_whole_footprint_sample() {
        let probe = WindowsProcesses::new();
        let mut counters = GpuCounters::new().unwrap();
        for round in 0..10 {
            std::thread::sleep(Duration::from_secs(1));
            let start = Instant::now();
            let all = probe.processes().unwrap();
            let listed = start.elapsed();
            // The processes with these names are enough to time the reading.
            let pids: Vec<u32> = all
                .iter()
                .filter(|p| ["warp.exe", "OpenConsole.exe", "conhost.exe"].contains(&&*p.image))
                .map(|p| p.pid)
                .collect();
            let read = pids.iter().filter(|&&pid| probe.usage(pid).is_ok()).count();
            let used = start.elapsed();
            counters.utilization(&pids).unwrap();
            let total = start.elapsed();
            println!(
                "{round}: list {} processes {listed:?} · read {read}/{} {:?} · gpu {:?} · total {total:?}",
                all.len(),
                pids.len(),
                used - listed,
                total - used
            );
        }
    }
}
