//! The picture of a glyph: one byte of coverage per pixel, smoothed in greys.
//! Not ClearType: that depends on what is behind.

use std::mem::ManuallyDrop;

use windows::Win32::Graphics::DirectWrite::{
    DWRITE_TEXTURE_ALIASED_1x1, IDWriteFactory2, IDWriteGlyphRunAnalysis, DWRITE_GLYPH_RUN,
    DWRITE_GRID_FIT_MODE_DEFAULT, DWRITE_MEASURING_MODE_NATURAL,
    DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC, DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
};

use super::face::Face;
use super::failed;
use crate::instances::Picture;

pub fn picture(
    factory: &IDWriteFactory2,
    face: &Face,
    glyph: u16,
    size: f32,
) -> Result<Picture, String> {
    let analysis = analyse(factory, face, glyph, size)?;
    // SAFETY: a plain query of a live analysis.
    let bounds = unsafe { analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1) }
        .map_err(failed("GetAlphaTextureBounds"))?;
    let side = |from: i32, to: i32| u32::try_from(to.saturating_sub(from)).unwrap_or(0);
    let width = side(bounds.left, bounds.right);
    let height = side(bounds.top, bounds.bottom);
    if width == 0 || height == 0 {
        // Nothing to see, as with a space.
        return Ok(Picture::default());
    }
    let mut coverage = vec![0u8; width as usize * height as usize];
    // SAFETY: the buffer has one byte for each pixel of the bounds, which this
    // kind of texture takes, and both live through the call.
    unsafe { analysis.CreateAlphaTexture(DWRITE_TEXTURE_ALIASED_1x1, &bounds, &mut coverage) }
        .map_err(failed("CreateAlphaTexture"))?;
    Ok(Picture {
        left: bounds.left,
        top: bounds.top,
        width,
        height,
        coverage,
    })
}

/// One glyph with its pen at 0, 0 on the baseline.
fn analyse(
    factory: &IDWriteFactory2,
    face: &Face,
    glyph: u16,
    size: f32,
) -> Result<IDWriteGlyphRunAnalysis, String> {
    let advance = 0.0f32;
    let mut run = DWRITE_GLYPH_RUN {
        fontFace: ManuallyDrop::new(Some(face.face.clone())),
        fontEmSize: size,
        glyphCount: 1,
        glyphIndices: &glyph,
        glyphAdvances: &advance,
        ..Default::default()
    };
    // SAFETY: the run, with the one glyph and advance it points to, lives
    // through the call, which keeps no pointer to them.
    let analysis = unsafe {
        factory.CreateGlyphRunAnalysis(
            &run,
            None,
            DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
            DWRITE_MEASURING_MODE_NATURAL,
            DWRITE_GRID_FIT_MODE_DEFAULT,
            DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
            0.0,
            0.0,
        )
    };
    // SAFETY: put there above and not used again.
    unsafe { ManuallyDrop::drop(&mut run.fontFace) };
    analysis.map_err(failed("CreateGlyphRunAnalysis"))
}
