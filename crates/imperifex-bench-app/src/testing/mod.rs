//! Fakes of the ports, for the tests of this crate. A fake records what it
//! was asked and answers what the test scripts.

pub mod adapters;
pub mod system;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use imperifex_bench_domain::ports::{FileStore, Output};
use imperifex_domain::ports::Clock;

/// What happened, in order, to the fakes that share a log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    ClockRead,
    Wrote(Vec<u8>),
    Flushed,
}

pub type Events = Rc<RefCell<Vec<Event>>>;

/// A clock that moves `step_ms` every time it is read.
pub struct FakeClock {
    step_ms: u64,
    now_ms: Cell<u64>,
    events: Events,
}

impl FakeClock {
    pub fn ticking(step_ms: u64, events: &Events) -> Self {
        Self {
            step_ms,
            now_ms: Cell::new(0),
            events: Rc::clone(events),
        }
    }
}

impl Clock for FakeClock {
    fn monotonic(&self) -> Duration {
        self.events.borrow_mut().push(Event::ClockRead);
        let now = self.now_ms.get();
        self.now_ms.set(now + self.step_ms);
        Duration::from_millis(now)
    }

    fn sleep(&self, time: Duration) {
        let waited = u64::try_from(time.as_millis()).unwrap_or(u64::MAX);
        self.now_ms.set(self.now_ms.get().saturating_add(waited));
    }
}

/// An output that keeps every piece it was handed.
pub struct FakeOutput {
    events: Events,
    failure: Option<String>,
}

impl FakeOutput {
    pub fn new(events: &Events) -> Self {
        Self {
            events: Rc::clone(events),
            failure: None,
        }
    }

    pub fn failing(events: &Events, reason: &str) -> Self {
        Self {
            failure: Some(reason.into()),
            ..Self::new(events)
        }
    }
}

impl Output for FakeOutput {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.events.borrow_mut().push(Event::Wrote(bytes.to_vec()));
        self.failure.clone().map_or(Ok(()), Err)
    }

    fn flush(&mut self) -> Result<(), String> {
        self.events.borrow_mut().push(Event::Flushed);
        Ok(())
    }
}

/// Files in memory.
#[derive(Default)]
pub struct FakeFiles {
    pub files: RefCell<HashMap<PathBuf, Vec<u8>>>,
    pub failure: Option<String>,
}

impl FileStore for FakeFiles {
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        if let Some(reason) = &self.failure {
            return Err(reason.clone());
        }
        self.files
            .borrow_mut()
            .insert(path.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    fn read(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        Ok(self.files.borrow().get(path).cloned())
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("temp"))
    }
}
