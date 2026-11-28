//! What the user asks for on the command line, and what the program prints.
//! Reading the arguments is a decision, so it is here and tested, not in `main`.

mod run;

use imperifex_domain::i18n::{fill, text, Language, Text};
use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser};

use crate::status::StatusService;

pub use run::RunOptions;

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Status {
        json: bool,
    },
    Run(RunOptions),
    /// Open the window: what happens with no arguments.
    Window,
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

impl Command {
    /// Reads the arguments after the program's name. `window` is the default.
    /// What is wrong with them is said in `language`.
    pub fn parse<S: AsRef<str>>(args: &[S], language: Language) -> Result<Self, String> {
        let mut args = args.iter().map(AsRef::as_ref);
        match args.next() {
            None => Ok(Self::Window),
            Some("window") => no_options(args, language).map(|()| Self::Window),
            Some("help" | "--help" | "-h") => Ok(Self::Help),
            Some("status") => {
                let mut json = false;
                for arg in args {
                    match arg {
                        "--json" => json = true,
                        other => return Err(fill(language, Text::UnknownOption, &[&other])),
                    }
                }
                Ok(Self::Status { json })
            }
            Some("run") => RunOptions::parse(args, language).map(Self::Run),
            Some(other) => Err(fill(language, Text::UnknownCommand, &[&other])),
        }
    }

    /// Runs it against the services. The window is not opened here: the program
    /// has its loop, and for [`Command::Window`] there is nothing to print.
    pub fn run<C: Clock>(
        &self,
        status: &StatusService<C>,
        shells: Shells<'_>,
        language: Language,
    ) -> Output {
        match self {
            Self::Window => Output::fine(String::new()),
            Self::Help => Output::fine(text(language, Text::Help).to_string()),
            Self::Status { json } => {
                let report = status.check();
                Output::fine(if *json {
                    report.to_json()
                } else {
                    report.to_text(language)
                })
            }
            Self::Run(options) => options.run(shells, language),
        }
    }
}

/// For an order that takes no options.
fn no_options<'a>(
    mut args: impl Iterator<Item = &'a str>,
    language: Language,
) -> Result<(), String> {
    match args.next() {
        None => Ok(()),
        Some(other) => Err(fill(language, Text::UnknownOption, &[&other])),
    }
}

#[cfg(test)]
mod tests;
