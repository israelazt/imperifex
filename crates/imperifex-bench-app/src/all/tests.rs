use imperifex_bench_domain::metric::Metric;

use super::*;
use crate::testing::adapters::FakeAdapters;

/// On the fakes of the orders, the footprint and the start work, and what types
/// into a window does not: their screen gives no picture.
fn run(adapters: &FakeAdapters, out: Option<&Path>) -> Result<All, String> {
    run_all("warp", out, adapters)
}

#[test]
fn a_part_that_fails_is_noted_and_the_rest_go_on() {
    let adapters = FakeAdapters::default();
    let all = run(&adapters, Some(Path::new("out"))).unwrap();
    let metrics: Vec<Metric> = all.report.measurements.iter().map(|m| m.metric).collect();
    assert!(metrics.contains(&Metric::CpuIdle) && metrics.contains(&Metric::StartupWarm));
    for part in [
        "latencia en el prompt",
        "latencia en un programa",
        "salida cargo",
        "memoria",
    ] {
        let noted = all.report.notes.iter().any(|n| n.contains(part));
        assert!(noted, "{part}: {:?}", all.report.notes);
    }
    assert_eq!(all.report.notes.len(), 9);
}

#[test]
fn the_report_is_left_in_the_folder_asked_for() {
    let adapters = FakeAdapters::default();
    let all = run(&adapters, Some(Path::new("out"))).unwrap();
    assert_eq!(all.file, Path::new("out").join(REPORT_FILE));
    let saved = adapters.files.read(&all.file).unwrap().unwrap();
    let saved = Report::from_json(&String::from_utf8(saved).unwrap()).unwrap();
    assert_eq!(saved, all.report);
}

#[test]
fn a_failure_with_a_personal_path_is_saved_without_it() {
    let adapters = FakeAdapters {
        no_gpu: Some("no se abre LOCAL/Temp/gpu.log".into()),
        ..Default::default()
    };
    let all = run(&adapters, Some(Path::new("out"))).unwrap();
    let saved = adapters.files.read(&all.file).unwrap().unwrap();
    let saved = Report::from_json(&String::from_utf8(saved).unwrap()).unwrap();
    let note = &saved.notes[0];
    assert!(note.ends_with("no se abre %TEMP%/gpu.log"), "{note}");
}

#[test]
fn without_a_folder_each_run_gets_a_new_numbered_one() {
    let adapters = FakeAdapters::default();
    let results = Path::new("elsewhere").join("bench").join("results");
    let first = run(&adapters, None).unwrap().file;
    let second = run(&adapters, None).unwrap().file;
    assert_eq!(first, results.join("2026-01-02-warp-1").join(REPORT_FILE));
    assert_eq!(second, results.join("2026-01-02-warp-2").join(REPORT_FILE));
}

#[test]
fn a_report_that_is_there_is_never_written_over_and_nothing_is_measured() {
    let adapters = FakeAdapters::default();
    let file = Path::new("out").join(REPORT_FILE);
    adapters.files.write(&file, b"kept").unwrap();
    let error = run(&adapters, Some(Path::new("out"))).unwrap_err();
    assert!(error.contains("no se sobrescribe"), "{error}");
    assert_eq!(adapters.files.read(&file), Ok(Some(b"kept".to_vec())));
    assert!(adapters.desktop.state.borrow().asked.is_empty());
}

#[test]
fn a_folder_that_cannot_be_written_is_found_out_before_measuring() {
    let adapters = FakeAdapters {
        files: crate::testing::FakeFiles {
            failure: Some("solo lectura".into()),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        run(&adapters, Some(Path::new("out"))).unwrap_err(),
        "solo lectura"
    );
    assert!(adapters.desktop.state.borrow().asked.is_empty());
}
