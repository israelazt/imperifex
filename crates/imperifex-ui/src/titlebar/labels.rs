//! What the bar writes: the tab of the shell, the icons of the buttons and the
//! help of the button the mouse rests on.

use imperifex_domain::draw::{DrawList, Icon, Rect, Rounded, UiFont};
use imperifex_domain::i18n::{text, Text};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::theme::Rgb;
use imperifex_domain::view::Model;

use super::{button_area, height, Button};
use crate::text::{half_of_the_rest, Ink};
use crate::{layout, metrics};

fn signed(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}

/// The tab of the shell that is open, at the left of the strip: its icon and
/// the name of its profile. One that does not fit before the buttons is not
/// drawn. A press on it moves the window, as on the rest of the strip: with one
/// tab there is nothing to choose.
pub fn tab(model: &Model, color: Rgb, measure: &dyn TextMeasure, into: &mut DrawList) {
    let Some(name) = &model.tab else { return };
    let scale = model.scale;
    let (padding, gap) = (Space::M.px(scale), Space::S.px(scale));
    let icon = metrics::px(metrics::TAB_ICON, scale);
    let size = metrics::px(metrics::TEXT, scale);
    let words = measure.width(name, UiFont::Text, size);
    let tall = metrics::px(metrics::TAB_HEIGHT, scale);
    let area = Rect {
        x: signed(Space::S.px(scale)),
        y: half_of_the_rest(height(scale), tall),
        width: padding * 2 + icon + gap + words,
        height: tall,
    };
    let room = button_area(Button::Minimize, model.window.0, scale).x;
    if area.x.saturating_add(signed(area.width)) > room {
        return;
    }
    into.rounded.push(Rounded {
        area,
        radius: metrics::px(metrics::TAB_RADIUS, scale),
        color: model.palette.raised,
    });
    let icon_area = Rect {
        x: area.x + signed(padding),
        width: icon,
        ..area
    };
    let glyph = Ink {
        font: UiFont::Icons,
        size: icon,
        color,
    };
    glyph.centred(Icon::Shell, icon_area, measure, into);
    let at = icon_area.x + signed(icon + gap);
    let letters = Ink {
        font: UiFont::Text,
        size,
        color,
    };
    letters.beside(at, area, name.clone(), measure, into);
}

fn icon_of(button: Button, maximized: bool) -> Icon {
    match button {
        Button::Minimize => Icon::Minimize,
        Button::Maximize if maximized => Icon::Restore,
        Button::Maximize => Icon::Maximize,
        Button::Close => Icon::Close,
    }
}

/// The icon of a button, in the middle of its `area`.
pub fn icon(
    button: Button,
    model: &Model,
    area: Rect,
    color: Rgb,
    measure: &dyn TextMeasure,
    into: &mut DrawList,
) {
    let glyph = Ink {
        font: UiFont::Icons,
        size: metrics::px(metrics::BUTTON_ICON, model.scale),
        color,
    };
    glyph.centred(icon_of(button, model.maximized), area, measure, into);
}

fn help_of(button: Button, maximized: bool) -> Text {
    match button {
        Button::Minimize => Text::Minimize,
        Button::Maximize if maximized => Text::Restore,
        Button::Maximize => Text::Maximize,
        Button::Close => Text::Close,
    }
}

/// What the button does, in a box under it that does not leave the window.
pub fn help(button: Button, model: &Model, measure: &dyn TextMeasure, into: &mut DrawList) {
    let said = text(model.language, help_of(button, model.maximized));
    let padding = Space::S.px(model.scale);
    let margin = Space::Xs.px(model.scale);
    let letters = Ink {
        font: UiFont::Text,
        size: metrics::px(metrics::SECONDARY_TEXT, model.scale),
        color: model.palette.text,
    };
    let width = measure.width(said, letters.font, letters.size) + padding * 2;
    let line = measure.line_height(letters.font, letters.size);
    let under = button_area(button, model.window.0, model.scale);
    // Centred under the button, and pushed left if it would stick out.
    let centred = i64::from(under.x) + (i64::from(under.width) - i64::from(width)) / 2;
    let furthest = i64::from(model.window.0) - i64::from(width) - i64::from(margin);
    let area = Rect {
        x: i32::try_from(centred.min(furthest).max(0)).unwrap_or(0),
        y: signed(height(model.scale) + margin),
        width,
        height: line + padding * 2,
    };
    into.floating_rects.push((area, model.palette.muted));
    let border = metrics::px(metrics::BORDER, model.scale).max(1);
    let inside = Rect {
        x: area.x + signed(border),
        y: area.y + signed(border),
        width: area.width.saturating_sub(border * 2),
        height: area.height.saturating_sub(border * 2),
    };
    into.floating_rects.push((inside, model.palette.raised));
    let place = layout::inset(area, Space::S, model.scale);
    into.floating_texts
        .push(letters.run((place.x, place.y), said.into()));
}
