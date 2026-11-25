use super::*;
use crate::testing::desktop::{window, Behaviour, DesktopProcesses, FakeDesktop, FIRST_PID};
use crate::testing::system::warp;
use crate::testing::{Events, FakeClock};

const AUTHORS_PID: u32 = 1;

fn home() -> &'static Path {
    Path::new("home")
}

struct Rig {
    service: StartupService<FakeClock, FakeDesktop, DesktopProcesses>,
    desktop: FakeDesktop,
    listings: DesktopProcesses,
}

/// With `authors_terminal`, the author has the terminal open: a window and its
/// process, which nothing may touch.
fn rig(behaviour: Behaviour, authors_terminal: bool) -> Rig {
    let (before, author) = if authors_terminal {
        (vec![window(0xA, AUTHORS_PID, "Session")], vec![AUTHORS_PID])
    } else {
        (vec![], vec![])
    };
    let desktop = FakeDesktop::new(before, behaviour);
    let listings = DesktopProcesses::new(&desktop, &author);
    Rig {
        service: StartupService::new(
            FakeClock::ticking(0, &Events::default()),
            desktop.clone(),
            listings.clone(),
        ),
        desktop,
        listings,
    }
}

impl Rig {
    fn asked(&self) -> Vec<String> {
        self.desktop.state.borrow().asked.clone()
    }
}

#[test]
fn each_run_opens_and_closes_one_window_of_its_own() {
    let rig = rig(Behaviour::default(), true);
    let measurement = rig.service.run(&warp(), 3, false, home()).unwrap();
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
    let measurement = rig(slow, false)
        .service
        .run(&warp(), 1, false, home())
        .unwrap();
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
    rig.service.run(&warp(), 1, false, home()).unwrap();
    // Once, when the window shows up.
    assert_eq!(rig.listings.calls(), 1);
}

#[test]
fn a_cold_start_is_refused_while_the_terminal_is_open() {
    let rig = rig(Behaviour::default(), true);
    let error = rig.service.run(&warp(), 2, true, home()).unwrap_err();
    assert!(
        error.contains("cierra warp") && error.contains("S4"),
        "{error}"
    );
    assert!(rig.asked().is_empty());
}

#[test]
fn a_cold_start_runs_when_the_terminal_leaves_nothing_behind() {
    let measurement = rig(Behaviour::default(), false)
        .service
        .run(&warp(), 2, true, home())
        .unwrap();
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
    let error = rig.service.run(&warp(), 3, true, home()).unwrap_err();
    assert!(error.contains("quedan procesos"), "{error}");
    assert_eq!(rig.asked().len(), 2, "{:?}", rig.asked());
}

#[test]
fn a_warm_start_does_not_mind_processes_that_stay() {
    let lingering = Behaviour {
        exits_on_close: false,
        ..Default::default()
    };
    assert!(rig(lingering, true)
        .service
        .run(&warp(), 2, false, home())
        .is_ok());
}

#[test]
fn a_window_that_will_not_close_stops_everything() {
    let stubborn = Behaviour {
        closes: false,
        ..Default::default()
    };
    let rig = rig(stubborn, true);
    let error = rig.service.run(&warp(), 3, false, home()).unwrap_err();
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
    let error = rig.service.run(&warp(), 3, false, home()).unwrap_err();
    assert!(error.contains("no apareció"), "{error}");
    assert_eq!(rig.asked(), ["launch warp.exe"]);
}

#[test]
fn a_new_window_of_another_program_is_not_taken_for_the_terminals() {
    let rig = rig(Behaviour::default(), false);
    // The process the launch starts is not listed as the terminal's.
    rig.listings.rename(FIRST_PID, "notepad.exe");
    assert!(rig.service.run(&warp(), 1, false, home()).is_err());
    assert_eq!(rig.asked(), ["launch warp.exe"]);
}

#[test]
fn no_runs_is_an_error() {
    assert!(rig(Behaviour::default(), false)
        .service
        .run(&warp(), 0, false, home())
        .is_err());
}
