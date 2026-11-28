//! The instances of a frame on the graphics card, in a buffer that grows.

use std::mem::size_of;

use windows::Win32::Graphics::Direct3D11::{
    ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC,
    D3D11_CPU_ACCESS_WRITE, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD, D3D11_USAGE_DYNAMIC,
};

use super::{created, failed};
use crate::instances::Instance;

/// How many fit in the first one.
const FIRST_ROOM: usize = 256;

#[derive(Default)]
pub struct InstanceBuffer {
    buffer: Option<ID3D11Buffer>,
    room: usize,
}

impl InstanceBuffer {
    /// Copies `instances` to the card and leaves them bound to `context`.
    pub fn upload(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        instances: &[Instance],
    ) -> Result<(), String> {
        if instances.len() > self.room || self.buffer.is_none() {
            self.grow(device, instances.len())?;
        }
        let Some(buffer) = &self.buffer else {
            return Err("no instance buffer".into());
        };
        write(context, buffer, instances)?;
        let stride = size_of::<Instance>() as u32;
        let bound = Some(buffer.clone());
        // SAFETY: one buffer, with its stride and its offset, which live
        // through the call.
        unsafe { context.IASetVertexBuffers(0, 1, Some(&bound), Some(&stride), Some(&0)) };
        Ok(())
    }

    fn grow(&mut self, device: &ID3D11Device, needed: usize) -> Result<(), String> {
        let room = needed.max(FIRST_ROOM).next_power_of_two();
        let bytes = u32::try_from(room * size_of::<Instance>())
            .map_err(|_| format!("too many instances: {needed}"))?;
        let description = D3D11_BUFFER_DESC {
            ByteWidth: bytes,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
            ..Default::default()
        };
        let mut buffer = None;
        // SAFETY: the description and the output live through the call.
        unsafe { device.CreateBuffer(&description, None, Some(&mut buffer)) }
            .map_err(failed("CreateBuffer"))?;
        self.buffer = Some(created(buffer)?);
        self.room = room;
        Ok(())
    }
}

/// `instances` must fit in `buffer`, which is dynamic.
fn write(
    context: &ID3D11DeviceContext,
    buffer: &ID3D11Buffer,
    instances: &[Instance],
) -> Result<(), String> {
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: a dynamic buffer mapped for writing; `mapped` lives through it.
    unsafe { context.Map(buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped)) }
        .map_err(failed("Map"))?;
    let into = mapped.pData.cast::<Instance>();
    // SAFETY: the mapped memory has room for them (`upload` made it so), is
    // aligned for floats and is not the slice's own.
    unsafe { std::ptr::copy_nonoverlapping(instances.as_ptr(), into, instances.len()) };
    // SAFETY: mapped above.
    unsafe { context.Unmap(buffer, 0) };
    Ok(())
}
