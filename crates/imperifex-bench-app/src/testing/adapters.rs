//! All the adapters at once, as an order gets them.

use std::path::Path;
use std::rc::Rc;

use imperifex_bench_domain::ports::{FileStore, MachineProbe};
use imperifex_bench_domain::report::Machine;

use super::system::{running, FakeGpu, FakeProcesses};
use super::{Events, FakeClock, FakeFiles, FakeOutput};
use crate::command::Adapters;

/// A machine with Warp running. Tests change what they care about.
pub struct FakeAdapters {
    pub files: FakeFiles,
    /// Why there is no console, if there is none.
    pub no_console: Option<String>,
    pub version: Result<String, String>,
    pub events: Events,
}

impl Default for FakeAdapters {
    fn default() -> Self {
        Self {
            files: FakeFiles::default(),
            no_console: None,
            version: Ok("v9".into()),
            events: Events::default(),
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
    type Processes = FakeProcesses;
    type Gpu = FakeGpu;
    type Machine = FakeMachine;

    fn clock(&self) -> FakeClock {
        FakeClock::ticking(100, &self.events)
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

    fn processes(&self) -> FakeProcesses {
        FakeProcesses::new(vec![vec![running(1, 0, "warp.exe", 1024, 2048, 0)]])
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
