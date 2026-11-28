//! The blocks over the output (T5.2): the line between them, the chosen one's
//! background, and at the right of each command how long it ran, how it ended
//! and, on the chosen one, the button that copies what it wrote.

use imperifex_domain::draw::{DrawList, Rect, Rounded};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::view::{Action, BlockView, Model, Point, Pointer};

use crate::metrics;

mod labels;

fn signed(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}

fn within(value: i32, start: i32, size: u32) -> bool {
    let offset = i64::from(value) - i64::from(start);
    (0..i64::from(size)).contains(&offset)
}

/// Whether the mouse is on the rows of the block.
fn under_mouse(block: &BlockView, pointer: &Pointer) -> bool {
    pointer
        .position
        .is_some_and(|(_, y)| within(y, block.top, block.height))
}

/// The line over each block that starts under `ceiling` (A10: it goes between
/// two rows and takes no room), and the background of the chosen one.
pub fn draw_behind(model: &Model, ceiling: i32, into: &mut DrawList) {
    let side = Space::L.px(model.scale);
    let out = side.saturating_sub(Space::S.px(model.scale));
    for block in &model.blocks {
        if block.selected {
            into.rounded.push(Rounded {
                area: Rect {
                    x: signed(out),
                    y: block.top,
                    width: model.window.0.saturating_sub(out * 2),
                    height: block.height,
                },
                radius: metrics::px(metrics::BLOCK_RADIUS, model.scale),
                color: model.palette.raised,
            });
        }
        if block.starts_here && block.top > ceiling {
            let line = Rect {
                x: signed(side),
                y: block.top,
                width: model.window.0.saturating_sub(side * 2),
                height: metrics::px(metrics::BORDER, model.scale).max(1),
            };
            into.rects.push((line, model.palette.line));
        }
    }
}

/// What is written at the right of each command. It covers no text (A9): where
/// it does not fit it shows only while the mouse is on its block.
pub fn draw_labels(
    model: &Model,
    pointer: &Pointer,
    measure: &dyn TextMeasure,
    into: &mut DrawList,
) {
    for block in &model.blocks {
        let hovered = under_mouse(block, pointer);
        if let Some(labels) = labels::place(model, block, hovered, measure) {
            labels.draw(model, measure, into);
        }
    }
}

/// What a press on the output asks: copying, on the button of the chosen
/// block; else choosing the block under the mouse, if it ran something.
pub fn input(model: &Model, pointer: &Pointer, measure: &dyn TextMeasure) -> Option<Action> {
    let at: Point = pointer.position.filter(|_| pointer.pressed)?;
    let block = model
        .blocks
        .iter()
        .find(|block| under_mouse(block, pointer))?;
    let button = labels::place(model, block, true, measure).and_then(|found| found.button());
    let on_button = button
        .is_some_and(|area| within(at.0, area.x, area.width) && within(at.1, area.y, area.height));
    if on_button {
        return Some(Action::CopyBlock(block.id));
    }
    (block.command.is_some() || block.ran.is_some()).then_some(Action::SelectBlock(block.id))
}

#[cfg(test)]
mod tests;
