//! What crosses between the window and the interface: what there is to show,
//! where the mouse is and what the user asked for.

use crate::i18n::Language;
use crate::theme::Palette;

/// Everything the interface needs to draw the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub title: String,
    pub language: Language,
    pub palette: &'static Palette,
    /// The system's scale: 1.0 is 100 %.
    pub scale: f32,
    /// Width and height in physical pixels.
    pub window: (u32, u32),
    pub maximized: bool,
    pub focused: bool,
}

/// A place in physical pixels from the top left of the window.
pub type Point = (i32, i32);

/// What the mouse is doing, as the window saw it since the last frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pointer {
    /// `None` when it is outside the window.
    pub position: Option<Point>,
    /// Where the left button went down, while it is down and on its release.
    pub held_from: Option<Point>,
    pub pressed: bool,
    pub released: bool,
    pub double_click: bool,
    pub right_click: bool,
    /// It has been still long enough to be told what is under it.
    pub rested: bool,
}

/// A side or a corner of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// What the user asked of the program through the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Minimize,
    ToggleMaximize,
    Close,
    /// Move the window with the mouse.
    Drag,
    /// Change its size from that edge.
    Resize(Edge),
    /// The menu of the window that Windows draws.
    SystemMenu,
}

/// The shape the mouse takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorIcon {
    #[default]
    Default,
    Resize(Edge),
}
