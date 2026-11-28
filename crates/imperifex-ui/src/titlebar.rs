//! The title bar of our own: the strip that moves the window, its three
//! buttons and the edges that resize it. The tabs will go in the strip.

use imperifex_domain::draw::{DrawList, Rect};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::theme::{Palette, Rgb};
use imperifex_domain::view::{Action, CursorIcon, Edge, Model, Point, Pointer};

mod labels;

/// In logical pixels: sizes of their own, not steps of the spacing scale.
const HEIGHT: f32 = 40.0;
const BUTTON_WIDTH: f32 = 46.0;
const EDGE: f32 = 8.0;

/// A logical measure in whole physical pixels at the window's `scale`.
pub(crate) fn physical(logical: f32, scale: f32) -> u32 {
    // Rounded and never negative before the cast, which then cannot wrap.
    (logical * scale).round().max(0.0) as u32
}

pub fn height(scale: f32) -> u32 {
    physical(HEIGHT, scale)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Minimize,
    Maximize,
    Close,
}

impl Button {
    /// From left to right.
    pub const ALL: [Self; 3] = [Self::Minimize, Self::Maximize, Self::Close];

    fn action(self) -> Action {
        match self {
            Self::Minimize => Action::Minimize,
            Self::Maximize => Action::ToggleMaximize,
            Self::Close => Action::Close,
        }
    }
}

/// Where a button is: side by side against the right of the window.
pub fn button_area(button: Button, window_width: u32, scale: f32) -> Rect {
    let width = physical(BUTTON_WIDTH, scale);
    let from_the_right: u32 = match button {
        Button::Close => 1,
        Button::Maximize => 2,
        Button::Minimize => 3,
    };
    let left = i64::from(window_width) - i64::from(width) * i64::from(from_the_right);
    Rect {
        x: i32::try_from(left).unwrap_or(0),
        y: 0,
        width,
        height: height(scale),
    }
}

/// What of the window's frame is at a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
    Edge(Edge),
    Button(Button),
    Strip,
    Below,
}

fn zone(model: &Model, at: Point) -> Zone {
    // A maximized window has no edges: its buttons reach the top of the screen.
    if let Some(edge) = (!model.maximized).then(|| edge(model, at)).flatten() {
        return Zone::Edge(edge);
    }
    if i64::from(at.1) >= i64::from(height(model.scale)) {
        return Zone::Below;
    }
    Button::ALL
        .into_iter()
        .find(|&button| contains(button_area(button, model.window.0, model.scale), at))
        .map_or(Zone::Strip, Zone::Button)
}

fn edge(model: &Model, at: Point) -> Option<Edge> {
    let size = i64::from(physical(EDGE, model.scale));
    let (x, y) = (i64::from(at.0), i64::from(at.1));
    let (west, east) = (x < size, x >= i64::from(model.window.0) - size);
    let (north, south) = (y < size, y >= i64::from(model.window.1) - size);
    Some(match (north, south, west, east) {
        (true, _, true, _) => Edge::NorthWest,
        (true, _, _, true) => Edge::NorthEast,
        (_, true, true, _) => Edge::SouthWest,
        (_, true, _, true) => Edge::SouthEast,
        (true, ..) => Edge::North,
        (_, true, ..) => Edge::South,
        (_, _, true, _) => Edge::West,
        (_, _, _, true) => Edge::East,
        _ => return None,
    })
}

fn contains(area: Rect, at: Point) -> bool {
    let inside = |value: i32, start: i32, size: u32| {
        let offset = i64::from(value) - i64::from(start);
        (0..i64::from(size)).contains(&offset)
    };
    inside(at.0, area.x, area.width) && inside(at.1, area.y, area.height)
}

/// What the mouse asks of the window, and the shape it takes.
pub fn input(model: &Model, pointer: &Pointer) -> (Option<Action>, CursorIcon) {
    let Some(at) = pointer.position else {
        return (None, CursorIcon::Default);
    };
    let zone = zone(model, at);
    let cursor = match zone {
        Zone::Edge(edge) => CursorIcon::Resize(edge),
        _ => CursorIcon::Default,
    };
    (action(model, pointer, zone), cursor)
}

fn action(model: &Model, pointer: &Pointer, zone: Zone) -> Option<Action> {
    match zone {
        Zone::Edge(edge) => pointer.pressed.then_some(Action::Resize(edge)),
        // A button acts when released on it, if the press began there too.
        Zone::Button(button) => {
            let began_here = pointer
                .held_from
                .is_some_and(|from| self::zone(model, from) == zone);
            (pointer.released && began_here).then(|| button.action())
        }
        Zone::Strip if pointer.right_click => Some(Action::SystemMenu),
        Zone::Strip if pointer.double_click => Some(Action::ToggleMaximize),
        Zone::Strip => pointer.pressed.then_some(Action::Drag),
        Zone::Below => None,
    }
}

/// The strip, its name and its buttons, the one under the mouse lit.
pub fn draw(model: &Model, pointer: &Pointer, measure: &dyn TextMeasure, into: &mut DrawList) {
    let strip = Rect {
        x: 0,
        y: 0,
        width: model.window.0,
        height: height(model.scale),
    };
    into.rects.push((strip, model.palette.background));
    let name = if model.focused {
        model.palette.text
    } else {
        model.palette.muted
    };
    labels::title(model, name, measure, into);
    let hovered = hovered(model, pointer);
    for button in Button::ALL {
        let area = button_area(button, model.window.0, model.scale);
        let lit = hovered == Some(button);
        let (back, ink) = colours(button, lit, model.focused, model.palette);
        if let Some(back) = back {
            into.rects.push((area, back));
        }
        labels::icon(button, model, area, ink, measure, into);
    }
}

/// The help of the button the mouse rests on. It goes over everything else, so
/// it is drawn last.
pub fn draw_help(model: &Model, pointer: &Pointer, measure: &dyn TextMeasure, into: &mut DrawList) {
    if let Some(button) = hovered(model, pointer).filter(|_| pointer.rested) {
        labels::help(button, model, measure, into);
    }
}

fn hovered(model: &Model, pointer: &Pointer) -> Option<Button> {
    match zone(model, pointer.position?) {
        Zone::Button(button) => Some(button),
        _ => None,
    }
}

/// The background of a button, when it has one, and the colour of its icon.
fn colours(button: Button, lit: bool, focused: bool, palette: &Palette) -> (Option<Rgb>, Rgb) {
    match (lit, button) {
        (true, Button::Close) => (Some(palette.error), palette.surface),
        (true, _) => (Some(palette.raised), palette.text),
        (false, _) if focused => (None, palette.text),
        (false, _) => (None, palette.muted),
    }
}

#[cfg(test)]
mod tests;
