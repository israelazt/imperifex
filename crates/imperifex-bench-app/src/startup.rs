//! How long a terminal takes to show a window, and to have it drawn (D13).

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{Desktop, ProcessProbe, ScreenWatch};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_domain::ports::Clock;

use crate::screen_wait::settle;
use crate::window_session::{running, wait_for, Closed, SessionPorts, WindowSession};

const EVERY_SECOND: Duration = Duration::from_secs(1);
const BETWEEN_RUNS: Duration = Duration::from_secs(2);
/// How long a terminal has to leave no process behind, between cold starts,
/// once the wait for its window's leftovers is over (D16).
const TO_LEAVE: Duration = Duration::from_secs(120);
/// A drawn window is one that has been still this long.
const DRAWN_QUIET: Duration = Duration::from_millis(500);
const TO_BE_DRAWN: Duration = Duration::from_secs(10);

const WHAT_IS_TIMED: &str = "Desde que se lanza el programa hasta que su ventana nueva sale en la \
    lista de ventanas, visible y con título; no mide cuándo tiene contenido. Se mira cada 5 ms, más \
    lo que tarda cada lectura.";
const WHAT_IS_DRAWN: &str = "Desde que se lanza el programa hasta el último cambio de la ventana \
    antes de 500 ms de calma. Solo cuenta un cambio del 0,5 % de la ventana, y solo se ven los \
    posteriores a la primera imagen recibida una vez visible.";

/// What one run found.
struct Run {
    visible: Duration,
    drawn: Result<Duration, String>,
    closed: Closed,
}

/// What each run left behind and how long it was waited for.
fn left_by(runs: &[Run]) -> String {
    let each: Vec<String> = runs
        .iter()
        .map(|r| format!("{} en {} s", r.closed.leftovers, r.closed.waited.as_secs()))
        .collect();
    format!(
        "Procesos que quedaban de cada ventana cerrada, y espera: {}.",
        each.join("; ")
    )
}

/// The runs that gave no drawn sample, and why.
fn not_drawn(runs: &[Run]) -> Vec<String> {
    (1..)
        .zip(runs)
        .filter_map(|(number, run)| {
            let why = run.drawn.as_ref().err()?;
            Some(format!("Tanda {number} sin ventana dibujada: {why}."))
        })
        .collect()
}

/// The visible times and, if any window could be watched until drawn, those.
fn measurements(runs: &[Run], cold: bool) -> Result<Vec<Measurement>, String> {
    let (visible, drawn) = if cold {
        (Metric::StartupCold, Metric::StartupColdDrawn)
    } else {
        (Metric::StartupWarm, Metric::StartupWarmDrawn)
    };
    let ms = |time: Duration| time.as_secs_f64() * 1000.0;
    let mut notes = vec![WHAT_IS_TIMED.to_string(), left_by(runs)];
    notes.extend(not_drawn(runs));
    let samples = runs.iter().map(|r| ms(r.visible)).collect();
    let first = Measurement::new(visible, samples, notes)
        .ok_or_else(|| "hace falta al menos una tanda".to_string())?;
    let mut notes = vec![WHAT_IS_DRAWN.to_string()];
    notes.extend(not_drawn(runs));
    let samples = runs
        .iter()
        .filter_map(|r| r.drawn.as_ref().ok().map(|&t| ms(t)))
        .collect();
    let second = Measurement::new(drawn, samples, notes);
    Ok([Some(first), second].into_iter().flatten().collect())
}

pub struct StartupService<C, D, P, S> {
    ports: SessionPorts<C, D, P>,
    screen: S,
}

impl<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch> StartupService<C, D, P, S> {
    pub fn new(ports: SessionPorts<C, D, P>, screen: S) -> Self {
        Self { ports, screen }
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
    ) -> Result<Vec<Measurement>, String> {
        let mut done = Vec::new();
        for number in 0..runs {
            if number > 0 {
                self.ports.clock.sleep(BETWEEN_RUNS);
            }
            done.push(self.once(profile, cold, dir)?);
        }
        measurements(&done, cold)
    }

    fn once(&self, profile: &TerminalProfile, cold: bool, dir: &Path) -> Result<Run, String> {
        if cold && !running(&self.ports.processes, profile)?.is_empty() {
            return Err(format!(
                "cierra {} del todo: el arranque en frío necesita que no esté abierto (regla S4)",
                profile.name
            ));
        }
        let session = WindowSession::open(&self.ports, profile, dir)?;
        let visible = session.visible_at.saturating_sub(session.started_at);
        // Whatever happens while watching it, the window is closed after.
        let drawn = self.drawn(&session);
        let closed = session.close()?;
        if cold {
            self.wait_until_gone(profile)?;
        }
        Ok(Run {
            visible,
            drawn,
            closed,
        })
    }

    /// From the launch until the window, left where it opened, was drawn.
    fn drawn(&self, session: &WindowSession<'_, C, D, P>) -> Result<Duration, String> {
        let mut watch = self.screen.watch(session.window().rect)?;
        let settled = settle(
            watch.as_mut(),
            &self.ports.clock,
            session.visible_at,
            DRAWN_QUIET,
            TO_BE_DRAWN,
        )?;
        Ok(settled.at.saturating_sub(session.started_at))
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
