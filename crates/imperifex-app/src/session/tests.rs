use super::*;
use crate::testing::{data, FakeLauncher, FakeParser, Script};
use imperifex_domain::ports::PtyLauncher;

fn session<'a>(launcher: &FakeLauncher, parser: &'a mut FakeParser) -> Session<'a> {
    let pty = launcher.start("shell", 10, 3).unwrap();
    Session::new(pty, parser, Terminal::new(10, 3))
}

#[test]
fn what_the_program_writes_reaches_the_screen() {
    let launcher = FakeLauncher::running(Script {
        output: vec![data("ho"), PtyRead::Nothing, data("la")],
        ..Script::default()
    });
    let mut parser = FakeParser;
    let mut session = session(&launcher, &mut parser);
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
    let mut parser = FakeParser;
    session(&launcher, &mut parser).send(b"dir\r").unwrap();
    assert_eq!(launcher.asked.borrow().written, b"dir\r");
}

#[test]
fn a_new_size_reaches_the_console_and_the_screen() {
    let launcher = FakeLauncher::default();
    let mut parser = FakeParser;
    let mut session = session(&launcher, &mut parser);
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
    let mut parser = FakeParser;
    let mut session = session(&launcher, &mut parser);
    assert!(session.resize(20, 5).is_err());
    assert_eq!(session.terminal.screen().columns(), 10);
}
