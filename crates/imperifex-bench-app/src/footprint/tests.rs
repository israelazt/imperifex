use super::*;
use crate::testing::system::{running, warp, FakeGpu, FakeProcesses, Listed};
use crate::testing::{Events, FakeClock};

const SECOND: Duration = Duration::from_secs(1);
/// A second of processor time, in ticks of 100 ns.
const BUSY: u64 = 10_000_000;
const MB: u64 = BYTES_PER_MB as u64;

fn sample(
    steps: Vec<Vec<Listed>>,
    gpu: &[f64],
    seconds: u64,
) -> (Result<Vec<Measurement>, String>, Vec<Vec<u32>>) {
    let asked = std::rc::Rc::default();
    let mut service = FootprintService::new(
        FakeClock::ticking(0, &Events::default()),
        FakeProcesses::new(steps),
        FakeGpu::new(gpu, &asked),
    );
    let result = service.sample(&warp(), seconds, SECOND);
    let asked = asked.borrow().clone();
    (result, asked)
}

fn samples_of(measurements: &[Measurement], metric: Metric) -> &[f64] {
    &measurements
        .iter()
        .find(|m| m.metric == metric)
        .unwrap()
        .samples
}

#[test]
fn memory_adds_up_the_terminals_processes_and_nothing_else() {
    let step = vec![
        running(1, 0, "warp.exe", 100 * MB, 200 * MB, 0),
        running(2, 1, "OpenConsole.exe", 5 * MB, 10 * MB, 0),
        running(3, 2, "pwsh.exe", 900 * MB, 900 * MB, 0),
    ];
    let (result, asked) = sample(vec![step], &[0.0], 2);
    let measurements = result.unwrap();
    assert_eq!(
        samples_of(&measurements, Metric::MemoryIdlePrivate),
        [105.0, 105.0]
    );
    assert_eq!(
        samples_of(&measurements, Metric::MemoryIdleWorkingSet),
        [210.0, 210.0]
    );
    assert_eq!(asked, [[1, 2], [1, 2], [1, 2]]);
}

#[test]
fn processor_time_is_taken_between_readings_of_each_process() {
    let warp_at = |cpu| running(1, 0, "warp.exe", MB, MB, cpu);
    let steps = vec![
        vec![warp_at(7 * BUSY)],
        vec![warp_at(7 * BUSY + BUSY / 10)],
        vec![
            warp_at(7 * BUSY + BUSY / 10),
            // Born between the two readings: all of its time is new.
            running(9, 1, "warp.exe", MB, MB, BUSY / 4),
        ],
    ];
    let (result, _) = sample(steps, &[0.0], 2);
    assert_eq!(samples_of(&result.unwrap(), Metric::CpuIdle), [10.0, 25.0]);
}

#[test]
fn the_cards_first_answer_only_starts_its_count() {
    let step = vec![running(1, 0, "warp.exe", MB, MB, 0)];
    let (result, _) = sample(vec![step], &[99.0, 3.0, 5.0], 2);
    assert_eq!(samples_of(&result.unwrap(), Metric::GpuIdle), [3.0, 5.0]);
}

#[test]
fn a_process_that_cannot_be_read_is_left_out_of_its_sample_and_noted() {
    let main = running(1, 0, "warp.exe", MB, MB, 0);
    let gone = Listed::unreadable(2, 1, "conhost.exe", "ya no estaba");
    let steps = vec![vec![main.clone()], vec![main.clone(), gone], vec![main]];
    let (result, _) = sample(steps, &[0.0], 2);
    let measurements = result.unwrap();
    assert_eq!(
        samples_of(&measurements, Metric::MemoryIdlePrivate),
        [1.0, 1.0]
    );
    let notes = measurements[0].notes.join(" ");
    assert!(notes.contains("1 (la última: ya no estaba)"), "{notes}");
}

#[test]
fn a_process_read_again_after_failing_does_not_bring_all_its_past_time() {
    let main = running(1, 0, "warp.exe", MB, MB, 0);
    let hidden = Listed::unreadable(2, 0, "warp.exe", "denegado");
    let back = running(2, 0, "warp.exe", MB, MB, 500 * BUSY);
    let steps = vec![vec![main.clone(), hidden], vec![main, back]];
    let (result, _) = sample(steps, &[0.0], 1);
    assert_eq!(samples_of(&result.unwrap(), Metric::CpuIdle), [0.0]);
}

#[test]
fn the_notes_say_how_many_processes_and_what_the_processor_misses() {
    let step = vec![running(1, 0, "warp.exe", MB, MB, 0)];
    let measurements = sample(vec![step], &[0.0], 1).0.unwrap();
    assert!(measurements[0].notes[0].contains("entre 1 y 1"));
    let cpu = measurements.iter().find(|m| m.metric == Metric::CpuIdle);
    assert!(cpu.unwrap().notes.iter().any(|n| n == SHORT_LIVED));
}

#[test]
fn without_the_terminal_running_there_is_nothing_to_measure() {
    let step = vec![running(3, 0, "pwsh.exe", MB, MB, 0)];
    let (result, asked) = sample(vec![step], &[0.0], 1);
    assert!(result.unwrap_err().contains("warp"));
    assert!(asked.is_empty());
}

#[test]
fn a_terminal_that_closes_while_measuring_stops_the_measure() {
    let step = vec![running(1, 0, "warp.exe", MB, MB, 0)];
    assert!(sample(vec![step, vec![]], &[0.0], 3).0.is_err());
}
