//! The arithmetic of what processes use.

use std::time::Duration;

/// The processor time used between two readings of the 100 ns counter, as a
/// percent of what one core could give in `wall`: two busy cores are 200. A
/// counter that went back, or no time at all, is 0.
pub fn cpu_percent_of_core(before_100ns: u64, after_100ns: u64, wall: Duration) -> f64 {
    let wall_100ns = wall.as_nanos() as f64 / 100.0;
    if wall_100ns > 0.0 {
        after_100ns.saturating_sub(before_100ns) as f64 / wall_100ns * 100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND_100NS: u64 = 10_000_000;

    #[test]
    fn one_second_of_processor_in_ten_is_ten_percent_of_a_core() {
        let percent = cpu_percent_of_core(0, SECOND_100NS, Duration::from_secs(10));
        assert_eq!(percent, 10.0);
    }

    #[test]
    fn two_busy_cores_are_two_hundred() {
        let percent = cpu_percent_of_core(5, 5 + 2 * SECOND_100NS, Duration::from_secs(1));
        assert_eq!(percent, 200.0);
    }

    #[test]
    fn a_counter_that_goes_back_or_no_time_is_nothing() {
        assert_eq!(cpu_percent_of_core(9, 3, Duration::from_secs(1)), 0.0);
        assert_eq!(cpu_percent_of_core(0, 9, Duration::ZERO), 0.0);
    }
}
