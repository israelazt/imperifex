//! The real adapters, handed to the orders. Only wiring.

use std::path::PathBuf;

use imperifex_bench_app::Adapters;
use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_platform::{
    ConsoleOut, DiskFiles, GpuCounters, WindowsDesktop, WindowsMachine, WindowsProcesses,
};
use imperifex_platform::SystemClock;

#[derive(Default)]
pub struct RealAdapters {
    files: DiskFiles,
}

impl Adapters for RealAdapters {
    type Clock = SystemClock;
    type Output = ConsoleOut;
    type Processes = WindowsProcesses;
    type Gpu = GpuCounters;
    type Machine = WindowsMachine;
    type Desktop = WindowsDesktop;

    fn clock(&self) -> SystemClock {
        SystemClock::new()
    }

    fn terminal_output(&self) -> Result<ConsoleOut, String> {
        ConsoleOut::new()
    }

    fn files(&self) -> &dyn FileStore {
        &self.files
    }

    fn current_dir(&self) -> Result<PathBuf, String> {
        std::env::current_dir()
            .map_err(|error| format!("no se pudo saber la carpeta actual: {error}"))
    }

    fn processes(&self) -> WindowsProcesses {
        WindowsProcesses::new()
    }

    fn gpu(&self) -> Result<GpuCounters, String> {
        GpuCounters::new()
    }

    fn machine(&self) -> WindowsMachine {
        WindowsMachine
    }

    fn desktop(&self) -> WindowsDesktop {
        WindowsDesktop
    }
}
