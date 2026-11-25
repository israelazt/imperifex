//! A PDH query of the GPU engines.

use windows::core::{w, PCWSTR};
use windows::Win32::System::Performance::*;

/// The engines' use, by instance name.
pub const GPU_ENGINES: PCWSTR = w!(r"\GPU Engine(*)\Utilization Percentage");

/// The largest answer accepted, so a misbehaving call cannot allocate without end.
const MAX_BYTES: u32 = 16 * 1024 * 1024;

fn check(what: &str, status: u32) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("{what} falló: 0x{status:08x}"))
    }
}

pub struct Query {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
}

impl Drop for Query {
    fn drop(&mut self) {
        // SAFETY: the query was opened by `open` and is closed once.
        let _ = unsafe { PdhCloseQuery(self.query) };
    }
}

impl Query {
    pub fn open(path: PCWSTR) -> Result<Self, String> {
        let mut query = PDH_HQUERY::default();
        // SAFETY: the out pointer is to a local; no data source means live data.
        let status = unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) };
        check("PdhOpenQueryW", status)?;
        let mut opened = Self {
            query,
            counter: PDH_HCOUNTER::default(),
        };
        // SAFETY: the query is open and the path is a static wide string.
        let status = unsafe { PdhAddEnglishCounterW(opened.query, path, 0, &mut opened.counter) };
        check("PdhAddEnglishCounterW", status)?;
        Ok(opened)
    }

    pub fn collect(&self) -> Result<(), String> {
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

    /// Every instance with its value, after at least two collections.
    pub fn values(&self) -> Result<Vec<(String, f64)>, String> {
        let (mut bytes, mut count) = (0, 0);
        let status = self.array(&mut bytes, &mut count, None);
        if status != PDH_MORE_DATA {
            check("PdhGetFormattedCounterArrayW (tamaño)", status)?;
        }
        if bytes > MAX_BYTES {
            return Err(format!("PDH pide demasiados bytes: {bytes}"));
        }
        let mut buffer = vec![0u64; (bytes as usize).div_ceil(8)];
        let status = self.array(&mut bytes, &mut count, Some(&mut buffer));
        check("PdhGetFormattedCounterArrayW", status)?;
        let first = buffer.as_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        // SAFETY: PDH wrote `count` items at the start of the aligned buffer.
        let items = unsafe { std::slice::from_raw_parts(first, count as usize) };
        Ok(items.iter().filter_map(read_item).collect())
    }
}

/// `None` for an instance without a valid value yet.
fn read_item(item: &PDH_FMT_COUNTERVALUE_ITEM_W) -> Option<(String, f64)> {
    let status = item.FmtValue.CStatus;
    if status != PDH_CSTATUS_VALID_DATA && status != PDH_CSTATUS_NEW_DATA {
        return None;
    }
    // SAFETY: the name points into the buffer the item came in, ends in a zero
    // and is read while the buffer is alive.
    let name = unsafe { item.szName.to_string() }.ok()?;
    // SAFETY: the double format was asked for, so this is the member written.
    let value = unsafe { item.FmtValue.Anonymous.doubleValue };
    Some((name, value))
}
