//! A face: a family in one weight and style, and what it says of its glyphs.

use imperifex_domain::font::FontMetrics;
use windows::core::{Interface, BOOL, PCWSTR};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory2, IDWriteFont, IDWriteFontCollection, IDWriteFontFace, IDWriteFontFamily,
    DWRITE_FONT_METRICS, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE, DWRITE_FONT_STYLE_ITALIC,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_FONT_WEIGHT_REGULAR, DWRITE_GLYPH_METRICS,
};

use super::{created, failed, GRID_FACES};

/// Bold, italic, both or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    bold: bool,
    italic: bool,
}

impl Look {
    pub const REGULAR: Self = Self {
        bold: false,
        italic: false,
    };

    /// The grid's four, in the order of their faces.
    pub const ALL: [Self; 4] = [
        Self::REGULAR,
        Self {
            bold: true,
            italic: false,
        },
        Self {
            bold: false,
            italic: true,
        },
        Self {
            bold: true,
            italic: true,
        },
    ];

    /// That of the face with this number; those not of the grid are regular.
    pub fn of_number(number: u32) -> Self {
        number
            .checked_sub(GRID_FACES)
            .and_then(|at| Self::ALL.get(usize::try_from(at).ok()?).copied())
            .unwrap_or(Self::REGULAR)
    }

    pub fn weight(self) -> DWRITE_FONT_WEIGHT {
        if self.bold {
            DWRITE_FONT_WEIGHT_BOLD
        } else {
            DWRITE_FONT_WEIGHT_REGULAR
        }
    }

    pub fn style(self) -> DWRITE_FONT_STYLE {
        if self.italic {
            DWRITE_FONT_STYLE_ITALIC
        } else {
            DWRITE_FONT_STYLE_NORMAL
        }
    }
}

pub fn system_fonts(factory: &IDWriteFactory2) -> Result<IDWriteFontCollection, String> {
    let mut collection = None;
    // SAFETY: the output lives through the call.
    unsafe { factory.GetSystemFontCollection(&mut collection, false) }
        .map_err(failed("GetSystemFontCollection"))?;
    created(collection)
}

/// The installed family called `name`, if there is one.
pub fn family(
    collection: &IDWriteFontCollection,
    name: &str,
) -> Result<Option<IDWriteFontFamily>, String> {
    let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
    let mut index = 0;
    let mut exists = BOOL::default();
    // SAFETY: the name ends in zero and lives through the call, as both outputs.
    unsafe { collection.FindFamilyName(PCWSTR(wide.as_ptr()), &mut index, &mut exists) }
        .map_err(failed("FindFamilyName"))?;
    if !exists.as_bool() {
        return Ok(None);
    }
    // SAFETY: the index is the one the collection just gave.
    unsafe { collection.GetFontFamily(index) }
        .map(Some)
        .map_err(failed("GetFontFamily"))
}

/// The first of `names` that is installed.
pub fn installed(
    collection: &IDWriteFontCollection,
    names: &[&str],
) -> Result<IDWriteFontFamily, String> {
    for name in names {
        if let Some(found) = family(collection, name)? {
            return Ok(found);
        }
    }
    let wanted = names.join(", ");
    Err(format!("none of these fonts is installed: {wanted}"))
}

pub struct Face {
    pub face: IDWriteFontFace,
    pub metrics: FontMetrics,
}

impl Face {
    /// The member of `family` closest to `look`. When it has none that is bold
    /// or italic, Windows makes one up from another.
    pub fn of(family: &IDWriteFontFamily, look: Look) -> Result<Self, String> {
        // SAFETY: plain values.
        let font = unsafe {
            family.GetFirstMatchingFont(look.weight(), DWRITE_FONT_STRETCH_NORMAL, look.style())
        }
        .map_err(failed("GetFirstMatchingFont"))?;
        Self::of_font(&font)
    }

    pub fn of_font(font: &IDWriteFont) -> Result<Self, String> {
        // SAFETY: a plain query of a live font.
        let face = unsafe { font.CreateFontFace() }.map_err(failed("CreateFontFace"))?;
        let metrics = metrics(&face)?;
        Ok(Self { face, metrics })
    }

    /// Whether both are the same object: DirectWrite gives the same one for the
    /// same file, weight and style.
    pub fn is_same(&self, other: &Face) -> bool {
        self.face.as_raw() == other.face.as_raw()
    }

    /// The glyph of a character; 0, the one of "missing", when it has none.
    pub fn glyph(&self, character: char) -> Result<u16, String> {
        glyph(&self.face, character)
    }

    /// How far the pen moves after `glyph`, with letters of `size` pixels.
    pub fn advance(&self, glyph: u16, size: f32) -> Result<f32, String> {
        let units = design_advance(&self.face, glyph)?;
        Ok(self.metrics.pixels(units as f32, size))
    }
}

fn glyph(face: &IDWriteFontFace, character: char) -> Result<u16, String> {
    let code = u32::from(character);
    let mut glyph = 0u16;
    // SAFETY: one code point in and one glyph out, both alive.
    unsafe { face.GetGlyphIndices(&code, 1, &mut glyph) }.map_err(failed("GetGlyphIndices"))?;
    Ok(glyph)
}

fn design_advance(face: &IDWriteFontFace, glyph: u16) -> Result<u32, String> {
    let mut metrics = DWRITE_GLYPH_METRICS::default();
    // SAFETY: one glyph in and one set of metrics out, both alive.
    unsafe { face.GetDesignGlyphMetrics(&glyph, 1, &mut metrics, false) }
        .map_err(failed("GetDesignGlyphMetrics"))?;
    Ok(metrics.advanceWidth)
}

fn metrics(face: &IDWriteFontFace) -> Result<FontMetrics, String> {
    let mut design = DWRITE_FONT_METRICS::default();
    // SAFETY: the output lives through the call.
    unsafe { face.GetMetrics(&mut design) };
    Ok(FontMetrics {
        units_per_em: design.designUnitsPerEm,
        ascent: design.ascent,
        descent: design.descent,
        line_gap: design.lineGap,
        zero_advance: design_advance(face, glyph(face, '0')?)?,
    })
}
