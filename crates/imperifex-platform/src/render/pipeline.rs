//! What stays the same from one frame to the next: the shaders, how an instance
//! is read, the blending and the glyph atlas.

use std::mem::offset_of;

use windows::core::{s, PCSTR};
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11PixelShader, ID3D11ShaderResourceView, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER,
    D3D11_BIND_SHADER_RESOURCE, D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE,
    D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA, D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL,
    D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_INSTANCE_DATA, D3D11_RENDER_TARGET_BLEND_DESC,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32_FLOAT, DXGI_FORMAT_R8_UNORM,
    DXGI_SAMPLE_DESC,
};

use super::shader::compile;
use super::{created, failed};
use crate::instances::Instance;

/// The side of the glyph atlas, in pixels: one byte of coverage each. It is
/// empty until there are fonts.
const ATLAS_SIDE: u32 = 2048;

pub struct Pipeline {
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    layout: ID3D11InputLayout,
    blend: ID3D11BlendState,
    /// The size of what is drawn on, for the vertex shader.
    frame: ID3D11Buffer,
    atlas: ID3D11ShaderResourceView,
}

impl Pipeline {
    pub fn new(device: &ID3D11Device) -> Result<Self, String> {
        let vertex_code = compile(s!("vs_main"), s!("vs_5_0"))?;
        let pixel_code = compile(s!("ps_main"), s!("ps_5_0"))?;
        let mut vertex_shader = None;
        // SAFETY: the code and the output live through the call.
        unsafe { device.CreateVertexShader(vertex_code.bytes(), None, Some(&mut vertex_shader)) }
            .map_err(failed("CreateVertexShader"))?;
        let mut pixel_shader = None;
        // SAFETY: the same.
        unsafe { device.CreatePixelShader(pixel_code.bytes(), None, Some(&mut pixel_shader)) }
            .map_err(failed("CreatePixelShader"))?;
        Ok(Self {
            vertex_shader: created(vertex_shader)?,
            pixel_shader: created(pixel_shader)?,
            layout: layout(device, vertex_code.bytes())?,
            blend: blend(device)?,
            frame: frame_buffer(device)?,
            atlas: atlas(device)?,
        })
    }

    /// Leaves `context` ready to draw instances on a target of `size` pixels.
    pub fn bind(&self, context: &ID3D11DeviceContext, size: (u32, u32)) {
        let data = [size.0 as f32, size.1 as f32, 0.0, 0.0];
        // SAFETY: the buffer is of this device and holds four floats, which
        // `data` is and keeps through the call.
        unsafe { context.UpdateSubresource(&self.frame, 0, None, data.as_ptr().cast(), 0, 0) };
        // SAFETY: everything bound is a live object of the context's device.
        unsafe { context.IASetInputLayout(&self.layout) };
        // SAFETY: a plain setting.
        unsafe { context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP) };
        // SAFETY: as for the layout.
        unsafe { context.VSSetShader(&self.vertex_shader, None) };
        // SAFETY: the same.
        unsafe { context.VSSetConstantBuffers(0, Some(&[Some(self.frame.clone())])) };
        // SAFETY: the same.
        unsafe { context.PSSetShader(&self.pixel_shader, None) };
        // SAFETY: the same.
        unsafe { context.PSSetShaderResources(0, Some(&[Some(self.atlas.clone())])) };
        // SAFETY: the same.
        unsafe { context.OMSetBlendState(&self.blend, None, u32::MAX) };
    }
}

/// How the fields of an [`Instance`] reach the vertex shader.
fn layout(device: &ID3D11Device, vertex_code: &[u8]) -> Result<ID3D11InputLayout, String> {
    let element = |name: PCSTR, format: DXGI_FORMAT, offset: usize| D3D11_INPUT_ELEMENT_DESC {
        SemanticName: name,
        SemanticIndex: 0,
        Format: format,
        InputSlot: 0,
        // A few bytes inside the instance.
        AlignedByteOffset: offset as u32,
        InputSlotClass: D3D11_INPUT_PER_INSTANCE_DATA,
        InstanceDataStepRate: 1,
    };
    let four = DXGI_FORMAT_R32G32B32A32_FLOAT;
    let elements = [
        element(s!("AREA"), four, offset_of!(Instance, area)),
        element(s!("COLOR"), four, offset_of!(Instance, color)),
        element(s!("SOURCE"), four, offset_of!(Instance, atlas)),
        element(
            s!("GLYPH"),
            DXGI_FORMAT_R32_FLOAT,
            offset_of!(Instance, glyph),
        ),
    ];
    let mut layout = None;
    // SAFETY: the elements, their constant names, the code and the output live
    // through the call.
    unsafe { device.CreateInputLayout(&elements, vertex_code, Some(&mut layout)) }
        .map_err(failed("CreateInputLayout"))?;
    created(layout)
}

/// What is drawn covers what is under it as much as its alpha says.
fn blend(device: &ID3D11Device) -> Result<ID3D11BlendState, String> {
    let mut description = D3D11_BLEND_DESC::default();
    description.RenderTarget[0] = D3D11_RENDER_TARGET_BLEND_DESC {
        BlendEnable: true.into(),
        SrcBlend: D3D11_BLEND_SRC_ALPHA,
        DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
        BlendOp: D3D11_BLEND_OP_ADD,
        SrcBlendAlpha: D3D11_BLEND_ONE,
        DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
        BlendOpAlpha: D3D11_BLEND_OP_ADD,
        // The four channels fit in its byte.
        RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
    };
    let mut state = None;
    // SAFETY: the description and the output live through the call.
    unsafe { device.CreateBlendState(&description, Some(&mut state)) }
        .map_err(failed("CreateBlendState"))?;
    created(state)
}

fn frame_buffer(device: &ID3D11Device) -> Result<ID3D11Buffer, String> {
    let description = D3D11_BUFFER_DESC {
        ByteWidth: 16,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
        ..Default::default()
    };
    let mut buffer = None;
    // SAFETY: the description and the output live through the call.
    unsafe { device.CreateBuffer(&description, None, Some(&mut buffer)) }
        .map_err(failed("CreateBuffer"))?;
    created(buffer)
}

fn atlas(device: &ID3D11Device) -> Result<ID3D11ShaderResourceView, String> {
    let description = D3D11_TEXTURE2D_DESC {
        Width: ATLAS_SIDE,
        Height: ATLAS_SIDE,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        ..Default::default()
    };
    let mut texture = None;
    // SAFETY: the description and the output live through the call.
    unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
        .map_err(failed("CreateTexture2D"))?;
    let texture = created(texture)?;
    let mut view = None;
    // SAFETY: the texture is of this device and the output lives through it.
    unsafe { device.CreateShaderResourceView(&texture, None, Some(&mut view)) }
        .map_err(failed("CreateShaderResourceView"))?;
    // The view keeps the texture alive.
    created(view)
}
