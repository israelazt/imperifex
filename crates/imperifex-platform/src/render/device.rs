//! The Direct3D device and the window's swap chain.

use windows::core::Interface;
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    IDXGIDevice, IDXGIFactory2, IDXGISwapChain1, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
    DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

use super::{created, failed};

pub fn device() -> Result<(ID3D11Device, ID3D11DeviceContext), String> {
    let mut device = None;
    let mut context = None;
    // SAFETY: no adapter nor software module; the list of levels and the two
    // outputs live through the call.
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
    }
    .map_err(failed("D3D11CreateDevice"))?;
    Ok((created(device)?, created(context)?))
}

/// Two buffers that take turns, shown whole or not at all. `window` is its
/// handle: a number that is not one gives an error.
pub fn swap_chain(
    device: &ID3D11Device,
    window: isize,
    (width, height): (u32, u32),
) -> Result<IDXGISwapChain1, String> {
    let dxgi: IDXGIDevice = device.cast().map_err(failed("IDXGIDevice"))?;
    // SAFETY: a plain query of a live device.
    let adapter = unsafe { dxgi.GetAdapter() }.map_err(failed("GetAdapter"))?;
    // SAFETY: a plain query of a live adapter.
    let factory: IDXGIFactory2 = unsafe { adapter.GetParent() }.map_err(failed("GetParent"))?;
    let description = DXGI_SWAP_CHAIN_DESC1 {
        Width: width,
        Height: height,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        Scaling: DXGI_SCALING_STRETCH,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        AlphaMode: DXGI_ALPHA_MODE_IGNORE,
        ..Default::default()
    };
    let window = HWND(window as *mut std::ffi::c_void);
    // SAFETY: the device and the description live through the call; Windows
    // checks the handle itself and answers with an error.
    unsafe { factory.CreateSwapChainForHwnd(device, window, &description, None, None) }
        .map_err(failed("CreateSwapChainForHwnd"))
}
