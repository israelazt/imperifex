//! What the interface asks to have drawn, in physical pixels from the top left
//! of the window.

use crate::theme::Rgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// The fonts of the interface; the grid has the one chosen in the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFont {
    Text,
    Icons,
}

/// One line of text, from its top left corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub color: Rgb,
    pub font: UiFont,
}

/// Rectangles first, in order, and the texts over them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DrawList {
    pub rects: Vec<(Rect, Rgb)>,
    pub texts: Vec<TextRun>,
}
