//! Turning ticks of the performance counter into time. The bench's clock and
//! the moment a frame was presented are both read through here, so the two can
//! be compared.

use std::time::Duration;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// The time those ticks are, at `frequency` ticks a second. A negative count or
/// a frequency that is not positive is no time.
pub fn qpc_to_duration(ticks: i64, frequency: i64) -> Duration {
    let (Ok(ticks), Ok(frequency)) = (u128::try_from(ticks), u128::try_from(frequency)) else {
        return Duration::ZERO;
    };
    let Some(nanos) = (ticks * NANOS_PER_SECOND).checked_div(frequency) else {
        return Duration::ZERO;
    };
    let seconds = u64::try_from(nanos / NANOS_PER_SECOND).unwrap_or(u64::MAX);
    Duration::new(seconds, (nanos % NANOS_PER_SECOND) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEN_MHZ: i64 = 10_000_000;

    #[test]
    fn a_tick_at_ten_megahertz_is_a_hundred_nanoseconds() {
        assert_eq!(qpc_to_duration(1, TEN_MHZ), Duration::from_nanos(100));
    }

    #[test]
    fn a_month_of_ticks_does_not_overflow() {
        const MONTH: i64 = 30 * 24 * 60 * 60;
        assert_eq!(
            qpc_to_duration(MONTH * TEN_MHZ, TEN_MHZ),
            Duration::from_secs(MONTH as u64)
        );
    }

    #[test]
    fn nonsense_is_no_time() {
        assert_eq!(qpc_to_duration(-1, TEN_MHZ), Duration::ZERO);
        assert_eq!(qpc_to_duration(1, 0), Duration::ZERO);
        assert_eq!(qpc_to_duration(1, -5), Duration::ZERO);
    }
}
