//! The interface of Imperifex: from a [`Model`] and the mouse to what is drawn
//! and what the user asked for. It has no colours or texts of its own: colours
//! come from the model's palette and texts from the domain. The grid is not
//! drawn here: the frame says where it goes.

#![forbid(unsafe_code)]

mod blocks;
mod editor;
pub mod layout;
pub mod metrics;
mod notice;
pub mod statusbar;
#[cfg(test)]
mod testing;
mod text;
pub mod titlebar;

use imperifex_domain::draw::{DrawList, Rect};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::view::{Action, CursorIcon, Model, Pointer};

/// One drawing of the window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frame {
    pub draw: DrawList,
    pub actions: Vec<Action>,
    /// All the room of the output: the console is as large as it.
    pub terminal_area: Rect,
    /// Where the renderer draws the grid: less than that with the command
    /// editor under it.
    pub output_area: Rect,
    pub cursor: CursorIcon,
}

/// The window: its title bar, the status bar at the bottom and between them the
/// output, with the base space around its grid unless a program uses the whole
/// screen.
pub fn frame(model: &Model, pointer: &Pointer, measure: &dyn TextMeasure) -> Frame {
    let (width, height) = model.window;
    let window = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let bar = titlebar::height(model.scale).min(height);
    let status = statusbar::height(model.scale).min(height - bar);
    let below = Rect {
        y: i32::try_from(bar).unwrap_or(0),
        height: height - bar - status,
        ..window
    };
    let bottom = Rect {
        y: i32::try_from(height - status).unwrap_or(0),
        height: status,
        ..window
    };
    let terminal_area = if model.full_screen {
        below
    } else {
        layout::inset(below, Space::L, model.scale)
    };
    let mut draw = DrawList::default();
    draw.rects.push((window, model.palette.background));
    draw.rects.push((below, model.palette.surface));
    titlebar::draw(model, pointer, measure, &mut draw);
    blocks::draw_behind(model, below.y, &mut draw);
    blocks::draw_labels(model, pointer, measure, &mut draw);
    notice::draw(model, terminal_area, measure, &mut draw);
    statusbar::draw(model, bottom, measure, &mut draw);
    let output_area = with_editor(model, terminal_area, bottom.y, measure, &mut draw);
    titlebar::draw_help(model, pointer, measure, &mut draw);
    let (action, cursor) = titlebar::input(model, pointer);
    let action = action
        .or_else(|| in_editor(model, pointer, bottom.y, measure))
        .or_else(|| blocks::input(model, pointer, measure));
    Frame {
        draw,
        actions: action.into_iter().collect(),
        terminal_area,
        output_area,
        cursor,
    }
}

/// Draws the editor, when there is one, against `bottom`, and says what of the
/// output's `area` is left for the grid.
fn with_editor(
    model: &Model,
    area: Rect,
    bottom: i32,
    measure: &dyn TextMeasure,
    draw: &mut DrawList,
) -> Rect {
    match &model.editor {
        Some(view) => {
            let top = editor::draw(model, view, bottom, measure, draw);
            over(area, top, model.scale)
        }
        None => area,
    }
}

/// A press in the box of the editor puts its cursor there.
fn in_editor(
    model: &Model,
    pointer: &Pointer,
    bottom: i32,
    measure: &dyn TextMeasure,
) -> Option<Action> {
    let view = model.editor.as_ref().filter(|_| pointer.pressed)?;
    let at = pointer.position?;
    let (line, column) = editor::place_at(model, view, bottom, at, measure)?;
    Some(Action::PlaceCursor { line, column })
}

/// What of `area` is over `top`, with (`M`) between them.
fn over(area: Rect, top: i32, scale: f32) -> Rect {
    let gap = i32::try_from(Space::M.px(scale)).unwrap_or(0);
    let end = top.saturating_sub(gap);
    let tall = u32::try_from(end.saturating_sub(area.y)).unwrap_or(0);
    Rect {
        height: tall.min(area.height),
        ..area
    }
}

#[cfg(test)]
mod tests;
