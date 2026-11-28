//! Fakes of the ports, for the tests of this crate.

use imperifex_domain::draw::UiFont;
use imperifex_domain::ports::TextMeasure;

/// Every character is as wide as the size of the letters, and a line is 4 more
/// than it.
pub struct FixedWidth;

impl TextMeasure for FixedWidth {
    fn width(&self, text: &str, _font: UiFont, size: u32) -> u32 {
        let characters = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
        characters.saturating_mul(size)
    }

    fn line_height(&self, _font: UiFont, size: u32) -> u32 {
        size + 4
    }
}
