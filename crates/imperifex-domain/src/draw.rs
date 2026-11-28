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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiFont {
    Text,
    Icons,
}

/// The icons of the interface, as characters of its icon font. The one of a
/// shell is not in it: it is drawn by [`crate::prompt_icon`], and its
/// character only names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Minimize,
    Maximize,
    Restore,
    Close,
    /// A tab with a shell.
    Shell,
}

impl Icon {
    pub const ALL: [Self; 5] = [
        Self::Minimize,
        Self::Maximize,
        Self::Restore,
        Self::Close,
        Self::Shell,
    ];

    /// The same in Segoe Fluent Icons and in Segoe MDL2 Assets.
    pub fn character(self) -> char {
        match self {
            Self::Minimize => '\u{E921}',
            Self::Maximize => '\u{E922}',
            Self::Restore => '\u{E923}',
            Self::Close => '\u{E8BB}',
            Self::Shell => '\u{F8FF}',
        }
    }
}

/// One line of text, from its top left corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub color: Rgb,
    pub font: UiFont,
    /// The size of its letters in physical pixels.
    pub size: u32,
}

/// A rectangle with its corners rounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rounded {
    pub area: Rect,
    /// In physical pixels.
    pub radius: u32,
    pub color: Rgb,
}

/// Rectangles first, in order, then the rounded ones, and the texts over them. What floats is drawn
/// last, the same way, over those and over the grid of the terminal, which goes
/// between the rectangles and the texts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DrawList {
    pub rects: Vec<(Rect, Rgb)>,
    pub rounded: Vec<Rounded>,
    pub texts: Vec<TextRun>,
    pub floating_rects: Vec<(Rect, Rgb)>,
    pub floating_texts: Vec<TextRun>,
}
