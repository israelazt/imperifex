//! The mouse as the window sees it, turned into the [`Pointer`] each frame of
//! the interface reads. What the window system does not tell is decided here:
//! where a press started and whether it was the second of a double click.

use std::time::Duration;

use crate::view::{Point, Pointer};

/// The longest time between the two presses of a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// How far, in physical pixels, the second press may be from the first.
const DOUBLE_CLICK_SLACK: u32 = 6;

#[derive(Debug, Default)]
pub struct Mouse {
    position: Option<Point>,
    /// Where the left button went down, while it is down.
    held_from: Option<Point>,
    /// When and where of a press that a second one would make double.
    first_press: Option<(Duration, Point)>,
}

impl Mouse {
    /// The pointer with nothing new but where it is.
    pub fn hover(&self) -> Pointer {
        Pointer {
            position: self.position,
            held_from: self.held_from,
            ..Pointer::default()
        }
    }

    pub fn moved(&mut self, to: Point) -> Pointer {
        self.position = Some(to);
        self.hover()
    }

    /// It left the window, or the window stopped being looked at: a press under
    /// way is forgotten.
    pub fn left(&mut self) -> Pointer {
        self.position = None;
        self.held_from = None;
        self.hover()
    }

    /// The left button went down at the time `at` of a monotonic clock.
    pub fn pressed(&mut self, at: Duration) -> Pointer {
        self.held_from = self.position;
        let double = self.is_second_press(at);
        // A third press starts again: three are not two double clicks.
        self.first_press = self.position.filter(|_| !double).map(|place| (at, place));
        Pointer {
            pressed: true,
            double_click: double,
            ..self.hover()
        }
    }

    /// The left button went up. The pointer still says where the press began.
    pub fn released(&mut self) -> Pointer {
        let pointer = Pointer {
            released: true,
            ..self.hover()
        };
        self.held_from = None;
        pointer
    }

    pub fn right_pressed(&mut self) -> Pointer {
        self.first_press = None;
        Pointer {
            right_click: true,
            ..self.hover()
        }
    }

    fn is_second_press(&self, at: Duration) -> bool {
        let (Some((first, place)), Some(here)) = (self.first_press, self.position) else {
            return false;
        };
        let near = |a: i32, b: i32| a.abs_diff(b) <= DOUBLE_CLICK_SLACK;
        at.saturating_sub(first) <= DOUBLE_CLICK && near(place.0, here.0) && near(place.1, here.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(time: u64) -> Duration {
        Duration::from_millis(time)
    }

    fn click(mouse: &mut Mouse, at: u64) -> bool {
        let double = mouse.pressed(ms(at)).double_click;
        mouse.released();
        double
    }

    #[test]
    fn a_release_says_where_its_press_began_and_then_it_is_forgotten() {
        let mut mouse = Mouse::default();
        mouse.moved((10, 10));
        mouse.pressed(ms(0));
        mouse.moved((90, 10));
        let released = mouse.released();
        assert!(released.released);
        assert_eq!(released.held_from, Some((10, 10)));
        assert_eq!(released.position, Some((90, 10)));
        assert_eq!(mouse.hover().held_from, None);
    }

    #[test]
    fn two_presses_soon_and_close_are_a_double_click_and_a_third_is_not() {
        let mut mouse = Mouse::default();
        mouse.moved((100, 20));
        assert!(!click(&mut mouse, 1000));
        mouse.moved((106, 14));
        assert!(click(&mut mouse, 1400));
        assert!(!click(&mut mouse, 1500));
        assert!(click(&mut mouse, 1600));
    }

    #[test]
    fn presses_that_are_late_far_or_split_by_the_right_button_are_single() {
        let mut mouse = Mouse::default();
        mouse.moved((100, 20));
        assert!(!click(&mut mouse, 0));
        assert!(!click(&mut mouse, 401));
        mouse.moved((107, 20));
        assert!(!click(&mut mouse, 500));
        assert!(mouse.right_pressed().right_click);
        assert!(!click(&mut mouse, 600));
    }

    #[test]
    fn leaving_forgets_where_it_was_and_the_press_under_way() {
        let mut mouse = Mouse::default();
        mouse.moved((5, 5));
        mouse.pressed(ms(0));
        assert_eq!(mouse.left(), Pointer::default());
        assert!(!mouse.pressed(ms(10)).double_click);
    }
}
