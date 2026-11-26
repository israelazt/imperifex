//! When the screen stopped changing after something was done.

use std::time::Duration;

/// For the screen to count as having changed, while waiting for it to settle,
/// at least this share of the region's pixels must differ (D17): in 1280x680,
/// 4352. A blinking cursor and a single letter stay under it; a screen of
/// output goes well over.
pub const SETTLE_SHARE: f64 = 0.005;

/// `changes` are the moments, in order, at which the screen changed; those
/// outside `start..=end` are not looked at. The answer is the last change that
/// was followed by `quiet` without another one before `end`; `None` if there
/// was never such a silence.
///
/// With no changes at all the screen was settled at `start`. With changes, the
/// silence before the first one does not count: it is only the wait for the
/// terminal to react.
pub fn settled_at(
    changes: &[Duration],
    start: Duration,
    quiet: Duration,
    end: Duration,
) -> Option<Duration> {
    let inside: Vec<Duration> = changes
        .iter()
        .copied()
        .filter(|at| (start..=end).contains(at))
        .collect();
    if inside.is_empty() {
        return (end.saturating_sub(start) >= quiet).then_some(start);
    }
    let next = inside.iter().skip(1).copied().chain([end]);
    inside
        .iter()
        .zip(next)
        .filter(|(at, next)| next.saturating_sub(**at) >= quiet)
        .map(|(at, _)| *at)
        .last()
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUIET: Duration = Duration::from_millis(500);

    fn ms(at: u64) -> Duration {
        Duration::from_millis(at)
    }

    fn settled(changes: &[u64], end: u64) -> Option<Duration> {
        let changes: Vec<Duration> = changes.iter().map(|&at| ms(at)).collect();
        settled_at(&changes, ms(100), QUIET, ms(end))
    }

    #[test]
    fn with_no_changes_it_was_settled_from_the_start() {
        assert_eq!(settled(&[], 600), Some(ms(100)));
    }

    #[test]
    fn with_no_changes_and_too_little_time_nothing_can_be_said() {
        assert_eq!(settled(&[], 599), None);
    }

    #[test]
    fn changes_and_then_silence_settle_at_the_last_one() {
        assert_eq!(settled(&[150, 200, 250], 750), Some(ms(250)));
    }

    #[test]
    fn if_the_silence_never_comes_there_is_no_answer() {
        assert_eq!(settled(&[200, 600, 1000, 1400], 1600), None);
    }

    #[test]
    fn a_slow_reaction_is_not_taken_for_the_end() {
        assert_eq!(settled(&[900, 950], 2000), Some(ms(950)));
        assert_eq!(settled(&[900, 950], 1000), None);
    }

    #[test]
    fn of_two_silences_the_later_one_counts() {
        assert_eq!(settled(&[200, 900], 1500), Some(ms(900)));
    }

    #[test]
    fn a_silence_followed_by_changes_up_to_the_end_still_counts() {
        assert_eq!(settled(&[200, 900, 1200], 1300), Some(ms(200)));
    }

    #[test]
    fn changes_before_the_start_or_after_the_end_are_not_looked_at() {
        assert_eq!(settled(&[50, 200, 5000], 800), Some(ms(200)));
    }
}
