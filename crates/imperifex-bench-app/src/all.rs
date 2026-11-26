//! The whole baseline of a terminal in one run. A part that fails is noted in
//! the report and the rest go on.

use std::path::{Path, PathBuf};

use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_domain::report::Report;
use imperifex_domain::log_info;

use crate::command::Adapters;
use crate::latency::LatencyMode;
use crate::reports::{
    empty_report, footprint_report, latency_report, memory_report, output_report, public_json,
    startup_report,
};

const FOOTPRINT_SECONDS: u64 = 60;
const STARTUP_RUNS: u32 = 10;
const PRESSES: u32 = 200;
const REPEATS: u32 = 3;
/// The sum of its parts as Warp took them, each run on its own: the footprint
/// 1 min, ten starts 11, both latencies 5, the six loads 8 and the memory 4.
const ESTIMATE: &str = "unos 30 minutos (medidos por partes el 2026-11-26)";
const REPORT_FILE: &str = "report.json";

/// One part of the baseline: what it is called and how it is measured.
type Part<'a> = (String, Box<dyn FnOnce() -> Result<Report, String> + 'a>);

/// In the order they run: the footprint first, before anything is opened. Each
/// mode of latency and each load is a part of its own, so that one that fails
/// takes none of the others with it.
fn parts<'a, A: Adapters>(terminal: &'a str, adapters: &'a A) -> Vec<Part<'a>> {
    let mut parts: Vec<Part<'a>> = vec![
        (
            "huella en reposo".into(),
            Box::new(move || footprint_report(terminal, FOOTPRINT_SECONDS, adapters)),
        ),
        (
            "arranque en caliente".into(),
            Box::new(move || startup_report(terminal, STARTUP_RUNS, false, adapters)),
        ),
    ];
    for (name, mode) in [
        ("latencia en el prompt", LatencyMode::Prompt),
        ("latencia en un programa", LatencyMode::Program),
    ] {
        parts.push((
            name.into(),
            Box::new(move || latency_report(terminal, PRESSES, &[mode], adapters)),
        ));
    }
    for load in Load::ALL {
        parts.push((
            format!("salida {}", load.name()),
            Box::new(move || output_report(terminal, &[load], REPEATS, adapters)),
        ));
    }
    parts.push((
        "memoria".into(),
        Box::new(move || memory_report(terminal, adapters)),
    ));
    parts
}

/// The first `<base>/<date>-<terminal>-<n>` with no report yet.
fn fresh_folder(files: &dyn FileStore, base: &Path, report: &Report) -> Result<PathBuf, String> {
    (1..=999)
        .map(|n| base.join(format!("{}-{}-{n}", report.date, report.terminal.name)))
        .find(|folder| !files.exists(&folder.join(REPORT_FILE)))
        .ok_or_else(|| format!("no queda una carpeta libre en {}", base.display()))
}

/// What a run of the whole baseline left.
#[derive(Debug)]
pub struct All {
    pub report: Report,
    /// The file with the report.
    pub file: PathBuf,
}

/// Where the report goes: the folder asked for, or a new one under
/// `bench/results` of the current folder.
fn target(
    out_dir: Option<&Path>,
    report: &Report,
    adapters: &impl Adapters,
) -> Result<PathBuf, String> {
    let folder = match out_dir {
        Some(folder) => folder.to_path_buf(),
        None => {
            let base = adapters.current_dir()?.join("bench").join("results");
            fresh_folder(adapters.files(), &base, report)?
        }
    };
    let file = folder.join(REPORT_FILE);
    if adapters.files().exists(&file) {
        return Err(format!(
            "ya hay un informe en {}: no se sobrescribe",
            file.display()
        ));
    }
    Ok(file)
}

pub fn run_all<A: Adapters>(
    terminal: &str,
    out_dir: Option<&Path>,
    adapters: &A,
) -> Result<All, String> {
    let mut report = empty_report(terminal, adapters)?;
    let file = target(out_dir, &report, adapters)?;
    log_info!(
        "toda la línea base de {terminal}: {ESTIMATE}. No toques el PC hasta que termine; el informe queda en {}",
        file.display()
    );
    let save = |report: &Report| {
        let json = public_json(report, &adapters.machine());
        adapters.files().write(&file, json.as_bytes())
    };
    // Written before anything is measured: a folder that cannot be written is
    // found out now, and not after half an hour.
    save(&report)?;
    for (name, measure) in parts(terminal, adapters) {
        log_info!("midiendo: {name}");
        match measure() {
            Ok(part) => report.measurements.extend(part.measurements),
            Err(error) => report.notes.push(format!("Falló {name}: {error}")),
        }
        save(&report)?;
    }
    Ok(All { report, file })
}

#[cfg(test)]
mod tests;
