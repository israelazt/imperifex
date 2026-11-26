//! The result of a run of the bench: on what machine, of which terminal, and
//! what was measured. Its JSON is what is kept in `bench/results/`, so the names
//! of its fields are a contract.

use serde::{Deserialize, Serialize};

use crate::metric::Metric;
use crate::private_paths::hide_folders;
use crate::stats::{summarize, Summary};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Monitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub hz: u32,
    pub primary: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    pub cpu: String,
    pub gpu: String,
    pub os: String,
    pub monitors: Vec<Monitor>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub name: String,
    pub version: String,
}

/// One metric with its samples and what they add up to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub metric: Metric,
    pub summary: Summary,
    /// In the unit of the metric, in the order they were taken.
    pub samples: Vec<f64>,
    /// What a reader should know about how they were taken.
    pub notes: Vec<String>,
}

impl Measurement {
    /// Keeps the finite samples (JSON cannot hold the others) and sums them up;
    /// `None` if there are none.
    pub fn new(metric: Metric, mut samples: Vec<f64>, notes: Vec<String>) -> Option<Self> {
        samples.retain(|s| s.is_finite());
        let summary = summarize(&samples)?;
        Some(Self {
            metric,
            summary,
            samples,
            notes,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// `YYYY-MM-DD`.
    pub date: String,
    pub machine: Machine,
    pub terminal: TerminalInfo,
    pub bench_version: String,
    pub measurements: Vec<Measurement>,
    /// What a reader should know about the run as a whole, such as a part that
    /// failed.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl Report {
    /// The same report with every note, its own and each measurement's, free of
    /// the folders given (see [`hide_folders`]).
    pub fn without_folders(&self, folders: &[(String, String)]) -> Self {
        let mut report = self.clone();
        let notes = report
            .measurements
            .iter_mut()
            .flat_map(|measurement| &mut measurement.notes);
        for note in notes.chain(&mut report.notes) {
            *note = hide_folders(note, folders);
        }
        report
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }

    /// What the reader gets on the screen: a line for each measurement.
    pub fn to_text(&self) -> String {
        let mut text = format!(
            "{} {} · {} · {}",
            self.terminal.name, self.terminal.version, self.date, self.machine.os
        );
        for measurement in &self.measurements {
            let unit = measurement.metric.unit().symbol();
            let s = &measurement.summary;
            text.push_str(&format!(
                "\n{}: p50 {:.1} {unit} · p95 {:.1} {unit} · máx {:.1} {unit} (n = {})",
                measurement.metric.label(),
                s.p50,
                s.p95,
                s.max,
                s.count
            ));
        }
        text
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| format!("el informe no se puede leer: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric::Load;

    fn report() -> Report {
        let measurement = |metric, samples| Measurement::new(metric, samples, vec![]).unwrap();
        Report {
            date: "2026-11-25".into(),
            machine: Machine {
                cpu: "cpu".into(),
                gpu: "gpu".into(),
                os: "os".into(),
                monitors: vec![Monitor {
                    name: "main".into(),
                    width: 1920,
                    height: 1080,
                    hz: 144,
                    primary: true,
                }],
            },
            terminal: TerminalInfo {
                name: "warp".into(),
                version: "1".into(),
            },
            bench_version: "0.1.0".into(),
            measurements: vec![
                measurement(Metric::KeyLatencyPrompt, vec![10.0, 12.0]),
                measurement(Metric::OutputSeen(Load::LongLines), vec![300.0]),
            ],
            notes: vec!["Falló algo".into()],
        }
    }

    #[test]
    fn a_report_comes_back_from_its_json_as_it_was() {
        assert_eq!(Report::from_json(&report().to_json()), Ok(report()));
    }

    #[test]
    fn the_json_has_stable_names() {
        let value: serde_json::Value = serde_json::from_str(&report().to_json()).unwrap();
        assert_eq!(value["date"], "2026-11-25");
        assert_eq!(value["bench_version"], "0.1.0");
        assert_eq!(value["notes"][0], "Falló algo");
        assert_eq!(value["machine"]["cpu"], "cpu");
        assert_eq!(value["machine"]["gpu"], "gpu");
        assert_eq!(value["machine"]["os"], "os");
        let monitor = &value["machine"]["monitors"][0];
        assert_eq!(monitor["name"], "main");
        assert_eq!(monitor["width"], 1920);
        assert_eq!(monitor["height"], 1080);
        assert_eq!(monitor["hz"], 144);
        assert_eq!(monitor["primary"], true);
        assert_eq!(value["terminal"]["name"], "warp");
        assert_eq!(value["terminal"]["version"], "1");
        let first = &value["measurements"][0];
        assert_eq!(first["metric"], "key_latency_prompt");
        assert_eq!(first["samples"][1], 12.0);
        assert!(first["notes"].as_array().unwrap().is_empty());
        for field in ["count", "min", "max", "mean", "stddev", "p50", "p95", "p99"] {
            assert!(first["summary"][field].is_number(), "{field}");
        }
        assert_eq!(
            value["measurements"][1]["metric"]["output_seen"],
            "long_lines"
        );
    }

    #[test]
    fn the_text_has_a_line_for_each_measurement() {
        let text = report().to_text();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "warp 1 · 2026-11-25 · os");
        assert_eq!(
            lines[1],
            "Tecla en el prompt: p50 10.0 ms · p95 12.0 ms · máx 12.0 ms (n = 2)"
        );
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn a_measurement_keeps_only_the_samples_json_can_hold() {
        let m = Measurement::new(Metric::KeyLatencyPrompt, vec![f64::NAN, 4.0], vec![]).unwrap();
        assert_eq!(m.samples, [4.0]);
        assert_eq!(
            Measurement::new(Metric::KeyLatencyPrompt, vec![], vec![]),
            None
        );
    }

    #[test]
    fn what_is_not_a_report_is_an_error_that_says_so() {
        assert!(Report::from_json("{}").unwrap_err().contains("informe"));
    }
}
