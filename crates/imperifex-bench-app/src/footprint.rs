//! What it costs to have a terminal open: its memory, and the processor and the
//! graphics card it uses, sampled over some time. It only reads.

use std::time::Duration;

use imperifex_bench_domain::emit::BYTES_PER_MB;
use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{GpuProbe, ProcessProbe};
use imperifex_bench_domain::process::ProcessUsage;
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_domain::ports::Clock;

use crate::usage_reading::{cpu_between, read, Reading};

const SHORT_LIVED: &str = "No se cuenta la CPU que un proceso gastó entre su última lectura y su \
    salida: los de vida corta pueden quedar fuera en parte o del todo.";

/// The samples taken so far, and what could not be read.
#[derive(Default)]
struct Samples {
    private_mb: Vec<f64>,
    working_set_mb: Vec<f64>,
    cpu: Vec<f64>,
    gpu: Vec<f64>,
    processes: Vec<usize>,
    missed: usize,
    last_error: String,
}

impl Samples {
    fn add(&mut self, before: &Reading, now: &Reading, gpu: f64) {
        let sum = |bytes: fn(&ProcessUsage) -> u64| {
            now.used.values().map(bytes).sum::<u64>() as f64 / BYTES_PER_MB as f64
        };
        self.private_mb.push(sum(|u| u.private_bytes));
        self.working_set_mb.push(sum(|u| u.working_set_bytes));
        let wall = now.at.saturating_sub(before.at);
        self.cpu.push(cpu_between(before, now, wall));
        self.gpu.push(gpu);
        self.processes.push(now.used.len());
    }

    fn measurements(self) -> Result<Vec<Measurement>, String> {
        let fewest = self.processes.iter().min().copied().unwrap_or_default();
        let most = self.processes.iter().max().copied().unwrap_or_default();
        let mut notes = vec![format!(
            "Procesos de la terminal leídos en cada muestra: entre {fewest} y {most}."
        )];
        if self.missed > 0 {
            notes.push(format!(
                "Lecturas de procesos que fallaron y no cuentan en su muestra: {} (la última: {}).",
                self.missed, self.last_error
            ));
        }
        let mut cpu_notes = notes.clone();
        cpu_notes.push(SHORT_LIVED.to_string());
        [
            (Metric::MemoryIdlePrivate, self.private_mb, notes.clone()),
            (
                Metric::MemoryIdleWorkingSet,
                self.working_set_mb,
                notes.clone(),
            ),
            (Metric::CpuIdle, self.cpu, cpu_notes),
            (Metric::GpuIdle, self.gpu, notes),
        ]
        .into_iter()
        .map(|(metric, samples, notes)| {
            Measurement::new(metric, samples, notes)
                .ok_or_else(|| format!("no hay muestras de {}", metric.label()))
        })
        .collect()
    }
}

pub struct FootprintService<C, P, G> {
    clock: C,
    processes: P,
    gpu: G,
}

impl<C: Clock, P: ProcessProbe, G: GpuProbe> FootprintService<C, P, G> {
    pub fn new(clock: C, processes: P, gpu: G) -> Self {
        Self {
            clock,
            processes,
            gpu,
        }
    }

    /// One sample every `interval` for `seconds`. The processes are looked for
    /// again each time, as they come and go.
    pub fn sample(
        &mut self,
        profile: &TerminalProfile,
        seconds: u64,
        interval: Duration,
    ) -> Result<Vec<Measurement>, String> {
        if interval.is_zero() {
            return Err("el intervalo entre muestras no puede ser cero".into());
        }
        let rounds = (Duration::from_secs(seconds).as_nanos() / interval.as_nanos()).max(1);
        let mut samples = Samples::default();
        let mut before = self.read(profile, &mut samples)?;
        // The first answer only starts the card's count.
        self.gpu.utilization(&pids_of(&before))?;
        for _ in 0..rounds {
            self.clock.sleep(interval);
            let now = self.read(profile, &mut samples)?;
            let gpu = self.gpu.utilization(&pids_of(&now))?;
            samples.add(&before, &now, gpu);
            before = now;
        }
        samples.measurements()
    }

    fn read(&self, profile: &TerminalProfile, samples: &mut Samples) -> Result<Reading, String> {
        let reading = read(&self.clock, &self.processes, profile)?;
        if reading.missed > 0 {
            samples.missed += reading.missed;
            samples.last_error.clone_from(&reading.last_error);
        }
        Ok(reading)
    }
}

/// In order, so that what is asked of the card does not depend on a hash.
fn pids_of(reading: &Reading) -> Vec<u32> {
    let mut pids: Vec<u32> = reading.used.keys().copied().collect();
    pids.sort_unstable();
    pids
}

#[cfg(test)]
mod tests;
