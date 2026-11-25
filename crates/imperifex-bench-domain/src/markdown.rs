//! The baseline as a document: the tables of one or more reports side by side,
//! each run (`tanda`) in its own rows.

use crate::metric::{Group, Metric};
use crate::report::{Machine, Measurement, Report};

/// The document for these reports, numbered as runs in the order given.
pub fn baseline_markdown(reports: &[Report]) -> String {
    let mut text = String::from("# Línea base\n");
    let Some(first) = reports.first() else {
        text.push_str("\nNo hay informes.\n");
        return text;
    };
    text.push_str(&machine_section(&first.machine));
    text.push_str(&runs_section(reports));
    for group in Group::ALL {
        text.push_str(&group_section(group, reports));
    }
    text
}

fn machine_section(machine: &Machine) -> String {
    let mut text = format!(
        "\n## Máquina\n\n- CPU: {}\n- GPU: {}\n- Sistema: {}\n",
        machine.cpu, machine.gpu, machine.os
    );
    for monitor in &machine.monitors {
        let primary = if monitor.primary { " · principal" } else { "" };
        text.push_str(&format!(
            "- Monitor: {} · {}×{} · {} Hz{primary}\n",
            monitor.name, monitor.width, monitor.height, monitor.hz
        ));
    }
    text
}

/// The runs, with a warning for each one from another machine: its numbers do
/// not compare with the rest.
fn runs_section(reports: &[Report]) -> String {
    let mut text =
        String::from("\n## Tandas\n\n| Tanda | Fecha | Terminal | Banco |\n|---|---|---|---|\n");
    for (number, report) in (1..).zip(reports) {
        text.push_str(&format!(
            "| {number} | {} | {} {} | {} |\n",
            report.date, report.terminal.name, report.terminal.version, report.bench_version
        ));
    }
    for (number, report) in (1..).zip(reports) {
        if report.machine != reports[0].machine {
            text.push_str(&format!(
                "\n> Aviso: la tanda {number} se midió con una máquina distinta de la descrita arriba.\n"
            ));
        }
    }
    text
}

/// The metrics of a group, in the order they first show up.
fn metrics_of(group: Group, reports: &[Report]) -> Vec<Metric> {
    let mut found = Vec::new();
    for measurement in reports.iter().flat_map(|r| &r.measurements) {
        let metric = measurement.metric;
        if metric.group() == group && !found.contains(&metric) {
            found.push(metric);
        }
    }
    found
}

/// Empty if nothing of the group was measured.
fn group_section(group: Group, reports: &[Report]) -> String {
    let mut rows = String::new();
    for metric in metrics_of(group, reports) {
        for (number, report) in (1..).zip(reports) {
            for measurement in report.measurements.iter().filter(|m| m.metric == metric) {
                rows.push_str(&row(number, measurement));
            }
        }
    }
    if rows.is_empty() {
        return rows;
    }
    format!(
        "\n## {}\n\n| Medida | Tanda | n | p50 | p95 | p99 | Dispersión |\n|---|---|---|---|---|---|---|\n{rows}",
        group.label()
    )
}

fn row(run: usize, measurement: &Measurement) -> String {
    let summary = &measurement.summary;
    let unit = measurement.metric.unit().symbol();
    let value = |v: f64| format!("{v:.1} {unit}");
    format!(
        "| {} | {run} | {} | {} | {} | {} | {} |\n",
        measurement.metric.label(),
        summary.count,
        value(summary.p50),
        value(summary.p95),
        value(summary.p99),
        spread(summary.stddev, summary.mean)
    )
}

/// The deviation as a part of the mean; a mean of zero has none.
fn spread(stddev: f64, mean: f64) -> String {
    if mean == 0.0 {
        "—".to_string()
    } else {
        format!("{:.1} %", stddev / mean * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric::Load;
    use crate::report::{Monitor, TerminalInfo};

    fn report(date: &str, measurements: &[(Metric, &[f64])]) -> Report {
        Report {
            date: date.into(),
            machine: Machine {
                cpu: "Ryzen".into(),
                gpu: "GTX".into(),
                os: "Windows 11".into(),
                monitors: vec![Monitor {
                    name: "DISPLAY1".into(),
                    width: 1920,
                    height: 1080,
                    hz: 144,
                    primary: true,
                }],
            },
            terminal: TerminalInfo {
                name: "warp".into(),
                version: "v1".into(),
            },
            bench_version: "0.1.0".into(),
            measurements: measurements
                .iter()
                .map(|(m, s)| Measurement::new(*m, s.to_vec(), vec![]).unwrap())
                .collect(),
        }
    }

    const EXPECTED: &str = r"# Línea base

## Máquina

- CPU: Ryzen
- GPU: GTX
- Sistema: Windows 11
- Monitor: DISPLAY1 · 1920×1080 · 144 Hz · principal

## Tandas

| Tanda | Fecha | Terminal | Banco |
|---|---|---|---|
| 1 | 2026-11-25 | warp v1 | 0.1.0 |
| 2 | 2026-11-26 | warp v1 | 0.1.0 |

## Arranque

| Medida | Tanda | n | p50 | p95 | p99 | Dispersión |
|---|---|---|---|---|---|---|
| Arranque en caliente | 1 | 2 | 100.0 ms | 300.0 ms | 300.0 ms | 50.0 % |
| Arranque en caliente | 2 | 1 | 120.0 ms | 120.0 ms | 120.0 ms | 0.0 % |

## Salida

| Medida | Tanda | n | p50 | p95 | p99 | Dispersión |
|---|---|---|---|---|---|---|
| Salida consumida (ascii) | 2 | 1 | 55.2 MB/s | 55.2 MB/s | 55.2 MB/s | 0.0 % |

## CPU y GPU

| Medida | Tanda | n | p50 | p95 | p99 | Dispersión |
|---|---|---|---|---|---|---|
| GPU en reposo | 1 | 2 | 0.0 % | 0.0 % | 0.0 % | — |
| CPU en reposo | 2 | 1 | 4.5 % de un núcleo | 4.5 % de un núcleo | 4.5 % de un núcleo | 0.0 % |
";

    #[test]
    fn two_runs_make_this_exact_document() {
        let reports = [
            report(
                "2026-11-25",
                &[
                    (Metric::GpuIdle, &[0.0, 0.0]),
                    (Metric::StartupWarm, &[100.0, 300.0]),
                ],
            ),
            report(
                "2026-11-26",
                &[
                    (Metric::CpuIdle, &[4.5]),
                    (Metric::OutputConsumed(Load::Ascii), &[55.25]),
                    (Metric::StartupWarm, &[120.0]),
                ],
            ),
        ];
        assert_eq!(baseline_markdown(&reports), EXPECTED);
    }

    #[test]
    fn a_run_from_another_machine_is_warned_about() {
        let mut other = report("2026-11-26", &[]);
        other.machine.gpu = "otra".into();
        let text = baseline_markdown(&[report("2026-11-25", &[]), other]);
        assert!(
            text.contains("la tanda 2 se midió con una máquina distinta"),
            "{text}"
        );
    }

    #[test]
    fn no_reports_say_so() {
        assert_eq!(baseline_markdown(&[]), "# Línea base\n\nNo hay informes.\n");
    }
}
