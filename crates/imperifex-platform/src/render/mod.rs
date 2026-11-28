//! The [`Renderer`] of Windows: Direct3D 11 (T-D13).

mod buffer;
mod device;
mod painter;
mod pipeline;
mod shader;

use std::rc::Rc;

use imperifex_domain::draw::DrawList;
use imperifex_domain::ports::Renderer;
use windows::Win32::Graphics::Direct3D11::{ID3D11RenderTargetView, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN;
use windows::Win32::Graphics::Dxgi::{IDXGISwapChain1, DXGI_PRESENT, DXGI_SWAP_CHAIN_FLAG};

use crate::text::DirectWriteText;
use painter::Painter;

pub struct D3dRenderer {
    painter: Painter,
    swap_chain: IDXGISwapChain1,
    /// What is drawn on: the swap chain's buffer of the moment.
    target: Option<ID3D11RenderTargetView>,
    size: (u32, u32),
}

impl D3dRenderer {
    /// For the window whose handle is `window`, of that size in physical
    /// pixels, with its letters from `text`.
    pub fn new(
        window: isize,
        width: u32,
        height: u32,
        text: Rc<DirectWriteText>,
    ) -> Result<Self, String> {
        let size = (width.max(1), height.max(1));
        let painter = Painter::new(text)?;
        let swap_chain = device::swap_chain(painter.device(), window, size)?;
        let mut renderer = Self {
            painter,
            swap_chain,
            target: None,
            size,
        };
        renderer.make_target()?;
        Ok(renderer)
    }

    /// Forgets the glyphs drawn so far: after a change of scale the old sizes
    /// would only take up room.
    pub fn forget_glyphs(&mut self) {
        self.painter.forget_glyphs();
    }

    fn make_target(&mut self) -> Result<(), String> {
        // SAFETY: buffer 0 of a live swap chain, asked for as the texture it is.
        let back: ID3D11Texture2D =
            unsafe { self.swap_chain.GetBuffer(0) }.map_err(failed("GetBuffer"))?;
        self.target = Some(self.painter.target(&back)?);
        Ok(())
    }
}

impl Renderer for D3dRenderer {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        if width == 0 || height == 0 || (width, height) == self.size {
            return Ok(());
        }
        // The buffers cannot change while anything holds them.
        self.target = None;
        self.painter.let_go();
        // SAFETY: no view of the buffers is left; 0 and unknown keep their
        // number and format, and the flags are those of its creation.
        unsafe {
            self.swap_chain.ResizeBuffers(
                0,
                width,
                height,
                DXGI_FORMAT_UNKNOWN,
                DXGI_SWAP_CHAIN_FLAG(0),
            )
        }
        .map_err(failed("ResizeBuffers"))?;
        self.size = (width, height);
        self.make_target()
    }

    fn draw(&mut self, list: &DrawList) -> Result<(), String> {
        if self.target.is_none() {
            self.make_target()?;
        }
        let Some(target) = &self.target else {
            return Err("nothing to draw on".into());
        };
        self.painter.paint(list, target, self.size)?;
        // SAFETY: a live swap chain; 1 waits for the screen's next refresh.
        unsafe { self.swap_chain.Present(1, DXGI_PRESENT(0)) }
            .ok()
            .map_err(failed("Present"))
    }
}

/// Says which call an error came from.
fn failed(call: &'static str) -> impl Fn(windows::core::Error) -> String {
    move |error| format!("{call}: {error}")
}

/// What a call that went well was to give.
fn created<T>(object: Option<T>) -> Result<T, String> {
    object.ok_or_else(|| "Direct3D gave no object".into())
}

#[cfg(test)]
mod tests;
