//! The wait for a ready window (D25), the same for every order that types.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::report::Measurement;

use crate::latency::{LatencyMode, LatencyService};
use crate::memory_load::MemoryService;
use crate::output_seen::OutputService;
use crate::testing::desktop::{home, rig, Behaviour};
use crate::testing::emitting::{emitting, Emitting};
use crate::testing::system::warp;

#[derive(Debug, Clone, Copy)]
enum Order {
    Latency,
    Output,
    Memory,
}

const ORDERS: [Order; 3] = [Order::Latency, Order::Output, Order::Memory];

fn run(order: Order, emit: &Emitting) -> Result<Vec<Measurement>, String> {
    let output = || OutputService {
        ports: emit.ports.clone(),
        files: &emit.files,
        bench_exe: "bench.exe".into(),
    };
    match order {
        Order::Latency => LatencyService::new(emit.ports.clone(), "bench.exe".into())
            .run(&warp(), 2, LatencyMode::Prompt, home())
            .map(|measured| vec![measured]),
        Order::Output => output().run(&warp(), Load::Color, 4, 1, home()),
        Order::Memory => {
            let readings = RefCell::new(VecDeque::from([100.0, 160.0]));
            let private_mb = || {
                readings
                    .borrow_mut()
                    .pop_front()
                    .ok_or_else(|| "sin más lecturas".to_string())
            };
            MemoryService {
                output: output(),
                private_mb: &private_mb,
            }
            .run(&warp(), home())
        }
    }
}

/// A desktop whose monitor shows these pictures once it is looked at.
fn opening(pictures: &[(u64, usize)]) -> Emitting {
    let emit = emitting(rig(Behaviour::default(), true), true);
    emit.ports.screen.shown.borrow_mut().opening = pictures.to_vec();
    emit
}

/// What was asked of the desktop, without the handles.
fn asked(emit: &Emitting) -> Vec<String> {
    let state = emit.ports.session.desktop.state.borrow();
    state
        .asked
        .iter()
        .filter_map(|a| a.split(' ').next().map(str::to_string))
        .collect()
}

/// The order stopped with this in its error, pressing nothing, and closed the
/// window it had opened without moving it.
fn stopped_without_typing(order: Order, emit: &Emitting, why: &str) {
    let error = run(order, emit).unwrap_err();
    assert!(
        error.contains("no se escribe nada") && error.contains(why),
        "{order:?}: {error}"
    );
    assert!(emit.ports.keyboard.typed().is_empty(), "{order:?}");
    assert_eq!(asked(emit), ["launch", "close"], "{order:?}");
}

#[test]
fn no_key_arrives_before_the_window_is_ready() {
    for order in ORDERS {
        // Nothing for 1.2 s after the launch, content, 1 s still while the
        // shell starts, and the last change.
        let emit = opening(&[(0, 0), (2200, 20_000), (3200, 60_000)]);
        let measured = run(order, &emit).unwrap();
        // Memory looks only after its 90 s of nothing.
        let looked = match order {
            Order::Memory => Duration::from_secs(90),
            _ => Duration::ZERO,
        };
        let earliest = looked + Duration::from_millis(3200 + 3000);
        let first = emit.ports.keyboard.times()[0];
        assert!(first >= earliest, "{order:?}: {first:?} < {earliest:?}");
        let note = "La ventana estuvo lista 2.2 s después";
        assert!(
            measured[0].notes.iter().any(|n| n.starts_with(note)),
            "{order:?}: {:?}",
            measured[0].notes
        );
    }
}

#[test]
fn a_window_that_never_changes_gets_nothing_typed_and_is_closed() {
    for order in ORDERS {
        stopped_without_typing(order, &opening(&[(0, 0)]), "no cambió");
    }
}

#[test]
fn a_window_off_the_primary_monitor_gets_nothing_typed_and_is_closed() {
    for order in ORDERS {
        let mut emit = opening(&[(0, 0), (1100, 60_000)]);
        emit.ports.primary = Rect {
            x: 0,
            y: 0,
            w: 1000,
            h: 700,
        };
        stopped_without_typing(order, &emit, "abrió en (0, 0), de 1280x800");
    }
}

#[test]
fn without_a_picture_from_before_no_window_is_opened() {
    for order in ORDERS {
        let unwatchable = opening(&[]);
        unwatchable.ports.screen.shown.borrow_mut().refusal = Some("sin duplicación".into());
        // A monitor that changes every half second for longer than the wait.
        let pictures: Vec<(u64, usize)> = (0..30u64)
            .map(|n| (n * 500, (n as usize % 2) * 60_000))
            .collect();
        for emit in [opening(&[]), unwatchable, opening(&pictures)] {
            let error = run(order, &emit).unwrap_err();
            assert!(
                error.ends_with("no se abre ninguna ventana"),
                "{order:?}: {error}"
            );
            assert!(asked(&emit).is_empty(), "{order:?}");
            assert!(emit.ports.keyboard.typed().is_empty(), "{order:?}");
        }
    }
}
