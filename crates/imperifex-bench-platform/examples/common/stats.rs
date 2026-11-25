//! What the probes share to time things and sum them up.

use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// The performance counter now, in its own ticks.
pub fn ticks() -> i64 {
    let mut now = 0;
    // SAFETY: the pointer is to a local that lives through the call. It cannot
    // fail on Windows XP or later; if it did, the value would stay 0.
    let _ = unsafe { QueryPerformanceCounter(&mut now) };
    now
}

/// How many ticks of [`ticks`] there are in a second.
pub fn ticks_per_second() -> i64 {
    let mut frequency = 0;
    // SAFETY: the pointer is to a local that lives through the call.
    let _ = unsafe { QueryPerformanceFrequency(&mut frequency) };
    frequency.max(1)
}

pub fn ms(from: i64, to: i64) -> f64 {
    (to - from) as f64 * 1000.0 / ticks_per_second() as f64
}

/// "p50 x · p95 y · máx z (n)" of some milliseconds, by nearest rank.
pub fn spread(mut samples: Vec<f64>) -> String {
    if samples.is_empty() {
        return "sin muestras".into();
    }
    samples.sort_by(f64::total_cmp);
    let rank = |p: usize| samples[(p * samples.len()).div_ceil(100).max(1) - 1];
    format!(
        "p50 {:.3} ms · p95 {:.3} ms · máx {:.3} ms (n = {})",
        rank(50),
        rank(95),
        rank(100),
        samples.len()
    )
}
