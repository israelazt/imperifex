use super::*;
use crate::testing::desktop::{home, rig, Behaviour, AUTHORS_WINDOW};
use crate::testing::keyboard::{typing, FakeTyping, Script, BEFORE_COUNTING};
use crate::testing::system::warp;

const BENCH: &str = "bench.exe";

fn run(
    script: Script,
    presses: u32,
    mode: LatencyMode,
) -> (Result<Measurement, String>, FakeTyping) {
    let ports = typing(rig(Behaviour::default(), true), script);
    let service = LatencyService::new(ports.clone(), BENCH.to_string());
    (service.run(&warp(), presses, mode, home()), ports)
}

fn closed(ports: &FakeTyping) -> bool {
    ports.session.desktop.state.borrow().opened.is_empty()
}

#[test]
fn a_sample_is_the_time_from_the_press_to_the_picture_that_shows_it() {
    let (result, ports) = run(Script::showing_after(12), 5, LatencyMode::Prompt);
    let measurement = result.unwrap();
    assert_eq!(measurement.metric, Metric::KeyLatencyPrompt);
    assert_eq!(measurement.samples, [12.0; 5]);
    assert!(closed(&ports));
}

#[test]
fn at_the_prompt_the_nonsense_is_typed_first_and_not_measured() {
    let (_, ports) = run(Script::showing_after(5), 2, LatencyMode::Prompt);
    let mut expected = vec!["text zqxj"];
    expected.extend(["a", "Backspace"].repeat(BEFORE_COUNTING + 2));
    assert_eq!(ports.keyboard.typed(), expected);
}

#[test]
fn in_a_program_the_echo_is_run_first_and_left_with_q() {
    let (result, ports) = run(Script::showing_after(5), 1, LatencyMode::Program);
    assert_eq!(result.unwrap().metric, Metric::KeyLatencyProgram);
    let mut expected = vec!["text & 'bench.exe' 'echo'", "Enter"];
    expected.extend(["a", "Backspace"].repeat(BEFORE_COUNTING + 1));
    expected.push("q");
    assert_eq!(ports.keyboard.typed(), expected);
}

#[test]
fn the_region_watched_is_the_placed_window_without_its_top() {
    let (_, ports) = run(Script::showing_after(5), 1, LatencyMode::Prompt);
    let region = ports.screen.shown.borrow().watched[1];
    assert_eq!(
        (region.x, region.y, region.w, region.h),
        (100, 140, 1280, 680)
    );
}

#[test]
fn a_change_before_the_press_starts_over_and_is_not_taken_for_the_key() {
    // Something else changes on the screen 100 ms after each erased letter.
    let script = Script {
        stray_after_erasing: Some(100),
        ..Script::showing_after(20)
    };
    let (result, _) = run(script, 4, LatencyMode::Prompt);
    assert_eq!(result.unwrap().samples, [20.0; 4]);
}

#[test]
fn if_the_window_loses_the_front_nothing_more_is_pressed() {
    let script = Script {
        focus_lost_after: Some(BEFORE_COUNTING + 2),
        ..Script::showing_after(5)
    };
    let (result, ports) = run(script, 10, LatencyMode::Prompt);
    assert!(result.unwrap_err().contains("S3"));
    let typed = ports.keyboard.typed();
    assert_eq!(
        typed.iter().filter(|t| *t == "a").count(),
        BEFORE_COUNTING + 2,
        "{typed:?}"
    );
    assert!(closed(&ports), "its own window is still closed");
}

#[test]
fn a_few_presses_that_show_nothing_are_counted_and_left_out() {
    let script = Script {
        unseen: vec![BEFORE_COUNTING + 3],
        ..Script::showing_after(5)
    };
    let measurement = run(script, 40, LatencyMode::Prompt).0.unwrap();
    assert_eq!(measurement.summary.count, 39);
    assert!(
        measurement.notes[0].ends_with("1."),
        "{:?}",
        measurement.notes
    );
}

