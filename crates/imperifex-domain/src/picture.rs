//! What the renderer needs to draw the grid of a terminal. The interface says
//! only where it goes: a list of thousands of texts each frame would be slow.

use crate::draw::Rect;
use crate::font::CellSize;
use crate::grid::Damage;
use crate::terminal::Terminal;
use crate::theme::Palette;

#[derive(Debug, Clone, Copy)]
pub struct TerminalPicture<'a> {
    pub terminal: &'a Terminal,
    /// The rows that changed since the last picture given to the same renderer.
    /// A new renderer draws them all, whatever this says.
    pub damage: &'a Damage,
    /// The row of the screen at the top of the area: negative ones are of the
    /// history. With any but 0, `damage` says nothing of what is seen.
    pub first: isize,
    /// Where the grid goes. Its top left cell is at the area's, and only whole
    /// cells are drawn.
    pub area: Rect,
    pub palette: &'static Palette,
    pub adjust_contrast: bool,
    pub cell: CellSize,
    /// The size of the letters in physical pixels.
    pub letters: f32,
    /// How thick the lines are: under and through the text, and around the
    /// cursor of a window without focus.
    pub line: u32,
    /// The window has the keyboard: the cursor is filled.
    pub focused: bool,
}
