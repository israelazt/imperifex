//! Fakes of the ports, for the tests of this crate.

use imperifex_domain::draw::UiFont;
use imperifex_domain::ports::TextMeasure;

/// Every character is 10 logical pixels wide and every line 16 high.
pub struct FixedWidth;

impl TextMeasure for FixedWidth {
    fn width(&self, text: &str, _font: UiFont, scale: f32) -> u32 {
        let characters = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
        (characters as f32 * 10.0 * scale).round() as u32
    }

    fn line_height(&self, _font: UiFont, scale: f32) -> u32 {
        (16.0 * scale).round() as u32
    }
}
