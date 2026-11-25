//! Fakes of the processes of the system and of the graphics card.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use imperifex_bench_domain::ports::{GpuProbe, ProcessProbe};
use imperifex_bench_domain::process::{ProcessEntry, ProcessUsage};
use imperifex_bench_domain::profile::TerminalProfile;

/// A process in one scripted list, and what asking for its usage answers.
#[derive(Debug, Clone)]
pub struct Listed {
    pub entry: ProcessEntry,
    pub usage: Result<ProcessUsage, String>,
}

impl Listed {
    pub fn unreadable(pid: u32, parent: u32, image: &str, why: &str) -> Self {
        Self {
            entry: ProcessEntry {
                pid,
                parent,
                image: image.into(),
            },
            usage: Err(why.into()),
        }
    }
}

pub fn running(
    pid: u32,
    parent: u32,
    image: &str,
    private_bytes: u64,
    working_set_bytes: u64,
    cpu_100ns: u64,
) -> Listed {
    Listed {
        usage: Ok(ProcessUsage {
            private_bytes,
            working_set_bytes,
            cpu_100ns,
            ..Default::default()
        }),
        ..Listed::unreadable(pid, parent, image, "")
    }
}

pub fn warp() -> TerminalProfile {
    TerminalProfile {
        name: "warp".into(),
        exe: "warp.exe".into(),
        new_window_args: vec![],
        own_images: vec!["warp.exe".into()],
        helper_images: vec!["OpenConsole.exe".into(), "conhost.exe".into()],
        new_tab_keys: None,
    }
}

/// Answers each call to `processes` with the next list, and the last one from
/// then on.
pub struct FakeProcesses {
    steps: Vec<Vec<Listed>>,
    calls: Cell<usize>,
}

impl FakeProcesses {
    pub fn new(steps: Vec<Vec<Listed>>) -> Self {
        Self {
            steps,
            calls: Cell::new(0),
        }
    }

    fn step(&self, call: usize) -> &[Listed] {
        let last = self.steps.len().saturating_sub(1);
        self.steps.get(call.min(last)).map_or(&[], Vec::as_slice)
    }
}

impl ProcessProbe for FakeProcesses {
    fn processes(&self) -> Result<Vec<ProcessEntry>, String> {
        let call = self.calls.get();
        self.calls.set(call + 1);
        Ok(self.step(call).iter().map(|p| p.entry.clone()).collect())
    }

    fn usage(&self, pid: u32) -> Result<ProcessUsage, String> {
        self.step(self.calls.get().saturating_sub(1))
            .iter()
            .find(|p| p.entry.pid == pid)
            .map_or_else(|| Err(format!("no existe {pid}")), |p| p.usage.clone())
    }
}

/// The pids asked about, call by call.
pub type Asked = Rc<RefCell<Vec<Vec<u32>>>>;

/// Answers with the numbers in order, and the last one from then on.
pub struct FakeGpu {
    answers: Vec<f64>,
    asked: Asked,
}

impl FakeGpu {
    pub fn new(answers: &[f64], asked: &Asked) -> Self {
        Self {
            answers: answers.to_vec(),
            asked: Rc::clone(asked),
        }
    }
}

impl GpuProbe for FakeGpu {
    fn utilization(&mut self, pids: &[u32]) -> Result<f64, String> {
        let call = self.asked.borrow().len();
        self.asked.borrow_mut().push(pids.to_vec());
        let last = self.answers.len().saturating_sub(1);
        Ok(self
            .answers
            .get(call.min(last))
            .copied()
            .unwrap_or_default())
    }
}
