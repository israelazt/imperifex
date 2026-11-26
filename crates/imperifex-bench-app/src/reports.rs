//! The reports of the orders: what was measured, with the machine, the
//! terminal's version and the date.

use std::path::Path;
use std::time::Duration;

use imperifex_bench_domain::ports::MachineProbe;
use imperifex_bench_domain::profile::{expand_env, find, parse_profiles, TerminalProfile};
use imperifex_bench_domain::report::{Measurement, Report, TerminalInfo};
use imperifex_domain::{log_info, log_warn};

use crate::command::Adapters;
use crate::footprint::FootprintService;
use crate::startup::StartupService;
use crate::window_session::SessionPorts;
use crate::workdir::measure_home;

/// The terminals the bench knows, built in so that it runs from any folder.
const PROFILES: &str = include_str!("../../../bench/terminals.json");
const INTERVAL: Duration = Duration::from_secs(1);
/// What a report says when the terminal's file will not tell its version.
const UNKNOWN_VERSION: &str = "desconocida";

fn expanded_exe(profile: &TerminalProfile, machine: &impl MachineProbe) -> Result<String, String> {
    expand_env(&profile.exe, &|name| machine.env_var(name))
}

fn version_of(profile: &TerminalProfile, machine: &impl MachineProbe) -> String {
    expanded_exe(profile, machine)
        .and_then(|exe| machine.product_version(Path::new(&exe)))
        .unwrap_or_else(|error| {
            log_warn!("no se pudo leer la versión de {}: {error}", profile.name);
            UNKNOWN_VERSION.to_string()
        })
}

/// A report of the terminal called `terminal` with what `measure` finds. The
/// machine is read first, so that reading it is not part of any sample.
fn report<A: Adapters>(
    terminal: &str,
    adapters: &A,
    measure: impl FnOnce(&TerminalProfile, &A::Machine) -> Result<Vec<Measurement>, String>,
) -> Result<Report, String> {
    let profiles = parse_profiles(PROFILES)?;
    let profile = find(&profiles, terminal)?;
    let probe = adapters.machine();
    let machine = probe.machine()?;
    let info = TerminalInfo {
        name: profile.name.clone(),
        version: version_of(profile, &probe),
    };
    Ok(Report {
        date: probe.today(),
        machine,
        terminal: info,
        bench_version: env!("CARGO_PKG_VERSION").to_string(),
        measurements: measure(profile, &probe)?,
    })
}

pub fn footprint_report<A: Adapters>(
    terminal: &str,
    seconds: u64,
    adapters: &A,
) -> Result<Report, String> {
    report(terminal, adapters, |profile, _| {
        FootprintService::new(adapters.clock(), adapters.processes(), adapters.gpu()?)
            .sample(profile, seconds, INTERVAL)
    })
}

pub fn startup_report<A: Adapters>(
    terminal: &str,
    runs: u32,
    cold: bool,
    adapters: &A,
) -> Result<Report, String> {
    report(terminal, adapters, |profile, machine| {
        let ready = TerminalProfile {
            exe: expanded_exe(profile, machine)?,
            ..profile.clone()
        };
        let home = measure_home(adapters.files(), &adapters.current_dir()?)?;
        log_info!(
            "no toques el PC hasta que termine: {runs} ventanas, cerca de un minuto cada una; se abren desde {}",
            home.display()
        );
        let ports = SessionPorts {
            clock: adapters.clock(),
            desktop: adapters.desktop(),
            processes: adapters.processes(),
        };
        StartupService::new(ports, adapters.screen()).run(&ready, runs, cold, &home)
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
    fn a_startup_report_launches_the_program_with_its_path_expanded() {
        let adapters = FakeAdapters::default();
        let report = startup_report("warp", 2, false, &adapters).unwrap();
        assert_eq!(report.measurements[0].summary.count, 2);
        let asked = adapters.desktop.state.borrow().asked.clone();
        assert!(
            asked[0].starts_with("launch local") && asked[0].ends_with("warp.exe"),
            "{asked:?}"
        );
    }

    #[test]
    fn a_startup_launches_from_the_measures_folder() {
        let adapters = FakeAdapters::default();
        startup_report("warp", 1, false, &adapters).unwrap();
        let dirs = adapters.desktop.state.borrow().dirs.clone();
        assert_eq!(dirs, [Path::new("temp").join("home")]);
    }

    #[test]
    fn a_startup_from_inside_a_repository_opens_nothing() {
        let adapters = FakeAdapters::default();
        let git = Path::new("elsewhere").join(".git");
        adapters.files.folders.borrow_mut().insert(git);
        let error = startup_report("warp", 1, false, &adapters).unwrap_err();
        assert!(error.contains("sin Git"), "{error}");
        assert!(adapters.desktop.state.borrow().asked.is_empty());
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
