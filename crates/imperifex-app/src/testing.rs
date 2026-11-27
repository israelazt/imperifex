//! Fakes of the ports, for the tests of this crate. A fake records what it
//! was asked and answers what the test scripts.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use imperifex_domain::ports::{Clock, Pty, PtyLauncher, PtyRead, SequenceParser};
use imperifex_domain::sequence::Sequence;

/// A clock that moves `step_ms` every time it is read, and jumps when waited on.
pub struct FakeClock {
    step_ms: u64,
    now_ms: Cell<u64>,
}

impl FakeClock {
    pub fn ticking(step_ms: u64) -> Self {
        Self {
            step_ms,
            now_ms: Cell::new(0),
        }
    }
}

impl Clock for FakeClock {
    fn monotonic(&self) -> Duration {
        let now = self.now_ms.get();
        self.now_ms.set(now + self.step_ms);
        Duration::from_millis(now)
    }

    fn sleep(&self, time: Duration) {
        let waited = u64::try_from(time.as_millis()).unwrap_or(u64::MAX);
        self.now_ms.set(self.now_ms.get().saturating_add(waited));
    }
}

/// Each byte is a character; `\r` and `\n` are controls. Enough to see what
/// reached the screen: the real parser is tested in the platform crate.
pub struct FakeParser;

impl SequenceParser for FakeParser {
    fn parse(&mut self, bytes: &[u8], receive: &mut dyn FnMut(Sequence<'_>)) {
        for byte in bytes {
            receive(match byte {
                b'\r' | b'\n' => Sequence::Control(*byte),
                other => Sequence::Print(char::from(*other)),
            });
        }
    }
}

/// How the fake program behaves.
#[derive(Clone, Default)]
pub struct Script {
    /// What each read gives while the console is open; then nothing more.
    pub output: Vec<PtyRead>,
    /// What still arrives after closing, before the end.
    pub after_close: Vec<&'static str>,
    /// It leaves by itself with this code once its output has been read.
    pub leaves: Option<u32>,
    /// Closing the console makes it leave with this code.
    pub leaves_when_closed: Option<u32>,
    /// Its output never ends, not even after closing.
    pub never_ends: bool,
    pub refuses_to_resize: bool,
}

/// What was asked of the fake.
#[derive(Debug, Default)]
pub struct Asked {
    pub started: Vec<(String, u16, u16)>,
    pub written: Vec<u8>,
    pub resized: Vec<(u16, u16)>,
    pub closed: usize,
}

#[derive(Default)]
pub struct FakeLauncher {
    pub script: Script,
    pub asked: Rc<RefCell<Asked>>,
}

impl FakeLauncher {
    pub fn running(script: Script) -> Self {
        Self {
            script,
            ..Self::default()
        }
    }
}

struct FakePty {
    script: Script,
    output: VecDeque<PtyRead>,
    asked: Rc<RefCell<Asked>>,
}

pub fn data(text: &str) -> PtyRead {
    PtyRead::Data(text.as_bytes().to_vec())
}

impl PtyLauncher for FakeLauncher {
    fn start(&self, command_line: &str, columns: u16, rows: u16) -> Result<Box<dyn Pty>, String> {
        let started = (command_line.to_string(), columns, rows);
        self.asked.borrow_mut().started.push(started);
        Ok(Box::new(FakePty {
            output: self.script.output.iter().cloned().collect(),
            script: self.script.clone(),
            asked: Rc::clone(&self.asked),
        }))
    }
}

impl FakePty {
    fn closed(&self) -> bool {
        self.asked.borrow().closed > 0
    }
}

impl Pty for FakePty {
    fn read(&mut self, _wait: Duration) -> PtyRead {
        if let Some(next) = self.output.pop_front() {
            return next;
        }
        if self.closed() && !self.script.never_ends {
            PtyRead::End
        } else {
            PtyRead::Nothing
        }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.asked.borrow_mut().written.extend_from_slice(bytes);
        Ok(())
    }

    fn resize(&mut self, columns: u16, rows: u16) -> Result<(), String> {
        if self.script.refuses_to_resize {
            return Err("la consola no quiso".into());
        }
        self.asked.borrow_mut().resized.push((columns, rows));
        Ok(())
    }

    fn exit_code(&mut self) -> Result<Option<u32>, String> {
        let by_itself = self.script.leaves.filter(|_| self.output.is_empty());
        let by_closing = self.script.leaves_when_closed.filter(|_| self.closed());
        Ok(by_itself.or(by_closing))
    }

    fn close(&mut self) {
        let first = !self.closed();
        self.asked.borrow_mut().closed += 1;
        if first {
            let rest = self.script.after_close.iter().map(|text| data(text));
            self.output.extend(rest);
        }
    }

    fn process_id(&self) -> u32 {
        4321
    }
}
