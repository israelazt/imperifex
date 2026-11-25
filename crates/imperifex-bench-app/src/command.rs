//! What the user asks of the bench on the command line, and what it prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

/// What the command line asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Help,
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
";

impl Command {
    /// Reads the arguments after the program's name. `help` is the default.
    pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Self, String> {
        let mut args = args.iter().map(AsRef::as_ref);
        match args.next() {
            None | Some("help" | "--help" | "-h") => match args.next() {
                None => Ok(Self::Help),
                Some(extra) => Err(format!("opción desconocida: {extra}")),
            },
            Some(other) => Err(format!("orden desconocida: {other}")),
        }
    }

    /// Runs it.
    pub fn run(self) -> Output {
        match self {
            Self::Help => Output {
                text: HELP.to_string(),
                ok: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_the_default_and_goes_well() {
        let none: [&str; 0] = [];
        assert_eq!(Command::parse(&none).unwrap(), Command::Help);
        assert_eq!(Command::parse(&["--help"]).unwrap(), Command::Help);
        let out = Command::Help.run();
        assert!(out.ok && out.text.contains("imperifex-bench help"));
    }

    #[test]
    fn what_it_does_not_know_is_an_error() {
        assert!(Command::parse(&["fly"]).unwrap_err().contains("fly"));
        assert!(Command::parse(&["help", "--fast"])
            .unwrap_err()
            .contains("--fast"));
    }
}
