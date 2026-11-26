//! What the user asks of the bench on the command line, and what it prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

use std::path::{Path, PathBuf};

use imperifex_bench_domain::emit::BYTES_PER_MB;
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::{
    Desktop, FileStore, GpuProbe, MachineProbe, Output as TerminalOutput, ProcessProbe,
};
use imperifex_bench_domain::report::Report;
use imperifex_domain::ports::Clock;

use crate::emit::EmitService;
use crate::reports::{footprint_report, startup_report};

mod parse;

/// What `emit` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitOrder {
    pub load: Load,
    pub mib: usize,
    pub seed: u64,
    /// Where the result is left, as JSON.
    pub out: PathBuf,
}

/// What `footprint` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintOrder {
    pub terminal: String,
    pub seconds: u64,
    /// Where the report is left; without it, the report is printed.
    pub out: Option<PathBuf>,
}

/// What `startup` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupOrder {
    pub terminal: String,
    pub runs: u32,
    pub cold: bool,
    pub out: Option<PathBuf>,
}

/// The adapters the orders run on. Each is made only when an order asks for
/// it, so that none needs what it does not use.
pub trait Adapters {
    type Clock: Clock;
    type Output: TerminalOutput;
    type Processes: ProcessProbe;
    type Gpu: GpuProbe;
    type Machine: MachineProbe;
    type Desktop: Desktop;

    fn clock(&self) -> Self::Clock;
    /// The output of the terminal the program runs in.
    fn terminal_output(&self) -> Result<Self::Output, String>;
    fn files(&self) -> &dyn FileStore;
    /// The folder the bench was started in.
    fn current_dir(&self) -> Result<PathBuf, String>;
    fn processes(&self) -> Self::Processes;
    fn gpu(&self) -> Result<Self::Gpu, String>;
    fn machine(&self) -> Self::Machine;
    fn desktop(&self) -> Self::Desktop;
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Emit(EmitOrder),
    Footprint(FootprintOrder),
    Startup(StartupOrder),
}

/// What the program should write, and whether it went well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub ok: bool,
}

pub const HELP: &str =
    "imperifex-bench: mide terminales con la misma herramienta y el mismo protocolo

Uso:
  imperifex-bench help   Esta ayuda
  imperifex-bench emit <carga> --out <archivo.json> [--mib N] [--seed S]
      Manda una carga a la terminal en la que corre y mide cuánto tarda en
      aceptarla. Cargas: ascii, long_lines, color, unicode, cursor, cargo.
      Tamaño por defecto: 32 MiB (ascii: 100). Semilla por defecto: 1.
  imperifex-bench footprint <terminal> [--seconds N] [--out <archivo.json>]
      Mide memoria, CPU y GPU de una terminal que ya está abierta, con una
      muestra por segundo durante 60 s. Solo lee: no abre ni cierra nada.
      Para medirla en reposo, no la uses mientras corre.
  imperifex-bench startup <terminal> [--runs N] [--cold] [--out <archivo.json>]
      ABRE Y CIERRA VENTANAS de la terminal, 10 veces, y mide cuánto tarda en
      verse cada una. No uses el PC mientras corre: cerca de un minuto por vez,
      por la espera de lo que deja cada ventana. Se lanza desde una carpeta sin
      Git. Con --cold la terminal tiene que estar cerrada del todo (README).
";

impl Command {
    /// Runs it.
    pub fn run(self, adapters: &impl Adapters) -> Output {
        let done = match self {
            Self::Help => Ok(HELP.to_string()),
            Self::Emit(order) => run_emit(&order, adapters),
            Self::Footprint(order) => run_footprint(&order, adapters),
            Self::Startup(order) => run_startup(&order, adapters),
        };
        match done {
            Ok(text) => Output { text, ok: true },
            Err(text) => Output { text, ok: false },
        }
    }
}

/// The terminal's output is closed before anything is printed afterwards.
fn run_emit<A: Adapters>(order: &EmitOrder, adapters: &A) -> Result<String, String> {
    let result = {
        let mut out = adapters.terminal_output()?;
        EmitService::new(adapters.clock()).run(
            &mut out,
            order.load,
            order.mib * BYTES_PER_MB,
            order.seed,
        )?
    };
    EmitService::<A::Clock>::save(adapters.files(), &order.out, &result)?;
    Ok(result.to_text())
}

fn run_footprint(order: &FootprintOrder, adapters: &impl Adapters) -> Result<String, String> {
    let report = footprint_report(&order.terminal, order.seconds, adapters)?;
    deliver(&report, order.out.as_deref(), adapters)
}

fn run_startup(order: &StartupOrder, adapters: &impl Adapters) -> Result<String, String> {
    let report = startup_report(&order.terminal, order.runs, order.cold, adapters)?;
    deliver(&report, order.out.as_deref(), adapters)
}

/// With a file, the report is kept there and its summary printed; without
/// one, the report itself is printed.
fn deliver(
    report: &Report,
    out: Option<&Path>,
    adapters: &impl Adapters,
) -> Result<String, String> {
    match out {
        Some(path) => {
            adapters.files().write(path, report.to_json().as_bytes())?;
            Ok(format!("{}\nInforme: {}", report.to_text(), path.display()))
        }
        None => Ok(report.to_json()),
    }
}

#[cfg(test)]
mod tests;
