use std::path::Path;

use imperifex_bench_domain::emit::EmitResult;
use imperifex_bench_domain::metric::{Load, Metric};
use imperifex_bench_domain::report::Report;

use super::*;
use crate::testing::adapters::FakeAdapters;
use crate::testing::FakeFiles;

fn parse(line: &str) -> Result<Command, String> {
    Command::parse(&line.split_whitespace().collect::<Vec<_>>())
}

fn emit_order(line: &str) -> EmitOrder {
    match parse(line) {
        Ok(Command::Emit(order)) => order,
        other => panic!("{other:?}"),
    }
}

/// Runs on fakes and gives back what they were left with.
fn run_on(command: Command, adapters: FakeAdapters) -> (Output, FakeAdapters) {
    let out = command.run(&adapters);
    (out, adapters)
}

#[test]
fn help_is_the_default_and_goes_well_without_a_terminal() {
    let none: [&str; 0] = [];
    assert_eq!(Command::parse(&none), Ok(Command::Help));
    assert_eq!(parse("--help"), Ok(Command::Help));
    let (out, _) = run_on(Command::Help, FakeAdapters::without_console("no console"));
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
    let command = parse("emit color --mib 1 --out r.json").unwrap();
    let (out, adapters) = run_on(command, FakeAdapters::default());
    assert!(out.ok, "{}", out.text);
    assert_eq!(out.text, "color: 1.0 MB en 0.100 s = 10.0 MB/s");
    let files = &adapters.files;
    let saved = files.read(Path::new("r.json")).unwrap().unwrap();
    let result = EmitResult::from_json(&String::from_utf8(saved).unwrap()).unwrap();
    assert_eq!((result.load, result.bytes), (Load::Color, BYTES_PER_MB));
}

#[test]
fn without_a_terminal_nothing_is_measured_or_saved() {
    let command = parse("emit ascii --mib 1 --out r.json").unwrap();
    let console = FakeAdapters::without_console("la salida no es una consola");
    let (out, adapters) = run_on(command, console);
    assert_eq!(
        (out.ok, out.text.as_str()),
        (false, "la salida no es una consola")
    );
    assert!(adapters.files.files.borrow().is_empty());
}

#[test]
fn a_result_that_cannot_be_saved_is_a_failure() {
    let adapters = FakeAdapters {
        files: FakeFiles {
            failure: Some("disco lleno".into()),
            ..Default::default()
        },
        ..Default::default()
    };
    let command = parse("emit ascii --mib 1 --out r.json").unwrap();
    assert!(!run_on(command, adapters).0.ok);
}

fn footprint_order(line: &str) -> FootprintOrder {
    match parse(line) {
        Ok(Command::Footprint(order)) => order,
        other => panic!("{other:?}"),
    }
}

#[test]
fn footprint_takes_a_minute_unless_told_otherwise() {
    let order = footprint_order("footprint warp");
    assert_eq!((order.seconds, order.out), (60, None));
    let order = footprint_order("footprint warp --out r.json --seconds 5");
    assert_eq!((order.seconds, order.out), (5, Some("r.json".into())));
}

#[test]
fn a_wrong_footprint_argument_is_an_error_that_shows_it() {
    for (line, shown) in [
        ("footprint", "terminal"),
        ("footprint warp --seconds 0", "0"),
        ("footprint warp --seconds long", "long"),
        ("footprint warp --seconds 4000", "4000"),
        ("footprint warp --runs 2", "--runs"),
    ] {
        let error = parse(line).unwrap_err();
        assert!(error.contains(shown), "{line}: {error}");
    }
}

#[test]
fn a_footprint_without_a_file_prints_the_report() {
    let command = parse("footprint warp --seconds 2").unwrap();
    let (out, adapters) = run_on(command, FakeAdapters::default());
    assert!(out.ok, "{}", out.text);
    let report = Report::from_json(&out.text).unwrap();
    assert_eq!(report.measurements.len(), 4);
    assert!(adapters.files.files.borrow().is_empty());
}

#[test]
fn a_footprint_with_a_file_saves_the_report_and_prints_its_summary() {
    let command = parse("footprint warp --seconds 2 --out r.json").unwrap();
    let (out, adapters) = run_on(command, FakeAdapters::default());
    assert!(
        out.ok && out.text.contains("Memoria privada en reposo"),
        "{}",
        out.text
    );
    let saved = adapters.files.read(Path::new("r.json")).unwrap().unwrap();
    assert!(Report::from_json(&String::from_utf8(saved).unwrap()).is_ok());
}

fn startup_order(line: &str) -> StartupOrder {
    match parse(line) {
        Ok(Command::Startup(order)) => order,
        other => panic!("{other:?}"),
    }
}

#[test]
fn startup_runs_ten_times_warm_unless_told_otherwise() {
    let order = startup_order("startup warp");
    assert_eq!((order.runs, order.cold, order.out), (10, false, None));
    let order = startup_order("startup warp --cold --out r.json --runs 3");
    assert_eq!(
        (order.runs, order.cold, order.out),
        (3, true, Some("r.json".into()))
    );
}

#[test]
fn a_wrong_startup_argument_is_an_error_that_shows_it() {
    for (line, shown) in [
        ("startup", "terminal"),
        ("startup warp --runs 0", "0"),
        ("startup warp --runs 101", "101"),
        ("startup warp --runs", "--runs"),
        ("startup warp --warm", "--warm"),
    ] {
        let error = parse(line).unwrap_err();
        assert!(error.contains(shown), "{line}: {error}");
    }
}

#[test]
fn startup_reports_a_warm_start() {
    let command = parse("startup warp --runs 2").unwrap();
    let (out, _) = run_on(command, FakeAdapters::default());
    assert!(out.ok, "{}", out.text);
    let report = Report::from_json(&out.text).unwrap();
    assert_eq!(report.measurements[0].metric, Metric::StartupWarm);
}
