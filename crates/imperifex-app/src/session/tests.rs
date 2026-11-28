use super::*;
use crate::testing::{data, fake_parser, no_wake, FakeLauncher, Script};
use imperifex_domain::ports::PtyLauncher;

fn session(launcher: &FakeLauncher) -> Session {
    let pty = launcher.start("shell", 10, 3, no_wake()).unwrap();
    Session::new(pty, fake_parser(), Terminal::new(10, 3))
}

#[test]
fn what_the_program_writes_reaches_the_screen() {
    let launcher = FakeLauncher::running(Script {
        output: vec![data("ho"), PtyRead::Nothing, data("la")],
        ..Script::default()
    });
    let mut session = session(&launcher);
    let steps: Vec<_> = (0..4).map(|_| session.step(Duration::ZERO)).collect();
    assert_eq!(
        steps,
        [Step::Changed, Step::Quiet, Step::Changed, Step::Quiet]
    );
    assert_eq!(session.terminal.screen().row_text(0), "hola");
}

#[test]
fn what_the_user_types_goes_to_the_console() {
    let launcher = FakeLauncher::default();
    session(&launcher).send(b"dir\r").unwrap();
    assert_eq!(launcher.asked.borrow().written, b"dir\r");
}

#[test]
fn a_new_size_reaches_the_console_and_the_screen() {
    let launcher = FakeLauncher::default();
    let mut session = session(&launcher);
    session.resize(20, 5).unwrap();
    assert_eq!(launcher.asked.borrow().resized, [(20, 5)]);
    let screen = session.terminal.screen();
    assert_eq!((screen.columns(), screen.rows()), (20, 5));
}

#[test]
fn a_console_that_refuses_a_size_leaves_the_screen_as_it_was() {
    let launcher = FakeLauncher::running(Script {
        refuses_to_resize: true,
        ..Script::default()
    });
    let mut session = session(&launcher);
    assert!(session.resize(20, 5).is_err());
    assert_eq!(session.terminal.screen().columns(), 10);
}
