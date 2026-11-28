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

impl UiFont {
    /// The size of its letters in physical pixels at the window's `scale`.
    /// In logical pixels they are what Windows uses in its title bars.
    pub fn size(self, scale: f32) -> u32 {
        let logical = match self {
            Self::Text => 12.0,
            Self::Icons => 10.0,
        };
        // Rounded and never negative before the cast, which then cannot wrap.
        (logical * scale).round().max(0.0) as u32
    }
}

/// The icons of the interface, as characters of its icon font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl Icon {
    pub const ALL: [Self; 4] = [Self::Minimize, Self::Maximize, Self::Restore, Self::Close];

    /// The same in Segoe Fluent Icons and in Segoe MDL2 Assets.
    pub fn character(self) -> char {
        match self {
            Self::Minimize => '\u{E921}',
            Self::Maximize => '\u{E922}',
            Self::Restore => '\u{E923}',
            Self::Close => '\u{E8BB}',
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

/// Rectangles first, in order, and the texts over them. What floats is drawn
/// last, the same way, over those and over the grid of the terminal, which goes
/// between the rectangles and the texts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DrawList {
    pub rects: Vec<(Rect, Rgb)>,
    pub texts: Vec<TextRun>,
    pub floating_rects: Vec<(Rect, Rgb)>,
    pub floating_texts: Vec<TextRun>,
}
