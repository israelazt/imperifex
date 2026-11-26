//! Fakes of the ports, for the tests of this crate. A fake records what it
//! was asked and answers what the test scripts.

pub mod adapters;
pub mod desktop;
pub mod keyboard;
pub mod processes_of;
pub mod screen;
pub mod system;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use imperifex_bench_domain::ports::{FileStore, KeyInput, Output};
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
#[derive(Debug, Clone)]
pub struct FakeClock {
    step_ms: u64,
    /// In microseconds, and shared by the clones: they all tell the same time.
    now_us: Rc<Cell<u64>>,
    events: Events,
}

impl FakeClock {
    pub fn ticking(step_ms: u64, events: &Events) -> Self {
        Self {
            step_ms,
            now_us: Rc::default(),
            events: Rc::clone(events),
        }
    }

    /// The time, without it counting as a reading.
    pub fn now(&self) -> Duration {
        Duration::from_micros(self.now_us.get())
    }

    /// Time never goes back.
    pub fn advance_to(&self, time: Duration) {
        let micros = u64::try_from(time.as_micros()).unwrap_or(u64::MAX);
        self.now_us.set(self.now_us.get().max(micros));
    }
}

impl Clock for FakeClock {
    fn monotonic(&self) -> Duration {
        self.events.borrow_mut().push(Event::ClockRead);
        let now = self.now();
        self.advance_to(now + Duration::from_millis(self.step_ms));
        now
    }

    fn sleep(&self, time: Duration) {
        self.advance_to(self.now() + time);
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

/// An input that hands over one scripted read at a time, then ends.
pub struct FakeKeys(std::collections::VecDeque<Vec<u8>>);

impl FakeKeys {
    pub fn new(reads: &[&[u8]]) -> Self {
        Self(reads.iter().map(|r| r.to_vec()).collect())
    }
}

impl KeyInput for FakeKeys {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        let next = self.0.pop_front().unwrap_or_default();
        buf[..next.len()].copy_from_slice(&next);
        Ok(next.len())
    }
}

/// Files in memory.
#[derive(Default)]
pub struct FakeFiles {
    pub files: RefCell<HashMap<PathBuf, Vec<u8>>>,
    pub folders: RefCell<HashSet<PathBuf>>,
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

    fn exists(&self, path: &Path) -> bool {
        self.files.borrow().contains_key(path) || self.folders.borrow().contains(path)
    }

    fn make_dir(&self, path: &Path) -> Result<(), String> {
        if let Some(reason) = &self.failure {
            return Err(reason.clone());
        }
        self.folders.borrow_mut().insert(path.to_path_buf());
        Ok(())
    }
}
