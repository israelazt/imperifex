//! What it costs to have a terminal open: its memory, and the processor and the
//! graphics card it uses, sampled over some time. It only reads.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use imperifex_bench_domain::emit::BYTES_PER_MB;
use imperifex_bench_domain::metric::Metric;
use imperifex_bench_domain::ports::{GpuProbe, ProcessProbe};
use imperifex_bench_domain::process::ProcessUsage;
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::Measurement;
use imperifex_bench_domain::tree::terminal_processes;
use imperifex_bench_domain::usage::cpu_percent_of_core;
use imperifex_domain::ports::Clock;

const SHORT_LIVED: &str = "No se cuenta la CPU que un proceso gastó entre su última lectura y su \
    salida: los de vida corta pueden quedar fuera en parte o del todo.";

/// The terminal's processes at one moment.
struct Reading {
    at: Duration,
    /// Every one of the terminal's, whether it could be read or not.
    listed: HashSet<u32>,
    used: HashMap<u32, ProcessUsage>,
}

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
        self.cpu.push(cpu_between(before, now));
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

/// The percent of one core used between two readings. A process that was not
/// there before spent all its time since then; one that was there but could not
/// be read is left out, as there is nothing to take its time from.
fn cpu_between(before: &Reading, now: &Reading) -> f64 {
    let wall = now.at.saturating_sub(before.at);
    now.used
        .iter()
        .filter_map(|(pid, usage)| {
            let from = match before.used.get(pid) {
                Some(earlier) => earlier.cpu_100ns,
                None if !before.listed.contains(pid) => 0,
                None => return None,
            };
            Some(cpu_percent_of_core(from, usage.cpu_100ns, wall))
        })
        .sum()
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
        let entries = self.processes.processes()?;
        let mut reading = Reading {
            at: self.clock.monotonic(),
            listed: HashSet::new(),
            used: HashMap::new(),
        };
        for pid in terminal_processes(&entries, profile) {
            reading.listed.insert(pid);
            match self.processes.usage(pid) {
                Ok(usage) => {
                    reading.used.insert(pid, usage);
                }
                Err(error) => {
                    samples.missed += 1;
                    samples.last_error = error;
                }
            }
        }
        if reading.used.is_empty() {
            return Err(format!(
                "no hay ningún proceso de {} que se pueda leer: ábrela antes de medir y no la cierres",
                profile.name
            ));
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
