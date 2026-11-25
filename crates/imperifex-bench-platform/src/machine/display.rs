//! The graphics card of the primary monitor and the monitors.

use imperifex_bench_domain::report::Monitor;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE, ENUM_CURRENT_SETTINGS,
};

use crate::decode::utf16;

/// Whether one of the adapter's outputs starts at (0, 0), as the primary does.
fn drives_primary(adapter: &IDXGIAdapter1) -> bool {
    for index in 0.. {
        // SAFETY: an index past the end is answered with an error.
        let Ok(output) = (unsafe { adapter.EnumOutputs(index) }) else {
            break;
        };
        // SAFETY: the output is alive; the description is returned by value.
        let Ok(desc) = (unsafe { output.GetDesc() }) else {
            continue;
        };
        let area = desc.DesktopCoordinates;
        if (area.left, area.top) == (0, 0) {
            return true;
        }
    }
    false
}

pub(super) fn primary_card() -> Result<String, String> {
    // SAFETY: no arguments; the factory is a counted reference.
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }
        .map_err(|error| format!("CreateDXGIFactory1 falló: {error}"))?;
    for index in 0.. {
        // SAFETY: an index past the end is answered with an error.
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(index) }) else {
            break;
        };
        if drives_primary(&adapter) {
            // SAFETY: the adapter is alive; the description is returned by value.
            let card = unsafe { adapter.GetDesc1() }
                .map_err(|error| format!("GetDesc1 falló: {error}"))?;
            return Ok(utf16(&card.Description));
        }
    }
    Err("ninguna tarjeta gráfica maneja el monitor principal".into())
}

/// The mode a monitor is in; `None` if the system will not say.
fn current_mode(device: &DISPLAY_DEVICEW) -> Option<DEVMODEW> {
    let mut mode = DEVMODEW {
        dmSize: size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };
    let name = PCWSTR(device.DeviceName.as_ptr());
    // SAFETY: the name ends in a zero inside `device`, and the mode says its size.
    let read = unsafe { EnumDisplaySettingsW(name, ENUM_CURRENT_SETTINGS, &mut mode) };
    read.as_bool().then_some(mode)
}

/// The monitors that are part of the desktop.
pub(super) fn monitors() -> Vec<Monitor> {
    let mut found = Vec::new();
    for index in 0.. {
        let mut device = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        // SAFETY: no device name asks for the adapters; the structure says its size.
        let more = unsafe { EnumDisplayDevicesW(PCWSTR::null(), index, &mut device, 0) };
        if !more.as_bool() {
            break;
        }
        if !device
            .StateFlags
            .contains(DISPLAY_DEVICE_ATTACHED_TO_DESKTOP)
        {
            continue;
        }
        if let Some(mode) = current_mode(&device) {
            found.push(Monitor {
                name: utf16(&device.DeviceName),
                width: mode.dmPelsWidth,
                height: mode.dmPelsHeight,
                hz: mode.dmDisplayFrequency,
                primary: device.StateFlags.contains(DISPLAY_DEVICE_PRIMARY_DEVICE),
            });
        }
    }
    found
}
