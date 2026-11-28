//! A shell in use in the window: what it writes is read a little at a time so
//! that the window goes on answering, what is typed reaches it, and its size
//! follows the window's once that stops changing.

use std::time::Duration;

use imperifex_domain::grid::Position;
use imperifex_domain::input::{bytes, KeyInput};
use imperifex_domain::log_warn;
use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser, Wake};
use imperifex_domain::resize_wait::ResizeWait;
use imperifex_domain::scrollback::{Turned, View, Wheel};
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
    /// The line that opened it.
    line: String,
    wait: ResizeWait,
    cursor: Cursor,
    view: View,
    wheel: Wheel,
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
                        line,
                        session: Session::new(pty, (opening.new_parser)(), terminal),
                        wait: ResizeWait::new(opening.size),
                        cursor: None,
                        view: View::default(),
                        wheel: Wheel::default(),
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

    pub fn line(&self) -> &str {
        &self.line
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
        self.view.follow(self.session.terminal.screen());
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

    /// Sends what a key writes, and goes back to the screen, where what it
    /// does is seen. A key that writes nothing sends nothing. `true` if the
    /// view moved.
    pub fn typed(&mut self, input: &KeyInput) -> Result<bool, String> {
        let sent = bytes(input, self.session.terminal.modes);
        if sent.is_empty() {
            return Ok(false);
        }
        let moved = self.view.back() > 0;
        self.view.to_screen();
        self.session.send(&sent)?;
        Ok(moved)
    }

    /// How many rows above the screen the view is.
    pub fn back(&self) -> usize {
        self.view.back()
    }

    /// The wheel turned `notches`, up when positive, with the pointer on the
    /// cell `at`. `true` if the view moved.
    pub fn wheel(&mut self, notches: f32, at: (u16, u16)) -> Result<bool, String> {
        let terminal = &self.session.terminal;
        let alternate = terminal.on_alternate_screen();
        match self.wheel.turn(notches, terminal.modes, alternate, at) {
            Turned::Nothing => Ok(false),
            Turned::Scroll(rows) => Ok(self.scroll(rows)),
            Turned::Send(sent) => self.session.send(&sent).map(|()| false),
        }
    }

    /// Moves the view a screen back, or forward, keeping one row of the last.
    /// `true` if it moved.
    pub fn page(&mut self, back: bool) -> bool {
        let rows = self.session.terminal.screen().rows();
        let step = i32::try_from(rows.saturating_sub(1).max(1)).unwrap_or(i32::MAX);
        self.scroll(if back { step } else { -step })
    }

    fn scroll(&mut self, rows: i32) -> bool {
        let before = self.view.back();
        self.view.scroll(rows, self.session.terminal.screen());
        self.view.back() != before
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
        // Its lines are cut again: where the view was is not that text now.
        self.view.to_screen();
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
