use super::*;
use crate::testing::desktop::{home, rig, Behaviour, Rig, FIRST_OPENED};
use crate::testing::system::warp;

#[test]
fn the_window_is_adopted_on_the_look_that_finds_it() {
    let rig = rig(
        Behaviour {
            looks_to_show: Some(3),
            ..Default::default()
        },
        true,
    );
    let profile = warp();
    let session = WindowSession::open(&rig.ports, &profile, home()).unwrap();
    assert_eq!(session.window().hwnd, FIRST_OPENED);
    // Seen on the third look, after two waits of 5 ms.
    assert_eq!(
        session.visible_at - session.started_at,
        Duration::from_millis(10)
    );
    assert_eq!(session.touch(), Ok(FIRST_OPENED));
    assert_eq!(rig.dirs(), [home()]);
}

fn open_error(rig: &Rig) -> String {
    let profile = warp();
    let error = WindowSession::open(&rig.ports, &profile, home()).unwrap_err();
    assert_eq!(rig.asked(), ["launch warp.exe"], "nothing is touched");
    error
}

#[test]
fn two_new_windows_stop_it_without_touching_either() {
    let two = Behaviour {
        windows_per_launch: 2,
        ..Default::default()
    };
    assert!(open_error(&rig(two, true)).contains("2 ventanas"));
}

#[test]
fn a_hidden_window_that_shows_up_stops_it_without_touching_it() {
    let reshows = Behaviour {
        looks_to_show: None,
        reshows_hidden: true,
        ..Default::default()
    };
    let error = open_error(&rig(reshows, true));
    assert!(error.contains("ya tenía oculta"), "{error}");
}

#[test]
fn with_no_window_it_gives_up_and_says_so() {
    let never = Behaviour {
        looks_to_show: None,
        ..Default::default()
    };
    assert!(open_error(&rig(never, true)).contains("no apareció"));
}

fn close(behaviour: Behaviour) -> (Result<Closed, String>, Vec<String>) {
    let rig = rig(behaviour, true);
    let profile = warp();
    let session = WindowSession::open(&rig.ports, &profile, home()).unwrap();
    let closed = session.close();
    (closed, rig.asked())
}

#[test]
fn a_window_that_leaves_nothing_is_closed_at_once() {
    let (closed, asked) = close(Behaviour::default());
    assert_eq!(
        closed,
        Ok(Closed {
            leftovers: 0,
            waited: Duration::ZERO
        })
    );
    assert_eq!(asked, ["launch warp.exe", "close 0x100"]);
}

#[test]
fn what_it_leaves_is_waited_for() {
    let lingers = Behaviour {
        lingers_for: 3,
        ..Default::default()
    };
    let closed = close(lingers).0.unwrap();
    assert_eq!(closed.leftovers, 0);
    assert_eq!(closed.waited, Duration::from_secs(3));
}

#[test]
fn what_stays_after_ninety_seconds_is_counted_and_not_forced() {
    let stays = Behaviour {
        exits_on_close: false,
        ..Default::default()
    };
    let closed = close(stays).0.unwrap();
    assert_eq!((closed.leftovers, closed.waited), (1, TO_LEAVE));
}

#[test]
fn a_question_before_closing_stops_it_and_nothing_is_pressed() {
    let asks = Behaviour {
        asks_to_confirm: true,
        ..Default::default()
    };
    let (closed, asked) = close(asks);
    assert!(closed.unwrap_err().contains("confirmar"));
    assert_eq!(asked, ["launch warp.exe", "close 0x100"]);
}

#[test]
fn a_window_that_will_not_close_is_not_forced() {
    let stubborn = Behaviour {
        closes: false,
        ..Default::default()
    };
    let (closed, asked) = close(stubborn);
    let error = closed.unwrap_err();
    assert!(
        error.contains("no se cerró") && error.contains("S2"),
        "{error}"
    );
    assert_eq!(asked, ["launch warp.exe", "close 0x100"]);
}

#[test]
fn placing_and_focusing_go_to_the_adopted_window() {
    let rig = rig(Behaviour::default(), true);
    let profile = warp();
    let session = WindowSession::open(&rig.ports, &profile, home()).unwrap();
    session.place(Rect::default()).unwrap();
    session.focus().unwrap();
    assert_eq!(
        rig.asked(),
        ["launch warp.exe", "place 0x100", "focus 0x100"]
    );
}
