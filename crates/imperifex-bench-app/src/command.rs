//! What the user asks of the bench on the command line, and what it prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

use std::path::PathBuf;

use imperifex_bench_domain::emit::BYTES_PER_MB;
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::{FileStore, Output as TerminalOutput};
use imperifex_domain::ports::Clock;

use crate::emit::EmitService;

/// The size of a load when none is asked for. ASCII is the fastest to take in,
/// so it needs more to be timed well.
const DEFAULT_MIB: usize = 32;
const DEFAULT_ASCII_MIB: usize = 100;
/// The whole load is held in memory, so it has a limit.
const MAX_MIB: usize = 1024;
const DEFAULT_SEED: u64 = 1;

/// What `emit` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitOrder {
    pub load: Load,
    pub mib: usize,
    pub seed: u64,
    /// Where the result is left, as JSON.
    pub out: PathBuf,
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Emit(EmitOrder),
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
";

/// The arguments still to be read.
type Args<'a> = dyn Iterator<Item = &'a str> + 'a;

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
    while let Some(option) = args.next() {
        if !["--mib", "--seed", "--out"].contains(&option) {
            return Err(format!("opción desconocida: {option}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("falta el valor de {option}"))?;
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
            Some(other) => Err(format!("orden desconocida: {other}")),
        }
    }

    /// Runs it. The terminal's output is opened only by the orders that write
    /// to it, and closed before what is printed afterwards.
    pub fn run<C, O>(
        self,
        emit: &EmitService<C>,
        files: &dyn FileStore,
        open_output: impl FnOnce() -> Result<O, String>,
    ) -> Output
    where
        C: Clock,
        O: TerminalOutput,
    {
        let done = match self {
            Self::Help => Ok(HELP.to_string()),
            Self::Emit(order) => run_emit(&order, emit, files, open_output),
        };
        match done {
            Ok(text) => Output { text, ok: true },
            Err(text) => Output { text, ok: false },
        }
    }
}

fn run_emit<C: Clock, O: TerminalOutput>(
    order: &EmitOrder,
    emit: &EmitService<C>,
    files: &dyn FileStore,
    open_output: impl FnOnce() -> Result<O, String>,
) -> Result<String, String> {
    let result = {
        let mut out = open_output()?;
        emit.run(&mut out, order.load, order.mib * BYTES_PER_MB, order.seed)?
    };
    EmitService::<C>::save(files, &order.out, &result)?;
    Ok(result.to_text())
}

#[cfg(test)]
mod tests;
