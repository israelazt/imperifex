//! What went wrong, told inside the window.

use imperifex_domain::draw::{DrawList, Rect, TextRun, UiFont};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::view::Model;

/// The model's lines from the top left of `area`, one under the other.
pub fn draw(model: &Model, area: Rect, measure: &dyn TextMeasure, into: &mut DrawList) {
    let line = measure.line_height(UiFont::Text, model.scale);
    let step = i32::try_from(line).unwrap_or(0);
    let mut y = area.y;
    for said in &model.notice {
        into.texts.push(TextRun {
            x: area.x,
            y,
            text: said.clone(),
            color: model.palette.error,
            font: UiFont::Text,
            size: UiFont::Text.size(model.scale),
        });
        y = y.saturating_add(step);
    }
}
