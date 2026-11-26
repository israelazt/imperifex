//! Telling a real change on screen from something that blinks. Frames are BGRA,
//! four bytes a pixel, row after row.

use crate::desktop::Rect;

/// A channel has to differ by more than this for a pixel to count as changed.
const TOLERANCE: u8 = 8;

/// How many changed pixels make a change, unless another number is asked for.
pub const DEFAULT_MIN_PIXELS: usize = 4;

const BYTES_PER_PIXEL: usize = 4;

/// Knows which pixels change on their own and leaves them out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeDetector {
    /// `true` for a pixel that is left out; one per pixel.
    mask: Vec<bool>,
    width: usize,
    height: usize,
    min_pixels: usize,
}

impl ChangeDetector {
    /// Watches frames taken while nothing is being done: every pixel that
    /// differs at all between one and the next is left out from then on. Any
    /// difference counts here, so that something that fades in small steps, and
    /// would add up to more than the tolerance later, is left out too.
    pub fn learn(
        frames: &[&[u8]],
        width: usize,
        height: usize,
        min_pixels: usize,
    ) -> Result<Self, String> {
        if min_pixels == 0 {
            return Err("hace falta al menos un píxel para hablar de un cambio".into());
        }
        let expected = width * height * BYTES_PER_PIXEL;
        if let Some(frame) = frames.iter().find(|f| f.len() != expected) {
            return Err(format!(
                "un frame de {width}x{height} tiene {expected} bytes, no {}",
                frame.len()
            ));
        }
        let mut detector = Self {
            mask: vec![false; width * height],
            width,
            height,
            min_pixels,
        };
        for pair in frames.windows(2) {
            detector.ignore_changes(pair[0], pair[1]);
        }
        Ok(detector)
    }

    /// Leaves out from now on every pixel that differs between these two, for
    /// learning from pictures as they come without keeping them all.
    pub fn ignore_changes(&mut self, before: &[u8], after: &[u8]) {
        for (ignored, (before, after)) in self.mask.iter_mut().zip(pixels(before, after)) {
            *ignored |= before != after;
        }
    }

    /// How many pixels are left out.
    pub fn masked(&self) -> usize {
        self.mask.iter().filter(|ignored| **ignored).count()
    }

    /// The smallest rectangle with every pixel outside the mask that differs,
    /// counted from the frames' top left corner; `None` unless there are enough
    /// of them to be a change. Both frames as for [`Self::changed`].
    pub fn changed_area(&self, before: &[u8], after: &[u8]) -> Option<Rect> {
        if !self.changed(before, after) {
            return None;
        }
        let (mut left, mut top, mut right, mut bottom) = (self.width, self.height, 0, 0);
        for (at, (ignored, (before, after))) in
            self.mask.iter().zip(pixels(before, after)).enumerate()
        {
            if !ignored && differ(before, after) {
                let (x, y) = (at % self.width, at / self.width);
                (left, top) = (left.min(x), top.min(y));
                (right, bottom) = (right.max(x + 1), bottom.max(y + 1));
            }
        }
        let side = |length: usize| i32::try_from(length).ok();
        Some(Rect {
            x: side(left)?,
            y: side(top)?,
            w: side(right - left)?,
            h: side(bottom - top)?,
        })
    }

    /// Whether enough pixels outside the mask differ. Both frames must have the
    /// size the detector learned with.
    pub fn changed(&self, before: &[u8], after: &[u8]) -> bool {
        let expected = self.width * self.height * BYTES_PER_PIXEL;
        let sized = before.len() == expected && after.len() == expected;
        debug_assert!(sized, "frames of another size than the detector's");
        sized
            && self
                .mask
                .iter()
                .zip(pixels(before, after))
                .filter(|(ignored, (a, b))| !**ignored && differ(a, b))
                .take(self.min_pixels)
                .count()
                == self.min_pixels
    }
}

/// A pixel as it comes in a frame.
type Pixel = [u8; BYTES_PER_PIXEL];

/// The pixels of two frames, side by side.
fn pixels<'a>(a: &'a [u8], b: &'a [u8]) -> impl Iterator<Item = (&'a Pixel, &'a Pixel)> {
    let (a, b) = (a.as_chunks().0, b.as_chunks().0);
    a.iter().zip(b)
}

