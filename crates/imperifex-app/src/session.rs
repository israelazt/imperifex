//! One shell and its screen: what the program writes reaches the terminal, and
//! what changes the size reaches both.

use std::time::Duration;

use imperifex_domain::interpret::apply;
use imperifex_domain::ports::{Pty, PtyRead, SequenceParser};
use imperifex_domain::terminal::Terminal;

pub struct Session {
    pty: Box<dyn Pty>,
    /// Each shell has its own: its output may be cut anywhere.
    parser: Box<dyn SequenceParser>,
    pub terminal: Terminal,
}

/// What one step of a session did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Output arrived and is on the screen.
    Changed,
    Quiet,
    /// There will be no more output.
    Ended,
}

impl Session {
    pub fn new(pty: Box<dyn Pty>, parser: Box<dyn SequenceParser>, terminal: Terminal) -> Self {
        Self {
            pty,
            parser,
            terminal,
        }
    }

    /// Waits up to `wait` for output and shows it.
    pub fn step(&mut self, wait: Duration) -> Step {
        match self.pty.read(wait) {
            PtyRead::Data(bytes) => {
                let terminal = &mut self.terminal;
                self.parser
                    .parse(&bytes, &mut |sequence| apply(terminal, sequence));
                Step::Changed
            }
            PtyRead::Nothing => Step::Quiet,
            PtyRead::End => Step::Ended,
        }
    }

    /// Sends what the user types.
    pub fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.pty.write(bytes)
    }

    /// The screen changes only if the console accepted the new size.
    pub fn resize(&mut self, columns: u16, rows: u16) -> Result<(), String> {
        self.pty.resize(columns, rows)?;
        self.terminal.resize(columns, rows);
        Ok(())
    }

    pub fn exit_code(&mut self) -> Result<Option<u32>, String> {
        self.pty.exit_code()
    }

    /// Asks the program to leave by closing its console. The output left still
    /// arrives through [`Self::step`].
    pub fn close(&mut self) {
        self.pty.close();
    }

    pub fn process_id(&self) -> u32 {
        self.pty.process_id()
    }
}

#[cfg(test)]
mod tests;
