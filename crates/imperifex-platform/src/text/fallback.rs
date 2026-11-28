//! Which installed font stands in for a character the chosen one does not have.
//! Windows asks for the text through an object of ours.

use windows::core::{implement, OutRef, PCWSTR};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory2, IDWriteFontCollection, IDWriteNumberSubstitution, IDWriteTextAnalysisSource,
    IDWriteTextAnalysisSource_Impl, DWRITE_FONT_STRETCH_NORMAL, DWRITE_READING_DIRECTION,
    DWRITE_READING_DIRECTION_LEFT_TO_RIGHT,
};

use super::face::{Face, Look};
use super::failed;

/// The text of one character, read left to right, with no language.
#[implement(IDWriteTextAnalysisSource)]
struct OneCharacter {
    text: Vec<u16>,
    /// An empty text ending in zero.
    locale: [u16; 1],
}

impl OneCharacter {
    fn length(&self) -> u32 {
        // One character is one or two units.
        self.text.len() as u32
    }

    /// The units from `from`, `count` of them, as a pointer and how many there
    /// are; a null one when there are none.
    fn part(&self, from: u32, count: u32) -> (*mut u16, u32) {
        let from = from as usize;
        let found = self.text.get(from..from + count as usize);
        match found.filter(|part| !part.is_empty()) {
            Some(part) => (part.as_ptr().cast_mut(), count),
            None => (std::ptr::null_mut(), 0),
        }
    }
}

/// # Safety
/// `into` must be a place that can be written.
unsafe fn put<T>(into: *mut T, value: T) {
    // SAFETY: what the caller promises.
    unsafe { into.write(value) }
}

impl IDWriteTextAnalysisSource_Impl for OneCharacter_Impl {
    fn GetTextAtPosition(
        &self,
        position: u32,
        text: *mut *mut u16,
        length: *mut u32,
    ) -> windows::core::Result<()> {
        let (start, count) = self.part(position, self.length().saturating_sub(position));
        // SAFETY: Windows passes a place for the pointer, which it only reads
        // through, while this object and so its text are alive.
        unsafe { put(text, start) };
        // SAFETY: and a place for the count.
        unsafe { put(length, count) };
        Ok(())
    }

    fn GetTextBeforePosition(
        &self,
        position: u32,
        text: *mut *mut u16,
        length: *mut u32,
    ) -> windows::core::Result<()> {
        let before = if position <= self.length() {
            position
        } else {
            0
        };
        let (start, count) = self.part(0, before);
        // SAFETY: as in `GetTextAtPosition`.
        unsafe { put(text, start) };
        // SAFETY: the same.
        unsafe { put(length, count) };
        Ok(())
    }

    fn GetParagraphReadingDirection(&self) -> DWRITE_READING_DIRECTION {
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
    }

    fn GetLocaleName(
        &self,
        position: u32,
        length: *mut u32,
        locale: *mut *mut u16,
    ) -> windows::core::Result<()> {
        // SAFETY: Windows passes a place for the count.
        unsafe { put(length, self.length().saturating_sub(position)) };
        // SAFETY: and one for the pointer, read while this object is alive.
        unsafe { put(locale, self.locale.as_ptr().cast_mut()) };
        Ok(())
    }

    fn GetNumberSubstitution(
        &self,
        position: u32,
        length: *mut u32,
        substitution: OutRef<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        // SAFETY: Windows passes a place for the count.
        unsafe { put(length, self.length().saturating_sub(position)) };
        substitution.write(None)
    }
}

/// The face that has `character`, and by how much to scale its letters so that
/// they match the rest. `None` when no installed font has it.
pub fn face_for(
    factory: &IDWriteFactory2,
    collection: &IDWriteFontCollection,
    character: char,
    look: Look,
) -> Result<Option<(Face, f32)>, String> {
    let mut units = [0u16; 2];
    let text = character.encode_utf16(&mut units).to_vec();
    let length = text.len() as u32;
    let source: IDWriteTextAnalysisSource = OneCharacter { text, locale: [0] }.into();
    // SAFETY: a plain query of a live factory.
    let fallback =
        unsafe { factory.GetSystemFontFallback() }.map_err(failed("GetSystemFontFallback"))?;
    let mut mapped = 0;
    let mut font = None;
    let mut scale = 1.0f32;
    // SAFETY: the source covers the whole length asked about; the outputs live
    // through the call, and no base family is given.
    unsafe {
        fallback.MapCharacters(
            &source,
            0,
            length,
            collection,
            PCWSTR::null(),
            look.weight(),
            look.style(),
            DWRITE_FONT_STRETCH_NORMAL,
            &mut mapped,
            &mut font,
            &mut scale,
        )
    }
    .map_err(failed("MapCharacters"))?;
    font.map(|font| Face::of_font(&font).map(|face| (face, scale)))
        .transpose()
}
