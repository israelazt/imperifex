//! What the user asks for on the command line, and what the program prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

mod run;

use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser};

use crate::status::StatusService;

pub use run::RunOptions;

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Status { json: bool },
    Run(RunOptions),
    Help,
}

/// What the program should write, and whether it went well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub ok: bool,
    /// What the user must know besides the text.
    pub notice: Option<String>,
}

impl Output {
    fn fine(text: String) -> Self {
        Self {
            text,
            ok: true,
            notice: None,
        }
    }
}

/// What running a program needs from the system.
pub struct Shells<'a> {
    pub launcher: &'a dyn PtyLauncher,
    pub parser: &'a mut dyn SequenceParser,
    pub clock: &'a dyn Clock,
}

pub const HELP: &str = "Imperifex: The development terminal of the Fexarium family

Uso:
  imperifex status [--json]   Comprueba que la app funciona
  imperifex run --headless [--columns N] [--rows N] [--timeout S] [--history]
                -- <programa> [argumentos]
      Corre un programa en una consola sin ventana y escribe la pantalla que
      deja (80x25 y 30 s si no se dice otra cosa). --history escribe también
      lo que salió por arriba.
  imperifex run --headless --profile <perfil> [opciones] -- \"<orden>\"
      Lo mismo, con la orden escrita para una shell: pwsh, powershell, cmd o wsl.
  imperifex help              Esta ayuda
";

impl Command {
    /// Reads the arguments after the program's name. `help` is the default.
    pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Self, String> {
        let mut args = args.iter().map(AsRef::as_ref);
        match args.next() {
            None | Some("help" | "--help" | "-h") => Ok(Self::Help),
            Some("status") => {
                let mut json = false;
                for arg in args {
                    match arg {
                        "--json" => json = true,
                        other => return Err(format!("opción desconocida: {other}")),
                    }
                }
                Ok(Self::Status { json })
            }
            Some("run") => RunOptions::parse(args).map(Self::Run),
            Some(other) => Err(format!("orden desconocida: {other}")),
        }
    }

    /// Runs it against the services.
    pub fn run<C: Clock>(&self, status: &StatusService<C>, shells: Shells<'_>) -> Output {
        match self {
            Self::Help => Output::fine(HELP.to_string()),
            Self::Status { json } => {
                let report = status.check();
                Output::fine(if *json {
                    report.to_json()
                } else {
                    report.to_text()
                })
            }
            Self::Run(options) => options.run(shells),
        }
    }
}

#[cfg(test)]
mod tests;
