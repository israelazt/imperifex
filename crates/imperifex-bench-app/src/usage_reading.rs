//! What a terminal's processes are using at one moment, and the processor time
//! between two such moments.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use imperifex_bench_domain::ports::ProcessProbe;
use imperifex_bench_domain::process::ProcessUsage;
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::tree::terminal_processes;
use imperifex_bench_domain::usage::cpu_percent_of_core;
use imperifex_domain::ports::Clock;

/// The terminal's processes at one moment.
pub(crate) struct Reading {
    pub(crate) at: Duration,
    /// Every one of the terminal's, whether it could be read or not.
    pub(crate) listed: HashSet<u32>,
    pub(crate) used: HashMap<u32, ProcessUsage>,
    /// Those that could not be read, and the last reason.
    pub(crate) missed: usize,
    pub(crate) last_error: String,
}

pub(crate) fn read(
    clock: &impl Clock,
    processes: &impl ProcessProbe,
    profile: &TerminalProfile,
) -> Result<Reading, String> {
    let entries = processes.processes()?;
    let mut reading = Reading {
        at: clock.monotonic(),
        listed: HashSet::new(),
        used: HashMap::new(),
        missed: 0,
        last_error: String::new(),
    };
    for pid in terminal_processes(&entries, profile) {
        reading.listed.insert(pid);
        match processes.usage(pid) {
            Ok(usage) => {
                reading.used.insert(pid, usage);
            }
            Err(error) => {
                reading.missed += 1;
                reading.last_error = error;
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

/// The percent of one core used in `wall`, between two readings. A process that
/// was not there before spent all its time since then; one that was there but
/// could not be read is left out, as there is nothing to take its time from.
pub(crate) fn cpu_between(before: &Reading, now: &Reading, wall: Duration) -> f64 {
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
