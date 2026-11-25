//! The adapters of the bench: they implement its ports. Calls to the system
//! live in `#[cfg(windows)]` modules; decoding what it returns is plain code.

#[cfg(windows)]
mod console_out;
#[cfg(any(windows, test))]
mod decode;
#[cfg(windows)]
mod desktop;
#[cfg(windows)]
mod dpi;
mod files;
#[cfg(windows)]
mod gpu;
#[cfg(any(windows, test))]
mod gpu_instance;
#[cfg(windows)]
mod machine;
#[cfg(windows)]
mod processes;

#[cfg(windows)]
pub use console_out::ConsoleOut;
#[cfg(windows)]
pub use desktop::WindowsDesktop;
#[cfg(windows)]
pub use dpi::use_physical_pixels;
pub use files::DiskFiles;
#[cfg(windows)]
pub use gpu::GpuCounters;
#[cfg(windows)]
pub use machine::WindowsMachine;
#[cfg(windows)]
pub use processes::WindowsProcesses;
