//! From the words of the interface to winit's, and back.

use imperifex_domain::input::{KeyInput, NamedKey};
use imperifex_domain::view::{CursorIcon, Edge, Point};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{KeyEvent, MouseScrollDelta};
use winit::keyboard::{Key, ModifiersState, NamedKey as Named};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
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

fn named_key(key: &Key) -> Option<NamedKey> {
    let Key::Named(named) = key else { return None };
    Some(match named {
        Named::Enter => NamedKey::Enter,
        Named::Backspace => NamedKey::Backspace,
        Named::Tab => NamedKey::Tab,
        Named::Escape => NamedKey::Escape,
        Named::ArrowUp => NamedKey::Up,
        Named::ArrowDown => NamedKey::Down,
        Named::ArrowRight => NamedKey::Right,
        Named::ArrowLeft => NamedKey::Left,
        Named::Home => NamedKey::Home,
        Named::End => NamedKey::End,
        Named::Insert => NamedKey::Insert,
        Named::Delete => NamedKey::Delete,
        Named::PageUp => NamedKey::PageUp,
        Named::PageDown => NamedKey::PageDown,
        Named::F1 => NamedKey::Function(1),
        Named::F2 => NamedKey::Function(2),
        Named::F3 => NamedKey::Function(3),
        Named::F4 => NamedKey::Function(4),
        Named::F5 => NamedKey::Function(5),
        Named::F6 => NamedKey::Function(6),
        Named::F7 => NamedKey::Function(7),
        Named::F8 => NamedKey::Function(8),
        Named::F9 => NamedKey::Function(9),
        Named::F10 => NamedKey::Function(10),
        Named::F11 => NamedKey::Function(11),
        Named::F12 => NamedKey::Function(12),
        _ => return None,
    })
}

/// `Alt`+`F4` closes the window and `Alt`+`Space` opens its menu: Windows has
/// them, and the program in the terminal is told nothing.
fn is_the_systems(event: &KeyEvent, modifiers: ModifiersState) -> bool {
    modifiers.alt_key()
        && !modifiers.control_key()
        && matches!(event.logical_key, Key::Named(Named::F4 | Named::Space))
}

/// What the key writes. `winit`'s own `text` leaves `Ctrl` out, so with it
/// `Ctrl`+`C` would write a `c`: with `Ctrl` held, the other one is asked.
fn text_of(event: &KeyEvent, modifiers: ModifiersState) -> Option<String> {
    let text = if modifiers.control_key() {
        event.text_with_all_modifiers()
    } else {
        event.text.as_deref()
    };
    let text = text.filter(|text| !text.is_empty()).map(str::to_string);
    // With only Alt, Windows may give no text: the key's own character is it.
    text.or_else(|| {
        let alone = modifiers.alt_key() && !modifiers.control_key();
        match event.key_without_modifiers() {
            Key::Character(character) if alone => Some(character.to_string()),
            _ => None,
        }
    })
}

pub fn key(event: &KeyEvent, modifiers: ModifiersState) -> KeyInput {
    if is_the_systems(event, modifiers) {
        return KeyInput::default();
    }
    KeyInput {
        text: text_of(event, modifiers),
        key: named_key(&event.logical_key),
        shift: modifiers.shift_key(),
        ctrl: modifiers.control_key(),
        alt: modifiers.alt_key(),
    }
    .without_alt_graph()
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
