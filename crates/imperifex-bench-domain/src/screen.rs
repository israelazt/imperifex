//! A piece of the screen as the bench sees it.

use std::time::Duration;

use crate::desktop::Rect;
use crate::report::Monitor;

const BYTES_PER_PIXEL: usize = 4;

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

impl ScreenFrame {
    /// The part of the picture inside `area`, counted from the picture's top
    /// left corner. `None` unless the whole area, with some size, is in it.
    pub fn crop(&self, area: Rect) -> Option<Self> {
        let left = usize::try_from(area.x).ok()?;
        let top = usize::try_from(area.y).ok()?;
        let width = usize::try_from(area.w).ok().filter(|w| *w > 0)?;
        let height = usize::try_from(area.h).ok().filter(|h| *h > 0)?;
        let fits = left.checked_add(width)? <= self.width
            && top.checked_add(height)? <= self.height
            && self.bgra.len() == self.width * self.height * BYTES_PER_PIXEL;
        if !fits {
            return None;
        }
        let row = width * BYTES_PER_PIXEL;
        let mut bgra = Vec::with_capacity(row * height);
        for y in top..top + height {
            let from = (y * self.width + left) * BYTES_PER_PIXEL;
            bgra.extend_from_slice(&self.bgra[from..from + row]);
        }
        Some(Self {
            width,
            height,
            bgra,
            ..*self
        })
    }
}

/// The whole of the primary monitor, whose top left corner is where the desktop
/// counts from; `None` if no monitor is the primary one or it says no size.
pub fn primary_area(monitors: &[Monitor]) -> Option<Rect> {
    let primary = monitors.iter().find(|m| m.primary)?;
    let side = |length: u32| i32::try_from(length).ok().filter(|l| *l > 0);
    Some(Rect {
        x: 0,
        y: 0,
        w: side(primary.width)?,
        h: side(primary.height)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4 by 3, each pixel's blue saying which one it is.
    fn numbered() -> ScreenFrame {
        ScreenFrame {
            at: Duration::from_millis(7),
            accumulated: 2,
            width: 4,
            height: 3,
            bgra: (0..12).flat_map(|n| [n, 0, 0, 0]).collect(),
        }
    }

    fn area(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn a_crop_keeps_the_rows_of_the_area_and_when_it_was_presented() {
        let cut = numbered().crop(area(1, 1, 2, 2)).unwrap();
        let blues: Vec<u8> = cut.bgra.chunks(4).map(|p| p[0]).collect();
        assert_eq!(blues, [5, 6, 9, 10]);
        assert_eq!((cut.width, cut.height), (2, 2));
        assert_eq!((cut.at, cut.accumulated), (Duration::from_millis(7), 2));
    }

    #[test]
    fn the_whole_picture_is_a_crop_of_itself() {
        assert_eq!(numbered().crop(area(0, 0, 4, 3)), Some(numbered()));
    }

    #[test]
    fn what_sticks_out_or_has_no_size_has_no_crop() {
        for outside in [
            area(-1, 0, 2, 2),
            area(0, -1, 2, 2),
            area(3, 0, 2, 1),
            area(0, 2, 1, 2),
            area(1, 1, 0, 1),
            area(1, 1, 1, -1),
        ] {
            assert_eq!(numbered().crop(outside), None, "{outside:?}");
        }
    }

    #[test]
    fn the_primary_monitor_starts_where_the_desktop_does() {
        let monitor = |width, primary| Monitor {
            name: "m".into(),
            width,
            height: 100,
            hz: 60,
            primary,
        };
        let both = [monitor(300, false), monitor(200, true)];
        assert_eq!(primary_area(&both), Some(area(0, 0, 200, 100)));
        assert_eq!(primary_area(&both[..1]), None);
        assert_eq!(primary_area(&[monitor(0, true)]), None);
    }
}
