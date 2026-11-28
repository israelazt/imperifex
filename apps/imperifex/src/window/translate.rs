//! From the words of the interface to winit's, and back.

use imperifex_domain::input::{KeyInput, NamedKey};
use imperifex_domain::view::{CursorIcon, Edge, Point};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{KeyEvent, MouseScrollDelta};
use winit::keyboard::{Key, ModifiersState, NamedKey as Named};
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

/// The size of what is seen of the window, in physical pixels. Minimized,
/// nothing is, though Windows gives it a small size (160×28 here) that would
/// cut the terminal's lines to fit in it.
pub fn seen_size(window: &Window) -> (u32, u32) {
    if window.is_minimized() == Some(true) {
        return (0, 0);
    }
    let size = window.inner_size();
    (size.width, size.height)
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

pub fn key(event: &KeyEvent, modifiers: ModifiersState) -> KeyInput {
    let named = match event.logical_key {
        Key::Named(Named::Enter) => Some(NamedKey::Enter),
        Key::Named(Named::Backspace) => Some(NamedKey::Backspace),
        Key::Named(Named::Tab) => Some(NamedKey::Tab),
        Key::Named(Named::Escape) => Some(NamedKey::Escape),
        Key::Named(Named::ArrowUp) => Some(NamedKey::Up),
        Key::Named(Named::ArrowDown) => Some(NamedKey::Down),
        Key::Named(Named::ArrowRight) => Some(NamedKey::Right),
        Key::Named(Named::ArrowLeft) => Some(NamedKey::Left),
        Key::Named(Named::Home) => Some(NamedKey::Home),
        Key::Named(Named::End) => Some(NamedKey::End),
        Key::Named(Named::Delete) => Some(NamedKey::Delete),
        _ => None,
    };
    KeyInput {
        text: event.text.as_ref().map(ToString::to_string),
        key: named,
        shift: modifiers.shift_key(),
        ctrl: modifiers.control_key(),
    }
}

/// The keys that are for the window and not for what is written.
pub enum Shortcut {
    /// `Shift` with `Page Up` or `Page Down`: look back, or forward.
    Page { back: bool },
    /// `Ctrl`+`Shift`+`E` (T-D25).
    ToggleEditor,
    /// `Ctrl`+`V`, with Shift or not, and `Shift`+`Insert`.
    Paste,
}

pub fn shortcut(event: &KeyEvent, modifiers: ModifiersState) -> Option<Shortcut> {
    let shift = modifiers.shift_key();
    match &event.logical_key {
        Key::Named(Named::PageUp) if shift => Some(Shortcut::Page { back: true }),
        Key::Named(Named::PageDown) if shift => Some(Shortcut::Page { back: false }),
        Key::Named(Named::Insert) if shift => Some(Shortcut::Paste),
        // With AltGr, which Windows gives as Ctrl and Alt, a letter writes.
        Key::Character(letter) if modifiers.control_key() && !modifiers.alt_key() => {
            if letter.eq_ignore_ascii_case("v") {
                Some(Shortcut::Paste)
            } else {
                (shift && letter.eq_ignore_ascii_case("e")).then_some(Shortcut::ToggleEditor)
            }
        }
        _ => None,
    }
}

/// How far the wheel turned, in notches, up when positive.
pub fn notches(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, up) => up,
        // As much as Windows counts for one notch.
        MouseScrollDelta::PixelDelta(pixels) => (pixels.y / 120.0) as f32,
    }
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
