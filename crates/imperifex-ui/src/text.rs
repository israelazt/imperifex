//! Putting one line of text, or one icon, in the list to draw.

use imperifex_domain::draw::{DrawList, Icon, Rect, TextRun, UiFont};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::theme::Rgb;

/// How some letters look: which, their size in physical pixels and their colour.
#[derive(Debug, Clone, Copy)]
pub struct Ink {
    pub font: UiFont,
    pub size: u32,
    pub color: Rgb,
}

impl Ink {
    /// A line whose top left corner is at `at`.
    pub fn run(self, at: (i32, i32), text: String) -> TextRun {
        TextRun {
            x: at.0,
            y: at.1,
            text,
            color: self.color,
            font: self.font,
            size: self.size,
        }
    }

    /// A line that starts at `x`, halfway down `area`. Returns how wide it is.
    pub fn beside(
        self,
        x: i32,
        area: Rect,
        text: String,
        measure: &dyn TextMeasure,
        into: &mut DrawList,
    ) -> u32 {
        let width = measure.width(&text, self.font, self.size);
        let line = measure.line_height(self.font, self.size);
        let y = area.y.saturating_add(half_of_the_rest(area.height, line));
        into.texts.push(self.run((x, y), text));
        width
    }

    /// An icon in the middle of `area`.
    pub fn centred(self, icon: Icon, area: Rect, measure: &dyn TextMeasure, into: &mut DrawList) {
        let drawn = icon.character().to_string();
        let width = measure.width(&drawn, self.font, self.size);
        let x = area.x.saturating_add(half_of_the_rest(area.width, width));
        self.beside(x, area, drawn, measure, into);
    }
}

/// How far to move something `inner` long to centre it in `outer`.
pub fn half_of_the_rest(outer: u32, inner: u32) -> i32 {
    i32::try_from(outer.saturating_sub(inner) / 2).unwrap_or(0)
}
