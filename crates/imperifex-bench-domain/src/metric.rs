//! What the bench measures: the loads it sends to a terminal and the metrics it
//! takes, each with its unit and its name for the reader.

use serde::{Deserialize, Serialize};

/// A kind of output that a terminal has to take in and show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Load {
    Ascii,
    LongLines,
    Color,
    Unicode,
    Cursor,
    Cargo,
}

impl Load {
    pub const ALL: [Self; 6] = [
        Self::Ascii,
        Self::LongLines,
        Self::Color,
        Self::Unicode,
        Self::Cursor,
        Self::Cargo,
    ];

    /// How the command line and the reports call it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::LongLines => "long_lines",
            Self::Color => "color",
            Self::Unicode => "unicode",
            Self::Cursor => "cursor",
            Self::Cargo => "cargo",
        }
    }

    /// The load with that [`name`](Self::name). The error says which there are.
    pub fn parse(text: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|load| load.name() == text)
            .ok_or_else(|| {
                let known = Self::ALL.map(Self::name).join(", ");
                format!("carga desconocida: {text}; las que hay: {known}")
            })
    }
}

/// One thing that is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    StartupCold,
    StartupWarm,
    StartupColdDrawn,
    StartupWarmDrawn,
    KeyLatency,
    OutputConsumed(Load),
    OutputSeen(Load),
    MemoryIdlePrivate,
    MemoryIdleWorkingSet,
    MemoryScrollbackPrivate,
    MemoryPerTabPrivate,
    CpuIdle,
    GpuIdle,
    CpuDuringOutput(Load),
}

/// The tables the metrics are read in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Startup,
    Key,
    Output,
    Memory,
    CpuAndGpu,
}

impl Group {
    /// In the order a reader gets them.
    pub const ALL: [Self; 5] = [
        Self::Startup,
        Self::Key,
        Self::Output,
        Self::Memory,
        Self::CpuAndGpu,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Startup => "Arranque",
            Self::Key => "Tecla",
            Self::Output => "Salida",
            Self::Memory => "Memoria",
            Self::CpuAndGpu => "CPU y GPU",
        }
    }
}

/// The unit of the samples of a metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Ms,
    /// Megabytes of output per second.
    MbPerS,
    Mb,
    /// Percent of one core: two full cores are 200.
    PercentOfCore,
    Percent,
}

impl Unit {
    /// What is written after a number.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Ms => "ms",
            Self::MbPerS => "MB/s",
            Self::Mb => "MB",
            Self::PercentOfCore => "% de un núcleo",
            Self::Percent => "%",
        }
    }
}

impl Metric {
    /// The one table of where each metric is read, what it is measured in and
    /// what it is called.
    fn describe(self) -> (Group, Unit, &'static str, Option<Load>) {
        use Group::{CpuAndGpu, Key, Memory, Output, Startup};
        use Unit::{MbPerS, Ms, Percent, PercentOfCore};
        const MB: Unit = Unit::Mb;
        let (group, unit, name) = match self {
            Self::StartupCold => (Startup, Ms, "Arranque en frío, hasta la ventana visible"),
            Self::StartupWarm => (
                Startup,
                Ms,
                "Arranque en caliente, hasta la ventana visible",
            ),
            Self::StartupColdDrawn => (Startup, Ms, "Arranque en frío, hasta la ventana dibujada"),
            Self::StartupWarmDrawn => (
                Startup,
                Ms,
                "Arranque en caliente, hasta la ventana dibujada",
            ),
            Self::KeyLatency => (Key, Ms, "Latencia de tecla"),
            Self::OutputConsumed(_) => (Output, MbPerS, "Salida consumida"),
            Self::OutputSeen(_) => (Output, Ms, "Salida vista"),
            Self::MemoryIdlePrivate => (Memory, MB, "Memoria privada en reposo"),
            Self::MemoryIdleWorkingSet => (Memory, MB, "Memoria de trabajo en reposo"),
            Self::MemoryScrollbackPrivate => (Memory, MB, "Memoria del historial largo"),
            Self::MemoryPerTabPrivate => (Memory, MB, "Memoria por pestaña"),
            Self::CpuIdle => (CpuAndGpu, PercentOfCore, "CPU en reposo"),
            Self::GpuIdle => (CpuAndGpu, Percent, "GPU en reposo"),
            Self::CpuDuringOutput(_) => (CpuAndGpu, PercentOfCore, "CPU durante la salida"),
        };
        (group, unit, name, self.load())
    }

    /// The load it was measured with, if it has one.
    fn load(self) -> Option<Load> {
        match self {
            Self::OutputConsumed(load) | Self::OutputSeen(load) | Self::CpuDuringOutput(load) => {
                Some(load)
            }
            _ => None,
        }
    }

    pub fn group(self) -> Group {
        self.describe().0
    }

    pub fn unit(self) -> Unit {
        self.describe().1
    }

    /// The name for the reader, with the load if it has one.
    pub fn label(self) -> String {
        match self.describe() {
            (_, _, name, Some(load)) => format!("{name} ({})", load.name()),
            (_, _, name, None) => name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_metric() -> Vec<Metric> {
        let mut all = vec![
            Metric::StartupCold,
            Metric::StartupWarm,
            Metric::StartupColdDrawn,
            Metric::StartupWarmDrawn,
            Metric::KeyLatency,
            Metric::MemoryIdlePrivate,
            Metric::MemoryIdleWorkingSet,
            Metric::MemoryScrollbackPrivate,
            Metric::MemoryPerTabPrivate,
            Metric::CpuIdle,
            Metric::GpuIdle,
        ];
        for load in Load::ALL {
            all.extend([
                Metric::OutputConsumed(load),
                Metric::OutputSeen(load),
                Metric::CpuDuringOutput(load),
            ]);
        }
        all
    }

    #[test]
    fn every_metric_has_a_name_of_its_own() {
        let labels: Vec<String> = every_metric().iter().map(|m| m.label()).collect();
        for (at, label) in labels.iter().enumerate() {
            assert!(
                !label.is_empty() && !labels[at + 1..].contains(label),
                "{label}"
            );
        }
    }

    #[test]
    fn units_follow_what_is_measured() {
        assert_eq!(Metric::KeyLatency.unit(), Unit::Ms);
        assert_eq!(Metric::OutputConsumed(Load::Ascii).unit(), Unit::MbPerS);
        assert_eq!(Metric::OutputSeen(Load::Ascii).unit(), Unit::Ms);
        assert_eq!(Metric::MemoryPerTabPrivate.unit(), Unit::Mb);
        assert_eq!(Metric::CpuIdle.unit(), Unit::PercentOfCore);
        assert_eq!(Metric::GpuIdle.unit(), Unit::Percent);
    }

    #[test]
    fn the_name_of_a_load_reads_back_as_the_same_load() {
        for load in Load::ALL {
            assert_eq!(Load::parse(load.name()), Ok(load));
        }
    }

    #[test]
    fn a_load_is_written_in_json_with_its_name() {
        for load in Load::ALL {
            let json = serde_json::to_string(&load).unwrap();
            assert_eq!(json, format!("\"{}\"", load.name()));
        }
    }

    #[test]
    fn an_unknown_load_says_which_ones_there_are() {
        let error = Load::parse("binary").unwrap_err();
        assert!(
            error.contains("binary") && error.contains("long_lines"),
            "{error}"
        );
    }
}
