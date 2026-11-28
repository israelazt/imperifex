//! An example of a report made in the domain: data with its text and JSON
//! forms. Replace it with the first real model of the app.

use serde::Serialize;

use crate::i18n::{fill, Language, Text};

/// What `status` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatusReport {
    pub version: String,
    /// How long the check took, in milliseconds.
    pub took_ms: u64,
}

impl StatusReport {
    pub fn to_text(&self, language: Language) -> String {
        fill(language, Text::StatusWorks, &[&self.version, &self.took_ms])
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> StatusReport {
        StatusReport {
            version: "0.1.0".into(),
            took_ms: 3,
        }
    }

    #[test]
    fn the_text_says_it_works_and_how_long_it_took() {
        let text = report().to_text(Language::Spanish);
        assert_eq!(text, "Imperifex 0.1.0 funciona · 3 ms");
    }

    #[test]
    fn the_json_has_stable_names() {
        let value: serde_json::Value = serde_json::from_str(&report().to_json()).unwrap();
        assert_eq!(value["version"], "0.1.0");
        assert_eq!(value["took_ms"], 3);
    }
}
