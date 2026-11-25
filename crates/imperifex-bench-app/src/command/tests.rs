use std::path::Path;

use imperifex_bench_domain::emit::EmitResult;

use super::*;
use crate::testing::{Events, FakeClock, FakeFiles, FakeOutput};

fn parse(line: &str) -> Result<Command, String> {
    Command::parse(&line.split_whitespace().collect::<Vec<_>>())
}

fn emit_order(line: &str) -> EmitOrder {
    match parse(line) {
        Ok(Command::Emit(order)) => order,
        other => panic!("{other:?}"),
    }
}

fn run(command: Command, files: &FakeFiles, output: Result<(), &str>) -> Output {
    let events = Events::default();
    let service = EmitService::new(FakeClock::ticking(100, &events));
    command.run(&service, files, || {
        output
            .map(|()| FakeOutput::new(&events))
            .map_err(str::to_string)
    })
}

#[test]
fn help_is_the_default_and_goes_well_without_a_terminal() {
    let none: [&str; 0] = [];
    assert_eq!(Command::parse(&none), Ok(Command::Help));
    assert_eq!(parse("--help"), Ok(Command::Help));
    let out = run(Command::Help, &FakeFiles::default(), Err("no console"));
    assert!(out.ok && out.text.contains("imperifex-bench emit"));
}

#[test]
fn what_it_does_not_know_is_an_error() {
    assert!(parse("fly").unwrap_err().contains("fly"));
    assert!(parse("help --fast").unwrap_err().contains("--fast"));
    assert!(parse("emit ascii --fast 1").unwrap_err().contains("--fast"));
}

#[test]
fn emit_has_a_size_and_a_seed_by_default_and_ascii_is_bigger() {
    let ascii = emit_order("emit ascii --out a.json");
    assert_eq!((ascii.mib, ascii.seed), (100, 1));
    assert_eq!(ascii.out, Path::new("a.json"));
    assert_eq!(emit_order("emit color --out a.json").mib, 32);
}

#[test]
fn emit_takes_its_options_in_any_order() {
    let order = emit_order("emit cursor --seed 9 --out a.json --mib 4");
    assert_eq!((order.load, order.mib, order.seed), (Load::Cursor, 4, 9));
}

#[test]
fn a_wrong_argument_is_an_error_that_shows_it() {
    for (line, shown) in [
        ("emit", "carga"),
        ("emit binary --out a", "binary"),
        ("emit ascii --mib many --out a", "many"),
        ("emit ascii --mib 0 --out a", "0"),
        ("emit ascii --mib 5000 --out a", "5000"),
        ("emit ascii --seed -1 --out a", "-1"),
        ("emit ascii --out", "--out"),
        ("emit ascii", "--out"),
    ] {
        let error = parse(line).unwrap_err();
        assert!(error.contains(shown), "{line}: {error}");
    }
}

#[test]
fn emitting_saves_the_result_and_says_it() {
    let files = FakeFiles::default();
    let out = run(
        parse("emit color --mib 1 --out r.json").unwrap(),
        &files,
        Ok(()),
    );
    assert!(out.ok, "{}", out.text);
    assert_eq!(out.text, "color: 1.0 MB en 0.100 s = 10.0 MB/s");
    let saved = files.read(Path::new("r.json")).unwrap().unwrap();
    let result = EmitResult::from_json(&String::from_utf8(saved).unwrap()).unwrap();
    assert_eq!((result.load, result.bytes), (Load::Color, BYTES_PER_MB));
}

#[test]
fn without_a_terminal_nothing_is_measured_or_saved() {
    let files = FakeFiles::default();
    let command = parse("emit ascii --mib 1 --out r.json").unwrap();
    let out = run(command, &files, Err("la salida no es una consola"));
    assert_eq!(
        (out.ok, out.text.as_str()),
        (false, "la salida no es una consola")
    );
    assert!(files.files.borrow().is_empty());
}

#[test]
fn a_result_that_cannot_be_saved_is_a_failure() {
    let files = FakeFiles {
        failure: Some("disco lleno".into()),
        ..Default::default()
    };
    let command = parse("emit ascii --mib 1 --out r.json").unwrap();
    assert!(!run(command, &files, Ok(())).ok);
}
