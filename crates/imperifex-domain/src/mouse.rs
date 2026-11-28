//! The mouse as the window sees it, turned into the [`Pointer`] each frame of
//! the interface reads. What the window system does not tell is decided here:
//! where a press started, whether it was the second of a double click and
//! whether the mouse has been still for a while.

use std::time::Duration;

use crate::view::{Point, Pointer};

/// The longest time between the two presses of a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// How far, in physical pixels, the second press may be from the first.
const DOUBLE_CLICK_SLACK: u32 = 6;
/// How long it must be still to be told what is under it.
const REST: Duration = Duration::from_millis(600);

#[derive(Debug, Default)]
pub struct Mouse {
    position: Option<Point>,
    /// Where the left button went down, while it is down.
    held_from: Option<Point>,
    /// When and where of a press that a second one would make double.
    first_press: Option<(Duration, Point)>,
    /// Since when it is still, unless a press or leaving ended the wait.
    still_since: Option<Duration>,
}

impl Mouse {
    /// The pointer with nothing new but where it is, at the time `now`.
    pub fn hover(&self, now: Duration) -> Pointer {
        let rested = self
            .still_since
            .is_some_and(|since| now.saturating_sub(since) >= REST);
        Pointer {
            rested,
            ..self.place()
        }
    }

    /// How long from `now` until it will have rested; `None` if it already has
    /// or will not.
    pub fn rests_in(&self, now: Duration) -> Option<Duration> {
        let rested_at = self.still_since?.saturating_add(REST);
        rested_at.checked_sub(now).filter(|left| !left.is_zero())
    }

    pub fn position(&self) -> Option<Point> {
        self.position
    }

    /// It is at `to`. Saying again where it already was does not move it.
    pub fn moved(&mut self, to: Point, at: Duration) -> Pointer {
        if self.position != Some(to) {
            self.position = Some(to);
            self.still_since = Some(at);
        }
        self.hover(at)
    }

    fn place(&self) -> Pointer {
        Pointer {
            position: self.position,
            held_from: self.held_from,
            ..Pointer::default()
        }
    }

    /// It left the window, or the window stopped being looked at: a press under
    /// way is forgotten.
    pub fn left(&mut self) -> Pointer {
        self.position = None;
        self.held_from = None;
        self.still_since = None;
        self.place()
    }

    /// The left button went down at the time `at` of a monotonic clock.
    pub fn pressed(&mut self, at: Duration) -> Pointer {
        self.held_from = self.position;
        self.still_since = None;
        let double = self.is_second_press(at);
        // A third press starts again: three are not two double clicks.
        self.first_press = self.position.filter(|_| !double).map(|place| (at, place));
        Pointer {
            pressed: true,
            double_click: double,
            ..self.place()
        }
    }

    /// The left button went up. The pointer still says where the press began.
    pub fn released(&mut self) -> Pointer {
        let pointer = Pointer {
            released: true,
            ..self.place()
        };
        self.held_from = None;
        pointer
    }

    pub fn right_pressed(&mut self) -> Pointer {
        self.first_press = None;
        self.still_since = None;
        Pointer {
            right_click: true,
            ..self.place()
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
        mouse.moved((10, 10), ms(0));
        mouse.pressed(ms(0));
        mouse.moved((90, 10), ms(0));
        let released = mouse.released();
        assert!(released.released);
        assert_eq!(released.held_from, Some((10, 10)));
        assert_eq!(released.position, Some((90, 10)));
        assert_eq!(mouse.hover(ms(0)).held_from, None);
    }

    #[test]
    fn two_presses_soon_and_close_are_a_double_click_and_a_third_is_not() {
        let mut mouse = Mouse::default();
        mouse.moved((100, 20), ms(0));
        assert!(!click(&mut mouse, 1000));
        mouse.moved((106, 14), ms(0));
        assert!(click(&mut mouse, 1400));
        assert!(!click(&mut mouse, 1500));
        assert!(click(&mut mouse, 1600));
    }

    #[test]
    fn presses_that_are_late_far_or_split_by_the_right_button_are_single() {
        let mut mouse = Mouse::default();
        mouse.moved((100, 20), ms(0));
        assert!(!click(&mut mouse, 0));
        assert!(!click(&mut mouse, 401));
        mouse.moved((107, 20), ms(0));
        assert!(!click(&mut mouse, 500));
        assert!(mouse.right_pressed().right_click);
        assert!(!click(&mut mouse, 600));
    }

    #[test]
    fn leaving_forgets_where_it_was_and_the_press_under_way() {
        let mut mouse = Mouse::default();
        mouse.moved((5, 5), ms(0));
        mouse.pressed(ms(0));
        assert_eq!(mouse.left(), Pointer::default());
        assert!(!mouse.pressed(ms(10)).double_click);
    }

    #[test]
    fn it_has_rested_600_ms_after_it_last_moved() {
        let mut mouse = Mouse::default();
        assert_eq!(mouse.rests_in(ms(0)), None);
        assert!(!mouse.moved((5, 5), ms(1000)).rested);
        assert_eq!(mouse.rests_in(ms(1100)), Some(ms(500)));
        assert!(!mouse.hover(ms(1599)).rested);
        assert!(mouse.hover(ms(1600)).rested);
        assert_eq!(mouse.rests_in(ms(1600)), None);
        assert!(mouse.moved((5, 5), ms(1700)).rested);
        assert!(!mouse.moved((6, 5), ms(1800)).rested);
    }

    #[test]
    fn a_press_or_leaving_ends_the_wait_until_it_moves_again() {
        let mut mouse = Mouse::default();
        mouse.moved((5, 5), ms(0));
        mouse.pressed(ms(100));
        mouse.released();
        assert!(!mouse.hover(ms(5000)).rested);
        assert_eq!(mouse.rests_in(ms(200)), None);
        mouse.moved((6, 5), ms(6000));
        mouse.right_pressed();
        assert!(!mouse.hover(ms(9000)).rested);
        mouse.moved((7, 5), ms(9000));
        mouse.left();
        assert!(!mouse.hover(ms(12000)).rested);
    }
}
