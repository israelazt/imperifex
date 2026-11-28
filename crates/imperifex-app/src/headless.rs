//! Running a program to its end with no window, to see the screen it leaves.

use std::sync::Arc;
use std::time::Duration;

use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser};
use imperifex_domain::terminal::Terminal;

use crate::session::{Session, Step};

/// How long one wait for output is.
const TICK: Duration = Duration::from_millis(20);
/// How long without output, once the program left, means it has all arrived.
const QUIET_AFTER_EXIT: Duration = Duration::from_millis(200);
/// How long closing may take: the rest of the output and the program leaving.
const TO_CLOSE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The whole line, already quoted.
    pub command_line: String,
    pub columns: u16,
    pub rows: u16,
    /// After this the console is closed, whatever the program is doing.
    pub timeout: Duration,
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// The program left by itself, with this code.
    Left(u32),
    /// Time ran out: its console was closed and then it left.
    TimedOut,
    /// It was still running after its console was closed. It is left alone.
    StillRunning { process_id: u32 },
}

pub struct Finished {
    pub terminal: Terminal,
    pub ending: Ending,
    /// False when the output had not ended after closing: the screen may lack
    /// the last of it.
    pub output_complete: bool,
}

pub fn run(
    launcher: &dyn PtyLauncher,
    parser: Box<dyn SequenceParser>,
    clock: &dyn Clock,
    request: &Request,
) -> Result<Finished, String> {
    // Nobody needs telling: the run waits for the output itself.
    let nobody = Arc::new(|| {});
    let pty = launcher.start(&request.command_line, request.columns, request.rows, nobody)?;
    let terminal = Terminal::new(request.columns, request.rows);
    let mut session = Session::new(pty, parser, terminal);
    let timed_out = wait_for_the_program(&mut session, clock, request.timeout)?;
    let output_complete = close_and_read(&mut session, clock);
    let ending = ending(&mut session, clock, timed_out)?;
    Ok(Finished {
        terminal: session.terminal,
        ending,
        output_complete,
    })
}

/// Reads until the program has left and gone quiet; `true` if time ran out
/// first.
fn wait_for_the_program(
    session: &mut Session,
    clock: &dyn Clock,
    timeout: Duration,
) -> Result<bool, String> {
    let start = clock.monotonic();
    let mut last_output = start;
    loop {
        let step = session.step(TICK);
        let now = clock.monotonic();
        match step {
            Step::Changed => last_output = now,
            Step::Ended => return Ok(false),
            Step::Quiet => {
                let quiet = now.saturating_sub(last_output) >= QUIET_AFTER_EXIT;
                if quiet && session.exit_code()?.is_some() {
                    return Ok(false);
                }
            }
        }
        if now.saturating_sub(start) >= timeout {
            // A program that already left did not run out of time.
            return Ok(session.exit_code()?.is_none());
        }
    }
}

/// Closes the console and reads what is left; `false` if it did not end.
fn close_and_read(session: &mut Session, clock: &dyn Clock) -> bool {
    session.close();
    let start = clock.monotonic();
    while clock.monotonic().saturating_sub(start) < TO_CLOSE {
        if session.step(TICK) == Step::Ended {
            return true;
        }
    }
    false
}

fn ending(session: &mut Session, clock: &dyn Clock, timed_out: bool) -> Result<Ending, String> {
    let start = clock.monotonic();
    loop {
        if let Some(code) = session.exit_code()? {
            return Ok(if timed_out {
                Ending::TimedOut
            } else {
                Ending::Left(code)
            });
        }
        if clock.monotonic().saturating_sub(start) >= TO_CLOSE {
            return Ok(Ending::StillRunning {
                process_id: session.process_id(),
            });
        }
        clock.sleep(TICK);
    }
}

#[cfg(test)]
mod tests;
