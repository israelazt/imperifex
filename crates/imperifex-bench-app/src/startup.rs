//! How long a terminal takes to show a window, its first content and the
//! ready window (D13).

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{Desktop, ProcessProbe, ScreenWatch};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_domain::ports::Clock;

use crate::window_session::{running, wait_for, Closed, SessionPorts, WindowSession};
use drawn::{look_before, watch_window, Drawn, Moment};

mod drawn;

const EVERY_SECOND: Duration = Duration::from_secs(1);
const BETWEEN_RUNS: Duration = Duration::from_secs(2);
/// How long a terminal has to leave no process behind, between cold starts,
/// once the wait for its window's leftovers is over (D16).
const TO_LEAVE: Duration = Duration::from_secs(120);
const WHAT_IS_TIMED: &str = "Desde que se lanza el programa hasta que su ventana nueva sale en la \
    lista de ventanas, visible y con título; no mide cuándo tiene contenido. Se mira cada 5 ms, más \
    lo que tarda cada lectura.";
const WHAT_IS_CONTENT: &str =
    "Desde que se lanza el programa hasta la primera imagen en la que el \
    sitio de la ventana difiere un 0,5 % o más de como estaba antes de lanzar.";
const WHAT_IS_READY: &str =
    "Desde que se lanza el programa hasta el último cambio del 0,5 % de la \
    ventana antes de 3 s de calma; la calma no se cuenta.";

/// What one run found.
struct Run {
    visible: Duration,
    drawn: Drawn,
    closed: Closed,
}

/// One of the two times the screen tells of a start.
type FromScreen = fn(&Drawn) -> &Result<Moment, String>;

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

/// The runs that gave no sample of one of the screen's times, and why.
fn missing(runs: &[Run], which: FromScreen, what: &str) -> Vec<String> {
    (1..)
        .zip(runs)
        .filter_map(|(number, run)| {
            let why = which(&run.drawn).as_ref().err()?;
            Some(format!("Tanda {number} sin {what}: {why}."))
        })
        .collect()
}

fn ms(time: Duration) -> f64 {
    time.as_secs_f64() * 1000.0
}

/// One of the screen's times over the runs; `None` if none gave it.
fn seen_on_screen(
    runs: &[Run],
    metric: Metric,
    which: FromScreen,
    definition: &str,
) -> Option<Measurement> {
    let moments: Vec<Moment> = runs
        .iter()
        .filter_map(|r| which(&r.drawn).as_ref().ok().copied())
        .collect();
    let joined = moments.iter().filter(|m| m.joined).count();
    let notes = vec![
        definition.to_string(),
        format!("Muestras cuya imagen juntaba varias presentaciones, y que pueden salir tarde: {joined}."),
    ];
    let samples = moments.iter().map(|m| ms(m.time)).collect();
    Measurement::new(metric, samples, notes)
}

/// The visible times and, for the runs whose window the screen showed, the
/// first content and the ready window.
fn measurements(runs: &[Run], cold: bool) -> Result<Vec<Measurement>, String> {
    let [visible, content, ready] = if cold {
        [
            Metric::StartupCold,
            Metric::StartupColdContent,
            Metric::StartupColdReady,
        ]
    } else {
        [
            Metric::StartupWarm,
            Metric::StartupWarmContent,
            Metric::StartupWarmReady,
        ]
    };
    let content_of: FromScreen = |drawn| &drawn.content;
    let ready_of: FromScreen = |drawn| &drawn.ready;
    // Why a run lacks them goes with the visible time, which is always there.
    let mut notes = vec![WHAT_IS_TIMED.to_string(), left_by(runs)];
    notes.extend(missing(runs, content_of, "primer contenido"));
    notes.extend(missing(runs, ready_of, "ventana lista"));
    let samples = runs.iter().map(|r| ms(r.visible)).collect();
    let first = Measurement::new(visible, samples, notes)
        .ok_or_else(|| "hace falta al menos una tanda".to_string())?;
    let rest = [
        seen_on_screen(runs, content, content_of, WHAT_IS_CONTENT),
        seen_on_screen(runs, ready, ready_of, WHAT_IS_READY),
    ];
    Ok(std::iter::once(first)
        .chain(rest.into_iter().flatten())
        .collect())
}

pub struct StartupService<C, D, P, S> {
    ports: SessionPorts<C, D, P>,
    screen: S,
    /// The whole of the primary monitor, where a window has to open to be seen
    /// being drawn.
    primary: Rect,
}

impl<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch> StartupService<C, D, P, S> {
    pub fn new(ports: SessionPorts<C, D, P>, screen: S, primary: Rect) -> Self {
        Self {
            ports,
            screen,
            primary,
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
        // The screen is looked at before there is anything new on it.
        let before = look_before(&self.screen, &self.ports.clock, self.primary);
        let session = WindowSession::open(&self.ports, profile, dir)?;
        let visible = session.visible_at.saturating_sub(session.started_at);
        // Whatever happens while watching it, the window is closed after.
        let drawn = match before {
            Ok(before) => watch_window(
                before,
                &self.ports.clock,
                session.window().rect,
                session.started_at,
                session.visible_at,
            ),
            Err(why) => Drawn::unseen(why),
        };
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
