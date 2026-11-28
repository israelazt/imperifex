use imperifex_domain::input::NamedKey;
use imperifex_domain::ports::PtyRead;

use super::*;
use crate::testing::{data, fake_parser, no_wake, FakeClock, FakeLauncher, Script};

fn open_with(launcher: &FakeLauncher, chosen: Option<&str>) -> Result<ShellTab, String> {
    ShellTab::open(&Opening {
        launcher,
        new_parser: &fake_parser,
        chosen,
        size: (10, 3),
        wake: no_wake(),
    })
}

fn open(launcher: &FakeLauncher) -> ShellTab {
    open_with(launcher, None).expect("a shell")
}

/// A tab with its first screen already drawn.
fn drawn(output: Vec<PtyRead>) -> ShellTab {
    let launcher = FakeLauncher::running(Script {
        output,
        ..Script::default()
    });
    let mut tab = open(&launcher);
    // The first look: a new screen and its cursor.
    tab.looks_different();
    tab.terminal_mut().screen_mut().take_damage();
    tab
}

fn started(launcher: &FakeLauncher) -> Vec<String> {
    let asked = launcher.asked.borrow();
    asked
        .started
        .iter()
        .map(|(line, ..)| line.clone())
        .collect()
}

#[test]
fn it_opens_the_first_shell_that_starts_with_the_size_given() {
    let launcher = FakeLauncher {
        missing: vec!["pwsh.exe"],
        ..FakeLauncher::default()
    };
    open(&launcher);
    assert_eq!(started(&launcher), ["pwsh.exe", "powershell.exe"]);
    assert_eq!(
        launcher.asked.borrow().started[1],
        ("powershell.exe".into(), 10, 3)
    );
}

#[test]
fn a_chosen_shell_that_does_not_start_is_not_replaced_by_another() {
    let launcher = FakeLauncher {
        missing: vec!["cmd.exe"],
        ..FakeLauncher::default()
    };
    let error = open_with(&launcher, Some("cmd")).err();
    assert_eq!(error.as_deref(), Some("cmd.exe is not installed"));
    assert_eq!(started(&launcher), ["cmd.exe"]);
}

#[test]
fn all_the_output_is_read_and_asks_for_one_redraw() {
    let mut tab = drawn(vec![data("ho"), data("la")]);
    let read = tab.read(&FakeClock::ticking(1));
    assert_eq!(
        read,
        Read {
            redraw: true,
            ..Read::default()
        }
    );
    assert_eq!(tab.terminal().screen().row_text(0), "hola");
}

#[test]
fn with_no_output_or_nothing_new_to_see_there_is_nothing_to_draw() {
    let mut tab = drawn(Vec::new());
    assert_eq!(tab.read(&FakeClock::ticking(1)), Read::default());
}

#[test]
fn a_cursor_that_only_moves_asks_for_a_redraw() {
    let mut tab = drawn(vec![data("\n")]);
    assert!(tab.read(&FakeClock::ticking(1)).redraw);
    assert!(!tab.read(&FakeClock::ticking(1)).redraw);
}

#[test]
fn hiding_the_cursor_asks_for_a_redraw() {
    let mut tab = drawn(Vec::new());
    tab.terminal_mut().modes.cursor_visible = false;
    assert!(tab.read(&FakeClock::ticking(1)).redraw);
}

#[test]
fn after_8_ms_of_reading_the_window_gets_a_turn_and_the_rest_waits() {
    let mut tab = drawn(vec![data("a"), data("b"), data("c"), data("d")]);
    // Each look at the clock is 3 ms later: the third read is past 8 ms.
    let first = tab.read(&FakeClock::ticking(3));
    assert!(first.more && first.redraw && !first.ended);
    assert_eq!(tab.terminal().screen().row_text(0), "abc");
    let rest = tab.read(&FakeClock::ticking(3));
    assert!(!rest.more);
    assert_eq!(tab.terminal().screen().row_text(0), "abcd");
}

#[test]
fn a_shell_that_is_gone_is_said() {
    let mut tab = drawn(vec![data("bye"), PtyRead::End]);
    let read = tab.read(&FakeClock::ticking(1));
    assert!(read.ended && read.redraw);
}

#[test]
fn what_is_typed_reaches_the_shell_and_a_silent_key_sends_nothing() {
    let launcher = FakeLauncher::default();
    let mut tab = open(&launcher);
    let typed = |text: Option<&str>, key| KeyInput {
        text: text.map(str::to_string),
        key,
    };
    tab.typed(&typed(Some("a"), None)).expect("sent");
    tab.typed(&typed(None, Some(NamedKey::Enter)))
        .expect("sent");
    tab.typed(&KeyInput::default()).expect("nothing");
    assert_eq!(launcher.asked.borrow().written, b"a\r");
}

fn ms(time: u64) -> Duration {
    Duration::from_millis(time)
}

#[test]
fn the_size_changes_only_once_it_has_settled() {
    let launcher = FakeLauncher::default();
    let mut tab = open(&launcher);
    tab.room_for((20, 5), ms(0));
    tab.room_for((30, 6), ms(50));
    assert_eq!(tab.resize_in(ms(100)), Some(ms(50)));
    assert!(!tab.resize_if_settled(ms(100)));
    assert!(tab.resize_if_settled(ms(150)));
    assert_eq!(launcher.asked.borrow().resized, [(30, 6)]);
    let screen = tab.terminal().screen();
    assert_eq!((screen.columns(), screen.rows()), (30, 6));
    assert_eq!(tab.resize_in(ms(200)), None);
}

#[test]
fn a_console_that_refuses_a_size_leaves_the_terminal_as_it_was() {
    let launcher = FakeLauncher::running(Script {
        refuses_to_resize: true,
        ..Script::default()
    });
    let mut tab = open(&launcher);
    tab.room_for((20, 5), ms(0));
    assert!(!tab.resize_if_settled(ms(200)));
    assert_eq!(tab.terminal().screen().columns(), 10);
}

#[test]
fn a_program_that_left_has_its_console_closed_and_then_ends() {
    let launcher = FakeLauncher::running(Script {
        leaves: Some(0),
        after_close: vec!["bye"],
        ..Script::default()
    });
    let mut tab = open(&launcher);
    assert!(!tab.read(&FakeClock::ticking(1)).ended);
    assert_eq!(launcher.asked.borrow().closed, 1);
    assert!(tab.read(&FakeClock::ticking(1)).ended);
    assert_eq!(tab.terminal().screen().row_text(0), "bye");
}
