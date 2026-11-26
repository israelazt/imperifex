//! The reports of the orders that open windows. All of them launch from the
//! measures' folder, wherever the bench was started (D15).

use std::path::PathBuf;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::metric::{Load, Metric};
use imperifex_bench_domain::ports::MachineProbe;
use imperifex_bench_domain::profile::TerminalProfile;
use imperifex_bench_domain::report::{Measurement, Report};
use imperifex_bench_domain::screen::primary_area;
use imperifex_domain::log_info;

use super::{expanded_exe, report};
use crate::command::Adapters;
use crate::emit::default_mib;
use crate::footprint::FootprintService;
use crate::latency::{LatencyMode, LatencyService};
use crate::measure_window::TypingPorts;
use crate::memory_load::MemoryService;
use crate::output_seen::OutputService;
use crate::startup::StartupService;
use crate::window_session::SessionPorts;
use crate::workdir::measure_home;

/// How long each reading of memory samples.
const MEMORY_SECONDS: u64 = 10;
const HANDS_OFF: &str = "no toques el teclado ni el ratón hasta que termine";

/// The profile ready to launch and the folder to launch it from, once it is
/// said what is about to happen.
fn ready<A: Adapters>(
    profile: &TerminalProfile,
    machine: &A::Machine,
    adapters: &A,
    what: &str,
) -> Result<(TerminalProfile, PathBuf), String> {
    let ready = TerminalProfile {
        exe: expanded_exe(profile, machine)?,
        ..profile.clone()
    };
    let home = measure_home(adapters.files())?;
    log_info!(
        "{what}; {HANDS_OFF}; las ventanas se abren desde {}",
        home.display()
    );
    Ok((ready, home))
}

fn session<A: Adapters>(adapters: &A) -> SessionPorts<A::Clock, A::Desktop, A::Processes> {
    SessionPorts {
        clock: adapters.clock(),
        desktop: adapters.desktop(),
        processes: adapters.processes(),
    }
}

type Typing<A> = TypingPorts<
    <A as Adapters>::Clock,
    <A as Adapters>::Desktop,
    <A as Adapters>::Processes,
    <A as Adapters>::Screen,
    <A as Adapters>::Keyboard,
>;

/// The whole of the primary monitor.
fn primary(machine: &impl MachineProbe) -> Result<Rect, String> {
    primary_area(&machine.machine()?.monitors)
        .ok_or_else(|| "no se sabe cuál es el monitor principal ni cuánto mide".to_string())
}

fn typing<A: Adapters>(adapters: &A, machine: &A::Machine) -> Result<Typing<A>, String> {
    Ok(TypingPorts {
        session: session(adapters),
        screen: adapters.screen(),
        keyboard: adapters.keyboard(),
        primary: primary(machine)?,
    })
}

pub fn startup_report<A: Adapters>(
    terminal: &str,
    runs: u32,
    cold: bool,
    adapters: &A,
) -> Result<Report, String> {
    report(terminal, adapters, |profile, machine| {
        let what = format!("{runs} ventanas, cerca de un minuto cada una");
        let (profile, home) = ready(profile, machine, adapters, &what)?;
        StartupService::new(session(adapters), adapters.screen(), primary(machine)?)
            .run(&profile, runs, cold, &home)
    })
}

pub fn latency_report<A: Adapters>(
    terminal: &str,
    presses: u32,
    modes: &[LatencyMode],
    adapters: &A,
) -> Result<Report, String> {
    report(terminal, adapters, |profile, machine| {
        let what = format!("{presses} pulsaciones por modo en una ventana propia");
        let (profile, home) = ready(profile, machine, adapters, &what)?;
        let service = LatencyService::new(typing(adapters, machine)?, adapters.current_exe()?);
        modes
            .iter()
            .map(|&mode| service.run(&profile, presses, mode, &home))
            .collect()
    })
}

type Output<'a, A> = OutputService<
    'a,
    <A as Adapters>::Clock,
    <A as Adapters>::Desktop,
    <A as Adapters>::Processes,
    <A as Adapters>::Screen,
    <A as Adapters>::Keyboard,
>;

fn output_service<'a, A: Adapters>(
    adapters: &'a A,
    machine: &A::Machine,
) -> Result<Output<'a, A>, String> {
    Ok(OutputService {
        ports: typing(adapters, machine)?,
        files: adapters.files(),
        bench_exe: adapters.current_exe()?,
    })
}

pub fn output_report<A: Adapters>(
    terminal: &str,
    loads: &[Load],
    repeats: u32,
    adapters: &A,
) -> Result<Report, String> {
    report(terminal, adapters, |profile, machine| {
        let what = format!("{} cargas, {repeats} veces cada una", loads.len());
        let (profile, home) = ready(profile, machine, adapters, &what)?;
        let service = output_service(adapters, machine)?;
        let mut measured = Vec::new();
        for &load in loads {
            measured.extend(service.run(&profile, load, default_mib(load), repeats, &home)?);
        }
        Ok(measured)
    })
}

/// The median of the terminal's private memory over a few seconds.
fn private_mb<A: Adapters>(profile: &TerminalProfile, adapters: &A) -> Result<f64, String> {
    let mut service =
        FootprintService::new(adapters.clock(), adapters.processes(), adapters.gpu()?);
    service
        .sample(profile, MEMORY_SECONDS, super::INTERVAL)?
        .iter()
        .find(|m: &&Measurement| m.metric == Metric::MemoryIdlePrivate)
        .map(|m| m.summary.p50)
        .ok_or_else(|| "la huella no trae la memoria privada".to_string())
}

pub fn memory_report<A: Adapters>(terminal: &str, adapters: &A) -> Result<Report, String> {
    report(terminal, adapters, |profile, machine| {
        let what = "un historial largo y tres pestañas en una ventana propia";
        let (profile, home) = ready(profile, machine, adapters, what)?;
        let reading = || private_mb(&profile, adapters);
        MemoryService {
            output: output_service(adapters, machine)?,
            private_mb: &reading,
        }
        .run(&profile, &home)
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::testing::adapters::FakeAdapters;

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

    fn with_a_repository(at: &str) -> FakeAdapters {
        let adapters = FakeAdapters::default();
        let git = Path::new(at).join(".git");
        adapters.files.folders.borrow_mut().insert(git);
        adapters
    }

    #[test]
    fn a_startup_launches_from_the_measures_folder_even_from_a_repository() {
        // The fakes start the bench in `elsewhere`.
        let adapters = with_a_repository("elsewhere");
        startup_report("warp", 1, false, &adapters).unwrap();
        let dirs = adapters.desktop.state.borrow().dirs.clone();
        assert_eq!(dirs, [Path::new("temp").join("home")]);
    }

    /// Every order that opens windows.
    fn each_order(adapters: &FakeAdapters) -> Vec<Result<Report, String>> {
        vec![
            startup_report("warp", 1, false, adapters),
            latency_report("warp", 1, &[LatencyMode::Prompt], adapters),
            output_report("warp", &[Load::Ascii], 1, adapters),
            memory_report("warp", adapters),
        ]
    }

    #[test]
    fn with_the_measures_folder_in_a_repository_no_order_opens_anything() {
        let adapters = with_a_repository("temp");
        for result in each_order(&adapters) {
            assert!(result.unwrap_err().contains("carpeta temporal"));
        }
        assert!(adapters.desktop.state.borrow().asked.is_empty());
    }
}
