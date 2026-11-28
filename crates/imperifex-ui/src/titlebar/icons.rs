//! The icons of the window's buttons, made of rectangles until there are fonts.

use imperifex_domain::draw::{DrawList, Rect};
use imperifex_domain::theme::Rgb;
use imperifex_domain::view::Model;

use super::{physical, Button};

/// In logical pixels.
const SIZE: f32 = 10.0;
const LINE: f32 = 1.0;
/// How far the second window of "restore" stands out from the first.
const OFFSET: f32 = 2.0;

/// An icon in the middle of `area`, in the colour `ink` over `under`. Close is
/// a filled square: rectangles cannot make its cross.
pub fn draw(
    button: Button,
    model: &Model,
    area: Rect,
    (ink, under): (Rgb, Rgb),
    into: &mut DrawList,
) {
    let size = physical(SIZE, model.scale);
    let line = physical(LINE, model.scale).max(1);
    let square = centred(area, size, size);
    match button {
        Button::Minimize => into.rects.push((centred(area, size, line), ink)),
        Button::Maximize if model.maximized => {
            let offset = physical(OFFSET, model.scale);
            let side = size.saturating_sub(offset);
            let back = at(square, offset, 0, side);
            let front = at(square, 0, offset, side);
            outline(back, line, (ink, under), into);
            outline(front, line, (ink, under), into);
        }
        Button::Maximize => outline(square, line, (ink, under), into),
        Button::Close => into.rects.push((square, ink)),
    }
}

fn centred(area: Rect, width: u32, height: u32) -> Rect {
    let middle = |outer: u32, inner: u32| i32::try_from(outer.saturating_sub(inner) / 2);
    Rect {
        x: area
            .x
            .saturating_add(middle(area.width, width).unwrap_or(0)),
        y: area
            .y
            .saturating_add(middle(area.height, height).unwrap_or(0)),
        width,
        height,
    }
}

/// A square of side `side`, moved from the corner of `from`.
fn at(from: Rect, right: u32, down: u32, side: u32) -> Rect {
    Rect {
        x: from.x.saturating_add(i32::try_from(right).unwrap_or(0)),
        y: from.y.saturating_add(i32::try_from(down).unwrap_or(0)),
        width: side,
        height: side,
    }
}

/// A frame of lines of thickness `line`: its inside is painted `under`, so it
/// hides what is behind it.
fn outline(area: Rect, line: u32, (ink, under): (Rgb, Rgb), into: &mut DrawList) {
    into.rects.push((area, ink));
    let step = i32::try_from(line).unwrap_or(0);
    let inside = Rect {
        x: area.x.saturating_add(step),
        y: area.y.saturating_add(step),
        width: area.width.saturating_sub(line.saturating_mul(2)),
        height: area.height.saturating_sub(line.saturating_mul(2)),
    };
    into.rects.push((inside, under));
}
