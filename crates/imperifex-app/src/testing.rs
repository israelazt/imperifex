//! Fakes of the ports, for the tests of this crate. A fake records what it
//! was asked and answers what the test scripts.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use imperifex_domain::ports::{
    Clipboard, Clock, CommandHistory, Folders, Pty, PtyLauncher, PtyRead, SequenceParser,
    SettingsStore, SystemAppearance, Wake,
};
use imperifex_domain::sequence::Sequence;

/// The system's theme as the test sets it. Its clones share it.
#[derive(Clone)]
pub struct FakeAppearance {
    says: Rc<RefCell<Result<bool, String>>>,
    watched: Rc<Cell<usize>>,
}

impl FakeAppearance {
    pub fn dark(dark: bool) -> Self {
        Self {
            says: Rc::new(RefCell::new(Ok(dark))),
            watched: Rc::default(),
        }
    }

    pub fn set(&self, says: Result<bool, String>) {
        *self.says.borrow_mut() = says;
    }

    /// How many times it was asked to watch.
    pub fn watched(&self) -> usize {
        self.watched.get()
    }
}

impl SystemAppearance for FakeAppearance {
    fn dark(&self) -> Result<bool, String> {
        self.says.borrow().clone()
    }

    fn watch(&self, _changed: Wake) -> Result<(), String> {
        self.watched.set(self.watched.get() + 1);
        Ok(())
    }
}

/// A settings file in memory that records what it was asked to write.
#[derive(Default)]
pub struct FakeStore {
    /// What `load` answers.
    pub file: Option<String>,
    pub load_fails: bool,
    pub save_fails: bool,
    pub copy_fails: bool,
    pub written: Rc<RefCell<Written>>,
}

#[derive(Debug, Default)]
pub struct Written {
    pub saved: Vec<String>,
    pub copies: Vec<String>,
}

impl SettingsStore for FakeStore {
    fn load(&self) -> Result<Option<String>, String> {
        if self.load_fails {
            return Err("acceso denegado".into());
        }
        Ok(self.file.clone())
    }

    fn save(&self, text: &str) -> Result<(), String> {
        if self.save_fails {
            return Err("disco lleno".into());
        }
        self.written.borrow_mut().saved.push(text.to_string());
        Ok(())
    }

    fn keep_damaged(&self, text: &str) -> Result<String, String> {
        if self.copy_fails {
            return Err("disco lleno".into());
        }
        self.written.borrow_mut().copies.push(text.to_string());
        Ok("la-copia.json".into())
    }
}

/// A history in memory that records what was added. Its clones share it.
#[derive(Clone, Default)]
pub struct FakeHistory {
    file: Option<String>,
    added: Rc<RefCell<Vec<(String, String)>>>,
}

impl FakeHistory {
    pub fn holding(text: &str) -> Self {
        Self {
            file: Some(text.into()),
            ..Self::default()
        }
    }

    /// Each profile and line added.
    pub fn added(&self) -> Vec<(String, String)> {
        self.added.borrow().clone()
    }
}

impl CommandHistory for FakeHistory {
    fn load(&self, _profile: &str) -> Result<Option<String>, String> {
        Ok(self.file.clone())
    }

    fn append(&self, profile: &str, line: &str) -> Result<(), String> {
        let added = (profile.to_string(), line.to_string());
        self.added.borrow_mut().push(added);
        Ok(())
    }

    fn replace(&self, _profile: &str, _text: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Folders that hold what the test says, by their path.
#[derive(Default)]
pub struct FakeFolders(pub Vec<(&'static str, Vec<(&'static str, bool)>)>);

impl Folders for FakeFolders {
    fn entries(&self, path: &str) -> Result<Vec<(String, bool)>, String> {
        let found = self.0.iter().find(|(folder, _)| *folder == path);
        let (_, names) = found.ok_or_else(|| format!("{path} is not there"))?;
        let owned = |&(name, is_folder): &(&str, bool)| (name.to_string(), is_folder);
        Ok(names.iter().map(owned).collect())
    }
}

/// A clipboard in memory. Its clones share it.
#[derive(Clone, Default)]
pub struct FakeClipboard {
    pub text: Rc<RefCell<Option<String>>>,
}

impl FakeClipboard {
    pub fn holding(text: &str) -> Self {
        let clipboard = Self::default();
        *clipboard.text.borrow_mut() = Some(text.into());
        clipboard
    }
}

impl Clipboard for FakeClipboard {
    fn read(&self) -> Result<Option<String>, String> {
        Ok(self.text.borrow().clone())
    }

    fn write(&self, text: &str) -> Result<(), String> {
        *self.text.borrow_mut() = Some(text.into());
        Ok(())
    }
}

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

pub fn fake_parser() -> Box<dyn SequenceParser> {
    Box::new(FakeParser)
}

pub fn no_wake() -> Wake {
    std::sync::Arc::new(|| {})
}

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
    /// The command lines that fail to start.
    pub missing: Vec<&'static str>,
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
    fn start(
        &self,
        command_line: &str,
        columns: u16,
        rows: u16,
        _wake: Wake,
    ) -> Result<Box<dyn Pty>, String> {
        let started = (command_line.to_string(), columns, rows);
        self.asked.borrow_mut().started.push(started);
        let program = command_line.split(' ').next().unwrap_or_default();
        if self.missing.contains(&command_line) || self.missing.contains(&program) {
            return Err(format!("{command_line} is not installed"));
        }
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
