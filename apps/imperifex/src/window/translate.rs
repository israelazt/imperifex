//! From the words of the interface to winit's, and back.

use imperifex_domain::view::{CursorIcon, Edge, Point};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{CursorIcon as Cursor, ResizeDirection, Window, WindowAttributes};

/// In logical pixels.
const FIRST_SIZE: LogicalSize<f64> = LogicalSize::new(960.0, 600.0);
const SMALLEST_SIZE: LogicalSize<f64> = LogicalSize::new(640.0, 400.0);

/// No system title bar (the interface draws its own), but the shadow and the
/// corners of a window with a frame. Hidden until its first frame is drawn.
pub fn attributes(title: String) -> WindowAttributes {
    Window::default_attributes()
        .with_title(title)
        .with_inner_size(FIRST_SIZE)
        .with_min_inner_size(SMALLEST_SIZE)
        .with_decorations(false)
        .with_undecorated_shadow(true)
        .with_visible(false)
}

/// The number Windows knows the window by.
pub fn handle(window: &Window) -> Result<isize, String> {
    match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Win32(handle)) => Ok(handle.hwnd.get()),
        _ => Err("the window has no Win32 handle".into()),
    }
}

pub fn point(position: PhysicalPosition<f64>) -> Point {
    let whole = position.cast::<i32>();
    (whole.x, whole.y)
}

pub fn direction(edge: Edge) -> ResizeDirection {
    match edge {
        Edge::North => ResizeDirection::North,
        Edge::South => ResizeDirection::South,
        Edge::East => ResizeDirection::East,
        Edge::West => ResizeDirection::West,
        Edge::NorthEast => ResizeDirection::NorthEast,
        Edge::NorthWest => ResizeDirection::NorthWest,
        Edge::SouthEast => ResizeDirection::SouthEast,
        Edge::SouthWest => ResizeDirection::SouthWest,
    }
}

pub fn cursor(icon: CursorIcon) -> Cursor {
    match icon {
        CursorIcon::Default => Cursor::Default,
        CursorIcon::Resize(edge) => direction(edge).into(),
    }
}