fn differ(a: &Pixel, b: &Pixel) -> bool {
    a.iter().zip(b).any(|(x, y)| x.abs_diff(*y) > TOLERANCE)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 16;
    const HEIGHT: usize = 8;

    /// A black frame with these pixels at this level of blue.
    fn frame(lit: &[usize], level: u8) -> Vec<u8> {
        let mut bytes = vec![0; WIDTH * HEIGHT * BYTES_PER_PIXEL];
        for &pixel in lit {
            bytes[pixel * BYTES_PER_PIXEL] = level;
        }
        bytes
    }

    fn learned(frames: &[&[u8]]) -> ChangeDetector {
        ChangeDetector::learn(frames, WIDTH, HEIGHT, DEFAULT_MIN_PIXELS).unwrap()
    }

    #[test]
    fn four_new_pixels_are_a_change_and_three_are_not() {
        let black = frame(&[], 0);
        let detector = learned(&[&black, &black]);
        assert!(detector.changed(&black, &frame(&[1, 2, 3, 4], 255)));
        assert!(!detector.changed(&black, &frame(&[1, 2, 3], 255)));
    }

    #[test]
    fn what_blinked_while_learning_does_not_count_later() {
        let off = frame(&[], 0);
        let on = frame(&[0, 1, 2, 3], 255);
        let detector = learned(&[&off, &on, &off]);
        assert!(!detector.changed(&off, &on));
        let typed = frame(&[0, 1, 2, 3, 20, 21, 22, 23], 255);
        assert!(detector.changed(&off, &typed));
    }

    #[test]
    fn the_area_of_a_change_holds_every_new_pixel_outside_the_mask() {
        let black = frame(&[], 0);
        let blink = frame(&[127], 255);
        let detector = learned(&[&black, &blink]);
        // Columns 2 to 5 of rows 1 and 2, and the masked last corner.
        let after = frame(&[18, 19, 37, 34, 127], 255);
        let expected = Rect {
            x: 2,
            y: 1,
            w: 4,
            h: 2,
        };
        assert_eq!(detector.changed_area(&black, &after), Some(expected));
    }

    #[test]
    fn too_few_pixels_have_no_area() {
        let black = frame(&[], 0);
        let detector = learned(&[]);
        assert_eq!(detector.changed_area(&black, &frame(&[1, 2, 3], 255)), None);
    }

    #[test]
    fn pictures_can_be_learned_from_one_pair_at_a_time() {
        let off = frame(&[], 0);
        let on = frame(&[0, 1], 255);
        let mut detector = learned(&[]);
        assert_eq!(detector.masked(), 0);
        detector.ignore_changes(&off, &on);
        assert_eq!(detector.masked(), 2);
    }

    #[test]
    fn a_pixel_that_fades_in_small_steps_is_left_out_too() {
        let steps = [frame(&[5, 6, 7, 8], 0), frame(&[5, 6, 7, 8], 6)];
        let detector = learned(&[&steps[0], &steps[1]]);
        assert!(!detector.changed(&steps[0], &frame(&[5, 6, 7, 8], 200)));
    }

    #[test]
    fn a_difference_within_the_tolerance_is_not_a_change() {
        let black = frame(&[], 0);
        let detector = learned(&[]);
        assert!(!detector.changed(&black, &frame(&[1, 2, 3, 4], TOLERANCE)));
        assert!(detector.changed(&black, &frame(&[1, 2, 3, 4], TOLERANCE + 1)));
    }

    #[test]
    fn every_channel_counts() {
        let black = frame(&[], 0);
        for channel in 0..BYTES_PER_PIXEL {
            let mut after = black.clone();
            for pixel in 0..DEFAULT_MIN_PIXELS {
                after[pixel * BYTES_PER_PIXEL + channel] = 255;
            }
            assert!(learned(&[]).changed(&black, &after), "channel {channel}");
        }
    }

    #[test]
    fn a_frame_of_another_size_is_an_error() {
        let short = vec![0; 10];
        let error = ChangeDetector::learn(&[&short], WIDTH, HEIGHT, 4).unwrap_err();
        assert!(error.contains("512") && error.contains("10"), "{error}");
    }

    #[test]
    fn asking_for_no_pixels_is_an_error() {
        assert!(ChangeDetector::learn(&[], WIDTH, HEIGHT, 0).is_err());
    }
}