#[test]
fn over_five_in_a_hundred_failures_stop_the_measure() {
    let script = Script {
        unseen: vec![6, 7, 8],
        ..Script::showing_after(5)
    };
    let (result, ports) = run(script, 40, LatencyMode::Prompt);
    let error = result.unwrap_err();
    assert!(error.contains("3 de 40"), "{error}");
    assert!(closed(&ports));
}

#[test]
fn a_screen_that_gives_no_picture_is_an_error_that_says_why() {
    let script = Script {
        blank: true,
        ..Script::showing_after(5)
    };
    let (result, ports) = run(script, 5, LatencyMode::Prompt);
    assert!(result.unwrap_err().contains("ninguna imagen"));
    assert!(closed(&ports));
}

#[test]
fn the_authors_window_is_never_typed_into() {
    let (_, ports) = run(Script::showing_after(5), 3, LatencyMode::Program);
    let targets = ports.keyboard.targets();
    assert!(targets.iter().all(|&hwnd| hwnd != AUTHORS_WINDOW));
}

/// The measure stopped before any press that counts, saying why.
fn stopped_finding_the_zone(script: Script, why: &str) {
    let (result, ports) = run(script, 10, LatencyMode::Prompt);
    let error = result.unwrap_err();
    assert!(
        error.contains("no cambia siempre el mismo sitio") && error.contains(why),
        "{error}"
    );
    let pressed = ports.keyboard.typed();
    assert!(
        pressed.iter().filter(|t| *t == "a").count() <= BEFORE_COUNTING,
        "{pressed:?}"
    );
    assert!(closed(&ports));
}

#[test]
fn a_clock_that_changes_elsewhere_right_after_the_key_is_not_the_key() {
    let script = Script {
        clock_ticks: (BEFORE_COUNTING + 1..=BEFORE_COUNTING + 4).collect(),
        ..Script::showing_after(20)
    };
    assert_eq!(
        run(script, 4, LatencyMode::Prompt).0.unwrap().samples,
        [20.0; 4]
    );
}

#[test]
fn the_presses_that_find_the_zone_are_not_samples() {
    let (result, _) = run(Script::showing_after(5), 3, LatencyMode::Prompt);
    let measurement = result.unwrap();
    assert_eq!(measurement.summary.count, 3);
    assert!(
        measurement.notes[4].starts_with("Zona de la tecla: 32x"),
        "{:?}",
        measurement.notes
    );
}

#[test]
fn a_clock_that_changes_during_a_press_that_finds_the_zone_stops_it() {
    let script = Script {
        clock_ticks: vec![3],
        ..Script::showing_after(20)
    };
    stopped_finding_the_zone(script, "no coinciden");
}

#[test]
fn a_press_that_finds_the_zone_and_shows_nothing_stops_it() {
    let script = Script {
        unseen: vec![3],
        ..Script::showing_after(5)
    };
    stopped_finding_the_zone(script, "previa 3 no cambió nada");
}

#[test]
fn a_key_that_changes_much_of_the_window_stops_it() {
    let script = Script {
        letter_pixels: Some(6000),
        ..Script::showing_after(5)
    };
    stopped_finding_the_zone(script, "pasa del 10 %");
}

#[test]
fn a_slow_sample_stays_and_is_told_with_its_number() {
    let script = Script {
        slow: vec![(BEFORE_COUNTING + 3, 100)],
        ..Script::showing_after(5)
    };
    let measurement = run(script, 10, LatencyMode::Prompt).0.unwrap();
    assert_eq!(measurement.summary.count, 10);
    assert!(
        measurement.notes[3].contains("la 3 (100.0 ms, a los "),
        "{:?}",
        measurement.notes
    );
}

#[test]
fn with_no_slow_sample_the_note_says_none() {
    let measurement = run(Script::showing_after(5), 5, LatencyMode::Prompt)
        .0
        .unwrap();
    assert!(measurement.notes[3].ends_with("ninguna."));
}
