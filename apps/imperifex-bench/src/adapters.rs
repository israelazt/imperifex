//! The real adapters, handed to the orders. Only wiring.

use std::path::PathBuf;

use imperifex_bench_app::Adapters;
use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_platform::{
    ConsoleIn, ConsoleOut, DiskFiles, DxgiScreen, GpuCounters, QpcClock, WindowsDesktop,
    WindowsKeyboard, WindowsMachine, WindowsProcesses,
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
    type Keyboard = WindowsKeyboard;

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

    fn keyboard(&self) -> WindowsKeyboard {
        WindowsKeyboard
    }

    fn current_exe(&self) -> Result<String, String> {
        std::env::current_exe()
            .map(|path| path.display().to_string())
            .map_err(|error| format!("no se pudo saber la ruta del banco: {error}"))
    }

    fn key_input(&self) -> Result<ConsoleIn, String> {
        ConsoleIn::new()
    }
}
