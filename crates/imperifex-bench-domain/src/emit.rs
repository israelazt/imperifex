//! What sending a load to a terminal found.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::metric::Load;

/// The bytes in a megabyte, as Windows counts them.
pub const BYTES_PER_MB: usize = 1024 * 1024;

/// How long a terminal took to take in a load. The program that emits it writes
/// this to a file, and the bench reads it from there.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmitResult {
    pub load: Load,
    pub bytes: usize,
    pub seconds: f64,
    pub mb_per_s: f64,
}

impl EmitResult {
    /// A time too short to be measured gives a speed of zero, not an infinite
    /// one, which JSON could not hold.
    pub fn new(load: Load, bytes: usize, took: Duration) -> Self {
        let seconds = took.as_secs_f64();
        let mb_per_s = if seconds > 0.0 {
            bytes as f64 / BYTES_PER_MB as f64 / seconds
        } else {
            0.0
        };
        Self {
            load,
            bytes,
            seconds,
            mb_per_s,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|error| format!("el resultado de emit no se puede leer: {error}"))
    }

    /// The line for the reader.
    pub fn to_text(&self) -> String {
        format!(
            "{}: {:.1} MB en {:.3} s = {:.1} MB/s",
            self.load.name(),
            self.bytes as f64 / BYTES_PER_MB as f64,
            self.seconds,
            self.mb_per_s
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result() -> EmitResult {
        EmitResult::new(Load::LongLines, 32 * BYTES_PER_MB, Duration::from_secs(2))
    }

    #[test]
    fn the_speed_is_the_megabytes_by_the_seconds() {
        assert_eq!((result().seconds, result().mb_per_s), (2.0, 16.0));
        assert_eq!(
            result().to_text(),
            "long_lines: 32.0 MB en 2.000 s = 16.0 MB/s"
        );
    }

    #[test]
    fn no_time_at_all_is_not_an_infinite_speed() {
        assert_eq!(
            EmitResult::new(Load::Ascii, 10, Duration::ZERO).mb_per_s,
            0.0
        );
    }

    #[test]
    fn the_json_has_stable_names_and_reads_back() {
        let value: serde_json::Value = serde_json::from_str(&result().to_json()).unwrap();
        assert_eq!(value["load"], "long_lines");
        assert_eq!(value["bytes"], 32 * BYTES_PER_MB);
        assert_eq!(value["seconds"], 2.0);
        assert_eq!(value["mb_per_s"], 16.0);
        assert_eq!(EmitResult::from_json(&result().to_json()), Ok(result()));
    }
}
