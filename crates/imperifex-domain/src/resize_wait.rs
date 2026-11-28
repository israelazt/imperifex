//! While the window is being dragged to another size, the terminal and its
//! console wait: they change once the size has stayed the same for a moment.
//! Joining and cutting the lines of a long history on every step would freeze
//! the drag.

use std::time::Duration;

/// How long a size must stay to be applied.
const SETTLE: Duration = Duration::from_millis(100);

#[derive(Debug, Default)]
pub struct ResizeWait {
    /// The size asked for that is not applied yet, and since when it is it.
    pending: Option<((u16, u16), Duration)>,
    /// What the terminal has: columns and rows.
    applied: (u16, u16),
}

impl ResizeWait {
    /// With the size the terminal starts with.
    pub fn new(applied: (u16, u16)) -> Self {
        Self {
            pending: None,
            applied,
        }
    }

    /// There is room for `size` at the time `now` of a monotonic clock. Going
    /// back to the applied size ends the wait.
    pub fn asked(&mut self, size: (u16, u16), now: Duration) {
        if size == self.applied {
            self.pending = None;
        } else if self.pending.map(|(pending, _)| pending) != Some(size) {
            self.pending = Some((size, now));
        }
    }

    /// The size to apply now, once it has settled. It is given only once.
    pub fn due(&mut self, now: Duration) -> Option<(u16, u16)> {
        let (size, since) = self.pending?;
        if now.saturating_sub(since) < SETTLE {
            return None;
        }
        self.pending = None;
        self.applied = size;
        Some(size)
    }

    /// How long from `now` until there is a size to apply; `None` when none is
    /// waiting.
    pub fn due_in(&self, now: Duration) -> Option<Duration> {
        let (_, since) = self.pending?;
        Some(since.saturating_add(SETTLE).saturating_sub(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(time: u64) -> Duration {
        Duration::from_millis(time)
    }

    #[test]
    fn a_size_is_applied_once_when_it_has_stayed_100_ms() {
        let mut wait = ResizeWait::new((80, 25));
        wait.asked((90, 30), ms(1000));
        assert_eq!(wait.due_in(ms(1040)), Some(ms(60)));
        assert_eq!(wait.due(ms(1099)), None);
        assert_eq!(wait.due(ms(1100)), Some((90, 30)));
        assert_eq!(wait.due(ms(1200)), None);
        assert_eq!(wait.due_in(ms(1200)), None);
    }

    #[test]
    fn each_new_size_starts_the_wait_again_and_the_same_one_does_not() {
        let mut wait = ResizeWait::new((80, 25));
        wait.asked((90, 30), ms(0));
        wait.asked((91, 30), ms(60));
        wait.asked((91, 30), ms(120));
        assert_eq!(wait.due(ms(159)), None);
        assert_eq!(wait.due(ms(160)), Some((91, 30)));
    }

    #[test]
    fn going_back_to_the_applied_size_leaves_nothing_to_do() {
        let mut wait = ResizeWait::new((80, 25));
        wait.asked((90, 30), ms(0));
        wait.asked((80, 25), ms(50));
        assert_eq!(wait.due_in(ms(50)), None);
        assert_eq!(wait.due(ms(500)), None);
    }

    #[test]
    fn a_wait_already_over_is_due_at_once() {
        let mut wait = ResizeWait::new((80, 25));
        wait.asked((90, 30), ms(0));
        assert_eq!(wait.due_in(ms(500)), Some(Duration::ZERO));
    }
}
