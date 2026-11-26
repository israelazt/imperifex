use super::*;
use crate::testing::desktop::{home, rig, Behaviour};
use crate::testing::emitting::{emitting, Emitting};
use crate::testing::system::warp;

fn run(emit: &Emitting, repeats: u32) -> Result<Vec<Measurement>, String> {
    let service = OutputService {
        ports: emit.ports.clone(),
        files: &emit.files,
        bench_exe: "bench.exe".into(),
    };
    service.run(&warp(), Load::Color, 4, repeats, home())
}

#[test]
fn what_is_seen_lasts_until_the_last_big_change_and_the_rest_comes_from_emit() {
    let emit = emitting(rig(Behaviour::default(), true), true);
    let measurements = run(&emit, 2).unwrap();
    assert_eq!(measurements[0].metric, Metric::OutputSeen(Load::Color));
    assert_eq!(measurements[0].samples, [800.0, 800.0]);
    assert_eq!(measurements[1].metric, Metric::OutputConsumed(Load::Color));
    assert_eq!(measurements[1].samples, [40.0, 40.0]);
}

#[test]
fn the_processor_is_the_total_spent_over_the_time_it_took() {
    let emit = emitting(rig(Behaviour::default(), true), true);
    let measurements = run(&emit, 1).unwrap();
    // The author's process and the window's each spent 80 ms of 800 ms.
    assert_eq!(measurements[2].metric, Metric::CpuDuringOutput(Load::Color));
    assert_eq!(measurements[2].samples, [20.0]);
}

#[test]
fn the_screen_is_cleared_between_repeats_and_each_has_its_own_file() {
    let emit = emitting(rig(Behaviour::default(), true), true);
    run(&emit, 2).unwrap();
    let typed = emit.ports.keyboard.typed();
    assert_eq!(typed.iter().filter(|t| *t == "text cls").count(), 1);
    let lines: Vec<&String> = typed.iter().filter(|t| t.contains("'emit'")).collect();
    assert!(
        lines[0].contains("'color' '--mib' '4' '--out'"),
        "{}",
        lines[0]
    );
    assert_ne!(lines[0], lines[1]);
}

#[test]
fn a_result_that_was_already_there_is_never_reused() {
    let emit = emitting(rig(Behaviour::default(), true), true);
    let old = fresh_result(&emit.files, "color").unwrap();
    emit.files.put(&old, b"old");
    assert_ne!(fresh_result(&emit.files, "color").unwrap(), old);
}

#[test]
fn a_result_that_never_appears_is_an_error_with_its_path_and_the_window_is_closed() {
    let emit = emitting(rig(Behaviour::default(), true), false);
    let error = run(&emit, 1).unwrap_err();
    assert!(
        error.contains("no apareció") && error.contains("color-1.json"),
        "{error}"
    );
    assert!(emit.ports.session.desktop.state.borrow().opened.is_empty());
}
