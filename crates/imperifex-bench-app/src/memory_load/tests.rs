use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

use super::*;
use crate::testing::desktop::{home, rig, Behaviour};
use crate::testing::emitting::{emitting, Emitting};
use crate::testing::system::warp;

/// The window's keys open this many tabs, each with its console helper.
fn with_tabs(behaviour: Behaviour, tabs: u32) -> Emitting {
    let emit = emitting(rig(behaviour, true), true);
    let desktop = emit.ports.session.desktop.clone();
    let processes = emit.ports.session.processes.clone();
    let opened = std::cell::Cell::new(0);
    emit.ports.keyboard.on_chord(move || {
        if opened.get() < tabs {
            opened.set(opened.get() + 1);
            let pid = 5000 + opened.get();
            processes.rename(pid, "OpenConsole.exe");
            let launched = desktop.state.borrow().alive.first().copied();
            processes.child_of(pid, launched.unwrap_or_default());
            desktop.state.borrow_mut().alive.push(pid);
        }
    });
    emit
}

fn run(
    emit: &Emitting,
    memory: &[f64],
    profile: &TerminalProfile,
) -> Result<Vec<Measurement>, String> {
    let readings = RefCell::new(VecDeque::from(memory.to_vec()));
    let private_mb = || {
        readings
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| "sin más lecturas".to_string())
    };
    let service = MemoryService {
        output: OutputService {
            ports: emit.ports.clone(),
            files: &emit.files,
            bench_exe: "bench.exe".into(),
        },
        private_mb: &private_mb,
    };
    service.run(profile, home())
}

fn with_keys() -> TerminalProfile {
    TerminalProfile {
        new_tab_keys: Some("Ctrl+Shift+T".into()),
        ..warp()
    }
}

#[test]
fn the_history_is_the_difference_and_each_tab_its_share_of_the_rise() {
    let emit = with_tabs(Behaviour::default(), 3);
    let measured = run(&emit, &[100.0, 160.0, 170.0, 180.0, 190.0], &with_keys()).unwrap();
    assert_eq!(measured[0].metric, Metric::MemoryScrollbackPrivate);
    assert_eq!(measured[0].samples, [60.0]);
    assert_eq!(measured[1].metric, Metric::MemoryPerTabPrivate);
    assert_eq!(measured[1].samples, [10.0]);
    let typed = emit.ports.keyboard.typed();
    assert!(
        typed.iter().any(|t| t.contains("'ascii' '--mib' '79'")),
        "{typed:?}"
    );
}

#[test]
fn the_window_opens_only_after_90_seconds_of_nothing() {
    let emit = with_tabs(Behaviour::default(), 0);
    run(&emit, &[100.0, 160.0], &warp()).unwrap();
    let launched = emit
        .ports
        .session
        .desktop
        .state
        .borrow()
        .launched_at
        .clone();
    assert_eq!(launched, [Duration::from_secs(90)]);
}

#[test]
fn keys_that_open_no_tab_leave_the_tabs_out_and_say_so() {
    let emit = with_tabs(Behaviour::default(), 0);
    let measured = run(&emit, &[100.0, 160.0], &with_keys()).unwrap();
    assert_eq!(measured.len(), 1);
    assert!(
        measured[0].notes[1].contains("no abrieron"),
        "{:?}",
        measured[0].notes
    );
}

#[test]
fn only_the_tabs_that_opened_share_the_rise() {
    let emit = with_tabs(Behaviour::default(), 2);
    let measured = run(&emit, &[100.0, 160.0, 175.0, 200.0], &with_keys()).unwrap();
    assert_eq!(measured[1].samples, [20.0]);
    assert!(measured[1].notes[1].ends_with("2."));
}

#[test]
fn a_profile_without_keys_has_no_tabs_to_measure() {
    let emit = with_tabs(Behaviour::default(), 3);
    let measured = run(&emit, &[100.0, 160.0], &warp()).unwrap();
    assert_eq!(measured.len(), 1);
    assert!(measured[0].notes[1].contains("no tiene teclas"));
}

#[test]
fn keys_that_cannot_be_read_open_nothing() {
    let emit = with_tabs(Behaviour::default(), 3);
    let profile = TerminalProfile {
        new_tab_keys: Some("Super+T".into()),
        ..warp()
    };
    assert!(run(&emit, &[], &profile).unwrap_err().contains("Super"));
    assert!(emit.ports.session.desktop.state.borrow().asked.is_empty());
    assert_eq!(emit.ports.session.clock.now(), Duration::ZERO, "no wait");
}

#[test]
fn a_question_when_closing_the_tabs_stops_it_and_nothing_else_is_pressed() {
    let asks = Behaviour {
        asks_to_confirm: true,
        ..Default::default()
    };
    let emit = with_tabs(asks, 3);
    let error = run(&emit, &[1.0, 2.0, 3.0, 4.0, 5.0], &with_keys()).unwrap_err();
    assert!(error.contains("confirmar"), "{error}");
    let asked = emit.ports.session.desktop.state.borrow().asked.clone();
    assert_eq!(asked.iter().filter(|a| a.starts_with("close")).count(), 1);
}
