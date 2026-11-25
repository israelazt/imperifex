//! The windows of the desktop as the bench sees them.

/// A rectangle in physical pixels, from the top left corner of the primary
/// monitor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// One top level window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowEntry {
    /// The system's handle for it.
    pub hwnd: u64,
    /// The process it belongs to.
    pub pid: u32,
    pub title: String,
    /// The name of its window class, as its program registered it.
    pub class: String,
    pub visible: bool,
    /// The frame the user sees; empty if the system will not say.
    pub rect: Rect,
}
