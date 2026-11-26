//! A load of output as the terminal shows it: how long until it is all on the
//! screen, how fast it was taken in, and what processor it cost (D18).

use std::path::{Path, PathBuf};
use std::time::Duration;

use imperifex_bench_domain::emit::EmitResult;
use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::metric::{Load, Metric};
use imperifex_bench_domain::ports::{
    Desktop, FileStore, Keyboard, ProcessProbe, ScreenWatch, Watch,
};
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_bench_domain::shell::powershell_command;
use imperifex_domain::ports::Clock;

use crate::measure_window::{settle_now, then_close, TypingPorts};
use crate::screen_wait::settle_when;
use crate::usage_reading::{cpu_between, read};
use crate::window_session::WindowSession;

const QUIET: Duration = Duration::from_millis(500);
/// How long one load may take to be emitted and shown.
const TO_BE_SHOWN: Duration = Duration::from_secs(120);

const CPU_NOTE: &str = "CPU = lo que gastaron los procesos de la terminal entre justo antes de \
    Enter y ver la pantalla asentada (con sus 500 ms de calma), dividido entre el tiempo hasta el \
    último cambio. Los procesos que salieron entre medias no se cuentan.";

/// What one repetition gave.
#[derive(Debug, Clone, Copy)]
struct Shown {
    seen_ms: f64,
    mb_per_s: f64,
    cpu: f64,
}

pub struct OutputService<'a, C, D, P, S, K> {
    pub ports: TypingPorts<C, D, P, S, K>,
    pub files: &'a dyn FileStore,
    /// The bench's own program, which the window runs to emit the load.
    pub bench_exe: String,
}

/// A file for a result that is not there yet: one left by an earlier run would
/// look as if the load were already over.
pub fn fresh_result(files: &dyn FileStore, name: &str) -> Result<PathBuf, String> {
    let folder = files.temp_dir()?;
    (1..)
        .map(|n| folder.join(format!("{name}-{n}.json")))
        .find(|path| !files.exists(path))
        .ok_or_else(|| "no queda un nombre libre para el resultado".to_string())
}

fn measurements(load: Load, shown: &[Shown]) -> Result<Vec<Measurement>, String> {
    let of = |metric: Metric, value: fn(&Shown) -> f64, notes: Vec<String>| {
        Measurement::new(metric, shown.iter().map(value).collect(), notes)
            .ok_or_else(|| format!("no hay muestras de {}", metric.label()))
    };
    Ok(vec![
        of(Metric::OutputSeen(load), |s| s.seen_ms, vec![])?,
        of(Metric::OutputConsumed(load), |s| s.mb_per_s, vec![])?,
        of(
            Metric::CpuDuringOutput(load),
            |s| s.cpu,
            vec![CPU_NOTE.to_string()],
        )?,
    ])
}

impl<C: Clock, D: Desktop, P: ProcessProbe, S: ScreenWatch, K: Keyboard>
    OutputService<'_, C, D, P, S, K>
{
    /// Emits `mib` of `load` `repeats` times in a window of its own.
    pub fn run(
        &self,
        profile: &TerminalProfile,
        load: Load,
        mib: usize,
        repeats: u32,
        dir: &Path,
    ) -> Result<Vec<Measurement>, String> {
        let before = self.ports.look()?;
        let window = WindowSession::open(&self.ports.session, profile, dir)?;
        let ((shown, ready_note), _) = then_close(window, |window| {
            let mut prepared = self.ports.prepare(window, before)?;
            let watch = prepared.watch.as_mut();
            let mut shown = Vec::new();
            for number in 0..repeats {
                if number > 0 {
                    self.ports.enter_line(window, "cls")?;
                    settle_now(watch, &self.ports.session.clock)?;
                }
                shown.push(self.emit_once(window, watch, profile, load, mib)?);
            }
            Ok((shown, prepared.ready_note()))
        })?;
        let mut measured = measurements(load, &shown)?;
        if let Some(seen) = measured.first_mut() {
            seen.notes.push(ready_note);
        }
        Ok(measured)
    }

    /// Runs one `emit` in the window and waits for it to be over and shown.
    /// It gives the result of `emit` and the time until the last change.
    pub fn emit_and_wait(
        &self,
        window: &WindowSession<'_, C, D, P>,
        watch: &mut dyn Watch,
        load: Load,
        mib: usize,
        before_enter: impl FnOnce() -> Result<(), String>,
    ) -> Result<(EmitResult, Duration), String> {
        let json = fresh_result(self.files, load.name())?;
        let out = json.display().to_string();
        let args = [
            "emit",
            load.name(),
            "--mib",
            &mib.to_string(),
            "--out",
            &out,
        ];
        self.ports
            .type_text(window, &powershell_command(&self.bench_exe, &args))?;
        before_enter()?;
        let clock = &self.ports.session.clock;
        let hwnd = window.ready_to_type()?;
        let entered = clock.monotonic();
        self.ports.keyboard.press(hwnd, Key::Enter)?;
        let settled = settle_when(watch, clock, entered, QUIET, TO_BE_SHOWN, || {
            self.files.exists(&json)
        })
        .map_err(|error| {
            if self.files.exists(&json) {
                error
            } else {
                format!("no apareció el resultado de emit en {out}: {error}")
            }
        })?;
        if settled.changes == 0 {
            return Err("emit terminó, pero no se vio ningún cambio en la ventana".into());
        }
        let result = self.result_in(&json)?;
        Ok((result, settled.at.saturating_sub(entered)))
    }

    fn result_in(&self, json: &Path) -> Result<EmitResult, String> {
        let bytes = self
            .files
            .read(json)?
            .ok_or_else(|| format!("desapareció {}", json.display()))?;
        EmitResult::from_json(&String::from_utf8_lossy(&bytes))
    }

    fn emit_once(
        &self,
        window: &WindowSession<'_, C, D, P>,
        watch: &mut dyn Watch,
        profile: &TerminalProfile,
        load: Load,
        mib: usize,
    ) -> Result<Shown, String> {
        let session = &self.ports.session;
        let mut before = None;
        let (result, seen) = self.emit_and_wait(window, watch, load, mib, || {
            before = Some(read(&session.clock, &session.processes, profile)?);
            Ok(())
        })?;
        let after = read(&session.clock, &session.processes, profile)?;
        let before = before.ok_or("no se leyó la CPU antes de la salida")?;
        Ok(Shown {
            seen_ms: seen.as_secs_f64() * 1000.0,
            mb_per_s: result.mb_per_s,
            cpu: cpu_between(&before, &after, seen),
        })
    }
}

#[cfg(test)]
mod tests;
