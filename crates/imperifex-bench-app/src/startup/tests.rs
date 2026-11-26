use std::time::Duration;

use super::*;
use crate::testing::desktop::{home, rig, Behaviour, FakeDesktop, Rig, FIRST_PID};
use crate::testing::processes_of::DesktopProcesses;
use crate::testing::screen::{picture, FakeScreen};
use crate::testing::system::warp;
use crate::testing::FakeClock;

type Service = StartupService<FakeClock, FakeDesktop, DesktopProcesses, FakeScreen>;

fn service(rig: &Rig) -> (Service, FakeScreen) {
    let screen = FakeScreen::new(&rig.ports.clock);
    (
        StartupService::new(rig.ports.clone(), screen.clone()),
        screen,
    )
}

/// The measurement of the time until the window is visible.
fn measure(rig: &Rig, runs: u32, cold: bool) -> Result<Measurement, String> {
    let mut all = service(rig).0.run(&warp(), runs, cold, home())?;
    Ok(all.remove(0))
}

#[test]
fn each_run_opens_and_closes_one_window_of_its_own() {
    let rig = rig(Behaviour::default(), true);
    let measurement = measure(&rig, 3, false).unwrap();
    assert_eq!(measurement.metric, Metric::StartupWarm);
    assert_eq!(measurement.summary.count, 3);
    assert_eq!(
        rig.asked(),
        [
            "launch warp.exe",
            "close 0x100",
            "launch warp.exe",
            "close 0x101",
            "launch warp.exe",
            "close 0x102"
        ]
    );
}

#[test]
fn a_sample_is_the_time_until_the_window_is_seen() {
    let slow = Behaviour {
        looks_to_show: Some(3),
        ..Default::default()
    };
    let measurement = measure(&rig(slow, false), 1, false).unwrap();
    // Seen on the third look, after two waits of 5 ms.
    assert_eq!(measurement.samples, [10.0]);
}

#[test]
fn the_processes_are_not_listed_while_nothing_has_shown_up() {
    let slow = Behaviour {
        looks_to_show: Some(50),
        ..Default::default()
    };
    let rig = rig(slow, false);
    measure(&rig, 1, false).unwrap();
    // The photo, the look that finds the window and the look for leftovers.
    assert_eq!(rig.ports.processes.calls(), 3);
}

#[test]
fn a_cold_start_is_refused_while_the_terminal_is_open() {
    let rig = rig(Behaviour::default(), true);
    let error = measure(&rig, 2, true).unwrap_err();
    assert!(
        error.contains("cierra warp") && error.contains("S4"),
        "{error}"
    );
    assert!(rig.asked().is_empty());
}

#[test]
fn a_cold_start_runs_when_the_terminal_leaves_nothing_behind() {
    let measurement = measure(&rig(Behaviour::default(), false), 2, true).unwrap();
    assert_eq!(measurement.metric, Metric::StartupCold);
    assert_eq!(measurement.summary.count, 2);
}

#[test]
fn a_cold_start_stops_if_processes_stay() {
    let lingering = Behaviour {
        exits_on_close: false,
        ..Default::default()
    };
    let rig = rig(lingering, false);
    let error = measure(&rig, 3, true).unwrap_err();
    assert!(
        error.contains("no salió") && error.contains("S2"),
        "{error}"
    );
    assert_eq!(rig.asked().len(), 2, "{:?}", rig.asked());
}

#[test]
fn a_warm_start_does_not_mind_processes_that_stay() {
    let lingering = Behaviour {
        exits_on_close: false,
        ..Default::default()
    };
    assert!(measure(&rig(lingering, true), 2, false).is_ok());
}

#[test]
fn a_window_that_will_not_close_stops_everything() {
    let stubborn = Behaviour {
        closes: false,
        ..Default::default()
    };
    let rig = rig(stubborn, true);
    let error = measure(&rig, 3, false).unwrap_err();
    assert!(
        error.contains("no se cerró") && error.contains("S2"),
        "{error}"
    );
    assert_eq!(rig.asked(), ["launch warp.exe", "close 0x100"]);
}

#[test]
fn if_no_window_shows_up_nothing_is_closed() {
    let never = Behaviour {
        looks_to_show: None,
        ..Default::default()
    };
    let rig = rig(never, true);
    let error = measure(&rig, 3, false).unwrap_err();
    assert!(error.contains("no apareció"), "{error}");
    assert_eq!(rig.asked(), ["launch warp.exe"]);
}

#[test]
fn a_new_window_of_another_program_is_not_taken_for_the_terminals() {
    let rig = rig(Behaviour::default(), false);
    // The process the launch starts is not listed as the terminal's.
    rig.ports.processes.rename(FIRST_PID, "notepad.exe");
    assert!(measure(&rig, 1, false).is_err());
    assert_eq!(rig.asked(), ["launch warp.exe"]);
}

#[test]
fn no_runs_is_an_error() {
    assert!(measure(&rig(Behaviour::default(), false), 0, false).is_err());
}

#[test]
fn the_notes_say_what_each_window_left_and_how_long_it_was_waited_for() {
    let lingers = Behaviour {
        lingers_for: 2,
        ..Default::default()
    };
    let measurement = measure(&rig(lingers, true), 2, false).unwrap();
    assert_eq!(
        measurement.notes[1],
        "Procesos que quedaban de cada ventana cerrada, y espera: 0 en 2 s; 0 en 2 s."
    );
}

#[test]
fn the_drawn_time_is_the_last_big_change_of_the_window() {
    let rig = rig(Behaviour::default(), true);
    let (service, screen) = service(&rig);
    for (at, lit) in [(20, 0), (100, 4000), (300, 8000)] {
        screen.show(picture(Duration::from_millis(at), lit));
    }
    let all = service.run(&warp(), 1, false, home()).unwrap();
    assert_eq!(all[1].metric, Metric::StartupWarmDrawn);
    assert_eq!(all[1].samples, [300.0]);
    assert_eq!(all[0].samples, [0.0]);
    let watched = screen.shown.borrow().watched.clone();
    assert_eq!((watched[0].w, watched[0].h), (1280, 800), "where it opened");
}

#[test]
fn a_window_that_cannot_be_watched_is_noted_and_the_visible_time_stays() {
    let rig = rig(Behaviour::default(), true);
    let (service, screen) = service(&rig);
    screen.shown.borrow_mut().refusal = Some("entre dos monitores".into());
    let all = service.run(&warp(), 2, false, home()).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].summary.count, 2);
    let notes = all[0].notes.join(" ");
    assert!(
        notes.contains("Tanda 2 sin ventana dibujada: entre dos monitores"),
        "{notes}"
    );
    assert_eq!(rig.asked().len(), 4, "both windows were closed");
}
