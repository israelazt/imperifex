use super::*;
use crate::testing::screen::{picture, FakeScreen};
use crate::testing::{Events, FakeClock};

const QUIET: Duration = Duration::from_millis(500);
const MAX: Duration = Duration::from_secs(10);

fn ms(at: u64) -> Duration {
    Duration::from_millis(at)
}

/// Pictures with so many pixels lit, at those milliseconds.
fn screen(pictures: &[(u64, usize)]) -> (FakeScreen, FakeClock) {
    let clock = FakeClock::ticking(0, &Events::default());
    let screen = FakeScreen::new(&clock);
    for &(at, lit) in pictures {
        screen.show(picture(ms(at), lit));
    }
    (screen, clock)
}

fn run(pictures: &[(u64, usize)]) -> Result<Settled, String> {
    let (screen, clock) = screen(pictures);
    settle(&mut screen.watching(), &clock, Duration::ZERO, QUIET, MAX)
}

/// A burst and then a cursor blinking every 530 ms: 20 pixels of 10 000, under
/// the half percent that counts.
#[test]
fn a_blinking_cursor_does_not_keep_the_screen_from_settling() {
    let mut pictures = vec![(10, 0), (100, 3000), (200, 6000)];
    pictures.extend((1..=20).map(|n| (200 + n * 530, 6000 + (n as usize % 2) * 20)));
    let settled = run(&pictures).unwrap();
    assert_eq!((settled.at, settled.changes), (ms(200), 2));
}

#[test]
fn a_big_burst_and_then_silence_gives_its_last_moment() {
    let settled = run(&[(10, 0), (400, 5000), (800, 200), (1200, 9000)]).unwrap();
    assert_eq!((settled.at, settled.changes), (ms(1200), 3));
    assert_eq!(settled.frame.unwrap().at, ms(1200));
}

#[test]
fn with_no_silence_it_gives_up_and_says_how_many_changes_there_were() {
    let pictures: Vec<(u64, usize)> = (0..80)
        .map(|n| (n * 200, (n as usize % 2) * 5000))
        .collect();
    let error = run(&pictures).unwrap_err();
    assert!(
        error.contains("10 s") && error.contains("cambios"),
        "{error}"
    );
}

#[test]
fn a_still_screen_was_settled_from_the_start() {
    let settled = run(&[]).unwrap();
    assert_eq!((settled.at, settled.frame), (Duration::ZERO, None));
}

#[test]
fn a_change_of_a_picture_that_adds_up_several_is_counted() {
    let (screen, clock) = screen(&[(10, 0)]);
    let mut joined = picture(ms(100), 5000);
    joined.accumulated = 3;
    screen.show(joined);
    let settled = settle(&mut screen.watching(), &clock, Duration::ZERO, QUIET, MAX).unwrap();
    assert_eq!(settled.joined, 1);
}

/// Two bursts with a long silence between: asked to wait for something that
/// happens after the second, the answer is the end of the second.
#[test]
fn until_what_is_waited_for_happens_nothing_is_settled() {
    let (screen, clock) = screen(&[(10, 0), (100, 5000), (2000, 9000)]);
    let time = clock.clone();
    let settled = settle_when(
        &mut screen.watching(),
        &clock,
        Duration::ZERO,
        QUIET,
        MAX,
        || time.now() >= ms(2100),
    )
    .unwrap();
    assert_eq!(settled.at, ms(2000));
}
