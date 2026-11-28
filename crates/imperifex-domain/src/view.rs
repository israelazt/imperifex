//! What crosses between the window and the interface: what there is to show,
//! where the mouse is and what the user asked for.

use crate::i18n::Language;
use crate::theme::Palette;

/// Everything the interface needs to draw the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub title: String,
    /// The name of the profile of the shell that is open, if one is.
    pub tab: Option<String>,
    pub language: Language,
    pub palette: &'static Palette,
    /// The system's scale: 1.0 is 100 %.
    pub scale: f32,
    /// Width and height in physical pixels.
    pub window: (u32, u32),
    pub maximized: bool,
    pub focused: bool,
    /// A program uses the whole screen: the output gets no margins (T-D5).
    pub full_screen: bool,
    /// What went wrong, one line each, shown where the output goes.
    pub notice: Vec<String>,
    /// The command editor, while a command is to be written in it.
    pub editor: Option<EditorView>,
}

/// The command editor as it is to be drawn.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorView {
    /// The shell's prompt, one line per row.
    pub prompt: Vec<String>,
    pub lines: Vec<String>,
    /// The line of the cursor and how many characters are before it.
    pub cursor: (usize, usize),
    /// What follows the text, in grey.
    pub suggestion: String,
    /// The command has several lines and this shell cannot take them.
    pub refused: bool,
    /// The size of the letters of the prompt and the command, in physical
    /// pixels: that of the output.
    pub letters: u32,
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
