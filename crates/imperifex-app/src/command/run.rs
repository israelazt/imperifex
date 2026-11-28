//! `imperifex run`: reading its options and saying what the program left.

use std::time::Duration;

use imperifex_domain::command_line::command_line;
use imperifex_domain::i18n::{fill, text, Language, Text};
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

/// The arguments left to read, and the language to say what is wrong in.
struct Reader<'a, I: Iterator<Item = &'a str>> {
    args: I,
    language: Language,
}

impl<'a, I: Iterator<Item = &'a str>> Reader<'a, I> {
    /// The number that follows the option `name`, from 1 to `max`.
    fn number(&mut self, name: &str, max: u64) -> Result<u64, String> {
        let language = self.language;
        let given = self
            .args
            .next()
            .ok_or_else(|| fill(language, Text::MissingNumber, &[&name]))?;
        given
            .parse()
            .ok()
            .filter(|value| (1..=max).contains(value))
            .ok_or_else(|| fill(language, Text::NumberOutOfRange, &[&name, &max, &given]))
    }

    fn side(&mut self, name: &str) -> Result<u16, String> {
        let value = self.number(name, MAX_SIDE)?;
        u16::try_from(value).map_err(|e| format!("{name}: {e}"))
    }

    fn profile(&mut self) -> Result<ShellProfile, String> {
        let id = self.args.next().unwrap_or("");
        shell::find(id)
            .ok_or_else(|| fill(self.language, Text::UnknownProfile, &[&id, &shell::ids()]))
    }
}

/// The line to start, from what came after `--`.
fn line(profile: Option<ShellProfile>, rest: &[&str]) -> Result<String, Text> {
    match (profile, rest) {
        (None, []) => Err(Text::MissingProgram),
        (None, program) => Ok(command_line(program)),
        (Some(profile), [command]) => Ok(profile.command_line(command)),
        (Some(_), _) => Err(Text::OneCommand),
    }
}

impl RunOptions {
    /// Reads what comes after `run`.
    pub(super) fn parse<'a>(
        args: impl Iterator<Item = &'a str>,
        language: Language,
    ) -> Result<Self, String> {
        let mut reader = Reader { args, language };
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
        while let Some(arg) = reader.args.next() {
            match arg {
                "--" => break,
                "--headless" => headless = true,
                "--history" => options.history = true,
                "--profile" => options.profile = Some(reader.profile()?),
                "--columns" => options.request.columns = reader.side(arg)?,
                "--rows" => options.request.rows = reader.side(arg)?,
                "--timeout" => {
                    let seconds = reader.number(arg, 24 * 3600)?;
                    options.request.timeout = Duration::from_secs(seconds);
                }
                other => return Err(fill(language, Text::UnknownOption, &[&other])),
            }
        }
        if !headless {
            return Err(text(language, Text::NeedsHeadless).to_string());
        }
        let rest: Vec<&str> = reader.args.collect();
        options.request.command_line =
            line(options.profile, &rest).map_err(|wrong| text(language, wrong).to_string())?;
        Ok(options)
    }

    pub(super) fn run(&self, shells: Shells<'_>, language: Language) -> Output {
        let Shells {
            launcher,
            parser,
            clock,
        } = shells;
        let finished = headless::run(launcher, parser, clock, &self.request);
        match finished {
            Ok(finished) => self.report(&finished, language),
            Err(error) => Output {
                text: String::new(),
                ok: false,
                notice: Some(match self.profile {
                    Some(profile) => fill(language, Text::ShellMissing, &[&profile.name, &error]),
                    None => error,
                }),
            },
        }
    }

    fn report(&self, finished: &Finished, language: Language) -> Output {
        let notice = notice(finished, self.request.timeout, language);
        Output {
            text: shown(&finished.terminal, self.history),
            ok: notice.is_none(),
            notice,
        }
    }
}

/// What is on the terminal, without the empty lines at the end.
fn shown(terminal: &Terminal, history: bool) -> String {
    let screen = terminal.screen();
    let mut output = if history && !terminal.on_alternate_screen() {
        screen.text_with_history()
    } else {
        screen.text()
    };
    output.truncate(output.trim_end_matches('\n').len());
    output
}

/// What went wrong, if something did.
fn notice(finished: &Finished, timeout: Duration, language: Language) -> Option<String> {
    let ending = match finished.ending {
        Ending::Left(0) => None,
        Ending::Left(code) => Some(fill(language, Text::ExitedWith, &[&code])),
        Ending::TimedOut => Some(fill(language, Text::TimedOut, &[&timeout.as_secs()])),
        Ending::StillRunning { process_id } => {
            Some(fill(language, Text::StillRunning, &[&process_id]))
        }
    };
    let cut = (!finished.output_complete).then(|| text(language, Text::OutputCut).to_string());
    let all: Vec<String> = ending.into_iter().chain(cut).collect();
    (!all.is_empty()).then(|| all.join("; "))
}
