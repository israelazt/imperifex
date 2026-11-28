//! What the bar writes: the name of the app, the icons of the buttons and the
//! help of the button the mouse rests on.

use imperifex_domain::draw::{DrawList, Icon, Rect, TextRun, UiFont};
use imperifex_domain::i18n::{text, Text};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::theme::Rgb;
use imperifex_domain::view::Model;

use super::{button_area, height, physical, Button};
use crate::layout;

/// In logical pixels: the line around the help.
const BORDER: f32 = 1.0;

/// A text in the font `font`, whose line starts at `at`.
fn run(model: &Model, at: (i32, i32), text: String, (font, color): (UiFont, Rgb)) -> TextRun {
    TextRun {
        x: at.0,
        y: at.1,
        text,
        color,
        font,
        size: font.size(model.scale),
    }
}

/// How far to move something `inner` long to centre it in `outer`.
fn half_of_the_rest(outer: u32, inner: u32) -> i32 {
    i32::try_from(outer.saturating_sub(inner) / 2).unwrap_or(0)
}

/// The name of the app, at the left of the strip and halfway down it.
pub fn title(model: &Model, ink: Rgb, measure: &dyn TextMeasure, into: &mut DrawList) {
    let line = measure.line_height(UiFont::Text, model.scale);
    let x = i32::try_from(Space::M.px(model.scale)).unwrap_or(0);
    let y = half_of_the_rest(height(model.scale), line);
    let style = (UiFont::Text, ink);
    into.texts
        .push(run(model, (x, y), model.title.clone(), style));
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
    ink: Rgb,
    measure: &dyn TextMeasure,
    into: &mut DrawList,
) {
    let drawn = icon_of(button, model.maximized).character().to_string();
    let width = measure.width(&drawn, UiFont::Icons, model.scale);
    let line = measure.line_height(UiFont::Icons, model.scale);
    let x = area.x.saturating_add(half_of_the_rest(area.width, width));
    let y = area.y.saturating_add(half_of_the_rest(area.height, line));
    into.texts
        .push(run(model, (x, y), drawn, (UiFont::Icons, ink)));
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
    let width = measure.width(said, UiFont::Text, model.scale) + padding * 2;
    let line = measure.line_height(UiFont::Text, model.scale);
    let under = button_area(button, model.window.0, model.scale);
    // Centred under the button, and pushed left if it would stick out.
    let centred = i64::from(under.x) + (i64::from(under.width) - i64::from(width)) / 2;
    let furthest = i64::from(model.window.0) - i64::from(width) - i64::from(margin);
    let area = Rect {
        x: i32::try_from(centred.min(furthest).max(0)).unwrap_or(0),
        y: i32::try_from(height(model.scale) + margin).unwrap_or(0),
        width,
        height: line + padding * 2,
    };
    into.floating_rects.push((area, model.palette.muted));
    let border = physical(BORDER, model.scale).max(1);
    let inside = Rect {
        x: area.x + i32::try_from(border).unwrap_or(0),
        y: area.y + i32::try_from(border).unwrap_or(0),
        width: area.width.saturating_sub(border * 2),
        height: area.height.saturating_sub(border * 2),
    };
    into.floating_rects.push((inside, model.palette.raised));
    let place = layout::inset(area, Space::S, model.scale);
    let style = (UiFont::Text, model.palette.text);
    into.floating_texts
        .push(run(model, (place.x, place.y), said.into(), style));
}
