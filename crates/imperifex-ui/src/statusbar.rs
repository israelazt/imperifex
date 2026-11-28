//! The bar at the bottom: which profile the shell is. The counts of agents and
//! the notices will go at its right.

use imperifex_domain::draw::{DrawList, Icon, Rect, UiFont};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::view::Model;

use crate::metrics;
use crate::text::Ink;

pub fn height(scale: f32) -> u32 {
    metrics::px(metrics::STATUS_BAR_HEIGHT, scale)
}

/// The bar in `area`.
pub fn draw(model: &Model, area: Rect, measure: &dyn TextMeasure, into: &mut DrawList) {
    into.rects.push((area, model.palette.background));
    let Some(name) = &model.tab else { return };
    let scale = model.scale;
    let icon = metrics::px(metrics::STATUS_ICON, scale);
    let left = area.x.saturating_add(signed(Space::L.px(scale)));
    let glyph = Ink {
        font: UiFont::Icons,
        size: icon,
        color: model.palette.muted,
    };
    let icon_area = Rect {
        x: left,
        width: icon,
        ..area
    };
    glyph.centred(Icon::Shell, icon_area, measure, into);
    let letters = Ink {
        font: UiFont::Text,
        size: metrics::px(metrics::SECONDARY_TEXT, scale),
        color: model.palette.muted,
    };
    let at = left.saturating_add(signed(icon + Space::Xs.px(scale)));
    letters.beside(at, area, name.clone(), measure, into);
}

fn signed(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}
