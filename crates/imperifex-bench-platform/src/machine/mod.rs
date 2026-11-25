//! What a report says about this computer: its processor, graphics card,
//! system and monitors. It only reads.

mod display;
mod version;

use std::path::Path;

use imperifex_bench_domain::ports::MachineProbe;
use imperifex_bench_domain::report::Machine;
use windows::core::w;
use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
use windows::Win32::System::SystemInformation::{GetLocalTime, OSVERSIONINFOW};

use crate::decode::{date_text, utf16};

fn processor() -> Result<String, String> {
    let mut name = [0u16; 256];
    let mut bytes = size_of_val(&name) as u32;
    // SAFETY: the buffer and its size in bytes go together, and both live
    // through the call.
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            w!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0"),
            w!("ProcessorNameString"),
            RRF_RT_REG_SZ,
            None,
            Some(name.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    };
    if status.is_err() {
        return Err(format!(
            "no se pudo leer el nombre del procesador: 0x{:08x}",
            status.0
        ));
    }
    Ok(utf16(&name).trim().to_string())
}

/// From the system itself: the version in `kernel32.dll` stays behind on some
/// updates (on the reference machine it says build 26100 and the system 26200).
fn system() -> Result<String, String> {
    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // SAFETY: the structure says its size and lives through the call.
    let status = unsafe { RtlGetVersion(&mut info) };
    if status.is_err() {
        return Err(format!("RtlGetVersion falló: 0x{:08x}", status.0));
    }
    Ok(format!(
        "Windows {}.{}.{}",
        info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
    ))
}

/// Reads the computer the program runs on.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsMachine;

impl MachineProbe for WindowsMachine {
    fn machine(&self) -> Result<Machine, String> {
        Ok(Machine {
            cpu: processor()?,
            gpu: display::primary_card()?,
            os: system()?,
            monitors: display::monitors(),
        })
    }

    fn product_version(&self, exe: &Path) -> Result<String, String> {
        version::product_version(exe)
    }

    fn today(&self) -> String {
        // SAFETY: no arguments; the time is returned by value.
        let now = unsafe { GetLocalTime() };
        date_text(now.wYear, now.wMonth, now.wDay)
    }

    fn env_var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_computer_has_a_processor_a_system_and_a_date() {
        assert!(!processor().unwrap().is_empty());
        assert!(system().unwrap().starts_with("Windows "));
        let today = WindowsMachine.today();
        assert_eq!(today.len(), 10, "{today}");
    }

    #[test]
    fn a_system_file_declares_its_product_version() {
        let windows = WindowsMachine.env_var("SystemRoot").unwrap();
        let kernel = Path::new(&windows).join("System32").join("kernel32.dll");
        let version = WindowsMachine.product_version(&kernel).unwrap();
        assert!(version.starts_with("10."), "{version}");
    }

    #[test]
    fn a_file_that_is_not_there_has_no_version_and_says_which_it_was() {
        let error = WindowsMachine
            .product_version(Path::new("no-such-file.exe"))
            .unwrap_err();
        assert!(error.contains("no-such-file.exe"), "{error}");
    }

    /// `cargo test -p imperifex-bench-platform show_this_machine -- --ignored --nocapture`
    #[test]
    #[ignore = "shows what is read of this computer, to check it by eye"]
    fn show_this_machine() {
        println!("{:#?}", WindowsMachine.machine().unwrap());
    }
}
