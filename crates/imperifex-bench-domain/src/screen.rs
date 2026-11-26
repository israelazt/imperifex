//! A piece of the screen as the bench sees it.

use std::time::Duration;

/// One new picture of the watched region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenFrame {
    /// When it was presented on the monitor, on the bench's clock: not when
    /// the bench got it.
    pub at: Duration,
    /// How many presentations this picture adds up. Over 1, some went by
    /// unseen, and `at` is that of the last one.
    pub accumulated: u32,
    pub width: usize,
    pub height: usize,
    /// Four bytes a pixel, blue first, row after row.
    pub bgra: Vec<u8>,
}
