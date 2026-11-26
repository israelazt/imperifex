//! A window in which running `emit` does what it would: output goes by on the
//! screen, the processes work and the result file shows up when it is over.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use imperifex_bench_domain::emit::{EmitResult, BYTES_PER_MB};
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::FileStore;

use super::desktop::Rig;
use super::keyboard::{typing, FakeTyping, Script};
use super::screen::{picture_of, KEY_SIDE};
use super::{FakeClock, FakeFiles};

/// A file and when it shows up.
type Due = (Duration, PathBuf, Vec<u8>);

/// Files that appear when their time comes.
#[derive(Debug, Clone)]
pub struct TimedFiles {
    clock: FakeClock,
    files: Rc<RefCell<Vec<Due>>>,
}

impl TimedFiles {
    fn at(&self, time: Duration, path: &Path, bytes: &[u8]) {
        self.files
            .borrow_mut()
            .push((time, path.to_path_buf(), bytes.to_vec()));
    }

    /// A file that is there from now.
    pub fn put(&self, path: &Path, bytes: &[u8]) {
        self.at(self.clock.now(), path, bytes);
    }
}

impl FileStore for TimedFiles {
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        self.put(path, bytes);
        Ok(())
    }

    fn read(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        let now = self.clock.now();
        Ok(self
            .files
            .borrow()
            .iter()
            .find(|(at, p, _)| *at <= now && p == path)
            .map(|(_, _, bytes)| bytes.clone()))
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        FakeFiles::default().temp_dir()
    }

    fn exists(&self, path: &Path) -> bool {
        matches!(self.read(path), Ok(Some(_)))
    }

    fn make_dir(&self, _path: &Path) -> Result<(), String> {
        Ok(())
    }
}

pub struct Emitting {
    pub ports: FakeTyping,
    pub files: TimedFiles,
}

/// The last quoted argument of a PowerShell line.
fn last_argument(line: &str) -> &str {
    line.trim_end_matches('\'')
        .rsplit('\'')
        .next()
        .unwrap_or("")
}

/// What running the line does: output at 200 and 800 ms, 80 ms of processor in
/// each process, and, if `finishes`, a result at 1 s.
pub fn emitting(rig: Rig, finishes: bool) -> Emitting {
    let ports = typing(rig, Script::showing_after(5));
    let files = TimedFiles {
        clock: ports.session.clock.clone(),
        files: Rc::default(),
    };
    let (clock, screen) = (ports.session.clock.clone(), ports.screen.clone());
    let busy = Rc::clone(&ports.session.processes.busy_100ns);
    let results = files.clone();
    ports.keyboard.on_enter(move |line| {
        if !line.contains("'emit'") {
            return;
        }
        let start = clock.now();
        let after = |ms| start + Duration::from_millis(ms);
        screen.show(picture_of(KEY_SIDE, after(200), 5000));
        screen.show(picture_of(KEY_SIDE, after(800), 9000));
        busy.set(busy.get() + 800_000);
        if finishes {
            let took = Duration::from_millis(100);
            let result = EmitResult::new(Load::Color, 4 * BYTES_PER_MB, took);
            results.at(
                after(1000),
                Path::new(last_argument(line)),
                result.to_json().as_bytes(),
            );
        }
    });
    Emitting { ports, files }
}
