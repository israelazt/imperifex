//! Reading the command line into a [`Command`].

use std::path::PathBuf;

use imperifex_bench_domain::metric::Load;

use super::{
    Command, EmitOrder, FootprintOrder, LatencyOrder, MemoryOrder, OutputOrder, StartupOrder,
};
use crate::emit::default_mib;
use crate::latency::LatencyMode;

/// The whole load is held in memory, so it has a limit.
const MAX_MIB: usize = 1024;
const DEFAULT_SEED: u64 = 1;
const DEFAULT_SECONDS: u64 = 60;
const MAX_SECONDS: u64 = 3600;
const DEFAULT_RUNS: u32 = 10;
const MAX_RUNS: u32 = 100;
const DEFAULT_PRESSES: u32 = 200;
const MAX_PRESSES: u32 = 2000;
const DEFAULT_REPEATS: u32 = 3;
const MAX_REPEATS: u32 = 20;

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
    Ok(Some((option, value_of(option, args)?)))
}

fn value_of<'a>(option: &str, args: &mut Args<'a>) -> Result<&'a str, String> {
    args.next()
        .ok_or_else(|| format!("falta el valor de {option}"))
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
        mib: mib.unwrap_or_else(|| default_mib(load)),
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

/// A whole number from 1 to `max`.
fn count(option: &str, value: &str, max: u32) -> Result<u32, String> {
    let count = number(option, value)?;
    if (1..=max).contains(&count) {
        Ok(count)
    } else {
        Err(format!("{option} va de 1 a {max}, no {value}"))
    }
}

fn terminal_of<'a>(order: &str, args: &mut Args<'a>) -> Result<String, String> {
    args.next()
        .map(str::to_string)
        .ok_or_else(|| format!("{order} necesita el nombre de una terminal"))
}

fn modes(value: &str) -> Result<Vec<LatencyMode>, String> {
    match value {
        "prompt" => Ok(vec![LatencyMode::Prompt]),
        "program" => Ok(vec![LatencyMode::Program]),
        "both" => Ok(vec![LatencyMode::Prompt, LatencyMode::Program]),
        other => Err(format!("--mode es prompt, program o both, no «{other}»")),
    }
}

fn parse_latency<'a>(args: &mut Args<'a>) -> Result<LatencyOrder, String> {
    let mut order = LatencyOrder {
        terminal: terminal_of("latency", args)?,
        presses: DEFAULT_PRESSES,
        modes: vec![LatencyMode::Prompt, LatencyMode::Program],
        out: None,
    };
    while let Some((option, value)) = next_option(args, &["--presses", "--mode", "--out"])? {
        match option {
            "--presses" => order.presses = count(option, value, MAX_PRESSES)?,
            "--mode" => order.modes = modes(value)?,
            _ => order.out = Some(PathBuf::from(value)),
        }
    }
    Ok(order)
}

fn parse_output<'a>(args: &mut Args<'a>) -> Result<OutputOrder, String> {
    let mut order = OutputOrder {
        terminal: terminal_of("output", args)?,
        loads: Load::ALL.to_vec(),
        repeats: DEFAULT_REPEATS,
        out: None,
    };
    while let Some((option, value)) = next_option(args, &["--load", "--repeats", "--out"])? {
        match option {
            "--load" if value == "all" => order.loads = Load::ALL.to_vec(),
            "--load" => order.loads = vec![Load::parse(value)?],
            "--repeats" => order.repeats = count(option, value, MAX_REPEATS)?,
            _ => order.out = Some(PathBuf::from(value)),
        }
    }
    Ok(order)
}

fn parse_memory<'a>(args: &mut Args<'a>) -> Result<MemoryOrder, String> {
    let mut order = MemoryOrder {
        terminal: terminal_of("memory", args)?,
        out: None,
    };
    while let Some((_, value)) = next_option(args, &["--out"])? {
        order.out = Some(PathBuf::from(value));
    }
    Ok(order)
}

fn parse_startup<'a>(args: &mut Args<'a>) -> Result<StartupOrder, String> {
    let terminal = args
        .next()
        .ok_or("startup necesita el nombre de una terminal")?;
    let mut order = StartupOrder {
        terminal: terminal.to_string(),
        runs: DEFAULT_RUNS,
        cold: false,
        out: None,
    };
    while let Some(option) = args.next() {
        match option {
            "--cold" => order.cold = true,
            "--runs" => order.runs = count(option, value_of(option, args)?, MAX_RUNS)?,
            "--out" => order.out = Some(PathBuf::from(value_of(option, args)?)),
            other => return Err(format!("opción desconocida: {other}")),
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
            Some("echo") => match args.next() {
                None => Ok(Self::Echo),
                Some(extra) => Err(format!("opción desconocida: {extra}")),
            },
            Some("emit") => parse_emit(&mut args).map(Self::Emit),
            Some("footprint") => parse_footprint(&mut args).map(Self::Footprint),
            Some("startup") => parse_startup(&mut args).map(Self::Startup),
            Some("latency") => parse_latency(&mut args).map(Self::Latency),
            Some("output") => parse_output(&mut args).map(Self::OutputLoads),
            Some("memory") => parse_memory(&mut args).map(Self::Memory),
            Some(other) => Err(format!("orden desconocida: {other}")),
        }
    }
}
