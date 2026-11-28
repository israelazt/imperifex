//! Drawing a frame on a texture, be it the window's or one of a test.

use std::rc::Rc;

use imperifex_domain::atlas::Atlas;
use imperifex_domain::draw::DrawList;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D, D3D11_VIEWPORT,
};

use super::buffer::InstanceBuffer;
use super::pipeline::{Pipeline, ATLAS_SIDE};
use super::{created, device, failed};
use crate::instances::Batch;
use crate::text::DirectWriteText;

pub struct Painter {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    pipeline: Pipeline,
    buffer: InstanceBuffer,
    batch: Batch,
    /// What the pipeline's texture holds: they live and die together.
    atlas: Atlas,
    text: Rc<DirectWriteText>,
}

impl Painter {
    pub fn new(text: Rc<DirectWriteText>) -> Result<Self, String> {
        let (device, context) = device::device()?;
        Ok(Self {
            pipeline: Pipeline::new(&device)?,
            buffer: InstanceBuffer::default(),
            batch: Batch::default(),
            atlas: Atlas::new(ATLAS_SIDE),
            device,
            context,
            text,
        })
    }

    pub fn device(&self) -> &ID3D11Device {
        &self.device
    }

    #[cfg(test)]
    pub fn context(&self) -> &ID3D11DeviceContext {
        &self.context
    }

    pub fn forget_glyphs(&mut self) {
        self.atlas.clear();
    }

    /// Lets go of everything bound; each frame binds what it uses.
    pub fn let_go(&self) {
        // SAFETY: a plain call on a live context.
        unsafe { self.context.ClearState() };
    }

    /// What draws on `texture`, which is of this device.
    pub fn target(&self, texture: &ID3D11Texture2D) -> Result<ID3D11RenderTargetView, String> {
        let mut target = None;
        // SAFETY: the texture is alive and so is the output through the call.
        unsafe {
            self.device
                .CreateRenderTargetView(texture, None, Some(&mut target))
        }
        .map_err(failed("CreateRenderTargetView"))?;
        created(target)
    }

    /// Draws `list` on `target`, of that size in pixels.
    pub fn paint(
        &mut self,
        list: &DrawList,
        target: &ID3D11RenderTargetView,
        size: (u32, u32),
    ) -> Result<(), String> {
        self.batch.fill(list, self.text.as_ref(), &mut self.atlas);
        self.pipeline
            .upload_glyphs(&self.context, &self.batch.uploads);
        self.aim(target, size);
        self.pipeline.bind(&self.context, size);
        let instances = &self.batch.instances;
        if instances.is_empty() {
            return Ok(());
        }
        self.buffer.upload(&self.device, &self.context, instances)?;
        let count = u32::try_from(instances.len()).map_err(|_| "too much to draw")?;
        // SAFETY: four corners per instance, and the buffer holds `count`.
        unsafe { self.context.DrawInstanced(4, count, 0, 0) };
        Ok(())
    }

    /// Points the context at the target. Showing a frame undoes it, so it is
    /// done for each one.
    fn aim(&self, target: &ID3D11RenderTargetView, size: (u32, u32)) {
        let viewport = D3D11_VIEWPORT {
            Width: size.0 as f32,
            Height: size.1 as f32,
            MaxDepth: 1.0,
            ..Default::default()
        };
        let targets = [Some(target.clone())];
        // SAFETY: the view is alive, and so is the list through the call.
        unsafe { self.context.OMSetRenderTargets(Some(&targets), None) };
        // SAFETY: one viewport, which lives through the call.
        unsafe { self.context.RSSetViewports(Some(&[viewport])) };
    }
}
