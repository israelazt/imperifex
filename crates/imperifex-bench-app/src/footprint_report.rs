//! The report of a terminal's footprint: what was measured, with the machine,
//! the terminal's version and the date.

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::ports::MachineProbe;
use imperifex_bench_domain::profile::{expand_env, find, parse_profiles, TerminalProfile};
use imperifex_bench_domain::report::{Report, TerminalInfo};
use imperifex_domain::log_warn;

use crate::command::Adapters;
use crate::footprint::FootprintService;

/// The terminals the bench knows, built in so that it runs from any folder.
const PROFILES: &str = include_str!("../../../bench/terminals.json");
const INTERVAL: Duration = Duration::from_secs(1);
/// What a report says when the terminal's file will not tell its version.
const UNKNOWN_VERSION: &str = "desconocida";

fn version_of(profile: &TerminalProfile, machine: &impl MachineProbe) -> String {
    expand_env(&profile.exe, &|name| machine.env_var(name))
        .and_then(|exe| machine.product_version(Path::new(&exe)))
        .unwrap_or_else(|error| {
            log_warn!("no se pudo leer la versión de {}: {error}", profile.name);
            UNKNOWN_VERSION.to_string()
        })
}

pub fn footprint_report<A: Adapters>(
    terminal: &str,
    seconds: u64,
    adapters: &A,
) -> Result<Report, String> {
    let profiles = parse_profiles(PROFILES)?;
    let profile = find(&profiles, terminal)?;
    let probe = adapters.machine();
    // Read before sampling, so that reading it is not part of any sample.
    let machine = probe.machine()?;
    let info = TerminalInfo {
        name: profile.name.clone(),
        version: version_of(profile, &probe),
    };
    let mut service =
        FootprintService::new(adapters.clock(), adapters.processes(), adapters.gpu()?);
    Ok(Report {
        date: probe.today(),
        machine,
        terminal: info,
        bench_version: env!("CARGO_PKG_VERSION").to_string(),
        measurements: service.sample(profile, seconds, INTERVAL)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::adapters::FakeAdapters;

    #[test]
    fn the_report_says_where_when_and_of_what() {
        let report = footprint_report("warp", 3, &FakeAdapters::default()).unwrap();
        assert_eq!(report.date, "2026-01-02");
        assert_eq!(report.machine.os, "os falso");
        assert_eq!(
            (
                report.terminal.name.as_str(),
                report.terminal.version.as_str()
            ),
            ("warp", "v9")
        );
        assert_eq!(report.measurements[0].summary.count, 3);
    }

    #[test]
    fn a_version_that_cannot_be_read_does_not_stop_the_measure() {
        let adapters = FakeAdapters {
            version: Err("no hay archivo".into()),
            ..Default::default()
        };
        let report = footprint_report("warp", 1, &adapters).unwrap();
        assert_eq!(report.terminal.version, UNKNOWN_VERSION);
    }

    #[test]
    fn an_unknown_terminal_is_an_error_before_anything_is_read() {
        let error = footprint_report("kitty", 1, &FakeAdapters::default()).unwrap_err();
        assert!(error.contains("kitty") && error.contains("warp"), "{error}");
    }
}
