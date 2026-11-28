//! Which part of the probe's window the mouse is over.

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::ResizeDirection;

pub const BAR: f64 = 40.0;
const EDGE: f64 = 8.0;
const BUTTON: f64 = 46.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zone {
    Edge(ResizeDirection),
    MaximizeSlot,
    Bar,
    Rest,
}

pub fn zone(
    at: PhysicalPosition<f64>,
    size: PhysicalSize<u32>,
    scale: f64,
    maximized: bool,
) -> Zone {
    let (width, height) = (f64::from(size.width), f64::from(size.height));
    if !maximized {
        let edge = EDGE * scale;
        let (west, east) = (at.x < edge, at.x >= width - edge);
        let (north, south) = (at.y < edge, at.y >= height - edge);
        if let Some(direction) = direction(north, south, west, east) {
            return Zone::Edge(direction);
        }
    }
    if at.y >= BAR * scale {
        return Zone::Rest;
    }
    let (left, right) = maximize_slot(width, scale);
    if (left..right).contains(&at.x) {
        Zone::MaximizeSlot
    } else {
        Zone::Bar
    }
}

/// From where to where, across the bar, the maximize button would be: the
/// second slot from the right.
pub fn maximize_slot(width: f64, scale: f64) -> (f64, f64) {
    (width - 2.0 * BUTTON * scale, width - BUTTON * scale)
}

fn direction(north: bool, south: bool, west: bool, east: bool) -> Option<ResizeDirection> {
    use ResizeDirection::*;
    Some(match (north, south, west, east) {
        (true, _, true, _) => NorthWest,
        (true, _, _, true) => NorthEast,
        (_, true, true, _) => SouthWest,
        (_, true, _, true) => SouthEast,
        (true, ..) => North,
        (_, true, ..) => South,
        (_, _, true, _) => West,
        (_, _, _, true) => East,
        _ => return None,
    })
}
