//! What a terminal's memory does with a long history, and with each tab.

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::keys::{parse_chord, Chord, Key};
use imperifex_bench_domain::metric::{Load, Metric};
use imperifex_bench_domain::ports::{Desktop, Keyboard, ProcessProbe, ScreenWatch, Watch};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_bench_domain::tree::{new_since, terminal_processes};
use imperifex_domain::log_info;
use imperifex_domain::ports::Clock;

use crate::measure_window::{settle_now, then_close};
use crate::output_seen::OutputService;
use crate::window_session::{running, wait_for, WindowSession};

/// How long nothing is opened before measuring, for what earlier windows left
/// to be gone (D20): their processes stay about a minute, and one that ended
/// halfway would falsify the differences. Which ones they are cannot be told
/// without a picture from before they were opened.
const QUIET_BEFORE: Duration = Duration::from_secs(90);
/// A million lines of the ASCII load, 82 bytes each, are 78.2 MiB.
const HISTORY_MIB: usize = 79;
const TABS: u32 = 3;
const TO_OPEN_A_TAB: Duration = Duration::from_secs(5);
const EVERY_LOOK: Duration = Duration::from_millis(50);

const SCROLLBACK_NOTE: &str = "Diferencia entre las medianas de memoria privada (10 s cada una) \
    antes y después de emitir 79 MiB de la carga ascii (algo más de un millón de líneas).";
const TAB_NOTE: &str = "Aumento medio de memoria privada por pestaña nueva. Una pestaña se da por \
    abierta cuando aparece un proceso auxiliar nuevo de la terminal.";

/// What the tabs gave: how many opened and the memory with all of them.
struct Tabs {
    opened: u32,
    memory: f64,
}

pub struct MemoryService<'a, C, D, P, S, K> {
    pub output: OutputService<'a, C, D, P, S, K>,
    /// The median of the terminal's private memory over a few seconds, in MB.
    pub private_mb: &'a dyn Fn() -> Result<f64, String>,
}

/// The helper processes of the terminal that were not in `before`.
fn new_helpers(
    processes: &impl ProcessProbe,
    profile: &TerminalProfile,
    before: &[u32],
) -> Result<bool, String> {
    let entries = processes.processes()?;
    let new = new_since(before, &terminal_processes(&entries, profile));
    Ok(entries.iter().any(|e| {
        new.contains(&e.pid)
            && profile
                .helper_images
                .iter()
                .any(|h| h.eq_ignore_ascii_case(&e.image))
    }))
}

fn one_sample(metric: Metric, value: f64, notes: Vec<String>) -> Result<Measurement, String> {
    Measurement::new(metric, vec![value], notes)
        .ok_or_else(|| format!("no hay muestra de {}", metric.label()))
}

impl<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>
    MemoryService<'_, C, D, P, S, K>
{
    /// In a window of its own: the memory a long history takes, and then what
    /// each of a few new tabs adds. The scrollback always comes out; the tabs
    /// only if the terminal opens them with the profile's keys.
    pub fn run(&self, profile: &TerminalProfile, dir: &Path) -> Result<Vec<Measurement>, String> {
        // Read before the window opens, so that a bad profile opens nothing.
        let keys = profile
            .new_tab_keys
            .as_deref()
            .map(parse_chord)
            .transpose()?;
        log_info!(
            "esperando {} s sin abrir nada, para que salga lo que dejaran otras ventanas",
            QUIET_BEFORE.as_secs()
        );
        self.output.ports.session.clock.sleep(QUIET_BEFORE);
        let window = WindowSession::open(&self.output.ports.session, profile, dir)?;
        let (measured, _) = then_close(window, |window| self.measure(window, profile, keys))?;
        Ok(measured)
    }

    fn measure(
        &self,
        window: &WindowSession<'_, C, D, P>,
        profile: &TerminalProfile,
        keys: Option<Chord>,
    ) -> Result<Vec<Measurement>, String> {
        let mut watch = self.output.ports.prepare(window)?;
        let before = (self.private_mb)()?;
        self.output
            .emit_and_wait(window, watch.as_mut(), Load::Ascii, HISTORY_MIB, || Ok(()))?;
        let full = (self.private_mb)()?;
        let mut notes = vec![SCROLLBACK_NOTE.to_string()];
        let tabs = match keys {
            Some(keys) => self.tabs(window, watch.as_mut(), profile, keys)?,
            None => None,
        };
        if tabs.is_none() {
            notes.push(
                match keys {
                    Some(_) => {
                        "Sin memoria por pestaña: las teclas del perfil no abrieron ninguna."
                    }
                    None => "Sin memoria por pestaña: el perfil no tiene teclas de pestaña nueva.",
                }
                .to_string(),
            );
        }
        let history = one_sample(Metric::MemoryScrollbackPrivate, full - before, notes)?;
        let mut measured = vec![history];
        if let Some(tabs) = tabs {
            let each = (tabs.memory - full) / f64::from(tabs.opened);
            let notes = vec![
                TAB_NOTE.to_string(),
                format!("Pestañas abiertas: {}.", tabs.opened),
            ];
            measured.push(one_sample(Metric::MemoryPerTabPrivate, each, notes)?);
        }
        Ok(measured)
    }

    /// Opens tabs one by one; `None` if not even the first one opened.
    fn tabs(
        &self,
        window: &WindowSession<'_, C, D, P>,
        watch: &mut dyn Watch,
        profile: &TerminalProfile,
        keys: Chord,
    ) -> Result<Option<Tabs>, String> {
        let ports = &self.output.ports;
        let mut tabs = None;
        for number in 1..=TABS {
            let before = running(&ports.session.processes, profile)?;
            ports.press(window, Key::Chord(keys))?;
            let opened = wait_for(&ports.session.clock, EVERY_LOOK, TO_OPEN_A_TAB, || {
                Ok(new_helpers(&ports.session.processes, profile, &before)?.then_some(()))
            })?;
            if opened.is_none() {
                break;
            }
            settle_now(watch, &ports.session.clock)?;
            tabs = Some(Tabs {
                opened: number,
                memory: (self.private_mb)()?,
            });
        }
        Ok(tabs)
    }
}

#[cfg(test)]
mod tests;
