//! The Direct3D and DXGI side of the probe.

use windows::core::Interface;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::Win32::Graphics::Dxgi::*;

fn failed(what: &str) -> impl Fn(windows::core::Error) -> String + '_ {
    move |error| format!("{what} falló: {error}")
}

pub struct Screen {
    pub description: String,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
}

/// A frame held by the duplication; it is released when dropped.
pub struct Frame<'a> {
    pub info: DXGI_OUTDUPL_FRAME_INFO,
    resource: Option<IDXGIResource>,
    duplication: &'a IDXGIOutputDuplication,
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        self.resource = None;
        // SAFETY: a frame was acquired from this duplication and is released once.
        let _ = unsafe { self.duplication.ReleaseFrame() };
    }
}

/// The adapter and output of the monitor whose desktop starts at (0, 0).
fn primary_output() -> Result<(IDXGIAdapter1, IDXGIOutput, String), String> {
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
            // SAFETY: the output is alive; the description is returned by value.
            let desc = unsafe { output.GetDesc() }.map_err(failed("IDXGIOutput::GetDesc"))?;
            let area = desc.DesktopCoordinates;
            if (area.left, area.top) != (0, 0) {
                continue;
            }
            // SAFETY: the adapter is alive; the description is returned by value.
            let card = unsafe { adapter.GetDesc1() }.map_err(failed("GetDesc1"))?;
            let description = format!(
                "salida {} {}x{} en {}",
                text(&desc.DeviceName),
                area.right - area.left,
                area.bottom - area.top,
                text(&card.Description)
            );
            return Ok((adapter, output, description));
        }
    }
    Err("no hay ninguna salida que empiece en (0, 0)".into())
}

fn text(wide: &[u16]) -> String {
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

fn device_on(adapter: &IDXGIAdapter1) -> Result<(ID3D11Device, ID3D11DeviceContext), String> {
    let (mut device, mut context) = (None, None);
    // SAFETY: the out pointers are to locals that live through the call.
    unsafe {
        D3D11CreateDevice(
            adapter,
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

pub fn open_primary() -> Result<Screen, String> {
    let (adapter, output, description) = primary_output()?;
    let (device, context) = device_on(&adapter)?;
    let output: IDXGIOutput1 = output.cast().map_err(failed("IDXGIOutput1"))?;
    // SAFETY: the device was made on the adapter that owns this output.
    let duplication =
        unsafe { output.DuplicateOutput(&device) }.map_err(failed("DuplicateOutput"))?;
    Ok(Screen {
        description,
        device,
        context,
        duplication,
    })
}

impl Screen {
    /// The next update within `timeout_ms`; `None` if there was none.
    pub fn next_frame(&self, timeout_ms: u32) -> Result<Option<Frame<'_>>, String> {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource = None;
        // SAFETY: the out pointers are to locals that live through the call.
        let acquired = unsafe {
            self.duplication
                .AcquireNextFrame(timeout_ms, &mut info, &mut resource)
        };
        match acquired {
            Ok(()) => Ok(Some(Frame {
                info,
                resource,
                duplication: &self.duplication,
            })),
            Err(error) if error.code() == DXGI_ERROR_WAIT_TIMEOUT => Ok(None),
            Err(error) => Err(failed("AcquireNextFrame")(error)),
        }
    }
}

/// Copies the top left region of a frame to memory the processor can read.
pub struct RegionReader<'a> {
    context: &'a ID3D11DeviceContext,
    staging: ID3D11Texture2D,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl<'a> RegionReader<'a> {
    pub fn new(screen: &'a Screen, width: u32, height: u32) -> Result<Self, String> {
        // SAFETY: the duplication is alive; the description is returned by value.
        let mode = unsafe { screen.duplication.GetDesc() }.ModeDesc;
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: mode.Format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut staging = None;
        // SAFETY: the description and the out pointer live through the call.
        unsafe {
            screen
                .device
                .CreateTexture2D(&desc, None, Some(&mut staging))
        }
        .map_err(failed("CreateTexture2D"))?;
        Ok(Self {
            context: &screen.context,
            staging: staging.ok_or("CreateTexture2D no devolvió la textura")?,
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        })
    }

    /// Reads the region and returns a sum of it, so the reading cannot be skipped.
    pub fn read(&mut self, frame: &Frame<'_>) -> Result<u64, String> {
        let source: ID3D11Texture2D = frame
            .resource
            .as_ref()
            .ok_or("el frame no trae imagen")?
            .cast()
            .map_err(failed("ID3D11Texture2D"))?;
        let area = D3D11_BOX {
            left: 0,
            top: 0,
            front: 0,
            right: self.width,
            bottom: self.height,
            back: 1,
        };
        // SAFETY: both textures belong to this device and have the same format;
        // the box fits in the staging texture and in a monitor of 1920x1080.
        unsafe {
            self.context
                .CopySubresourceRegion(&self.staging, 0, 0, 0, 0, &source, 0, Some(&area))
        };
        self.copy_out()?;
        Ok(self.pixels.iter().step_by(997).map(|&b| u64::from(b)).sum())
    }

    fn copy_out(&mut self) -> Result<(), String> {
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: a staging texture with read access; unmapped below.
        unsafe {
            self.context
                .Map(&self.staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
        }
        .map_err(failed("Map"))?;
        let row = self.width as usize * 4;
        for (y, target) in self.pixels.chunks_exact_mut(row).enumerate() {
            // SAFETY: the mapping has `height` rows `RowPitch` bytes apart.
            let start = unsafe { mapped.pData.cast::<u8>().add(y * mapped.RowPitch as usize) };
            // SAFETY: each row holds at least `width * 4` readable bytes while mapped.
            let source = unsafe { std::slice::from_raw_parts(start, row) };
            target.copy_from_slice(source);
        }
        // SAFETY: it was mapped above and `source` is no longer used.
        unsafe { self.context.Unmap(&self.staging, 0) };
        Ok(())
    }
}
