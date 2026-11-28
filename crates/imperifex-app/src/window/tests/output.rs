//! The shell inside the window.

use std::time::Duration;

use imperifex_domain::draw::Rect;
use imperifex_domain::font::CellSize;
use imperifex_domain::input::KeyInput;

use super::*;
use crate::testing::{data, fake_parser, no_wake, FakeClock, FakeLauncher, Script};

const CELL: CellSize = CellSize {
    width: 10,
    height: 20,
    baseline: 15,
};

fn area(width: u32, height: u32) -> Rect {
    Rect {
        x: 16,
        y: 56,
        width,
        height,
    }
}

fn ms(time: u64) -> Duration {
    Duration::from_millis(time)
}

fn open(core: &mut Core, launcher: &FakeLauncher) -> bool {
    core.open_shell(&Start {
        launcher,
        new_parser: &fake_parser,
        wake: no_wake(),
    })
}

/// A window of 800 by 600 with room for 40 by 10, and its shell.
fn running(script: Script) -> (Core, FakeLauncher) {
    let launcher = FakeLauncher::running(script);
    let mut core = core_with(ThemeChoice::Dark, &FakeAppearance::dark(true));
    core.resized((800, 600), false);
    core.fit(area(400, 200), CELL, ms(0));
    assert!(open(&mut core, &launcher));
    (core, launcher)
}

fn says(text: &str) -> Script {
    Script {
        output: vec![data(text)],
        ..Script::default()
    }
}

fn read(core: &mut Core) -> Vec<WindowCommand> {
    core.read_output(&FakeClock::ticking(1))
}

fn draw(core: &mut Core) {
    core.take_damage();
    core.was_shown(DrawList::default());
}

#[test]
fn the_shell_opens_once_with_the_columns_and_rows_that_fit() {
    let mut core = core_with(ThemeChoice::Dark, &FakeAppearance::dark(true));
    let launcher = FakeLauncher::default();
    assert!(!open(&mut core, &launcher), "no room is known yet");
    core.fit(area(409, 219), CELL, ms(0));
    assert!(open(&mut core, &launcher));
    assert!(!open(&mut core, &launcher));
    let started = launcher.asked.borrow().started.clone();
    assert_eq!(started, [("pwsh.exe".into(), 40, 10)]);
}

#[test]
fn with_no_shell_the_window_says_it_in_its_language_and_why() {
    let launcher = FakeLauncher {
        missing: vec!["pwsh.exe", "powershell.exe"],
        ..FakeLauncher::default()
    };
    let mut core = core_with(ThemeChoice::Dark, &FakeAppearance::dark(true));
    core.fit(area(400, 200), CELL, ms(0));
    assert!(open(&mut core, &launcher));
    assert_eq!(
        core.model().notice,
        [
            "The shell could not be opened.",
            "powershell.exe is not installed"
        ]
    );
    assert!(core.picture(&Default::default()).is_none());
}

#[test]
fn output_asks_for_one_frame_and_waits_for_it_before_reading_more() {
    let (mut core, _) = running(Script {
        output: vec![
            data("a"),
            imperifex_domain::ports::PtyRead::Nothing,
            data("b"),
        ],
        ..Script::default()
    });
    assert_eq!(read(&mut core), [WindowCommand::Redraw]);
    assert!(!core.has_unread());
    // More arrives before the frame is drawn: it stays where it is.
    assert_eq!(read(&mut core), []);
    assert!(!core.has_unread(), "not before the frame");
    draw(&mut core);
    assert!(core.has_unread());
    assert_eq!(read(&mut core), [WindowCommand::Redraw]);
}

#[test]
fn a_window_that_is_not_seen_goes_on_reading() {
    let (mut core, _) = running(Script {
        output: vec![
            data("a"),
            imperifex_domain::ports::PtyRead::Nothing,
            data("b"),
        ],
        ..Script::default()
    });
    read(&mut core);
    core.resized((0, 0), false);
    assert_eq!(read(&mut core), [WindowCommand::Redraw]);
}

#[test]
fn a_frame_that_failed_does_not_hold_the_output_back() {
    let (mut core, _) = running(says("a"));
    read(&mut core);
    read(&mut core);
    core.not_shown();
    assert!(core.has_unread());
}

#[test]
fn a_shell_that_ends_closes_the_window() {
    let (mut core, launcher) = running(Script {
        leaves: Some(0),
        ..Script::default()
    });
    read(&mut core);
    assert_eq!(launcher.asked.borrow().closed, 1);
    // The first frame of the new screen.
    draw(&mut core);
    assert_eq!(read(&mut core), [WindowCommand::Close]);
    assert!(!core.has_unread());
}

#[test]
fn the_shell_takes_the_windows_new_size_once_it_settles() {
    let (mut core, launcher) = running(Script::default());
    core.fit(area(400, 200), CELL, ms(0));
    assert_eq!(core.settles_in(ms(0)), None);
    core.fit(area(500, 300), CELL, ms(10));
    assert_eq!(core.settles_in(ms(60)), Some(ms(50)));
    assert_eq!(core.settle_size(ms(60)), []);
    assert_eq!(core.settle_size(ms(110)), [WindowCommand::Redraw]);
    assert_eq!(launcher.asked.borrow().resized, [(50, 15)]);
}

#[test]
fn the_tab_is_named_after_the_shell_that_opened_and_there_is_none_without_it() {
    let (core, _) = running(Script::default());
    assert_eq!(core.model().tab.as_deref(), Some("PowerShell"));
    let none = core_with(ThemeChoice::Dark, &FakeAppearance::dark(true));
    assert_eq!(none.model().tab, None);
}

#[test]
fn what_is_typed_reaches_the_shell() {
    let (mut core, launcher) = running(Script::default());
    core.typed(&KeyInput {
        text: Some("ls".into()),
        key: None,
    });
    assert_eq!(launcher.asked.borrow().written, b"ls");
}

#[test]
fn the_picture_is_the_terminal_where_it_was_fitted_with_the_windows_looks() {
    let (mut core, _) = running(says("hi"));
    core.set_scale(1.5);
    core.set_focused(false);
    read(&mut core);
    let damage = core.take_damage();
    assert!(damage.any());
    let picture = core.picture(&damage).expect("a picture");
    assert_eq!(picture.terminal.screen().row_text(0), "hi");
    assert_eq!((picture.area, picture.cell), (area(400, 200), CELL));
    assert_eq!((picture.line, picture.focused), (2, false));
    // 12 points at 150 % are 24 pixels.
    assert!((picture.letters - 24.0).abs() < 0.001);
    assert!(!core.take_damage().any());
}
