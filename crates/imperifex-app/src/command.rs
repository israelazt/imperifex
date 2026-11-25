//! What the user asks for on the command line, and what the program prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

use imperifex_domain::ports::Clock;

use crate::status::StatusService;

/// What the command line asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Status { json: bool },
    Help,
}

/// What the program should write, and whether it went well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub ok: bool,
}

pub const HELP: &str = "Imperifex: The development terminal of the Fexarium family

Uso:
  imperifex status [--json]   Comprueba que la app funciona
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
            Some(other) => Err(format!("orden desconocida: {other}")),
        }
    }

    /// Runs it against the services.
    pub fn run<C: Clock>(self, status: &StatusService<C>) -> Output {
        match self {
            Self::Help => Output {
                text: HELP.to_string(),
                ok: true,
            },
            Self::Status { json } => {
                let report = status.check();
                Output {
                    text: if json {
                        report.to_json()
                    } else {
                        report.to_text()
                    },
                    ok: true,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeClock;

    #[test]
    fn help_is_the_default_and_status_takes_its_options() {
        let none: [&str; 0] = [];
        assert_eq!(Command::parse(&none).unwrap(), Command::Help);
        assert_eq!(
            Command::parse(&["status", "--json"]).unwrap(),
            Command::Status { json: true }
        );
    }

    #[test]
    fn what_it_does_not_know_is_an_error() {
        assert!(Command::parse(&["fly"]).unwrap_err().contains("fly"));
        assert!(Command::parse(&["status", "--fast"])
            .unwrap_err()
            .contains("--fast"));
    }

    #[test]
    fn status_prints_the_report() {
        let service = StatusService::new(FakeClock::ticking(2));
        let out = Command::Status { json: false }.run(&service);
        assert!(out.ok && out.text.contains("2 ms"), "{}", out.text);
    }
}
