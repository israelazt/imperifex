//! `imperifex run`: reading its options and saying what the program left.

use std::time::Duration;

use imperifex_domain::command_line::command_line;
use imperifex_domain::shell::{self, ShellProfile};
use imperifex_domain::terminal::Terminal;

use crate::headless::{self, Ending, Finished, Request};

use super::{Output, Shells};

const DEFAULT_COLUMNS: u16 = 80;
const DEFAULT_ROWS: u16 = 25;
const DEFAULT_TIMEOUT_S: u64 = 30;
/// The most columns or rows asked for: what a screen takes.
const MAX_SIDE: u64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOptions {
    pub request: Request,
    /// Print the history too, and not only the screen.
    pub history: bool,
    /// The shell that runs the command, if one was named.
    pub profile: Option<ShellProfile>,
}

fn number<'a>(
    name: &str,
    args: &mut impl Iterator<Item = &'a str>,
    max: u64,
) -> Result<u64, String> {
    let text = args
        .next()
        .ok_or_else(|| format!("falta el número de {name}"))?;
    text.parse()
        .ok()
        .filter(|value| (1..=max).contains(value))
        .ok_or_else(|| format!("{name} pide un número de 1 a {max}, no «{text}»"))
}

fn profile<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<ShellProfile, String> {
    let id = args.next().unwrap_or("");
    shell::find(id)
        .ok_or_else(|| format!("no hay un perfil «{id}»; los que hay son: {}", shell::ids()))
}

/// The line to start, from what came after `--`.
fn line(profile: Option<ShellProfile>, rest: &[&str]) -> Result<String, String> {
    match (profile, rest) {
        (None, []) => Err("falta el programa: run --headless -- <programa> [argumentos]".into()),
        (None, program) => Ok(command_line(program)),
        (Some(profile), [command]) => Ok(profile.command_line(command)),
        (Some(_), _) => Err("con --profile va una sola orden entre comillas después de --".into()),
    }
}

fn side<'a>(name: &str, args: &mut impl Iterator<Item = &'a str>) -> Result<u16, String> {
    let value = number(name, args, MAX_SIDE)?;
    u16::try_from(value).map_err(|e| format!("{name}: {e}"))
}

impl RunOptions {
    /// Reads what comes after `run`.
    pub(super) fn parse<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<Self, String> {
        let mut options = Self {
            request: Request {
                command_line: String::new(),
                columns: DEFAULT_COLUMNS,
                rows: DEFAULT_ROWS,
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_S),
            },
            history: false,
            profile: None,
        };
        let mut headless = false;
        while let Some(arg) = args.next() {
            match arg {
                "--" => break,
                "--headless" => headless = true,
                "--history" => options.history = true,
                "--profile" => options.profile = Some(profile(&mut args)?),
                "--columns" => options.request.columns = side(arg, &mut args)?,
                "--rows" => options.request.rows = side(arg, &mut args)?,
                "--timeout" => {
                    let seconds = number(arg, &mut args, 24 * 3600)?;
                    options.request.timeout = Duration::from_secs(seconds);
                }
                other => return Err(format!("opción desconocida: {other}")),
            }
        }
        if !headless {
            return Err("todavía no hay ventana: usa run --headless".into());
        }
        let rest: Vec<&str> = args.collect();
        options.request.command_line = line(options.profile, &rest)?;
        Ok(options)
    }

    pub(super) fn run(&self, shells: Shells<'_>) -> Output {
        let Shells {
            launcher,
            parser,
            clock,
        } = shells;
        let finished = headless::run(launcher, parser, clock, &self.request);
        match finished {
            Ok(finished) => report(&finished, self.history, self.request.timeout),
            Err(error) => Output {
                text: String::new(),
                ok: false,
                notice: Some(match self.profile {
                    Some(profile) => format!("{} (¿está instalado?): {error}", profile.name),
                    None => error,
                }),
            },
        }
    }
}

/// What is on the terminal, without the empty lines at the end.
fn shown(terminal: &Terminal, history: bool) -> String {
    let screen = terminal.screen();
    let mut text = if history && !terminal.on_alternate_screen() {
        screen.text_with_history()
    } else {
        screen.text()
    };
    text.truncate(text.trim_end_matches('\n').len());
    text
}

/// What went wrong, if something did.
fn notice(finished: &Finished, timeout: Duration) -> Option<String> {
    let ending = match finished.ending {
        Ending::Left(0) => None,
        Ending::Left(code) => Some(format!("el programa salió con el código {code}")),
        Ending::TimedOut => Some(format!(
            "no terminó en {} s: se cerró su consola",
            timeout.as_secs()
        )),
        Ending::StillRunning { process_id } => Some(format!(
            "sigue vivo después de cerrar su consola: proceso {process_id} (no se termina)"
        )),
    };
    let cut = (!finished.output_complete)
        .then(|| "la salida no terminó de llegar: puede faltar el final".to_string());
    let all: Vec<String> = ending.into_iter().chain(cut).collect();
    (!all.is_empty()).then(|| all.join("; "))
}

fn report(finished: &Finished, history: bool, timeout: Duration) -> Output {
    let notice = notice(finished, timeout);
    Output {
        text: shown(&finished.terminal, history),
        ok: notice.is_none(),
        notice,
    }
}
