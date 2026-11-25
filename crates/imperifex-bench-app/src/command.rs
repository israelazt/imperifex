//! What the user asks of the bench on the command line, and what it prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

use std::path::PathBuf;

use imperifex_bench_domain::emit::BYTES_PER_MB;
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::{
    FileStore, GpuProbe, MachineProbe, Output as TerminalOutput, ProcessProbe,
};
use imperifex_domain::ports::Clock;

use crate::emit::EmitService;
use crate::footprint_report::footprint_report;

/// The size of a load when none is asked for. ASCII is the fastest to take in,
/// so it needs more to be timed well.
const DEFAULT_MIB: usize = 32;
const DEFAULT_ASCII_MIB: usize = 100;
/// The whole load is held in memory, so it has a limit.
const MAX_MIB: usize = 1024;
const DEFAULT_SEED: u64 = 1;
const DEFAULT_SECONDS: u64 = 60;
const MAX_SECONDS: u64 = 3600;

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

/// The adapters the orders run on. Each is made only when an order asks for
/// it, so that none needs what it does not use.
pub trait Adapters {
    type Clock: Clock;
    type Output: TerminalOutput;
    type Processes: ProcessProbe;
    type Gpu: GpuProbe;
    type Machine: MachineProbe;

    fn clock(&self) -> Self::Clock;
    /// The output of the terminal the program runs in.
    fn terminal_output(&self) -> Result<Self::Output, String>;
    fn files(&self) -> &dyn FileStore;
    fn processes(&self) -> Self::Processes;
    fn gpu(&self) -> Result<Self::Gpu, String>;
    fn machine(&self) -> Self::Machine;
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Emit(EmitOrder),
    Footprint(FootprintOrder),
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
";

/// The arguments still to be read.
type Args<'a> = dyn Iterator<Item = &'a str> + 'a;

/// The next option, which must be one of `known`, with its value.
fn next_option<'a>(
    args: &mut Args<'a>,
    known: &[&str],
) -> Result<Option<(&'a str, &'a str)>, String> {
    let Some(option) = args.next() else {
        return Ok(None);
    };
    if !known.contains(&option) {
        return Err(format!("opción desconocida: {option}"));
    }
    let value = args
        .next()
        .ok_or_else(|| format!("falta el valor de {option}"))?;
    Ok(Some((option, value)))
}

fn number<T: std::str::FromStr>(option: &str, value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{option} necesita un número entero, no «{value}»"))
}

fn parse_mib(value: &str) -> Result<usize, String> {
    let mib = number("--mib", value)?;
    if (1..=MAX_MIB).contains(&mib) {
        Ok(mib)
    } else {
        Err(format!("--mib va de 1 a {MAX_MIB}, no {value}"))
    }
}

fn parse_emit<'a>(args: &mut Args<'a>) -> Result<EmitOrder, String> {
    let name = args.next().ok_or("emit necesita el nombre de una carga")?;
    let load = Load::parse(name)?;
    let (mut mib, mut seed, mut out) = (None, DEFAULT_SEED, None);
    while let Some((option, value)) = next_option(args, &["--mib", "--seed", "--out"])? {
        match option {
            "--mib" => mib = Some(parse_mib(value)?),
            "--seed" => seed = number(option, value)?,
            _ => out = Some(PathBuf::from(value)),
        }
    }
    Ok(EmitOrder {
        load,
        mib: mib.unwrap_or(if load == Load::Ascii {
            DEFAULT_ASCII_MIB
        } else {
            DEFAULT_MIB
        }),
        seed,
        out: out.ok_or("emit necesita --out <archivo.json>")?,
    })
}

fn parse_seconds(value: &str) -> Result<u64, String> {
    let seconds = number("--seconds", value)?;
    if (1..=MAX_SECONDS).contains(&seconds) {
        Ok(seconds)
    } else {
        Err(format!("--seconds va de 1 a {MAX_SECONDS}, no {value}"))
    }
}

fn parse_footprint<'a>(args: &mut Args<'a>) -> Result<FootprintOrder, String> {
    let terminal = args
        .next()
        .ok_or("footprint necesita el nombre de una terminal")?;
    let mut order = FootprintOrder {
        terminal: terminal.to_string(),
        seconds: DEFAULT_SECONDS,
        out: None,
    };
    while let Some((option, value)) = next_option(args, &["--seconds", "--out"])? {
        match option {
            "--seconds" => order.seconds = parse_seconds(value)?,
            _ => order.out = Some(PathBuf::from(value)),
        }
    }
    Ok(order)
}

impl Command {
    /// Reads the arguments after the program's name. `help` is the default.
    pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Self, String> {
        let mut args = args.iter().map(AsRef::as_ref);
        match args.next() {
            None | Some("help" | "--help" | "-h") => match args.next() {
                None => Ok(Self::Help),
                Some(extra) => Err(format!("opción desconocida: {extra}")),
            },
            Some("emit") => parse_emit(&mut args).map(Self::Emit),
            Some("footprint") => parse_footprint(&mut args).map(Self::Footprint),
            Some(other) => Err(format!("orden desconocida: {other}")),
        }
    }

    /// Runs it.
    pub fn run(self, adapters: &impl Adapters) -> Output {
        let done = match self {
            Self::Help => Ok(HELP.to_string()),
            Self::Emit(order) => run_emit(&order, adapters),
            Self::Footprint(order) => run_footprint(&order, adapters),
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
    match &order.out {
        Some(path) => {
            adapters.files().write(path, report.to_json().as_bytes())?;
            Ok(format!("{}\nInforme: {}", report.to_text(), path.display()))
        }
        None => Ok(report.to_json()),
    }
}

#[cfg(test)]
mod tests;
