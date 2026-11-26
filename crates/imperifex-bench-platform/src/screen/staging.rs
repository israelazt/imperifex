//! Bringing a region of a frame to memory the processor can read.

use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BOX, D3D11_CPU_ACCESS_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_SAMPLE_DESC};

use super::failed;

const BYTES_PER_PIXEL: usize = 4;

/// A texture of the region's size, made once and used for every frame.
pub(super) struct Staging {
    context: ID3D11DeviceContext,
    texture: ID3D11Texture2D,
    /// The corner of the region on the monitor.
    left: u32,
    top: u32,
    pub(super) width: u32,
    pub(super) height: u32,
}

impl Staging {
    /// The region must lie inside the frames it will be copied from.
    pub(super) fn new(
        device: &ID3D11Device,
        context: ID3D11DeviceContext,
        format: DXGI_FORMAT,
        (left, top): (u32, u32),
        (width, height): (u32, u32),
    ) -> Result<Self, String> {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut texture = None;
        // SAFETY: the description and the out pointer live through the call.
        unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) }
            .map_err(failed("CreateTexture2D"))?;
        Ok(Self {
            context,
            texture: texture.ok_or("CreateTexture2D no devolvió la textura")?,
            left,
            top,
            width,
            height,
        })
    }

    /// The region of `source` as rows of BGRA with nothing between them.
    pub(super) fn read(&self, source: &ID3D11Texture2D) -> Result<Vec<u8>, String> {
        let area = D3D11_BOX {
            left: self.left,
            top: self.top,
            front: 0,
            right: self.left + self.width,
            bottom: self.top + self.height,
            back: 1,
        };
        // SAFETY: both textures belong to this device and have the same format,
        // and the box lies inside the source, as `new` requires.
        unsafe {
            self.context
                .CopySubresourceRegion(&self.texture, 0, 0, 0, 0, source, 0, Some(&area))
        };
        self.copy_out()
    }

    fn copy_out(&self) -> Result<Vec<u8>, String> {
        let row = self.width as usize * BYTES_PER_PIXEL;
        let mut pixels = vec![0; row * self.height as usize];
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: a staging texture with read access; unmapped below.
        unsafe {
            self.context
                .Map(&self.texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
        }
        .map_err(failed("Map"))?;
        for (y, target) in pixels.chunks_exact_mut(row).enumerate() {
            // SAFETY: the mapping has `height` rows `RowPitch` bytes apart.
            let start = unsafe { mapped.pData.cast::<u8>().add(y * mapped.RowPitch as usize) };
            // SAFETY: each row holds at least `width * 4` readable bytes while mapped.
            let source = unsafe { std::slice::from_raw_parts(start, row) };
            target.copy_from_slice(source);
        }
        // SAFETY: it was mapped above and no borrow of the mapping is left.
        unsafe { self.context.Unmap(&self.texture, 0) };
        Ok(pixels)
    }
}
