//! A shell in use in the window: what it writes is read a little at a time so
//! that the window goes on answering, what is typed reaches it, and its size
//! follows the window's once that stops changing.

use std::time::Duration;

use imperifex_domain::grid::Position;
use imperifex_domain::input::{bytes, KeyInput};
use imperifex_domain::log_warn;
use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser, Wake};
use imperifex_domain::resize_wait::ResizeWait;
use imperifex_domain::shell::opening_lines;
use imperifex_domain::terminal::Terminal;

use crate::session::{Session, Step};

/// The longest time spent reading before the window gets a turn.
const READING: Duration = Duration::from_millis(8);

/// What reading the output did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Read {
    /// What is seen of the terminal changed.
    pub redraw: bool,
    /// Output is left: read again after drawing.
    pub more: bool,
    /// The shell is gone: there will be no more output.
    pub ended: bool,
}

/// What of the cursor is seen: where it is, if it is shown.
type Cursor = Option<Position>;

pub struct ShellTab {
    session: Session,
    wait: ResizeWait,
    cursor: Cursor,
}

/// What opening a shell needs.
pub struct Opening<'a> {
    pub launcher: &'a dyn PtyLauncher,
    pub new_parser: &'a dyn Fn() -> Box<dyn SequenceParser>,
    /// The one chosen in the settings, if any.
    pub chosen: Option<&'a str>,
    /// Columns and rows.
    pub size: (u16, u16),
    pub wake: Wake,
}

impl ShellTab {
    /// Opens the first shell that starts. The error is that of the last one
    /// tried.
    pub fn open(opening: &Opening<'_>) -> Result<Self, String> {
        let (columns, rows) = opening.size;
        let mut error = String::new();
        for line in opening_lines(opening.chosen) {
            let wake = opening.wake.clone();
            match opening.launcher.start(&line, columns, rows, wake) {
                Ok(pty) => {
                    let terminal = Terminal::new(columns, rows);
                    return Ok(Self {
                        session: Session::new(pty, (opening.new_parser)(), terminal),
                        wait: ResizeWait::new(opening.size),
                        cursor: None,
                    });
                }
                Err(why) => {
                    log_warn!("{line} did not start: {why}");
                    error = why;
                }
            }
        }
        Err(error)
    }

    pub fn terminal(&self) -> &Terminal {
        &self.session.terminal
    }

    /// For whoever draws it, which takes its damage.
    pub fn terminal_mut(&mut self) -> &mut Terminal {
        &mut self.session.terminal
    }

    /// Reads the output there is, for 8 ms at most. Once the program has left,
    /// its console is closed: what is left of the output arrives and ends.
    pub fn read(&mut self, clock: &dyn Clock) -> Read {
        let start = clock.monotonic();
        let mut read = Read::default();
        loop {
            match self.session.step(Duration::ZERO) {
                Step::Quiet => break,
                Step::Ended => {
                    read.ended = true;
                    break;
                }
                Step::Changed => {}
            }
            if clock.monotonic().saturating_sub(start) >= READING {
                read.more = true;
                break;
            }
        }
        if !read.ended && matches!(self.session.exit_code(), Ok(Some(_))) {
            self.session.close();
        }
        read.redraw = self.looks_different();
        read
    }

    /// Whether a row changed, or the cursor that is seen.
    fn looks_different(&mut self) -> bool {
        let terminal = &self.session.terminal;
        let seen = terminal.modes.cursor_visible;
        let cursor = seen.then(|| terminal.screen().cursor());
        let moved = cursor != self.cursor;
        self.cursor = cursor;
        moved || terminal.screen().is_damaged()
    }

    /// Sends what a key writes. A key that writes nothing sends nothing.
    pub fn typed(&mut self, input: &KeyInput) -> Result<(), String> {
        let sent = bytes(input, self.session.terminal.modes);
        if sent.is_empty() {
            return Ok(());
        }
        self.session.send(&sent)
    }

    /// There is now room for `size`, columns and rows.
    pub fn room_for(&mut self, size: (u16, u16), now: Duration) {
        self.wait.asked(size, now);
    }

    /// Takes the new size if it has settled; `true` if the terminal changed.
    /// A console that refuses it keeps the old one.
    pub fn resize_if_settled(&mut self, now: Duration) -> bool {
        let Some((columns, rows)) = self.wait.due(now) else {
            return false;
        };
        let resized = self.session.resize(columns, rows);
        if let Err(error) = &resized {
            log_warn!("the console did not take {columns}x{rows}: {error}");
        }
        resized.is_ok()
    }

    /// How long from `now` until [`Self::resize_if_settled`] has work.
    pub fn resize_in(&self, now: Duration) -> Option<Duration> {
        self.wait.due_in(now)
    }
}

#[cfg(test)]
mod tests;
