use super::*;
use crate::testing::{data, FakeClock, FakeLauncher, FakeParser, Script};
use imperifex_domain::ports::PtyRead;

fn request(timeout: Duration) -> Request {
    Request {
        command_line: "programa".into(),
        columns: 20,
        rows: 3,
        timeout,
    }
}

/// Runs a fake program; each read of the clock moves it 10 ms.
fn finish(script: Script) -> (Finished, FakeLauncher) {
    let launcher = FakeLauncher::running(script);
    let clock = FakeClock::ticking(10);
    let request = request(Duration::from_secs(2));
    let finished = run(&launcher, &mut FakeParser, &clock, &request).unwrap();
    (finished, launcher)
}

#[test]
fn a_program_that_leaves_gives_its_code_and_its_screen() {
    let (finished, launcher) = finish(Script {
        output: vec![data("uno\r\n"), data("dos")],
        leaves: Some(3),
        ..Script::default()
    });
    assert_eq!(finished.ending, Ending::Left(3));
    assert!(finished.output_complete);
    assert_eq!(finished.terminal.screen().text(), "uno\ndos\n");
    assert_eq!(
        launcher.asked.borrow().started,
        [("programa".to_string(), 20, 3)]
    );
    assert_eq!(launcher.asked.borrow().closed, 1);
}

#[test]
fn output_that_arrives_after_it_left_is_not_lost() {
    // A pause shorter than the quiet wanted, then more output; and the last
    // of it only once the console is closing.
    let mut output = vec![data("a")];
    output.extend(std::iter::repeat_n(PtyRead::Nothing, 5));
    output.push(data("b"));
    let (finished, _) = finish(Script {
        output,
        after_close: vec!["c"],
        leaves: Some(0),
        ..Script::default()
    });
    assert_eq!(finished.terminal.screen().row_text(0), "abc");
}

#[test]
fn out_of_time_the_console_is_closed_and_it_is_said() {
    let (finished, launcher) = finish(Script {
        output: vec![data("esperando")],
        leaves_when_closed: Some(1),
        ..Script::default()
    });
    assert_eq!(finished.ending, Ending::TimedOut);
    assert_eq!(finished.terminal.screen().row_text(0), "esperando");
    assert_eq!(launcher.asked.borrow().closed, 1);
}

#[test]
fn a_program_that_does_not_leave_is_left_alone_and_named() {
    let (finished, _) = finish(Script::default());
    assert_eq!(finished.ending, Ending::StillRunning { process_id: 4321 });
}

#[test]
fn output_that_never_ends_is_said_to_be_incomplete() {
    let (finished, _) = finish(Script {
        leaves: Some(0),
        never_ends: true,
        ..Script::default()
    });
    assert!(!finished.output_complete);
    assert_eq!(finished.ending, Ending::Left(0));
}

#[test]
fn output_that_does_not_stop_still_runs_out_of_time() {
    let (finished, _) = finish(Script {
        output: vec![data("x"); 10_000],
        leaves_when_closed: Some(1),
        never_ends: true,
        ..Script::default()
    });
    assert_eq!(finished.ending, Ending::TimedOut);
}
