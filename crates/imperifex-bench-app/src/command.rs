//! What the user asks of the bench on the command line, and what it prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

use std::path::{Path, PathBuf};

use imperifex_bench_domain::emit::BYTES_PER_MB;
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::{
    Desktop, FileStore, GpuProbe, KeyInput, Keyboard, MachineProbe, Output as TerminalOutput,
    ProcessProbe, ScreenWatch,
};
use imperifex_bench_domain::report::Report;
use imperifex_domain::ports::Clock;

use crate::all::run_all;
use crate::baseline::write_baseline;
use crate::echo::run_echo;
use crate::emit::EmitService;
use crate::latency::LatencyMode;
use crate::reports::{
    footprint_report, latency_report, memory_report, output_report, public_json, startup_report,
};

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

/// What `latency` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyOrder {
    pub terminal: String,
    pub presses: u32,
    pub modes: Vec<LatencyMode>,
    pub out: Option<PathBuf>,
}

/// What `output` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputOrder {
    pub terminal: String,
    pub loads: Vec<Load>,
    pub repeats: u32,
    pub out: Option<PathBuf>,
}

/// What `memory` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryOrder {
    pub terminal: String,
    pub out: Option<PathBuf>,
}

/// What `all` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllOrder {
    pub terminal: String,
    /// Where the report goes; without it, a new folder under `bench/results`.
    pub out_dir: Option<PathBuf>,
}

/// What `report` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportOrder {
    pub folders: Vec<PathBuf>,
    pub markdown: PathBuf,
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
    type Screen: ScreenWatch;
    type Input: KeyInput;
    type Keyboard: Keyboard;

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
    fn screen(&self) -> Self::Screen;
    fn keyboard(&self) -> Self::Keyboard;
    /// The bench's own program, for a window to run it.
    fn current_exe(&self) -> Result<String, String>;
    /// The program's own input, key by key.
    fn key_input(&self) -> Result<Self::Input, String>;
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Echo,
    Emit(EmitOrder),
    Footprint(FootprintOrder),
    Startup(StartupOrder),
    Latency(LatencyOrder),
    OutputLoads(OutputOrder),
    Memory(MemoryOrder),
    All(AllOrder),
    Report(ReportOrder),
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
  imperifex-bench echo
      Repite cada tecla hasta la «q». Lo usa la medida de latencia.
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
      por la espera de lo que deja cada ventana. Con --cold la terminal tiene
      que estar cerrada del todo (README).

  Las tres que siguen ABREN UNA VENTANA Y ESCRIBEN EN ELLA. No toques el
  teclado ni el ratón mientras corren.
  imperifex-bench latency <terminal> [--presses 200] [--mode prompt|program|both]
      Cuánto tarda una tecla en verse, en el prompt y en un programa.
  imperifex-bench output <terminal> [--load <carga>|all] [--repeats 3]
      Cuánto tarda en verse una carga, a qué velocidad y con cuánta CPU.
  imperifex-bench memory <terminal>
      Memoria de un historial largo y de cada pestaña.
  Todas admiten --out <archivo.json>.

  imperifex-bench all <terminal> [--out-dir <carpeta>]
      TODA LA LÍNEA BASE, unos 35 minutos (sin medir) sin tocar el PC. Deja
      report.json; sin --out-dir, en bench/results/<fecha>-<terminal>-<n> de la
      carpeta actual.
  imperifex-bench report <carpeta>... --md <archivo.md>
      El documento de la línea base a partir de los report.json de esas carpetas.
";

impl Command {
    /// Runs it.
    pub fn run(self, adapters: &impl Adapters) -> Output {
        let done = match self {
            Self::Help => Ok(HELP.to_string()),
            Self::Echo => run_echo_order(adapters),
            Self::Emit(order) => run_emit(&order, adapters),
            Self::Footprint(order) => run_footprint(&order, adapters),
            Self::Startup(order) => run_startup(&order, adapters),
            Self::Latency(order) => run_latency(&order, adapters),
            Self::OutputLoads(order) => run_output(&order, adapters),
            Self::Memory(order) => run_memory(&order, adapters),
            Self::All(order) => run_all_order(&order, adapters),
            Self::Report(order) => run_report(&order, adapters),
        };
        match done {
            Ok(text) => Output { text, ok: true },
            Err(text) => Output { text, ok: false },
        }
    }
}

fn run_echo_order(adapters: &impl Adapters) -> Result<String, String> {
    let mut input = adapters.key_input()?;
    let mut out = adapters.terminal_output()?;
    run_echo(&mut input, &mut out).map(|()| String::new())
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

fn run_latency(order: &LatencyOrder, adapters: &impl Adapters) -> Result<String, String> {
    let report = latency_report(&order.terminal, order.presses, &order.modes, adapters)?;
    deliver(&report, order.out.as_deref(), adapters)
}

fn run_output(order: &OutputOrder, adapters: &impl Adapters) -> Result<String, String> {
    let report = output_report(&order.terminal, &order.loads, order.repeats, adapters)?;
    deliver(&report, order.out.as_deref(), adapters)
}

fn run_memory(order: &MemoryOrder, adapters: &impl Adapters) -> Result<String, String> {
    let report = memory_report(&order.terminal, adapters)?;
    deliver(&report, order.out.as_deref(), adapters)
}

/// A run with a part that failed does not count as having gone well.
fn run_all_order(order: &AllOrder, adapters: &impl Adapters) -> Result<String, String> {
    let all = run_all(&order.terminal, order.out_dir.as_deref(), adapters)?;
    let mut text = format!("{}\nInforme: {}", all.report.to_text(), all.file.display());
    for note in &all.report.notes {
        text.push_str(&format!("\n{note}"));
    }
    if all.report.notes.is_empty() {
        Ok(text)
    } else {
        Err(text)
    }
}

fn run_report(order: &ReportOrder, adapters: &impl Adapters) -> Result<String, String> {
    write_baseline(adapters.files(), &order.folders, &order.markdown)?;
    Ok(format!("Escrito {}", order.markdown.display()))
}

/// With a file, the report is kept there and its summary printed; without
/// one, the report itself is printed.
fn deliver(
    report: &Report,
    out: Option<&Path>,
    adapters: &impl Adapters,
) -> Result<String, String> {
    let json = public_json(report, &adapters.machine());
    match out {
        Some(path) => {
            adapters.files().write(path, json.as_bytes())?;
            Ok(format!("{}\nInforme: {}", report.to_text(), path.display()))
        }
        None => Ok(json),
    }
}

#[cfg(test)]
mod tests;
