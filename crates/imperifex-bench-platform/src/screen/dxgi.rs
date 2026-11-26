//! Desktop duplication: the pictures a monitor presents, as probe 1.1 found
//! them. Updates that only move the pointer (`LastPresentTime == 0`, hundreds a
//! second) are not pictures and are passed over.

use windows::core::Interface;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1, IDXGIOutput, IDXGIOutput1,
    IDXGIOutputDuplication, IDXGIResource, DXGI_OUTDUPL_FRAME_INFO,
};

use imperifex_bench_domain::desktop::Rect;

use super::failed;
use crate::screen_region::inside;

/// The monitor that holds a whole region, and the region's corner on it.
pub(super) struct Found {
    adapter: IDXGIAdapter1,
    output: IDXGIOutput,
    pub(super) corner: (u32, u32),
}

/// The corner of the region on an output, if it holds all of it.
fn corner_on(output: &IDXGIOutput, region: Rect) -> Result<Option<(u32, u32)>, String> {
    // SAFETY: the output is alive; the description is returned by value.
    let desc = unsafe { output.GetDesc() }.map_err(failed("IDXGIOutput::GetDesc"))?;
    let area = desc.DesktopCoordinates;
    let monitor = Rect {
        x: area.left,
        y: area.top,
        w: area.right - area.left,
        h: area.bottom - area.top,
    };
    Ok(inside(monitor, region))
}

pub(super) fn monitor_with(region: Rect) -> Result<Found, String> {
    // SAFETY: no arguments; the factory is a counted reference.
    let factory: IDXGIFactory1 =
        unsafe { CreateDXGIFactory1() }.map_err(failed("CreateDXGIFactory1"))?;
    for a in 0.. {
        // SAFETY: an index past the end is answered with an error.
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(a) }) else {
            break;
        };
        for o in 0.. {
            // SAFETY: an index past the end is answered with an error.
            let Ok(output) = (unsafe { adapter.EnumOutputs(o) }) else {
                break;
            };
            if let Some(corner) = corner_on(&output, region)? {
                return Ok(Found {
                    adapter,
                    output,
                    corner,
                });
            }
        }
    }
    Err(format!(
        "ningún monitor contiene entera la región {region:?}"
    ))
}

pub(super) fn device_for(found: &Found) -> Result<(ID3D11Device, ID3D11DeviceContext), String> {
    let (mut device, mut context) = (None, None);
    // SAFETY: the out pointers are to locals that live through the call.
    unsafe {
        D3D11CreateDevice(
            &found.adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
    }
    .map_err(failed("D3D11CreateDevice"))?;
    device
        .zip(context)
        .ok_or_else(|| "D3D11CreateDevice no devolvió el dispositivo".to_string())
}

pub(super) fn duplicate(
    found: &Found,
    device: &ID3D11Device,
) -> Result<IDXGIOutputDuplication, String> {
    let output: IDXGIOutput1 = found.output.cast().map_err(failed("IDXGIOutput1"))?;
    // SAFETY: the device was made on the adapter that owns this output.
    unsafe { output.DuplicateOutput(device) }.map_err(failed("DuplicateOutput"))
}

/// A frame held by the duplication; it is released when dropped.
pub(super) struct Held<'a> {
    pub(super) info: DXGI_OUTDUPL_FRAME_INFO,
    resource: Option<IDXGIResource>,
    duplication: &'a IDXGIOutputDuplication,
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        self.resource = None;
        // SAFETY: a frame was acquired from this duplication and is released once.
        let _ = unsafe { self.duplication.ReleaseFrame() };
    }
}

impl Held<'_> {
    pub(super) fn picture(&self) -> Result<ID3D11Texture2D, String> {
        self.resource
            .as_ref()
            .ok_or("el frame no trae imagen")?
            .cast()
            .map_err(failed("ID3D11Texture2D"))
    }
}

/// The next update within `timeout_ms`, with the system's own error so that
/// the caller can tell a quiet screen from a lost duplication.
pub(super) fn acquire(
    duplication: &IDXGIOutputDuplication,
    timeout_ms: u32,
) -> windows::core::Result<Held<'_>> {
    let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
    let mut resource = None;
    // SAFETY: the out pointers are to locals that live through the call.
    unsafe { duplication.AcquireNextFrame(timeout_ms, &mut info, &mut resource) }?;
    Ok(Held {
        info,
        resource,
        duplication,
    })
}
