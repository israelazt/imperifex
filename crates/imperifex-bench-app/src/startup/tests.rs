use std::time::Duration;

use imperifex_bench_domain::screen::ScreenFrame;

use super::*;
use crate::testing::desktop::{home, rig, Behaviour, FakeDesktop, Rig, FIRST_PID};
use crate::testing::processes_of::DesktopProcesses;
use crate::testing::screen::{monitor, FakeScreen};
use crate::testing::system::warp;
use crate::testing::FakeClock;

/// A monitor with room for the fake windows, 1280x800 at its corner, and some
/// to spare below them.
const MONITOR: (usize, usize) = (1920, 1080);
const PRIMARY: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1080,
};
/// A row below the windows.
const UNDER_THE_WINDOW: usize = 900;

type Service = StartupService<FakeClock, FakeDesktop, DesktopProcesses, FakeScreen>;

fn service(rig: &Rig) -> (Service, FakeScreen) {
    let screen = FakeScreen::new(&rig.ports.clock);
    (
        StartupService::new(rig.ports.clone(), screen.clone(), PRIMARY),
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

/// A monitor with `pixels` lit from the top, presented at `ms`. The screen is
/// found calm 1000 ms after the first of them, and the launch comes then.
fn at(ms: u64, pixels: usize) -> ScreenFrame {
    monitor(Duration::from_millis(ms), MONITOR, 0, pixels)
}

/// What one warm start measures with these pictures.
fn watched(pictures: impl IntoIterator<Item = ScreenFrame>) -> Vec<Measurement> {
    let rig = rig(Behaviour::default(), true);
    let (service, screen) = service(&rig);
    for picture in pictures {
        screen.show(picture);
    }
    let all = service.run(&warp(), 1, false, home()).unwrap();
    assert_eq!(rig.asked().len(), 2, "the window was closed");
    all
}

fn samples(all: &[Measurement]) -> Vec<(Metric, &[f64])> {
    all.iter()
        .map(|m| (m.metric, m.samples.as_slice()))
        .collect()
}

fn notes(all: &[Measurement]) -> String {
    all[0].notes.join(" ")
}

#[test]
fn content_a_pause_and_a_second_change_give_both_times() {
    let all = watched([at(10, 0), at(2300, 20_000), at(3500, 60_000)]);
    assert_eq!(
        samples(&all),
        [
            (Metric::StartupWarm, [0.0].as_slice()),
            (Metric::StartupWarmContent, [1300.0].as_slice()),
            (Metric::StartupWarmReady, [2500.0].as_slice()),
        ]
    );
}

#[test]
fn a_window_drawn_by_the_first_picture_after_it_shows_has_its_content_there() {
    let all = watched([at(10, 0), at(1001, 60_000)]);
    assert_eq!(all[1].samples, [1.0]);
    assert_eq!(all[2].samples, [1.0]);
}

#[test]
fn a_change_outside_the_window_is_not_its_content() {
    let elsewhere = monitor(
        Duration::from_millis(1500),
        MONITOR,
        UNDER_THE_WINDOW,
        60_000,
    );
    let all = watched([at(10, 0), elsewhere]);
    assert_eq!(all.len(), 1, "{:?}", samples(&all));
}

#[test]
fn with_no_change_there_is_neither_time_and_the_visible_one_stays() {
    let all = watched([at(10, 0)]);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].samples, [0.0]);
    let notes = notes(&all);
    assert!(
        notes.contains("Tanda 1 sin primer contenido: la ventana no cambió")
            && notes.contains("Tanda 1 sin ventana lista"),
        "{notes}"
    );
}

#[test]
fn a_window_that_never_stays_still_keeps_its_first_content() {
    let blinking = (0..40).map(|n| at(1500 + n * 500, (n as usize % 2) * 30_000 + 20_000));
    let all = watched(std::iter::once(at(10, 0)).chain(blinking));
    assert_eq!(
        all.iter().map(|m| m.metric).collect::<Vec<_>>(),
        [Metric::StartupWarm, Metric::StartupWarmContent]
    );
    let notes = notes(&all);
    assert!(
        notes.contains("sin ventana lista: la pantalla no se asentó"),
        "{notes}"
    );
}

#[test]
fn a_screen_that_is_not_calm_before_the_launch_gives_only_the_visible_time() {
    let busy = (0..30).map(|n| at(10 + n * 500, (n as usize % 2) * 60_000));
    let all = watched(busy);
    assert_eq!(all.len(), 1);
    let notes = notes(&all);
    assert!(
        notes.contains("antes de lanzar, la pantalla no se asentó"),
        "{notes}"
    );
}

#[test]
fn a_window_that_does_not_fit_on_the_primary_monitor_is_told_where_it_opened() {
    let rig = rig(Behaviour::default(), true);
    let screen = FakeScreen::new(&rig.ports.clock);
    screen.show(monitor(Duration::from_millis(10), (1000, 700), 0, 0));
    let small = Rect {
        w: 1000,
        h: 700,
        ..PRIMARY
    };
    let service = StartupService::new(rig.ports, screen, small);
    let all = service.run(&warp(), 1, false, home()).unwrap();
    assert_eq!(all.len(), 1);
    let notes = notes(&all);
    assert!(notes.contains("abrió en (0, 0), de 1280x800"), "{notes}");
}

#[test]
fn the_whole_primary_monitor_is_watched_from_before_the_launch() {
    let rig = rig(Behaviour::default(), true);
    let (service, screen) = service(&rig);
    service.run(&warp(), 1, false, home()).unwrap();
    assert_eq!(screen.shown.borrow().watched, [PRIMARY]);
}

#[test]
fn a_time_from_a_picture_that_adds_up_several_is_noted() {
    let mut joined = at(2300, 60_000);
    joined.accumulated = 3;
    let all = watched([at(10, 0), joined]);
    for measured in &all[1..] {
        assert!(measured.notes[1].ends_with(": 1."), "{:?}", measured.notes);
    }
}

#[test]
fn a_screen_that_cannot_be_watched_is_noted_and_the_visible_time_stays() {
    let rig = rig(Behaviour::default(), true);
    let (service, screen) = service(&rig);
    screen.shown.borrow_mut().refusal = Some("sin duplicación".into());
    let all = service.run(&warp(), 2, false, home()).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].summary.count, 2);
    let notes = notes(&all);
    assert!(
        notes.contains("Tanda 2 sin primer contenido: sin duplicación"),
        "{notes}"
    );
    assert_eq!(rig.asked().len(), 4, "both windows were closed");
}
