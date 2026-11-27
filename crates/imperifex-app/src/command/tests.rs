use std::time::Duration;

use super::*;
use crate::testing::{data, FakeClock, FakeLauncher, FakeParser, Script};

fn parse(line: &str) -> Result<Command, String> {
    let args: Vec<&str> = line.split_whitespace().collect();
    Command::parse(&args)
}

fn output(command: &Command, launcher: &FakeLauncher) -> Output {
    let service = StatusService::new(FakeClock::ticking(2));
    let clock = FakeClock::ticking(10);
    let shells = Shells {
        launcher,
        parser: &mut FakeParser,
        clock: &clock,
    };
    command.run(&service, shells)
}

fn ran(line: &str, script: Script) -> Output {
    output(&parse(line).unwrap(), &FakeLauncher::running(script))
}

#[test]
fn help_is_the_default_and_status_takes_its_options() {
    let none: [&str; 0] = [];
    assert_eq!(Command::parse(&none).unwrap(), Command::Help);
    assert_eq!(
        parse("status --json").unwrap(),
        Command::Status { json: true }
    );
}

#[test]
fn what_it_does_not_know_is_an_error() {
    assert!(parse("fly").unwrap_err().contains("fly"));
    assert!(parse("status --fast").unwrap_err().contains("--fast"));
}

#[test]
fn status_prints_the_report() {
    let out = output(&Command::Status { json: false }, &FakeLauncher::default());
    assert!(out.ok && out.text.contains("2 ms"), "{}", out.text);
}

#[test]
fn run_takes_a_size_a_time_and_the_program_with_its_arguments() {
    let line = "run --headless --columns 100 --rows 40 --timeout 5 --history -- cmd /c dir --x";
    let Command::Run(options) = parse(line).unwrap() else {
        panic!("not a run");
    };
    assert_eq!(options.request.command_line, "cmd /c dir --x");
    assert_eq!((options.request.columns, options.request.rows), (100, 40));
    assert_eq!(options.request.timeout, Duration::from_secs(5));
    assert!(options.history);
}

#[test]
fn with_a_profile_the_one_command_is_written_for_its_shell() {
    let args = ["run", "--headless", "--profile", "cmd", "--", "echo a b"];
    let Command::Run(options) = Command::parse(&args).unwrap() else {
        panic!("not a run");
    };
    assert_eq!(
        options.request.command_line,
        "cmd.exe /d /s /c \"echo a b\""
    );
}

#[test]
fn the_help_names_every_profile() {
    for profile in imperifex_domain::shell::PROFILES {
        assert!(HELP.contains(profile.id), "{}", profile.id);
    }
}

#[test]
fn run_without_what_it_needs_says_why() {
    for (line, said) in [
        ("run -- cmd", "--headless"),
        ("run --headless", "falta el programa"),
        ("run --headless --", "falta el programa"),
        ("run --headless --columns 0 -- cmd", "1 a 1000"),
        ("run --headless --rows 5000 -- cmd", "1 a 1000"),
        ("run --headless --timeout pronto -- cmd", "pronto"),
        ("run --headless --columns", "falta el número"),
        ("run --headless cmd", "cmd"),
        ("run --headless --profile fish -- ls", "powershell"),
        ("run --headless --profile cmd -- dir /b", "una sola orden"),
        ("run --headless --profile cmd --", "una sola orden"),
    ] {
        let error = parse(line).unwrap_err();
        assert!(error.contains(said), "{line}: {error}");
    }
}

fn leaving(code: u32) -> Script {
    Script {
        output: vec![data("uno\r\n\ndos\r\n")],
        leaves: Some(code),
        ..Script::default()
    }
}

#[test]
fn run_prints_the_screen_without_the_empty_lines_at_the_end() {
    let out = ran("run --headless -- p", leaving(0));
    assert_eq!(out.text, "uno\n\ndos");
    assert!(out.ok && out.notice.is_none());
}

#[test]
fn the_history_is_printed_only_when_asked() {
    let script = Script {
        output: vec![data("a\r\nb\r\nc\r\nd")],
        leaves: Some(0),
        ..Script::default()
    };
    let screen = ran("run --headless --rows 2 -- p", script.clone());
    assert_eq!(screen.text, "c\nd");
    let all = ran("run --headless --rows 2 --history -- p", script);
    assert_eq!(all.text, "a\nb\nc\nd");
}

#[test]
fn a_bad_ending_is_not_fine_and_is_explained() {
    let failed = ran("run --headless -- p", leaving(7));
    assert!(!failed.ok);
    assert!(failed.notice.unwrap().contains('7'));
    assert!(failed.text.contains("dos"));

    let stuck = ran("run --headless --timeout 1 -- p", Script::default());
    assert!(!stuck.ok);
    assert!(stuck.notice.unwrap().contains("4321"));
}
