//! The interface of Imperifex: from a [`Model`] and the mouse to what is drawn
//! and what the user asked for. It has no colours or texts of its own: colours
//! come from the model's palette and texts from the domain. The grid is not
//! drawn here: the frame says where it goes.

#![forbid(unsafe_code)]

pub mod layout;

use imperifex_domain::draw::{DrawList, Rect};
use imperifex_domain::ports::TextMeasure;
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

/// The window so far: its background and the place of the output. The title bar
/// brings the use of the mouse and of the text measure.
pub fn frame(model: &Model, _pointer: &Pointer, _measure: &dyn TextMeasure) -> Frame {
    let (width, height) = model.window;
    let window = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let terminal_area = layout::inset(window, Space::L, model.scale);
    let mut draw = DrawList::default();
    draw.rects.push((window, model.palette.background));
    draw.rects.push((terminal_area, model.palette.surface));
    Frame {
        draw,
        terminal_area,
        ..Frame::default()
    }
}

#[cfg(test)]
mod tests;
