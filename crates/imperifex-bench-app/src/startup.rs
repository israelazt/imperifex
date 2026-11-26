//! How long a terminal takes to show a window.

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{Desktop, ProcessProbe};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_domain::ports::Clock;

use crate::window_session::{running, wait_for, Closed, SessionPorts, WindowSession};

const EVERY_SECOND: Duration = Duration::from_secs(1);
const BETWEEN_RUNS: Duration = Duration::from_secs(2);
/// How long a terminal has to leave no process behind, between cold starts,
/// once the wait for its window's leftovers is over (D16).
const TO_LEAVE: Duration = Duration::from_secs(120);

const WHAT_IS_TIMED: &str = "Desde que se lanza el programa hasta que su ventana nueva sale en la \
    lista de ventanas, visible y con título; no mide cuándo tiene contenido. Se mira cada 5 ms, más \
    lo que tarda cada lectura.";

/// What each run left behind and how long it was waited for.
fn left_by(runs: &[Closed]) -> String {
    let each: Vec<String> = runs
        .iter()
        .map(|c| format!("{} en {} s", c.leftovers, c.waited.as_secs()))
        .collect();
    format!(
        "Procesos que quedaban de cada ventana cerrada, y espera: {}.",
        each.join("; ")
    )
}

pub struct StartupService<C, D, P> {
    ports: SessionPorts<C, D, P>,
}

impl<C: Clock, D: Desktop, P: ProcessProbe> StartupService<C, D, P> {
    pub fn new(ports: SessionPorts<C, D, P>) -> Self {
        Self { ports }
    }

    /// Opens and closes a window `runs` times. A cold start also needs, and
    /// leaves, no process of the terminal. It stops at the first thing that does
    /// not go as expected. The profile's `exe` must already be expanded.
    pub fn run(
        &self,
        profile: &TerminalProfile,
        runs: u32,
        cold: bool,
        dir: &Path,
    ) -> Result<Measurement, String> {
        let mut samples = Vec::new();
        let mut left = Vec::new();
        for number in 0..runs {
            if number > 0 {
                self.ports.clock.sleep(BETWEEN_RUNS);
            }
            let (took, closed) = self.once(profile, cold, dir)?;
            samples.push(took.as_secs_f64() * 1000.0);
            left.push(closed);
        }
        let metric = if cold {
            Metric::StartupCold
        } else {
            Metric::StartupWarm
        };
        Measurement::new(
            metric,
            samples,
            vec![WHAT_IS_TIMED.to_string(), left_by(&left)],
        )
        .ok_or_else(|| "hace falta al menos una tanda".to_string())
    }

    fn once(
        &self,
        profile: &TerminalProfile,
        cold: bool,
        dir: &Path,
    ) -> Result<(Duration, Closed), String> {
        if cold && !running(&self.ports.processes, profile)?.is_empty() {
            return Err(format!(
                "cierra {} del todo: el arranque en frío necesita que no esté abierto (regla S4)",
                profile.name
            ));
        }
        let session = WindowSession::open(&self.ports, profile, dir)?;
        let took = session.visible_at.saturating_sub(session.started_at);
        let closed = session.close()?;
        if cold {
            self.wait_until_gone(profile)?;
        }
        Ok((took, closed))
    }

    fn wait_until_gone(&self, profile: &TerminalProfile) -> Result<(), String> {
        let ports = &self.ports;
        wait_for(&ports.clock, EVERY_SECOND, TO_LEAVE, || {
            Ok(running(&ports.processes, profile)?.is_empty().then_some(()))
        })?
        .ok_or_else(|| {
            format!(
                "{} no salió al cerrar su única ventana: quedan procesos suyos {} s después; no se fuerzan (regla S2)",
                profile.name,
                TO_LEAVE.as_secs()
            )
        })
    }
}

#[cfg(test)]
mod tests;
