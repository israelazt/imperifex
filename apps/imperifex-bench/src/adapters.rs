//! The real adapters, handed to the orders. Only wiring.

use std::path::PathBuf;

use imperifex_bench_app::Adapters;
use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_platform::{
    ConsoleIn, ConsoleOut, DiskFiles, DxgiScreen, GpuCounters, QpcClock, WindowsDesktop,
    WindowsMachine, WindowsProcesses,
};

#[derive(Default)]
pub struct RealAdapters {
    files: DiskFiles,
}

impl Adapters for RealAdapters {
    type Clock = QpcClock;
    type Output = ConsoleOut;
    type Processes = WindowsProcesses;
    type Gpu = GpuCounters;
    type Machine = WindowsMachine;
    type Desktop = WindowsDesktop;
    type Screen = DxgiScreen;
    type Input = ConsoleIn;

    fn clock(&self) -> QpcClock {
        QpcClock::new()
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

    fn screen(&self) -> DxgiScreen {
        DxgiScreen
    }

    fn key_input(&self) -> Result<ConsoleIn, String> {
        ConsoleIn::new()
    }
}
