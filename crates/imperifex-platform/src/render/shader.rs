//! The shaders, compiled when the renderer is made.

use windows::core::{s, PCSTR};
use windows::Win32::Graphics::Direct3D::Fxc::{D3DCompile, D3DCOMPILE_OPTIMIZATION_LEVEL3};
use windows::Win32::Graphics::Direct3D::ID3DBlob;

const SOURCE: &str = include_str!("shaders.hlsl");

/// The compiled code of the entry point `entry` for the shader model `model`.
pub fn compile(entry: PCSTR, model: PCSTR) -> Result<Blob, String> {
    let mut code = None;
    let mut errors = None;
    // SAFETY: the source lives through the call with the length given, the
    // names are constant texts ending in zero, and both outputs live through it.
    let result = unsafe {
        D3DCompile(
            SOURCE.as_ptr().cast(),
            SOURCE.len(),
            s!("shaders.hlsl"),
            None,
            None,
            entry,
            model,
            D3DCOMPILE_OPTIMIZATION_LEVEL3,
            0,
            &mut code,
            Some(&mut errors),
        )
    };
    if let Err(error) = result {
        let said = errors.map(|blob| String::from_utf8_lossy(Blob(blob).bytes()).into_owned());
        return Err(format!("D3DCompile: {error} {}", said.unwrap_or_default()));
    }
    code.map(Blob)
        .ok_or_else(|| "D3DCompile gave no code".into())
}

/// A block of memory of the compiler.
pub struct Blob(ID3DBlob);

impl Blob {
    pub fn bytes(&self) -> &[u8] {
        // SAFETY: a plain query of a live blob.
        let start = unsafe { self.0.GetBufferPointer() };
        // SAFETY: the same.
        let length = unsafe { self.0.GetBufferSize() };
        // SAFETY: the blob owns `length` bytes at `start`, which do not change
        // and live as long as `self`, borrowed by the slice.
        unsafe { std::slice::from_raw_parts(start.cast::<u8>(), length) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// It needs the compiler of Direct3D only, no graphics card.
    #[test]
    fn both_shaders_compile() {
        for (entry, model) in [(s!("vs_main"), s!("vs_5_0")), (s!("ps_main"), s!("ps_5_0"))] {
            let compiled = compile(entry, model).map(|code| code.bytes().len());
            assert!(compiled.as_ref().is_ok_and(|&n| n > 0), "{compiled:?}");
        }
    }
}
