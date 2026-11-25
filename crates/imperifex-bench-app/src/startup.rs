//! How long a terminal takes to show a window.

use std::time::Duration;

use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{Desktop, ProcessProbe};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_domain::ports::Clock;

use crate::own_window::{running, wait_for, Opener};

const BETWEEN_RUNS: Duration = Duration::from_secs(2);
/// How long a terminal has to leave no process behind, between cold starts.
const TO_LEAVE: Duration = Duration::from_secs(10);

const WHAT_IS_TIMED: &str = "Desde que se lanza el programa hasta que su ventana nueva sale en la \
    lista de ventanas, visible y con título; no mide cuándo tiene contenido. Se mira cada 5 ms, más \
    lo que tarda cada lectura.";

pub struct StartupService<C, D, P> {
    clock: C,
    desktop: D,
    processes: P,
}

impl<C: Clock, D: Desktop, P: ProcessProbe> StartupService<C, D, P> {
    pub fn new(clock: C, desktop: D, processes: P) -> Self {
        Self {
            clock,
            desktop,
            processes,
        }
    }

    /// Opens and closes a window `runs` times. A cold start also needs, and
    /// leaves, no process of the terminal. It stops at the first thing that does
    /// not go as expected. The profile's `exe` must already be expanded.
    pub fn run(
        &self,
        profile: &TerminalProfile,
        runs: u32,
        cold: bool,
    ) -> Result<Measurement, String> {
        let mut samples = Vec::new();
        for number in 0..runs {
            if number > 0 {
                self.clock.sleep(BETWEEN_RUNS);
            }
            samples.push(self.once(profile, cold)?.as_secs_f64() * 1000.0);
        }
        let metric = if cold {
            Metric::StartupCold
        } else {
            Metric::StartupWarm
        };
        Measurement::new(metric, samples, vec![WHAT_IS_TIMED.to_string()])
            .ok_or_else(|| "hace falta al menos una tanda".to_string())
    }

    fn once(&self, profile: &TerminalProfile, cold: bool) -> Result<Duration, String> {
        if cold && !running(&self.processes, profile)?.is_empty() {
            return Err(format!(
                "cierra {} del todo: el arranque en frío necesita que no esté abierto (regla S4)",
                profile.name
            ));
        }
        let opener = Opener {
            clock: &self.clock,
            desktop: &self.desktop,
            processes: &self.processes,
        };
        let own = opener.open(profile)?;
        opener.close(&own)?;
        if cold {
            self.wait_until_gone(profile)?;
        }
        Ok(own.took)
    }

    fn wait_until_gone(&self, profile: &TerminalProfile) -> Result<(), String> {
        wait_for(&self.clock, TO_LEAVE, || {
            Ok(running(&self.processes, profile)?.is_empty().then_some(()))
        })?
        .ok_or_else(|| {
            format!(
                "quedan procesos de {} {} s después de cerrar su ventana: no se fuerzan (regla S2)",
                profile.name,
                TO_LEAVE.as_secs()
            )
        })
    }
}

#[cfg(test)]
mod tests;
