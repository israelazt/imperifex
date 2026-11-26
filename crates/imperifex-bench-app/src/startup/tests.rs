use super::*;
use crate::testing::desktop::{home, rig, Behaviour, FakeDesktop, Rig, FIRST_PID};
use crate::testing::processes_of::DesktopProcesses;
use crate::testing::system::warp;
use crate::testing::FakeClock;

type Service = StartupService<FakeClock, FakeDesktop, DesktopProcesses>;

fn service(rig: &Rig) -> Service {
    StartupService::new(rig.ports.clone())
}

fn measure(rig: &Rig, runs: u32, cold: bool) -> Result<Measurement, String> {
    service(rig).run(&warp(), runs, cold, home())
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
    assert!(error.contains("quedan procesos"), "{error}");
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
