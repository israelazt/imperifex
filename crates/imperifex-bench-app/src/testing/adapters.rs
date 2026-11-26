//! All the adapters at once, as an order gets them.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use imperifex_bench_domain::ports::{FileStore, MachineProbe};
use imperifex_bench_domain::report::Machine;

use super::desktop::{Behaviour, FakeDesktop};
use super::processes_of::DesktopProcesses;
use super::screen::FakeScreen;
use super::system::FakeGpu;
use super::{Events, FakeClock, FakeFiles, FakeOutput};
use crate::command::Adapters;

/// A machine with Warp running. Tests change what they care about.
pub struct FakeAdapters {
    pub files: FakeFiles,
    /// Why there is no console, if there is none.
    pub no_console: Option<String>,
    pub version: Result<String, String>,
    /// Every clock handed out tells the same time as this one.
    pub clock: FakeClock,
    pub events: Events,
    pub desktop: FakeDesktop,
}

impl Default for FakeAdapters {
    fn default() -> Self {
        let events = Events::default();
        Self {
            files: FakeFiles::default(),
            clock: FakeClock::ticking(100, &events),
            no_console: None,
            version: Ok("v9".into()),
            events,
            desktop: FakeDesktop::new(vec![], Behaviour::default()),
        }
    }
}

impl FakeAdapters {
    pub fn without_console(why: &str) -> Self {
        Self {
            no_console: Some(why.into()),
            ..Default::default()
        }
    }
}

pub struct FakeMachine {
    version: Result<String, String>,
}

impl MachineProbe for FakeMachine {
    fn machine(&self) -> Result<Machine, String> {
        Ok(Machine {
            os: "os falso".into(),
            ..Default::default()
        })
    }

    fn product_version(&self, _exe: &Path) -> Result<String, String> {
        self.version.clone()
    }

    fn today(&self) -> String {
        "2026-01-02".into()
    }

    fn env_var(&self, name: &str) -> Option<String> {
        (name == "LOCALAPPDATA").then(|| "local".into())
    }
}

impl Adapters for FakeAdapters {
    type Clock = FakeClock;
    type Output = FakeOutput;
    type Processes = DesktopProcesses;
    type Desktop = FakeDesktop;
    type Screen = FakeScreen;
    type Gpu = FakeGpu;
    type Machine = FakeMachine;

    fn clock(&self) -> FakeClock {
        self.clock.clone()
    }

    fn terminal_output(&self) -> Result<FakeOutput, String> {
        match &self.no_console {
            Some(why) => Err(why.clone()),
            None => Ok(FakeOutput::new(&self.events)),
        }
    }

    fn files(&self) -> &dyn FileStore {
        &self.files
    }

    /// Outside any repository, unless a test makes one there.
    fn current_dir(&self) -> Result<PathBuf, String> {
        Ok("elsewhere".into())
    }

    /// Warp is already running, as process 1.
    fn processes(&self) -> DesktopProcesses {
        DesktopProcesses::new(&self.desktop, &[1])
    }

    fn desktop(&self) -> FakeDesktop {
        self.desktop.clone()
    }

    fn screen(&self) -> FakeScreen {
        FakeScreen::new(&self.clock)
    }

    fn gpu(&self) -> Result<FakeGpu, String> {
        Ok(FakeGpu::new(&[0.0], &Rc::default()))
    }

    fn machine(&self) -> FakeMachine {
        FakeMachine {
            version: self.version.clone(),
        }
    }
}
