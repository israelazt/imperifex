//! The interface of Imperifex: from a [`Model`] and the mouse to what is drawn
//! and what the user asked for. It has no colours or texts of its own: colours
//! come from the model's palette and texts from the domain. The grid is not
//! drawn here: the frame says where it goes.

#![forbid(unsafe_code)]

pub mod layout;
pub mod titlebar;

use imperifex_domain::draw::{DrawList, Rect};
use imperifex_domain::space::Space;
use imperifex_domain::view::{Action, CursorIcon, Model, Pointer};

/// One drawing of the window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frame {
    pub draw: DrawList,
    pub actions: Vec<Action>,
    /// Where the renderer draws the grid.
    pub terminal_area: Rect,
    pub cursor: CursorIcon,
}

/// The window: its title bar, and under it the place of the output with the
/// base space around it.
pub fn frame(model: &Model, pointer: &Pointer) -> Frame {
    let (width, height) = model.window;
    let window = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let bar = titlebar::height(model.scale).min(height);
    let below = Rect {
        y: i32::try_from(bar).unwrap_or(0),
        height: height - bar,
        ..window
    };
    let terminal_area = layout::inset(below, Space::L, model.scale);
    let mut draw = DrawList::default();
    draw.rects.push((window, model.palette.background));
    draw.rects.push((terminal_area, model.palette.surface));
    titlebar::draw(model, pointer, &mut draw);
    let (action, cursor) = titlebar::input(model, pointer);
    Frame {
        draw,
        actions: action.into_iter().collect(),
        terminal_area,
        cursor,
    }
}

#[cfg(test)]
mod tests;
