//! An example service: checks the app and times it, through the `Clock` port.
//! Replace it with the first real service.

use imperifex_domain::example::StatusReport;
use imperifex_domain::ports::Clock;

pub struct StatusService<C> {
    clock: C,
}

impl<C: Clock> StatusService<C> {
    pub fn new(clock: C) -> Self {
        Self { clock }
    }

    pub fn check(&self) -> StatusReport {
        let started = self.clock.monotonic();
        let took = self.clock.monotonic().saturating_sub(started);
        StatusReport {
            version: env!("CARGO_PKG_VERSION").to_string(),
            took_ms: u64::try_from(took.as_millis()).unwrap_or(u64::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeClock;

    #[test]
    fn it_reports_the_version_and_times_the_check() {
        let report = StatusService::new(FakeClock::ticking(4)).check();
        assert_eq!(report.took_ms, 4);
        assert_eq!(report.version, env!("CARGO_PKG_VERSION"));
    }
}
